use std::io::Read;
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::image::types::ImageFormat;

pub struct KritaParser;

impl PreviewParser for KritaParser {
    fn supported_extensions(&self) -> &[&str] {
        &["kra", "ora"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let file =
            std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let mut entry = archive
            .by_name("mergedimage.png")
            .map_err(|_| ParseError::ParseFailed("no mergedimage.png in archive".into()))?;

        let mut png_data = Vec::new();
        entry
            .read_to_end(&mut png_data)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let cursor = std::io::Cursor::new(&png_data);
        let reader = image::ImageReader::new(cursor)
            .with_guessed_format()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let (width, height) = reader
            .into_dimensions()
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        Ok(ParsedContent::Image {
            data: png_data,
            width,
            height,
            format: ImageFormat::Png,
            exif: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_kra_and_ora_extensions() {
        let parser = KritaParser;
        assert!(parser.is_supported(Path::new("drawing.kra")));
        assert!(parser.is_supported(Path::new("painting.ora")));
        assert!(!parser.is_supported(Path::new("image.png")));
        assert!(!parser.is_supported(Path::new("file.xcf")));
    }
}
