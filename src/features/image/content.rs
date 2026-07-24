use crate::app::Message;
use crate::core::preview::{ContentType, PreviewContent};
use crate::core::types::{ImageState, KglanceState};
use crate::features::image::ImageLoadState;
use crate::features::image::types::{ExifData, ImageFormat};
use iced::Element;

pub struct ImageContent {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub exif: Option<Box<ExifData>>,
}

impl PreviewContent<Message> for ImageContent {
    fn populate_state(&self, state: &mut KglanceState) {
        let exif_content = self.exif.as_ref().map(|exif| {
            format!(
                "Camera Make: {}\nCamera Model: {}\nDate Taken: {}\nGPS Lat: {}\nGPS Lon: {}\nExposure: {}\nF-Number: {}\nISO: {}\nFocal Length: {}",
                exif.camera_make.as_deref().unwrap_or("N/A"),
                exif.camera_model.as_deref().unwrap_or("N/A"),
                exif.date_taken.as_deref().unwrap_or("N/A"),
                exif.gps_lat.as_deref().unwrap_or("N/A"),
                exif.gps_lon.as_deref().unwrap_or("N/A"),
                exif.exposure.as_deref().unwrap_or("N/A"),
                exif.f_number.as_deref().unwrap_or("N/A"),
                exif.iso.as_deref().unwrap_or("N/A"),
                exif.focal_length.as_deref().unwrap_or("N/A"),
            )
        }).unwrap_or_default();

        state.image = ImageState {
            handle: Some(iced::widget::image::Handle::from_bytes(self.data.clone())),
            image_bytes: self.data.clone(),
            width: self.width,
            height: self.height,
            exif_content,
            format_info: format!("{:?} - {}x{}", self.format, self.width, self.height),
            load_state: ImageLoadState::Ready,
            ..Default::default()
        };
        state.file_type_text = format!("{:?}", self.format);
    }

    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message> {
        crate::features::image::view_image(&state.image)
    }

    fn content_type(&self) -> ContentType {
        ContentType::Image
    }

    fn supports_zoom(&self) -> bool {
        true
    }
}
