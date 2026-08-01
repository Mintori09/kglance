pub mod archive;
pub mod csv;
pub mod helpers;
pub use archive::{ExtractedFile, extract_entry};

pub mod audio;
pub mod epub;
pub mod folder;
pub mod font;
pub mod image;
pub mod json;
pub mod markdown;
pub mod office;
pub mod pdf;
pub mod svg;
pub mod text;
pub mod typst;
pub mod video;

use crate::parsers::helpers::file_limit::preview_size_limit;
use crate::parsers::helpers::icon::icon_for_entry;
use crate::{log_debug, log_error, log_info};
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
    UnsupportedFormat,
    FileNotFound,
    PermissionDenied,
    TooLarge,
    ParseFailed(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat => write!(f, "unsupported file format"),
            Self::FileNotFound => write!(f, "file not found"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::TooLarge => write!(f, "file too large"),
            Self::ParseFailed(msg) => write!(f, "parse failed: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub(crate) fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

pub fn human_time(datetime: std::time::SystemTime) -> String {
    let now = chrono::Local::now();
    let dt = chrono::DateTime::<chrono::Local>::from(datetime);
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 0 {
        return dt.format("%b %d").to_string();
    }

    if duration.num_minutes() < 1 {
        return "Just now".to_string();
    } else if duration.num_hours() < 1 {
        return format!("{}m ago", duration.num_minutes());
    } else if duration.num_days() < 1 {
        return format!("{}h ago", duration.num_hours());
    } else if duration.num_days() == 1 {
        return "Yesterday".to_string();
    } else if duration.num_days() < 7 {
        return format!("{}d ago", duration.num_days());
    }

    dt.format("%b %d, %Y").to_string()
}

pub fn format_timestamp(secs: u64) -> String {
    let dur = std::time::Duration::from_secs(secs);
    let sys_time = std::time::UNIX_EPOCH + dur;
    let dt: chrono::DateTime<chrono::Local> = sys_time.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub trait PreviewParser: Send + Sync {
    fn supported_extensions(&self) -> &[&str];
    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .map(|ext| self.supported_extensions().contains(&ext.as_str()))
            .unwrap_or(false)
    }
    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError>;
}

#[derive(Debug, Clone)]
pub struct SheetData {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub raw_modified: i64,
}

#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified: String,
    pub archive_path: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageRef {
    pub alt_text: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
}

#[derive(Debug, Clone)]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub date_taken: Option<String>,
    pub gps_lat: Option<String>,
    pub gps_lon: Option<String>,
    pub exposure: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ParsedContent {
    Text {
        content: String,
        language: String,
        line_count: usize,
        highlighted_html: Option<String>,
    },
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
        format: ImageFormat,
        exif: Option<Box<ExifData>>,
    },
    Pdf {
        page_count: u32,
        first_page: PageData,
    },
    Typst {
        source: String,
        page_count: u32,
        first_page: PageData,
        error: Option<String>,
    },
    Archive {
        entries: Vec<ArchiveEntry>,
        total_files: usize,
    },
    Folder {
        entries: Vec<DirEntry>,
    },
    Markdown {
        content: String,
        images: Vec<ImageRef>,
        blocks: Vec<markdown::Block>,
    },
    Video {
        path: String,
        duration: f64,
        thumbnail: Vec<u8>,
    },
    Audio {
        path: String,
        metadata: String,
        waveform: Vec<u8>,
        waveform_width: u32,
        waveform_height: u32,
    },
    Office {
        content: String,
        format: String,
        page_count: usize,
    },
    Json {
        content: String,
        pretty: String,
        nodes: Vec<json::JsonNode>,
        has_parse_error: bool,
    },
    Epub {
        title: String,
        author: String,
        chapters: Vec<(String, u8, Option<String>, Vec<markdown::Block>)>,
        images: std::collections::HashMap<String, Vec<u8>>,
    },
    Spreadsheet {
        sheets: Vec<SheetData>,
    },
    Font {
        name: String,
        metadata: String,
        sample: Vec<u8>,
        sample_width: u32,
        sample_height: u32,
    },
}

pub struct ParserRegistry {
    parsers: Vec<Box<dyn PreviewParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register(&mut self, parser: Box<dyn PreviewParser>) {
        self.parsers.push(parser);
    }

    pub fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let path_str = path.to_string_lossy();
        log_info!("ParserRegistry: Parsing path: {}", path_str);
        if !path.exists() {
            log_error!("ParserRegistry: File not found: {}", path_str);
            return Err(ParseError::FileNotFound);
        }
        if path.is_dir() {
            log_info!("ParserRegistry: Path is a directory: {}", path_str);
            for parser in &self.parsers {
                if parser.is_supported(path) {
                    log_info!("ParserRegistry: Delegating to folder/directory parser");
                    return parser.parse(path);
                }
            }
            log_error!(
                "ParserRegistry: No directory parser found for: {}",
                path_str
            );
            return Err(ParseError::UnsupportedFormat);
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let metadata = path.metadata().map_err(|e| {
            log_error!(
                "ParserRegistry: Failed to read metadata for {}: {}",
                path_str,
                e
            );
            ParseError::PermissionDenied
        })?;
        let limit = preview_size_limit(&ext);
        if metadata.len() > limit {
            log_error!(
                "ParserRegistry: File too large. Size: {}, Limit: {} for extension: {}",
                human_size(metadata.len()),
                human_size(limit),
                ext
            );
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown file");
            let _ = std::process::Command::new("notify-send")
                .args([
                    "-u",
                    "normal",
                    "-i",
                    "dialog-warning",
                    "Kglance Preview",
                    &format!(
                        "File \"{}\" is too large to preview.\nSize: {} (Limit: {})",
                        file_name,
                        human_size(metadata.len()),
                        human_size(limit)
                    ),
                ])
                .status();
            return Err(ParseError::TooLarge);
        }

        log_debug!("ParserRegistry: Matching by extension: .{}", ext);
        for parser in &self.parsers {
            if parser.supported_extensions().contains(&ext.as_str()) {
                log_info!(
                    "ParserRegistry: Found matching parser by extension for: .{}",
                    ext
                );
                let start = std::time::Instant::now();
                let res = parser.parse(path);
                log_info!("ParserRegistry: Parsing completed in {:?}", start.elapsed());
                return res;
            }
        }

        log_debug!("ParserRegistry: Attempting fallback matching by content check");
        for parser in &self.parsers {
            if parser.is_supported(path) {
                log_info!("ParserRegistry: Found fallback parser by is_supported check");
                let start = std::time::Instant::now();
                let res = parser.parse(path);
                log_info!("ParserRegistry: Parsing completed in {:?}", start.elapsed());
                return res;
            }
        }

        log_debug!("ParserRegistry: Falling back to plain text read");
        if let Ok(content) = std::fs::read_to_string(path) {
            let line_count = content.lines().count();
            log_info!(
                "ParserRegistry: Read file as plain text ({} lines)",
                line_count
            );
            return Ok(ParsedContent::Text {
                content,
                language: "Plain Text".into(),
                line_count,
                highlighted_html: None,
            });
        }

        log_error!(
            "ParserRegistry: Unsupported format and cannot be read as plain text: {}",
            path_str
        );
        Err(ParseError::UnsupportedFormat)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::core::preview::FilePreviewer for ParserRegistry {
    fn parse(&self, path: &Path) -> Result<crate::core::preview::PreviewData, ParseError> {
        let content = self.parse(path)?;
        let preview = match content {
            ParsedContent::Text {
                content,
                language,
                line_count,
                ..
            } => {
                let line_numbers = (1..=line_count)
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                crate::core::preview::PreviewData::Text {
                    content,
                    line_numbers,
                    language,
                }
            }
            ParsedContent::Office {
                content,
                format,
                page_count,
            } => crate::core::preview::PreviewData::Text {
                content,
                line_numbers: String::new(),
                language: format!("Office ({}, {} pages)", format, page_count),
            },
            ParsedContent::Json {
                content,
                pretty,
                nodes,
                has_parse_error,
            } => crate::core::preview::PreviewData::Json {
                nodes,
                content,
                pretty,
                has_parse_error,
            },
            ParsedContent::Epub {
                title,
                author,
                chapters,
                images,
            } => {
                let epub_chapters = chapters
                    .into_iter()
                    .map(|(t, lvl, anc, b)| crate::core::types::EpubChapterInfo {
                        title: t,
                        level: lvl,
                        anchor: anc,
                        blocks: b,
                    })
                    .collect();
                crate::core::preview::PreviewData::Epub {
                    title,
                    author,
                    chapters: epub_chapters,
                    active_chapter: 0,
                    images,
                }
            }
            ParsedContent::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            } => crate::core::preview::PreviewData::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            },
            ParsedContent::Image {
                data,
                width,
                height,
                format,
                exif,
            } => {
                let format_info = format!("Image ({:?} - {}x{})", format, width, height);
                let exif_content = exif.map(|exif_data| {
                    format!(
                        "Camera Make: {}\nCamera Model: {}\nDate Taken: {}\nGPS Lat: {}\nGPS Lon: {}\nExposure: {}\nF-Number: {}\nISO: {}\nFocal Length: {}",
                        exif_data.camera_make.as_deref().unwrap_or("N/A"),
                        exif_data.camera_model.as_deref().unwrap_or("N/A"),
                        exif_data.date_taken.as_deref().unwrap_or("N/A"),
                        exif_data.gps_lat.as_deref().unwrap_or("N/A"),
                        exif_data.gps_lon.as_deref().unwrap_or("N/A"),
                        exif_data.exposure.as_deref().unwrap_or("N/A"),
                        exif_data.f_number.as_deref().unwrap_or("N/A"),
                        exif_data.iso.as_deref().unwrap_or("N/A"),
                        exif_data.focal_length.as_deref().unwrap_or("N/A")
                    )
                });
                crate::core::preview::PreviewData::Image {
                    data,
                    width,
                    height,
                    format_info,
                    exif_content,
                }
            }
            ParsedContent::Pdf {
                page_count,
                first_page,
            } => crate::core::preview::PreviewData::Pdf {
                page_count: page_count as usize,
                current_page: 0,
                data: first_page.data,
                width: first_page.width,
                height: first_page.height,
            },
            ParsedContent::Typst {
                source,
                page_count,
                first_page,
                error,
            } => crate::core::preview::PreviewData::Typst {
                page_count: page_count as usize,
                current_page: 0,
                data: first_page.data,
                width: first_page.width,
                height: first_page.height,
                source,
                error,
            },
            ParsedContent::Spreadsheet { sheets } => {
                crate::core::preview::PreviewData::Spreadsheet {
                    sheets: sheets
                        .into_iter()
                        .map(|s| crate::core::types::SheetInfo {
                            name: s.name,
                            headers: s.headers,
                            rows: s.rows,
                        })
                        .collect(),
                    active_sheet: 0,
                }
            }
            ParsedContent::Archive { entries, .. } => {
                let total_size = entries.iter().map(|e| e.size).sum();
                let rows = entries
                    .into_iter()
                    .map(|entry| {
                        let icon = icon_for_entry(&entry.path, entry.is_dir);
                        crate::core::FolderRowState {
                            name: entry.path.clone(),
                            kind: if entry.is_dir {
                                "Directory".to_string()
                            } else {
                                "File".to_string()
                            },
                            size: crate::parsers::human_size(entry.size),
                            raw_size: entry.size,
                            modified: entry.modified.clone(),
                            raw_modified: 0,
                            path: entry.path,
                            is_dir: entry.is_dir,
                            icon,
                        }
                    })
                    .collect();
                crate::core::preview::PreviewData::Folder { rows, total_size }
            }
            ParsedContent::Folder { entries } => {
                let total_size = entries.iter().map(|e| e.size).sum();
                let rows = entries
                    .into_iter()
                    .map(|entry| {
                        let icon = icon_for_entry(&entry.name, entry.is_dir);
                        crate::core::FolderRowState {
                            name: entry.name.clone(),
                            kind: if entry.is_dir {
                                "Directory".to_string()
                            } else {
                                "File".to_string()
                            },
                            size: crate::parsers::human_size(entry.size),
                            raw_size: entry.size,
                            modified: entry.modified.clone(),
                            raw_modified: entry.raw_modified,
                            path: entry.name,
                            is_dir: entry.is_dir,
                            icon,
                        }
                    })
                    .collect();
                crate::core::preview::PreviewData::Folder { rows, total_size }
            }
            ParsedContent::Markdown {
                blocks, content, ..
            } => crate::core::preview::PreviewData::Markdown {
                blocks,
                raw_text: content,
            },
            ParsedContent::Video {
                path,
                duration,
                thumbnail,
            } => crate::core::preview::PreviewData::Media {
                url: path,
                metadata: format!("Video Duration: {:.2}s", duration),
                thumbnail_or_waveform: thumbnail,
                width: 320,
                height: 240,
            },
            ParsedContent::Audio {
                path,
                metadata,
                waveform,
                waveform_width,
                waveform_height,
            } => crate::core::preview::PreviewData::Media {
                url: path,
                metadata,
                thumbnail_or_waveform: waveform,
                width: waveform_width,
                height: waveform_height,
            },
        };
        Ok(preview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockParser;

    impl PreviewParser for MockParser {
        fn supported_extensions(&self) -> &[&str] {
            &["mock"]
        }
        fn is_supported(&self, _path: &Path) -> bool {
            true
        }
        fn parse(&self, _path: &Path) -> Result<ParsedContent, ParseError> {
            Ok(ParsedContent::Text {
                content: "mock".into(),
                language: "plaintext".into(),
                line_count: 1,
                highlighted_html: None,
            })
        }
    }

    #[test]
    fn registry_matches_by_extension() {
        let mut registry = ParserRegistry::new();
        registry.register(Box::new(MockParser));

        let dir = std::env::temp_dir().join("kglance-test-mock");
        let _ = std::fs::create_dir_all(&dir);
        let file_path = dir.join("test.mock");
        assert!(std::fs::write(&file_path, b"content").is_ok());

        let result = registry.parse(&file_path);
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn registry_returns_error_for_missing_file() {
        let registry = ParserRegistry::new();
        let result = registry.parse(Path::new("/nonexistent/file.xyz"));
        assert!(matches!(result, Err(ParseError::FileNotFound)));
    }

    #[test]
    fn test_preview_parser_default_is_supported() {
        struct TestParser;
        impl PreviewParser for TestParser {
            fn supported_extensions(&self) -> &[&str] {
                &["txt", "doc"]
            }
            fn parse(&self, _path: &Path) -> Result<ParsedContent, ParseError> {
                Err(ParseError::UnsupportedFormat)
            }
        }

        let parser = TestParser;
        assert!(parser.is_supported(Path::new("file.txt")));
        assert!(parser.is_supported(Path::new("FILE.TXT")));
        assert!(parser.is_supported(Path::new("document.doc")));
        assert!(!parser.is_supported(Path::new("file.pdf")));
        assert!(!parser.is_supported(Path::new("no_extension")));
    }

    #[test]
    fn test_format_timestamp_valid() {
        let formatted = format_timestamp(1700000000);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_font_conversion_to_preview_data() {
        let content = ParsedContent::Font {
            name: "TestFont".to_string(),
            metadata: "Name: TestFont\nGlyphs: 2".to_string(),
            sample: vec![0u8; 100 * 50 * 4],
            sample_width: 100,
            sample_height: 50,
        };

        let preview = match content {
            ParsedContent::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            } => crate::core::preview::PreviewData::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            },
            _ => unreachable!(),
        };

        match preview {
            crate::core::preview::PreviewData::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            } => {
                assert_eq!(name, "TestFont");
                assert!(metadata.contains("TestFont"));
                assert!(!sample.is_empty());
                assert_eq!(sample_width, 100);
                assert_eq!(sample_height, 50);
                assert_eq!(sample.len(), (100 * 50 * 4) as usize);
            }
            other => panic!("expected PreviewData::Font, got {other:?}"),
        }
    }
}
