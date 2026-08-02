use iced::Size;

use crate::core::ImageState;

use super::constants::{
    DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, HEADER_HEIGHT, MAX_WINDOW_HEIGHT,
    MAX_WINDOW_WIDTH, MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH,
};

pub fn is_loaded(state: &ImageState) -> bool {
    state.handle.is_some()
}

pub fn calculate_window_size(img_width: u32, img_height: u32) -> Size {
    if img_width == 0 || img_height == 0 {
        return Size::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);
    }

    let img_w = img_width as f32;
    let img_h = img_height as f32;

    let scale = (MAX_WINDOW_WIDTH / img_w)
        .min(MAX_WINDOW_HEIGHT / img_h)
        .min(1.0);

    let content_width = (img_w * scale).clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH);
    let content_height = (img_h * scale).clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);

    Size::new(content_width, content_height + HEADER_HEIGHT)
}
