use std::fs;
use std::io::{BufReader, Read};
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
        let mut archive = open_archive(path)?;
        let png_data = read_preview_image(&mut archive)?;
        let (width, height) = read_image_dimensions(&png_data)?;

        Ok(ParsedContent::Image {
            data: png_data,
            width,
            height,
            format: ImageFormat::Png,
            exif: None,
        })
    }
}

fn open_archive(path: &Path) -> Result<zip::ZipArchive<BufReader<std::fs::File>>, ParseError> {
    let file = fs::File::open(path).map_err(|error| ParseError::ParseFailed(error.to_string()))?;
    zip::ZipArchive::new(BufReader::new(file))
        .map_err(|error| ParseError::ParseFailed(error.to_string()))
}

fn read_preview_image<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Result<Vec<u8>, ParseError> {
    for entry_name in ["mergedimage.png", "preview.png"] {
        match archive.by_name(entry_name) {
            Ok(mut entry) => {
                let mut image_data = Vec::new();
                entry
                    .read_to_end(&mut image_data)
                    .map_err(|error| ParseError::ParseFailed(error.to_string()))?;

                return Ok(image_data);
            }
            Err(zip::result::ZipError::FileNotFound) => continue,
            Err(error) => return Err(ParseError::ParseFailed(error.to_string())),
        }
    }

    Err(ParseError::ParseFailed(
        "No preview image found in archive".to_string(),
    ))
}

fn read_image_dimensions(image_data: &[u8]) -> Result<(u32, u32), ParseError> {
    let reader = image::ImageReader::new(std::io::Cursor::new(image_data))
        .with_guessed_format()
        .map_err(|error| ParseError::ParseFailed(error.to_string()))?;

    reader
        .into_dimensions()
        .map_err(|error| ParseError::ParseFailed(error.to_string()))
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
