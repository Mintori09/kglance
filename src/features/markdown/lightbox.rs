use iced::Task;

use crate::app::{KglanceApp, Message};
use crate::features::lightbox::types::LightboxItem;
use crate::features::markdown::{Block, Inline};

fn get_image_subtitle(path: &str) -> String {
    if path.starts_with("data:image/") {
        "Embedded image".to_string()
    } else {
        std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string()
    }
}

fn collect_inline_images(
    inlines: &[Inline],
    cached_handles: &std::collections::HashMap<String, iced::widget::image::Handle>,
    items: &mut Vec<LightboxItem>,
) {
    for inline in inlines {
        match inline {
            Inline::Image { alt, url, .. } => {
                if cached_handles.contains_key(url) {
                    let subtitle = get_image_subtitle(url);
                    let title = if alt.is_empty() {
                        None
                    } else {
                        Some(alt.clone())
                    };
                    items.push(LightboxItem {
                        id: url.clone(),
                        title,
                        subtitle,
                        link_url: None,
                        in_memory_bytes: None,
                    });
                }
            }
            Inline::Link { text, url } => {
                for sub in text {
                    if let Inline::Image {
                        alt, url: img_url, ..
                    } = sub
                    {
                        if cached_handles.contains_key(img_url) {
                            let subtitle = get_image_subtitle(img_url);
                            let title = if alt.is_empty() {
                                None
                            } else {
                                Some(alt.clone())
                            };
                            items.push(LightboxItem {
                                id: img_url.clone(),
                                title,
                                subtitle,
                                link_url: Some(url.clone()),
                                in_memory_bytes: None,
                            });
                        }
                    } else {
                        collect_inline_images(std::slice::from_ref(sub), cached_handles, items);
                    }
                }
            }
            Inline::Bold(subs)
            | Inline::Italic(subs)
            | Inline::Strikethrough(subs)
            | Inline::Underline(subs) => {
                collect_inline_images(subs, cached_handles, items);
            }
            _ => {}
        }
    }
}

pub fn collect_markdown_lightbox_items(
    blocks: &[Block],
    cached_handles: &std::collections::HashMap<String, iced::widget::image::Handle>,
) -> Vec<LightboxItem> {
    let mut items = Vec::new();
    let mut mermaid_counter = 0;

    fn collect_from_blocks(
        blocks: &[Block],
        cached_handles: &std::collections::HashMap<String, iced::widget::image::Handle>,
        items: &mut Vec<LightboxItem>,
        mermaid_counter: &mut usize,
        prefix: &str,
    ) {
        for (idx, block) in blocks.iter().enumerate() {
            let block_id = if prefix.is_empty() {
                format!("{idx}")
            } else {
                format!("{prefix}.{idx}")
            };

            match block {
                Block::Image {
                    alt,
                    path,
                    link_url,
                    ..
                } => {
                    if cached_handles.contains_key(path) {
                        let subtitle = get_image_subtitle(path);
                        let title = if alt.is_empty() {
                            None
                        } else {
                            Some(alt.clone())
                        };
                        items.push(LightboxItem {
                            id: path.clone(),
                            title,
                            subtitle,
                            link_url: link_url.clone(),
                            in_memory_bytes: None,
                        });
                    }
                }
                Block::Mermaid { rendered, .. } => {
                    *mermaid_counter += 1;
                    let counter = *mermaid_counter;
                    if let Some(png) = rendered {
                        items.push(LightboxItem {
                            id: format!("mermaid:{block_id}"),
                            title: None,
                            subtitle: format!("Mermaid Diagram #{counter}"),
                            link_url: None,
                            in_memory_bytes: Some(png.clone()),
                        });
                    }
                }
                Block::Paragraph(inlines)
                | Block::Heading {
                    content: inlines, ..
                } => {
                    collect_inline_images(inlines, cached_handles, items);
                }
                Block::List {
                    items: list_items, ..
                } => {
                    for (item_idx, item) in list_items.iter().enumerate() {
                        collect_inline_images(&item.content, cached_handles, items);
                        let sub_prefix = format!("{block_id}.{item_idx}");
                        collect_from_blocks(
                            &item.sub_blocks,
                            cached_handles,
                            items,
                            mermaid_counter,
                            &sub_prefix,
                        );
                    }
                }
                Block::Table(table) => {
                    for header in &table.headers {
                        collect_inline_images(&header.content, cached_handles, items);
                    }
                    for row in &table.rows {
                        for cell in row {
                            collect_inline_images(&cell.content, cached_handles, items);
                        }
                    }
                }
                Block::Quote(sub_blocks)
                | Block::Alert {
                    content: sub_blocks,
                    ..
                }
                | Block::FootnoteDefinition {
                    content: sub_blocks,
                    ..
                } => {
                    collect_from_blocks(
                        sub_blocks,
                        cached_handles,
                        items,
                        mermaid_counter,
                        &block_id,
                    );
                }
                _ => {}
            }
        }
    }

    collect_from_blocks(blocks, cached_handles, &mut items, &mut mermaid_counter, "");

    items
}

pub fn handle_open_markdown_image_lightbox(app: &mut KglanceApp, target_id: &str) -> Task<Message> {
    let Some(crate::core::PreviewData::Markdown { blocks, .. }) = &app.current_content else {
        return Task::none();
    };

    let items = collect_markdown_lightbox_items(blocks, &app.state.markdown.cached_image_handles);

    if items.is_empty() {
        return Task::none();
    }

    let initial_index = items.iter().position(|it| it.id == target_id).unwrap_or(0);

    crate::features::lightbox::ops::handle_open_lightbox(app, items, initial_index)
}

#[cfg(test)]
#[path = "lightbox_tests.rs"]
mod tests;
