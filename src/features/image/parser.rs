use std::io::Read;
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::image::types::{ImageFormat, ImageMetadata};

pub struct ImageParser;

impl PreviewParser for ImageParser {
    fn supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "webp", "gif", "bmp", "ico"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let mut file =
            std::fs::File::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let (width, height) =
            image::image_dimensions(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let format = match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref()
        {
            Some("png") => ImageFormat::Png,
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("webp") => ImageFormat::WebP,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            _ => ImageFormat::Png,
        };

        let exif = extract_exif(path);

        Ok(ParsedContent::Image {
            data,
            width,
            height,
            format,
            exif,
        })
    }
}

pub(crate) fn extract_exif(path: &Path) -> Option<Box<ImageMetadata>> {
    let file = std::fs::File::open(path).ok()?;
    let mut buf_reader = std::io::BufReader::new(file);
    let exif_reader = exif::Reader::new();
    if let Ok(reader) = exif_reader.read_from_container(&mut buf_reader) {
        return Some(Box::new(exif_from_reader(&reader)));
    }

    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    extract_exif_from_bytes(&buf)
}

pub(crate) fn extract_exif_from_bytes(bytes: &[u8]) -> Option<Box<ImageMetadata>> {
    let exif_reader = exif::Reader::new();
    let mut cursor = std::io::Cursor::new(bytes);
    if let Ok(reader) = exif_reader.read_from_container(&mut cursor) {
        return Some(Box::new(exif_from_reader(&reader)));
    }
    if let Ok(reader) = exif_reader.read_raw(bytes.to_vec()) {
        return Some(Box::new(exif_from_reader(&reader)));
    }
    None
}

fn exif_from_reader(reader: &exif::Exif) -> ImageMetadata {
    let fmt_val = |tag: exif::Tag| {
        reader.get_field(tag, exif::In::PRIMARY).map(|f| {
            let s = f.value.display_as(f.tag).to_string();
            s.trim_matches('"').trim().to_string()
        })
    };

    let gps_val = |tag: exif::Tag| {
        reader.get_field(tag, exif::In::PRIMARY).and_then(|f| {
            let v = f.value.display_as(f.tag).to_string();
            let trimmed = v.trim_matches('"').trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    };

    ImageMetadata {
        camera_make: fmt_val(exif::Tag::Make),
        camera_model: fmt_val(exif::Tag::Model),
        date_taken: fmt_val(exif::Tag::DateTimeOriginal),
        gps_lat: gps_val(exif::Tag::GPSLatitude),
        gps_lon: gps_val(exif::Tag::GPSLongitude),
        exposure: fmt_val(exif::Tag::ExposureTime),
        f_number: fmt_val(exif::Tag::FNumber),
        iso: fmt_val(exif::Tag::ISOSpeed),
        focal_length: fmt_val(exif::Tag::FocalLength),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_png() {
        let img = image::DynamicImage::new_rgba8(2, 2);
        let mut tmp = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        img.write_to(&mut tmp, image::ImageFormat::Png).unwrap();
        let parser = ImageParser;
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Image {
                data,
                width,
                height,
                format,
                ..
            } => {
                assert_eq!(width, 2);
                assert_eq!(height, 2);
                assert!(matches!(format, ImageFormat::Png));

                // Assert that the generated data can be decoded back to an image correctly
                let decoded = image::load_from_memory(&data);
                assert!(
                    decoded.is_ok(),
                    "Parsed image data could not be decoded: {:?}",
                    decoded.err()
                );
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
