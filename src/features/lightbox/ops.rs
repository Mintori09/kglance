use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use iced::Task;

use super::types::{LightboxItem, LightboxState};
use crate::app::{KglanceApp, Message};

pub fn create_lightbox_image_state(bytes: &[u8]) -> crate::features::image::ImageState {
    let (display_handle, width, height) =
        if let Some((h, w, h_dim)) = crate::features::image::decode_to_rgba_handle(bytes) {
            (Some(h), w, h_dim)
        } else {
            (
                Some(iced::widget::image::Handle::from_bytes(bytes.to_vec())),
                0,
                0,
            )
        };

    crate::features::image::ImageState {
        handle: display_handle.clone(),
        preview_handle: None,
        image_bytes: bytes.to_vec(),
        exif_content: String::new(),
        exif_parsed_lines: Vec::new(),
        format_info: String::new(),
        load_state: crate::features::image::ImageLoadState::Ready,
        load_id: Arc::new(AtomicU64::new(0)),
        display_handle,
        display_width: width,
        display_height: height,
        camera: crate::features::image::Camera::new(),
        show_info: false,
        animated_gif: None,
        animation_start_time: None,
        prev_frame_handle: None,
    }
}

pub fn resolve_item_bytes(app: &KglanceApp, item: &LightboxItem) -> Option<Vec<u8>> {
    if let Some(ref bytes) = item.in_memory_bytes {
        return Some(bytes.clone());
    }

    if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Epub { .. })
    ) {
        return crate::features::epub::lightbox_ops::get_epub_image_bytes(app, &item.id);
    }

    if item.id.starts_with("data:image/") {
        return crate::features::markdown::tasks::parse_data_image_uri(&item.id);
    }

    if crate::core::net::is_remote_url(&item.id) {
        if let Some(cached_path) =
            crate::core::disk_cache::get_cached_url_path("remote_images", &item.id, "")
            && let Ok(bytes) = std::fs::read(&cached_path)
            && !bytes.is_empty()
        {
            return Some(bytes);
        }
        return None;
    }

    let resolved = if !app.state.file_name.is_empty() {
        crate::core::utils::resolve_path(&item.id, &app.state.file_name)
    } else {
        std::path::PathBuf::from(&item.id)
    };
    std::fs::read(&resolved).ok()
}

pub fn handle_open_lightbox(
    app: &mut KglanceApp,
    items: Vec<LightboxItem>,
    initial_index: usize,
) -> Task<Message> {
    if items.is_empty() {
        return Task::none();
    }
    let idx = initial_index.min(items.len().saturating_sub(1));
    let Some(bytes) = resolve_item_bytes(app, &items[idx]) else {
        return Task::none();
    };

    let image_state = create_lightbox_image_state(&bytes);
    app.state.lightbox = Some(LightboxState {
        items,
        current_index: idx,
        image_state,
    });
    Task::none()
}

pub fn handle_close_lightbox(app: &mut KglanceApp) -> Task<Message> {
    app.state.lightbox = None;
    Task::none()
}

pub fn handle_next_lightbox(app: &mut KglanceApp) -> Task<Message> {
    let Some(lb) = &app.state.lightbox else {
        return Task::none();
    };
    if lb.items.is_empty() {
        return Task::none();
    }
    let next_idx = (lb.current_index + 1) % lb.items.len();
    let bytes = resolve_item_bytes(app, &lb.items[next_idx]);

    if let Some(bytes) = bytes {
        let new_image_state = create_lightbox_image_state(&bytes);
        if let Some(lb) = &mut app.state.lightbox {
            lb.current_index = next_idx;
            lb.image_state = new_image_state;
        }
    }
    Task::none()
}

pub fn handle_prev_lightbox(app: &mut KglanceApp) -> Task<Message> {
    let Some(lb) = &app.state.lightbox else {
        return Task::none();
    };
    if lb.items.is_empty() {
        return Task::none();
    }
    let prev_idx = if lb.current_index == 0 {
        lb.items.len().saturating_sub(1)
    } else {
        lb.current_index - 1
    };
    let bytes = resolve_item_bytes(app, &lb.items[prev_idx]);

    if let Some(bytes) = bytes {
        let new_image_state = create_lightbox_image_state(&bytes);
        if let Some(lb) = &mut app.state.lightbox {
            lb.current_index = prev_idx;
            lb.image_state = new_image_state;
        }
    }
    Task::none()
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
