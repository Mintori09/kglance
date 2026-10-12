use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use iced::Task;

use crate::app::messages::{MarkdownMsg, Message};
use crate::log_debug;

/// Decode a base64 data URI (e.g. `data:image/png;base64,...`) into raw bytes.
pub fn parse_data_image_uri(uri: &str) -> Option<Vec<u8>> {
    if !uri.starts_with("data:image/") {
        return None;
    }
    let comma_idx = uri.find(',')?;
    let metadata = &uri[..comma_idx];
    if !metadata.contains(";base64") {
        return None;
    }
    let payload = &uri[comma_idx + 1..];
    BASE64_STANDARD.decode(payload.trim()).ok()
}

/// Spawns an asynchronous task to load, decode, and cache an image
/// (remote URL, data URI, or local file) for Markdown or EPUB.
pub fn spawn_image_load_task(
    path_str: String,
    file_context_path: Option<String>,
    gen_arc: Arc<AtomicUsize>,
    current_gen: usize,
) -> Task<Message> {
    let path_for_msg = path_str.clone();

    if path_str.starts_with("data:image/") {
        log_debug!("Spawning async load for base64 data image");
        Task::perform(
            async move {
                if gen_arc.load(Ordering::Relaxed) != current_gen {
                    return MarkdownMsg::ImageLoaded {
                        generation_id: current_gen,
                        path: path_for_msg,
                        payload: None,
                    }
                    .into();
                }

                let payload = tokio::task::spawn_blocking(move || {
                    let bytes = parse_data_image_uri(&path_str)?;
                    crate::features::image::decode_image_bytes(bytes)
                })
                .await
                .ok()
                .flatten();

                MarkdownMsg::ImageLoaded {
                    generation_id: current_gen,
                    path: path_for_msg,
                    payload,
                }
                .into()
            },
            |msg| msg,
        )
    } else if crate::core::net::is_remote_url(&path_str) {
        log_debug!("Spawning async load for remote image: {}", path_str);
        Task::perform(
            async move {
                if gen_arc.load(Ordering::Relaxed) != current_gen {
                    return MarkdownMsg::ImageLoaded {
                        generation_id: current_gen,
                        path: path_for_msg,
                        payload: None,
                    }
                    .into();
                }

                let bytes = crate::core::net::fetch_remote_image(&path_str).await;
                let payload = if let Some(bytes) = bytes {
                    if gen_arc.load(Ordering::Relaxed) != current_gen {
                        None
                    } else {
                        tokio::task::spawn_blocking(move || {
                            crate::features::image::decode_image_bytes(bytes)
                        })
                        .await
                        .ok()
                        .flatten()
                    }
                } else {
                    None
                };

                MarkdownMsg::ImageLoaded {
                    generation_id: current_gen,
                    path: path_for_msg,
                    payload,
                }
                .into()
            },
            |msg| msg,
        )
    } else {
        log_debug!("Spawning async load for local image: {}", path_str);
        let resolved = if let Some(ref base_file) = file_context_path {
            crate::core::utils::resolve_path(&path_str, base_file)
        } else {
            std::path::PathBuf::from(&path_str)
        };

        Task::perform(
            async move {
                if gen_arc.load(Ordering::Relaxed) != current_gen {
                    return MarkdownMsg::ImageLoaded {
                        generation_id: current_gen,
                        path: path_for_msg,
                        payload: None,
                    }
                    .into();
                }

                let payload = tokio::task::spawn_blocking(move || {
                    let bytes = std::fs::read(&resolved).ok()?;
                    crate::features::image::decode_image_bytes(bytes)
                })
                .await
                .ok()
                .flatten();

                MarkdownMsg::ImageLoaded {
                    generation_id: current_gen,
                    path: path_for_msg,
                    payload,
                }
                .into()
            },
            |msg| msg,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_image_uri() {
        let sample_png = b"\x89PNG\r\n\x1a\n";
        let b64 = BASE64_STANDARD.encode(sample_png);
        let uri = format!("data:image/png;base64,{b64}");

        let decoded = parse_data_image_uri(&uri).expect("Should decode valid data uri");
        assert_eq!(decoded, sample_png);

        // Invalid scheme or missing base64
        assert!(parse_data_image_uri("https://example.com/test.png").is_none());
        assert!(parse_data_image_uri("data:image/png,notbase64").is_none());
    }
}
