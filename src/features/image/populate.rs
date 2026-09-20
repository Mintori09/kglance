use crate::core::types::{ImageState, KglanceState};
use crate::features::image::ImageLoadState;
use std::sync::atomic::Ordering;

pub fn populate_state(
    state: &mut KglanceState,
    data: &[u8],
    width: u32,
    height: u32,
    format_info: &str,
    exif_content: Option<&str>,
) {
    let new_load_id = state
        .image
        .load_id
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1);

    debug_assert_eq!(
        state.image.load_id.load(Ordering::Relaxed),
        new_load_id,
        "load_id should equal new_load_id after fetch_add"
    );

    let shared_load_id = state.image.load_id.clone();

    let exif_str = exif_content.unwrap_or_default();
    let exif_parsed_lines: Vec<(String, String)> = exif_str
        .lines()
        .map(|line| {
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().trim_end_matches(':');
                (key.to_string(), v.trim().to_string())
            } else {
                (line.trim().to_string(), String::new())
            }
        })
        .filter(|(k, _)| !k.is_empty())
        .collect();

    let cached = state
        .cache
        .get(&state.file_name)
        .and_then(|c| c.as_decoded_image())
        .map(|(h, w, ht)| (h.clone(), w, ht));

    let (display_handle, preview_handle, load_state) = if let Some((h, cw, ch)) = cached {
        if cw < width || ch < height {
            // Cached entry is a downsampled preview
            (None, Some(h), ImageLoadState::Ready)
        } else {
            // Cached entry is full resolution
            (Some(h), None, ImageLoadState::Ready)
        }
    } else if data.len() <= 64 * 1024 {
        // Small image (<= 64KB, e.g. tests, icons): decode synchronously in <1ms
        if let Some((h, _, _)) = crate::features::image::decode_to_rgba_handle(data) {
            (Some(h), None, ImageLoadState::Ready)
        } else {
            (
                Some(iced::widget::image::Handle::from_bytes(data.to_vec())),
                None,
                ImageLoadState::Ready,
            )
        }
    } else {
        // Large image: leave handle as None and let spawn_decode_task decode in background
        (None, None, ImageLoadState::Loading)
    };

    state.image = ImageState {
        handle: display_handle.clone().or_else(|| preview_handle.clone()),
        preview_handle,
        image_bytes: Vec::new(),
        exif_content: exif_str.to_string(),
        exif_parsed_lines,
        format_info: format_info.to_string(),
        load_state,
        load_id: shared_load_id,
        display_handle,
        display_width: width,
        display_height: height,
        camera: crate::features::image::Camera::new(),
        show_info: false,
    };

    state.file_type_text = format_info.to_string();
}
