use crate::core::types::{ImageState, KglanceState};
use crate::features::image::ImageLoadState;
use std::sync::atomic::Ordering;

pub fn populate_state(
    state: &mut KglanceState,
    data: &[u8],
    format_info: &str,
    exif_content: Option<&str>,
) {
    let bytes = data.to_vec();

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

    let previous_handle = state.image.handle.clone();
    let shared_load_id = state.image.load_id.clone();

    let prev_display_handle = state.image.display_handle.clone();
    let prev_display_width = state.image.display_width;
    let prev_display_height = state.image.display_height;
    let prev_camera = state.image.camera;

    state.image = ImageState {
        handle: previous_handle,
        preview_handle: None,
        image_bytes: bytes,
        exif_content: exif_content.unwrap_or_default().to_string(),
        format_info: format_info.to_string(),
        load_state: ImageLoadState::Loading,
        load_id: shared_load_id,
        display_handle: prev_display_handle,
        display_width: prev_display_width,
        display_height: prev_display_height,
        camera: prev_camera,
        show_info: false,
    };

    state.file_type_text = format_info.to_string();
}
