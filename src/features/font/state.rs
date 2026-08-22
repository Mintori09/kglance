use crate::core::types::{ImageState, KglanceState};
use crate::features::image::ImageLoadState;

pub fn populate_state(
    state: &mut KglanceState,
    name: &str,
    metadata: &str,
    sample: &[u8],
    sample_width: u32,
    sample_height: u32,
) {
    state.image = ImageState {
        handle: Some(iced::widget::image::Handle::from_rgba(
            sample_width,
            sample_height,
            sample.to_vec(),
        )),
        image_bytes: sample.to_vec(),
        width: sample_width,
        height: sample_height,
        format_info: format!("Font — {name}"),
        exif_content: metadata.to_string(),
        load_state: ImageLoadState::Ready,
        ..Default::default()
    };
    state.file_type_text = "Font".to_string();
}
