use iced::Size;

/// Calculates the target window size to fit the image.
/// The long side of the image will fit within the maximum allowed boundary (e.g. 1000px)
/// without maximizing the window (not "open full"), and the short side will be scaled proportionally.
pub fn calculate_window_size(img_width: u32, img_height: u32) -> Size {
    let img_width = img_width as f32;
    let img_height = img_height as f32;

    if img_width == 0.0 || img_height == 0.0 {
        return Size::new(1024.0, 768.0);
    }

    let aspect_ratio = img_width / img_height;
    let max_long_side = 1000.0;
    let min_width = 400.0;
    let min_height = 300.0;

    let (mut w, mut h) = if img_width >= img_height {
        // Landscape or square: fit width to max_long_side
        let target_w = img_width.min(max_long_side);
        let target_h = target_w / aspect_ratio;
        (target_w, target_h)
    } else {
        // Portrait: fit height to max_long_side
        let target_h = img_height.min(max_long_side);
        let target_w = target_h * aspect_ratio;
        (target_w, target_h)
    };

    // Add padding for window decoration / header (around 50px)
    let header_height = 50.0;

    // Constrain to minimum dimensions while preserving aspect ratio
    if w < min_width {
        w = min_width;
        h = w / aspect_ratio;
    }
    if h < min_height {
        h = min_height;
        w = h * aspect_ratio;
    }

    Size::new(w, h + header_height)
}
