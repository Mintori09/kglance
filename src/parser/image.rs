use std::io::Read;
use std::path::Path;

use crate::parser::{ExifData, ImageFormat, ParseError, ParsedContent, PreviewParser};

pub struct ImageParser;

impl PreviewParser for ImageParser {
    fn supported_extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "webp", "gif", "bmp", "ico"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.supported_extensions().contains(&e))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let img = image::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

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

        let exif = extract_exif(path);

        Ok(ParsedContent::Image {
            data: buf.into_inner(),
            width,
            height,
            format,
            exif,
        })
    }
}

fn extract_exif(path: &Path) -> Option<Box<ExifData>> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    let exif_reader = exif::Reader::new();
    let reader = exif_reader.read_raw(buf).ok()?;

    let fmt_val = |tag: exif::Tag| {
        reader
            .get_field(tag, exif::In::PRIMARY)
            .map(|f| f.value.display_as(f.tag).to_string())
    };

    let gps_val = |tag: exif::Tag| {
        reader.get_field(tag, exif::In::PRIMARY).and_then(|f| {
            let v = f.value.display_as(f.tag).to_string();
            if v.is_empty() { None } else { Some(v) }
        })
    };

    Some(Box::new(ExifData {
        camera_make: fmt_val(exif::Tag::Make),
        camera_model: fmt_val(exif::Tag::Model),
        date_taken: fmt_val(exif::Tag::DateTimeOriginal),
        gps_lat: gps_val(exif::Tag::GPSLatitude),
        gps_lon: gps_val(exif::Tag::GPSLongitude),
        exposure: fmt_val(exif::Tag::ExposureTime),
        f_number: fmt_val(exif::Tag::FNumber),
        iso: fmt_val(exif::Tag::ISOSpeed),
        focal_length: fmt_val(exif::Tag::FocalLength),
    }))
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
                width,
                height,
                format,
                ..
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
