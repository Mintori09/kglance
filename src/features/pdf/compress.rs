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
}
