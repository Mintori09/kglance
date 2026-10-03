/// Smoothly masks the 4 corners of an RGBA image buffer with anti-aliased rounded corners.
pub fn round_rgba_corners(data: &mut [u8], width: u32, height: u32, radius: f32) {
    if width == 0 || height == 0 || radius <= 0.0 {
        return;
    }
    let r = radius.min((width as f32) / 2.0).min((height as f32) / 2.0);
    let r_ceil = r.ceil() as u32;
    let r_sq = r * r;
    let stride = (width * 4) as usize;

    for cy in 0..r_ceil {
        let dy = r - (cy as f32 + 0.5);
        let dy_sq = dy * dy;

        for cx in 0..r_ceil {
            let dx = r - (cx as f32 + 0.5);
            let dist_sq = dx * dx + dy_sq;

            if dist_sq > r_sq {
                let dist = dist_sq.sqrt();
                let factor = (r + 0.5 - dist).clamp(0.0, 1.0);

                let corners = [
                    (cx, cy),
                    (width - 1 - cx, cy),
                    (cx, height - 1 - cy),
                    (width - 1 - cx, height - 1 - cy),
                ];

                for (x, y) in corners {
                    let idx = (y as usize) * stride + (x as usize) * 4;
                    if idx + 3 < data.len() {
                        let original_a = data[idx + 3] as f32;
                        data[idx + 3] = (original_a * factor).round() as u8;
                    }
                }
            }
        }
    }
}

pub fn compress_rgba_to_png(data: &[u8], width: u32, height: u32) -> Option<Vec<u8>> {
    let img = image::RgbaImage::from_raw(width, height, data.to_vec())?;
    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png).ok()?;
    Some(png_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_small_image() {
        // 2x2 red RGBA image
        let rgba = vec![
            255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255,
        ];
        let png = compress_rgba_to_png(&rgba, 2, 2);
        assert!(png.is_some());
        let png_bytes = png.unwrap();
        // PNG should start with magic bytes
        assert_eq!(&png_bytes[..4], &[0x89, 0x50, 0x4E, 0x47]);
        assert!(!png_bytes.is_empty());
    }

    #[test]
    fn compress_invalid_dimensions_returns_none() {
        let rgba = vec![0u8; 16];
        // width*height*4 = 100 != 16
        assert!(compress_rgba_to_png(&rgba, 5, 5).is_none());
    }

    #[test]
    fn test_round_rgba_corners_masks_corner_pixels() {
        let width = 10u32;
        let height = 10u32;
        let mut data = vec![255u8; (width * height * 4) as usize];
        round_rgba_corners(&mut data, width, height, 4.0);

        // Top-left corner (0, 0) should be fully transparent (alpha == 0)
        assert_eq!(data[3], 0);

        // Center pixel (5, 5) should remain untouched (alpha == 255)
        let center_idx = (5 * width + 5) as usize * 4;
        assert_eq!(data[center_idx + 3], 255);
    }
}
