use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::{KglanceState, MarkdownState};
use crate::features::markdown::types::ImageRef;
use crate::features::markdown::{Block, extract_toc};
use iced::{Element, Task};
use std::collections::HashMap;
use std::path::Path;

pub struct MarkdownContent {
    pub content: String,
    pub images: Vec<ImageRef>,
    pub blocks: Vec<Block>,
}

impl PreviewContent<Message> for MarkdownContent {
    fn populate_state(&self, state: &mut KglanceState) {
        let fs = state.font_size;
        state.markdown = MarkdownState {
            toc: extract_toc(&self.blocks, fs, &HashMap::new()),
            ..Default::default()
        };
        for (i, block) in self.blocks.iter().enumerate() {
            if let Block::Mermaid {
                rendered: Some(png),
                ..
            } = block
            {
                let handle = match image::load_from_memory(png) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        iced::widget::image::Handle::from_rgba(w, h, rgba.into_raw())
                    }
                    Err(_) => iced::widget::image::Handle::from_bytes(png.clone()),
                };
                state.markdown.cached_mermaid_handles.insert(i, handle);
            }
        }
        state.file_type_text = "Markdown Document".to_string();
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        crate::features::markdown::view_markdown(
            &self.blocks,
            &state.markdown,
            state.font_size,
            state.theme_dark,
        )
    }

    fn content_type(&self) -> ContentType {
        ContentType::Markdown
    }
    fn supports_toc(&self) -> bool {
        true
    }
    fn supports_text_operations(&self) -> bool {
        true
    }

    fn on_loaded(&self, _state: &KglanceState, path: &str) -> Task<Message> {
        let mut tasks = Vec::new();
        for (i, block) in self.blocks.iter().enumerate() {
            match block {
                Block::Mermaid {
                    lines,
                    rendered: None,
                } => {
                    let code = lines.join("\n");
                    tasks.push(Task::perform(
                        async move {
                            let png = tokio::task::spawn_blocking(move || {
                                crate::features::markdown::render_mermaid_to_png(&code)
                            })
                            .await
                            .ok()
                            .flatten();
                            Message::MermaidBlockRendered {
                                index: i,
                                png_bytes: png,
                            }
                        },
                        |msg| msg,
                    ));
                }
                Block::Image { path: img_path, .. } => {
                    let resolved = if Path::new(img_path).is_absolute() {
                        std::path::PathBuf::from(img_path)
                    } else {
                        Path::new(path)
                            .parent()
                            .unwrap_or(Path::new("."))
                            .join(img_path)
                    };
                    tasks.push(Task::perform(
                        async move {
                            let bytes =
                                tokio::task::spawn_blocking(move || std::fs::read(&resolved).ok())
                                    .await
                                    .ok()
                                    .flatten();
                            Message::MarkdownImageLoaded {
                                index: i,
                                png_bytes: bytes,
                            }
                        },
                        |msg| msg,
                    ));
                }
                _ => {}
            }
        }
        for (bi, block) in self.blocks.iter().enumerate() {
            let inlines = match block {
                Block::Paragraph(inlines)
                | Block::Heading {
                    content: inlines, ..
                } => inlines,
                _ => continue,
            };
            for (ii, inline) in inlines.iter().enumerate() {
                if let crate::features::markdown::Inline::Image { url, .. } = inline {
                    let resolved = if Path::new(url).is_absolute() {
                        std::path::PathBuf::from(url)
                    } else {
                        Path::new(path).parent().unwrap_or(Path::new(".")).join(url)
                    };
                    tasks.push(Task::perform(
                        async move {
                            let bytes =
                                tokio::task::spawn_blocking(move || std::fs::read(&resolved).ok())
                                    .await
                                    .ok()
                                    .flatten();
                            Message::InlineImageLoaded {
                                block_index: bi,
                                inline_index: ii,
                                png_bytes: bytes,
                            }
                        },
                        |msg| msg,
                    ));
                }
            }
        }
        Task::batch(tasks)
    }
}
