use std::path::Path;

use crate::core::MmapFile;
use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::image::types::ImageFormat;

pub struct PsdParser;

impl PreviewParser for PsdParser {
    fn supported_extensions(&self) -> &[&str] {
        &["psd"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        // 1. Check persistent disk cache first (near-instant subsequent opens)
        if let Some(cached_path) = crate::core::disk_cache::get_cached_path("psd", path, "png")
            && let Ok(cached_png) = std::fs::read(&cached_path)
            && !cached_png.is_empty()
        {
            let (w, h) = image::ImageReader::new(std::io::Cursor::new(&cached_png))
                .with_guessed_format()
                .ok()
                .and_then(|r| r.into_dimensions().ok())
                .unwrap_or((0, 0));

            if w > 0 && h > 0 {
                return Ok(ParsedContent::Image {
                    data: cached_png,
                    width: w,
                    height: h,
                    format: ImageFormat::Png,
                    exif: None,
                });
            }
        }

        let mmap = MmapFile::open(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        let psd = psd::Psd::from_bytes(mmap.as_bytes())
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let width = psd.width();
        let height = psd.height();

        if width == 0 || height == 0 {
            return Err(ParseError::ParseFailed(
                "Invalid PSD dimensions (0x0)".to_string(),
            ));
        }

        // 2. Try composite image first (pre-rendered by Photoshop with all blend modes and effects)
        let composite = psd.rgba();
        let rgba = if !composite.is_empty() {
            composite
        } else if !psd.layers().is_empty() {
            // 3. Fallback: Flatten visible layers
            match psd.flatten_layers_rgba(&|(_, layer)| layer.visible()) {
                Ok(data) if !data.is_empty() => data,
                _ => psd
                    .flatten_layers_rgba(&|(_, _)| true)
                    .map_err(|e| ParseError::ParseFailed(e.to_string()))?,
            }
        } else {
            psd.flatten_layers_rgba(&|(_, _)| true)
                .map_err(|e| ParseError::ParseFailed(e.to_string()))?
        };

        if rgba.is_empty() {
            return Err(ParseError::ParseFailed("Empty PSD pixel data".to_string()));
        }

        let mut png_data = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        image::ImageEncoder::write_image(
            encoder,
            &rgba,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        // 4. Save to persistent disk cache
        let _ = crate::core::disk_cache::put_bytes("psd", path, "png", &png_data);

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
    fn supports_psd_extension() {
        let parser = PsdParser;
        assert!(parser.is_supported(Path::new("design.psd")));
        assert!(parser.is_supported(Path::new("artwork.PSD")));
        assert!(!parser.is_supported(Path::new("large.psb")));
        assert!(!parser.is_supported(Path::new("image.png")));
    }

    #[test]
    fn test_parse_minimal_psd() {
        let mut psd_bytes = Vec::new();
        // 1. Header (26 bytes)
        psd_bytes.extend_from_slice(b"8BPS"); // Signature
        psd_bytes.extend_from_slice(&1u16.to_be_bytes()); // Version 1
        psd_bytes.extend_from_slice(&[0u8; 6]); // Reserved
        psd_bytes.extend_from_slice(&3u16.to_be_bytes()); // 3 channels (RGB)
        psd_bytes.extend_from_slice(&1u32.to_be_bytes()); // Height: 1
        psd_bytes.extend_from_slice(&1u32.to_be_bytes()); // Width: 1
        psd_bytes.extend_from_slice(&8u16.to_be_bytes()); // Depth: 8 bits
        psd_bytes.extend_from_slice(&3u16.to_be_bytes()); // Mode: RGB (3)

        // 2. Color Mode Data (4 bytes length = 0)
        psd_bytes.extend_from_slice(&0u32.to_be_bytes());

        // 3. Image Resources (4 bytes length = 0)
        psd_bytes.extend_from_slice(&0u32.to_be_bytes());

        // 4. Layer and Mask Information (4 bytes length = 0)
        psd_bytes.extend_from_slice(&0u32.to_be_bytes());

        // 5. Image Data
        psd_bytes.extend_from_slice(&0u16.to_be_bytes()); // Compression: 0 (Raw)
        psd_bytes.extend_from_slice(&[255, 128, 64]); // R=255, G=128, B=64

        let mut tmp = tempfile::Builder::new().suffix(".psd").tempfile().unwrap();
        use std::io::Write;
        tmp.write_all(&psd_bytes).unwrap();

        let parser = PsdParser;
        let result = parser.parse(tmp.path()).expect("parse minimal psd");
        match result {
            ParsedContent::Image {
                data,
                width,
                height,
                format,
                exif,
            } => {
                assert_eq!(width, 1);
                assert_eq!(height, 1);
                assert!(matches!(format, ImageFormat::Png));
                assert!(exif.is_none());
                assert!(!data.is_empty());

                let decoded = image::load_from_memory(&data).expect("decode generated png");
                assert_eq!(decoded.width(), 1);
                assert_eq!(decoded.height(), 1);
            }
            _ => panic!("Expected ParsedContent::Image variant"),
        }
    }

    #[test]
    fn test_parse_invalid_psd() {
        let mut tmp = tempfile::Builder::new().suffix(".psd").tempfile().unwrap();
        use std::io::Write;
        tmp.write_all(b"not a valid psd file").unwrap();

        let parser = PsdParser;
        let result = parser.parse(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_psd_disk_cache() {
        let mut psd_bytes = Vec::new();
        psd_bytes.extend_from_slice(b"8BPS");
        psd_bytes.extend_from_slice(&1u16.to_be_bytes());
        psd_bytes.extend_from_slice(&[0u8; 6]);
        psd_bytes.extend_from_slice(&3u16.to_be_bytes());
        psd_bytes.extend_from_slice(&2u32.to_be_bytes()); // Height: 2
        psd_bytes.extend_from_slice(&2u32.to_be_bytes()); // Width: 2
        psd_bytes.extend_from_slice(&8u16.to_be_bytes());
        psd_bytes.extend_from_slice(&3u16.to_be_bytes());
        psd_bytes.extend_from_slice(&0u32.to_be_bytes());
        psd_bytes.extend_from_slice(&0u32.to_be_bytes());
        psd_bytes.extend_from_slice(&0u32.to_be_bytes());
        psd_bytes.extend_from_slice(&0u16.to_be_bytes());
        psd_bytes.extend_from_slice(&[200; 12]); // 2x2 RGB

        let mut tmp = tempfile::Builder::new().suffix(".psd").tempfile().unwrap();
        use std::io::Write;
        tmp.write_all(&psd_bytes).unwrap();

        let parser = PsdParser;
        // First parse: encodes and saves to disk cache
        let result1 = parser.parse(tmp.path()).expect("first parse");
        // Second parse: hits disk cache
        let result2 = parser.parse(tmp.path()).expect("cached parse");

        match (result1, result2) {
            (
                ParsedContent::Image {
                    width: w1,
                    height: h1,
                    data: d1,
                    ..
                },
                ParsedContent::Image {
                    width: w2,
                    height: h2,
                    data: d2,
                    ..
                },
            ) => {
                assert_eq!(w1, w2);
                assert_eq!(h1, h2);
                assert_eq!(d1, d2);
                assert_eq!(w1, 2);
                assert_eq!(h1, 2);
            }
            _ => panic!("Expected Image variant"),
        }
    }
}
