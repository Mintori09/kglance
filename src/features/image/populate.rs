use crate::core::types::{ImageState, KglanceState};
use crate::features::image::ImageLoadState;

pub fn populate_state(
    state: &mut KglanceState,
    data: &[u8],
    width: u32,
    height: u32,
    format_info: &str,
    exif_content: Option<&str>,
) {
    state.image = ImageState {
        handle: Some(iced::widget::image::Handle::from_bytes(data.to_vec())),
        image_bytes: data.to_vec(),
        width,
        height,
        exif_content: exif_content.unwrap_or_default().to_string(),
        format_info: format_info.to_string(),
        load_state: ImageLoadState::Ready,
        ..Default::default()
    };
    state.file_type_text = format_info.to_string();
}
