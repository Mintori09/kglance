use std::path::Path;

use crate::parser::{ImageFormat, ParseError, ParsedContent, PreviewParser};

pub struct ImageParser;

impl PreviewParser for ImageParser {
    fn name(&self) -> &'static str {
        "image"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "webp", "gif", "bmp", "ico"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.supported_extensions().contains(&e))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let img =
            image::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let (width, height) = (img.width(), img.height());
        let format = match path.extension().and_then(|e| e.to_str()) {
            Some("png") => ImageFormat::Png,
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("webp") => ImageFormat::WebP,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            _ => ImageFormat::Png,
        };

        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        Ok(ParsedContent::Image {
            data: buf.into_inner(),
            width,
            height,
            format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_png() {
        let img = image::DynamicImage::new_rgba8(2, 2);
        let mut tmp = tempfile::Builder::new()
            .suffix(".png")
            .tempfile()
            .unwrap();
        img.write_to(&mut tmp, image::ImageFormat::Png).unwrap();
        let parser = ImageParser;
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Image {
                width, height, format, ..
            } => {
                assert_eq!(width, 2);
                assert_eq!(height, 2);
                assert!(matches!(format, ImageFormat::Png));
            }
            _ => panic!("expected Image variant"),
        }
    }

    #[test]
    fn supports_common_image_extensions() {
        let parser = ImageParser;
        assert!(parser.is_supported(Path::new("photo.png")));
        assert!(parser.is_supported(Path::new("photo.jpg")));
        assert!(parser.is_supported(Path::new("photo.jpeg")));
        assert!(parser.is_supported(Path::new("photo.webp")));
        assert!(parser.is_supported(Path::new("photo.gif")));
        assert!(parser.is_supported(Path::new("photo.bmp")));
        assert!(!parser.is_supported(Path::new("file.txt")));
    }
}
