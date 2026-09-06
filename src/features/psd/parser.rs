use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::image::types::ImageFormat;

pub struct PsdParser;

impl PreviewParser for PsdParser {
    fn supported_extensions(&self) -> &[&str] {
        &["psd", "psb"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let bytes =
            std::fs::read(path).map_err(|error| ParseError::ParseFailed(error.to_string()))?;
        let psd = psd::Psd::from_bytes(&bytes)
            .map_err(|error| ParseError::ParseFailed(format!("{error:?}")))?;

        let width = psd.width();
        let height = psd.height();
        let rgba = psd.flatten_layers_rgba(&|(_, _)| true)
            .map_err(|error| ParseError::ParseFailed(format!("{error:?}")))?;

        let mut png_data = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        image::ImageEncoder::write_image(
            encoder,
            &rgba,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|error| ParseError::ParseFailed(error.to_string()))?;

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
    fn supports_psd_and_psb_extensions() {
        let parser = PsdParser;
        assert!(parser.is_supported(Path::new("design.psd")));
        assert!(parser.is_supported(Path::new("large.psb")));
        assert!(!parser.is_supported(Path::new("image.png")));
    }
}
