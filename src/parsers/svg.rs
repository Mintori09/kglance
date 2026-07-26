use std::path::Path;

use crate::parsers::{ImageFormat, ParseError, ParsedContent, PreviewParser};

pub struct SvgParser;

impl PreviewParser for SvgParser {
    fn supported_extensions(&self) -> &[&str] {
        &["svg"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let svg_data =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let opt = resvg::usvg::Options::default();
        let rtree = resvg::usvg::Tree::from_str(&svg_data, &opt)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let pixmap_size = rtree.size().to_int_size();
        let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| ParseError::ParseFailed("invalid SVG dimensions".into()))?;

        resvg::render(
            &rtree,
            resvg::usvg::Transform::default(),
            &mut pixmap.as_mut(),
        );

        let png_data = pixmap
            .encode_png()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        Ok(ParsedContent::Image {
            data: png_data,
            width: pixmap_size.width(),
            height: pixmap_size.height(),
            format: ImageFormat::Png,
            exif: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_svg() {
        let svg_content = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="red"/></svg>"#;
        let mut tmp = tempfile::Builder::new().suffix(".svg").tempfile().unwrap();
        use std::io::Write;
        write!(tmp, "{svg_content}").unwrap();
        let parser = SvgParser;
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Image {
                width,
                height,
                format,
                ..
            } => {
                assert_eq!(width, 16);
                assert_eq!(height, 16);
                assert!(matches!(format, ImageFormat::Png));
            }
            _ => panic!("expected Image variant"),
        }
    }

    #[test]
    fn supports_svg_extension() {
        let parser = SvgParser;
        assert!(parser.is_supported(Path::new("image.svg")));
        assert!(!parser.is_supported(Path::new("image.png")));
        assert!(!parser.is_supported(Path::new("file.txt")));
    }

    #[test]
    fn returns_error_for_invalid_svg() {
        let mut tmp = tempfile::Builder::new().suffix(".svg").tempfile().unwrap();
        use std::io::Write;
        write!(tmp, "not valid svg content").unwrap();
        let parser = SvgParser;
        let result = parser.parse(tmp.path());
        assert!(result.is_err());
    }
}
