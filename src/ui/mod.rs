pub mod generated {
    slint::include_modules!();
}

use std::error::Error;

use slint::ComponentHandle;

use crate::parser::ParsedContent;

pub struct PreviewWindow {
    ui: generated::PreviewWindow,
}

impl PreviewWindow {
    pub fn new(standalone: bool) -> Result<Self, Box<dyn Error>> {
        let ui = generated::PreviewWindow::new()?;
        if standalone {
            ui.window().on_close_requested(move || {
                let _ = slint::quit_event_loop();
                slint::CloseRequestResponse::HideWindow
            });
        }
        Ok(Self { ui })
    }

    pub fn show(&self, path: &str, content: &ParsedContent) {
        self.ui.set_file_name(path.into());
        self.ui.set_show_text(false);
        self.ui.set_show_image(false);
        self.ui.set_show_tree(false);
        self.ui.set_show_markdown(false);

        match content {
            ParsedContent::Text { content, language, line_count } => {
                self.ui.set_text_content(content.into());
                self.ui.set_show_text(true);
                self.ui.set_status_text(format!("{language}  |  {line_count} lines").into());
            }
            ParsedContent::Image { data, width, height, format } => {
                if let Ok(img) = image::load_from_memory(data) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let raw = rgba.as_raw();
                    let mut buffer =
                        slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(w, h);
                    let pixel_slice = buffer.make_mut_slice();
                    for (i, pixel) in pixel_slice.iter_mut().enumerate() {
                        let offset = i * 4;
                        pixel.r = raw[offset];
                        pixel.g = raw[offset + 1];
                        pixel.b = raw[offset + 2];
                        pixel.a = raw[offset + 3];
                    }
                    self.ui.set_preview_image(slint::Image::from_rgba8(buffer));
                }
                self.ui.set_show_image(true);
                self.ui.set_status_text(format!("{format:?}  |  {width}x{height}").into());
            }
            ParsedContent::Pdf { page_count, .. } => {
                self.ui.set_text_content(
                    format!("PDF document - {page_count} pages\n(rendering not yet supported)")
                        .into(),
                );
                self.ui.set_show_text(true);
                self.ui.set_status_text(format!("PDF  |  {page_count} pages").into());
            }
            ParsedContent::Archive { entries, total_files } => {
                let tree = entries
                    .iter()
                    .map(|e| {
                        let prefix = if e.is_dir { "[DIR] " } else { "[FILE] " };
                        format!("{}{}  ({} bytes)", prefix, e.path, e.size)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                self.ui.set_tree_content(tree.into());
                self.ui.set_show_tree(true);
                self.ui.set_status_text(format!("Archive  |  {total_files} files").into());
            }
            ParsedContent::Markdown { content, images } => {
                match slint::StyledText::from_markdown(content) {
                    Ok(st) => {
                        self.ui.set_styled_content(st);
                        self.ui.set_show_markdown(true);
                        self.ui.set_status_text(format!("Markdown  |  {} images", images.len()).into());
                    }
                    Err(_) => {
                        self.ui.set_text_content(content.as_str().into());
                        self.ui.set_show_markdown(false);
                        self.ui.set_show_text(true);
                        self.ui.set_status_text("Markdown  |  parse error, showing plain text".into());
                    }
                }
            }
            ParsedContent::Folder { entries } => {
                let tree = entries
                    .iter()
                    .map(|e| {
                        let icon = if e.is_dir { "[DIR] " } else { "" };
                        format!("{}{}  {}  {}", icon, e.name, e.modified, e.size)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                self.ui.set_tree_content(tree.into());
                self.ui.set_show_tree(true);
                self.ui.set_status_text(format!("Folder  |  {} items", entries.len()).into());
            }
        }

        let _ = self.ui.window().show();
    }

    pub fn hide(&self) {
        let _ = self.ui.window().hide();
    }

    pub fn window(&self) -> &slint::Window {
        self.ui.window()
    }
}
