pub mod archive;
pub use archive::{ExtractedFile, extract_entry};

pub mod audio;
pub mod folder;
pub mod font;
pub mod image;
pub mod markdown;
pub mod office;
pub mod pdf;
pub mod svg;
pub mod text;
pub mod video;

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

pub trait PreviewParser: Send + Sync {
    fn supported_extensions(&self) -> &[&str];
    fn is_supported(&self, path: &Path) -> bool;
    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError>;
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
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
        let limit = match ext.as_str() {
            // Video & Audio: 10 GB limit
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "mp3" | "wav" | "flac" | "ogg"
            | "aac" | "m4a" => 10 * 1024 * 1024 * 1024,
            // Archives: 2 GB
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => 2 * 1024 * 1024 * 1024,
            // PDF/Office: 500 MB
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" => {
                500 * 1024 * 1024
            }
            // Images & Fonts: 100 MB
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "ttf" | "otf"
            | "woff" | "woff2" => 100 * 1024 * 1024,
            // Default (Text/Code fallback): 20 MB
            _ => 20 * 1024 * 1024,
        };

        if metadata.len() > limit {
            log_error!(
                "ParserRegistry: File too large. Size: {}, Limit: {} for extension: {}",
                human_size(metadata.len()),
                human_size(limit),
                ext
            );
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
            ParsedContent::Font { name, metadata, .. } => crate::core::preview::PreviewData::Text {
                content: format!("Font: {}\n\n{}", name, metadata),
                line_numbers: String::new(),
                language: "Font File".to_string(),
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
            ParsedContent::Archive { entries, .. } => {
                let rows = entries
                    .into_iter()
                    .map(|entry| crate::core::TableRowState {
                        name: entry.path.clone(),
                        kind: if entry.is_dir {
                            "Directory".to_string()
                        } else {
                            "File".to_string()
                        },
                        size: crate::parsers::human_size(entry.size),
                        modified: entry.modified.clone(),
                        path: entry.path,
                        is_dir: entry.is_dir,
                    })
                    .collect();
                crate::core::preview::PreviewData::Folder { rows }
            }
            ParsedContent::Folder { entries } => {
                let rows = entries
                    .into_iter()
                    .map(|entry| crate::core::TableRowState {
                        name: entry.name.clone(),
                        kind: if entry.is_dir {
                            "Directory".to_string()
                        } else {
                            "File".to_string()
                        },
                        size: crate::parsers::human_size(entry.size),
                        modified: entry.modified.clone(),
                        path: entry.name,
                        is_dir: entry.is_dir,
                    })
                    .collect();
                crate::core::preview::PreviewData::Folder { rows }
            }
            ParsedContent::Markdown { blocks, .. } => {
                crate::core::preview::PreviewData::Markdown { blocks }
            }
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
                metadata,
                waveform,
                waveform_width,
                waveform_height,
            } => crate::core::preview::PreviewData::Media {
                url: String::new(),
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
}
