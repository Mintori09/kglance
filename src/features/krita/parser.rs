use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::features::image::extract_exif_from_bytes;
use crate::features::image::types::{ImageFormat, ImageMetadata};

pub struct KritaParser;

impl PreviewParser for KritaParser {
    fn supported_extensions(&self) -> &[&str] {
        &["kra", "ora"]
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let mut archive = open_archive(path)?;
        let png_data = read_preview_image(&mut archive)?;
        let (width, height) = read_image_dimensions(&png_data)?;

        let mut exif = extract_exif_from_bytes(&png_data);

        if let Some(doc_info) = read_document_info(&mut archive) {
            match exif.as_mut() {
                Some(existing) => {
                    if existing.title.is_none() {
                        existing.title = doc_info.title;
                    }
                    if existing.author.is_none() {
                        existing.author = doc_info.author;
                    }
                    if existing.software.is_none() {
                        existing.software = doc_info.software;
                    }
                    if existing.creation_date.is_none() {
                        existing.creation_date = doc_info.creation_date;
                    }
                }
                None => {
                    exif = Some(Box::new(doc_info));
                }
            }
        }

        Ok(ParsedContent::Image {
            data: png_data,
            width,
            height,
            format: ImageFormat::Png,
            exif,
        })
    }
}

fn read_document_info<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> Option<ImageMetadata> {
    let mut entry = archive.by_name("documentinfo.xml").ok()?;
    let mut xml_str = String::new();
    entry.read_to_string(&mut xml_str).ok()?;
    parse_document_info_xml(&xml_str)
}

fn parse_document_info_xml(xml: &str) -> Option<ImageMetadata> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut data = ImageMetadata::default();
    let mut current_tag = String::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_tag = name;
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Text(ref e)) => {
                let text = match e.decode() {
                    Ok(txt) => txt.trim().to_string(),
                    Err(_) => continue,
                };
                if text.is_empty() {
                    continue;
                }

                if current_tag.ends_with("title") {
                    data.title = Some(text);
                } else if current_tag.ends_with("creator") || current_tag.ends_with("author") {
                    data.author = Some(text);
                } else if current_tag.ends_with("generator") {
                    data.software = Some(text);
                } else if current_tag.ends_with("creation-date")
                    || (current_tag.ends_with("date") && data.creation_date.is_none())
                {
                    data.creation_date = Some(text);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if data.title.is_some()
        || data.author.is_some()
        || data.software.is_some()
        || data.creation_date.is_some()
    {
        Some(data)
    } else {
        None
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

    #[test]
    fn parses_document_info_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<document-info xmlns="http://www.calligra.org/document-info">
 <about>
  <title>Artwork Title</title>
  <description>Sample description</description>
  <date>2026-09-06T12:00:00</date>
  <creation-date>2026-09-01T10:00:00</creation-date>
 </about>
 <author>
  <creator>Krita Artist</creator>
 </author>
 <meta:generator>Krita 5.2.2</meta:generator>
</document-info>"#;

        let data = parse_document_info_xml(xml).expect("should parse document info");
        assert_eq!(data.title.as_deref(), Some("Artwork Title"));
        assert_eq!(data.author.as_deref(), Some("Krita Artist"));
        assert_eq!(data.software.as_deref(), Some("Krita 5.2.2"));
        assert_eq!(data.creation_date.as_deref(), Some("2026-09-01T10:00:00"));
    }
}
