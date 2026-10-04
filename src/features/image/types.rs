use crate::features::image::{Camera, ImageLoadState};
use iced::widget::image;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageRef {
    pub alt_text: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
}

#[derive(Debug, Clone, Default)]
pub struct ImageMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub software: Option<String>,
    pub creation_date: Option<String>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_taken: Option<String>,
    pub gps_lat: Option<String>,
    pub gps_lon: Option<String>,
    pub exposure: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageState {
    pub exif_content: String,
    pub exif_parsed_lines: Vec<(String, String)>,
    pub image_bytes: Vec<u8>,
    pub handle: Option<image::Handle>,
    pub preview_handle: Option<image::Handle>,
    pub format_info: String,
    pub camera: Camera,
    pub load_state: ImageLoadState,
    pub load_id: Arc<AtomicU64>,
    pub display_handle: Option<image::Handle>,
    pub display_width: u32,
    pub display_height: u32,
    pub show_info: bool,
}

impl Default for ImageState {
    fn default() -> Self {
        Self {
            exif_content: String::new(),
            exif_parsed_lines: Vec::new(),
            image_bytes: Vec::new(),
            handle: None,
            preview_handle: None,
            format_info: String::new(),
            camera: Camera::new(),
            load_state: ImageLoadState::Loading,
            load_id: Arc::new(AtomicU64::new(0)),
            display_handle: None,
            display_width: 0,
            display_height: 0,
            show_info: false,
        }
    }
}
