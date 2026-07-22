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

    let max_w = 900.0;
    let max_h = 700.0;
    let min_w = 400.0;
    let min_h = 300.0;

    // Calculate scale to fit within max bounds
    let scale_w = max_w / img_width;
    let scale_h = max_h / img_height;
    let scale = scale_w.min(scale_h).min(1.0);

    let mut w = img_width * scale;
    let mut h = img_height * scale;

    // Apply minimum constraints
    if w < min_w {
        w = min_w;
    }
    if h < min_h {
        h = min_h;
    }

    // Double check that we don't exceed max bounds after applying min bounds
    if w > max_w {
        w = max_w;
    }
    if h > max_h {
        h = max_h;
    }

    let header_height = 50.0;
    Size::new(w, h + header_height)
}
