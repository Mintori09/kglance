use crate::core::limit::preview_size_limit;
use crate::core::utils::human_size;
use crate::features::common::parser::traits::ParseError;
use crate::features::common::parser::traits::{ParserRegistry, PreviewParser};
use crate::features::common::parser::types::ParsedContent;
use crate::ui::theme::icon_theme::icon_for_entry;
use crate::{log_debug, log_error, log_info};
use std::path::Path;

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
                outline,
            } => crate::core::preview::PreviewData::Pdf {
                page_count: page_count as usize,
                current_page: 0,
                data: first_page.data,
                width: first_page.width,
                height: first_page.height,
                outline,
            },
            ParsedContent::Typst {
                source,
                page_count,
                first_page,
                error,
                outline,
            } => crate::core::preview::PreviewData::Typst {
                page_count: page_count as usize,
                current_page: 0,
                data: first_page.data,
                width: first_page.width,
                height: first_page.height,
                source,
                error,
                outline,
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
                            size: human_size(entry.size),
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
                            size: human_size(entry.size),
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
    use crate::core::utils::format_timestamp;

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
