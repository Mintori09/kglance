use crate::app::messages::Message;
use iced::Task;

/// Copies clean formatted plain text and optional Rich Text (HTML) to the system clipboard.
pub fn copy_to_clipboard(plain_text: String, html_text: Option<String>) -> Task<Message> {
    let plain_bytes = plain_text.clone().into_bytes().into_boxed_slice();

    if let Some(html) = html_text {
        let html_bytes = html.into_bytes().into_boxed_slice();

        let sources = vec![
            wl_clipboard_rs::copy::MimeSource {
                source: wl_clipboard_rs::copy::Source::Bytes(plain_bytes),
                mime_type: wl_clipboard_rs::copy::MimeType::Text,
            },
            wl_clipboard_rs::copy::MimeSource {
                source: wl_clipboard_rs::copy::Source::Bytes(html_bytes),
                mime_type: wl_clipboard_rs::copy::MimeType::Specific("text/html".to_string()),
            },
        ];

        let opts = wl_clipboard_rs::copy::Options::new();
        match opts.copy_multi(sources) {
            Ok(()) => {
                return Task::none();
            }
            Err(err) => {
                crate::log_info!("[ERROR]: {err}, fallback to iced clipboard");
            }
        }
    } else {
        let opts = wl_clipboard_rs::copy::Options::new();
        match opts.copy(
            wl_clipboard_rs::copy::Source::Bytes(plain_bytes),
            wl_clipboard_rs::copy::MimeType::Text,
        ) {
            Ok(()) => {
                return Task::none();
            }
            Err(err) => {
                crate::log_info!("[ERROR]: {err}, fallback to iced clipboard");
            }
        }
    }

    iced::clipboard::write(plain_text)
}

/// Copies bitmap image data (PNG formatted) to the system clipboard.
pub fn copy_image_to_clipboard(bytes: &[u8]) -> Task<Message> {
    if bytes.is_empty() {
        return Task::none();
    }

    let png_bytes = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        bytes.to_vec()
    } else {
        match image::load_from_memory(bytes) {
            Ok(img) => {
                let mut buf = std::io::Cursor::new(Vec::new());
                if img.write_to(&mut buf, image::ImageFormat::Png).is_ok() {
                    buf.into_inner()
                } else {
                    bytes.to_vec()
                }
            }
            Err(_) => bytes.to_vec(),
        }
    };

    let opts = wl_clipboard_rs::copy::Options::new();
    match opts.copy(
        wl_clipboard_rs::copy::Source::Bytes(png_bytes.into_boxed_slice()),
        wl_clipboard_rs::copy::MimeType::Specific("image/png".to_string()),
    ) {
        Ok(()) => Task::none(),
        Err(err) => {
            crate::log_info!("[ERROR]: {err}, failed to copy image to clipboard");
            Task::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_image_empty_bytes() {
        let task = copy_image_to_clipboard(&[]);
        let _ = task;
    }

    #[test]
    fn test_copy_image_png_bytes() {
        let dummy_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89";
        let task = copy_image_to_clipboard(dummy_png);
        let _ = task;
    }
}
