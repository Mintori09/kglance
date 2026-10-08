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
