use crate::core::error::ParseError;
use crate::core::preview::{PreviewContent, content_to_preview_data};
use crate::core::utils::human_size;
use crate::features::text::content::TextContent;
use crate::parsers::types::PreviewParser;
use crate::{log_debug, log_error, log_info};
use std::path::Path;

pub struct ParserRegistry {
    parsers: Vec<Box<dyn PreviewParser<crate::app::Message>>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register(&mut self, parser: Box<dyn PreviewParser<crate::app::Message>>) {
        self.parsers.push(parser);
    }

    pub fn parse(
        &self,
        path: &Path,
    ) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
        let path_str = path.to_string_lossy();
        log_info!("ParserRegistry: Parsing path: {}", path_str);

        verify_path_exists(path)?;

        if path.is_dir() {
            return self.parse_directory(path);
        }

        let ext = extract_extension(path);
        enforce_size_limit(path, &ext)?;

        if let Some(result) = self.try_extension_match(path, &ext) {
            return result;
        }
        if let Some(result) = self.try_content_match(path) {
            return result;
        }
        try_fallback_text(path)
    }

    pub fn parse_to_preview_data(
        &self,
        path: &Path,
    ) -> Result<crate::core::preview::PreviewData, ParseError> {
        use crate::core::types::KglanceState;

        let content = self.parse(path)?;
        let mut state = KglanceState {
            file_name: path.to_string_lossy().to_string(),
            ..Default::default()
        };
        content.populate_state(&mut state);
        Ok(content_to_preview_data(&*content, &state))
    }

    fn parse_directory(
        &self,
        path: &Path,
    ) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
        log_info!(
            "ParserRegistry: Path is a directory: {}",
            path.to_string_lossy()
        );
        for parser in &self.parsers {
            if parser.is_supported(path) {
                log_info!("ParserRegistry: Delegating to folder/directory parser");
                return parser.parse(path);
            }
        }
        log_error!(
            "ParserRegistry: No directory parser found for: {}",
            path.to_string_lossy()
        );
        Err(ParseError::UnsupportedFormat)
    }

    fn try_extension_match(
        &self,
        path: &Path,
        ext: &str,
    ) -> Option<Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError>> {
        log_debug!("ParserRegistry: Matching by extension: .{}", ext);
        for parser in &self.parsers {
            if parser.supported_extensions().contains(&ext) {
                log_info!(
                    "ParserRegistry: Found matching parser by extension for: .{}",
                    ext
                );
                let start = std::time::Instant::now();
                let res = parser.parse(path);
                log_info!("ParserRegistry: Parsing completed in {:?}", start.elapsed());
                return Some(res);
            }
        }
        None
    }

    fn try_content_match(
        &self,
        path: &Path,
    ) -> Option<Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError>> {
        log_debug!("ParserRegistry: Attempting fallback matching by content check");
        for parser in &self.parsers {
            if parser.is_supported(path) {
                log_info!("ParserRegistry: Found fallback parser by is_supported check");
                let start = std::time::Instant::now();
                let res = parser.parse(path);
                log_info!("ParserRegistry: Parsing completed in {:?}", start.elapsed());
                return Some(res);
            }
        }
        None
    }

    pub fn all_extensions(&self, exclude_av: bool) -> Vec<String> {
        let mut exts: Vec<String> = self
            .parsers
            .iter()
            .flat_map(|p| p.supported_extensions().iter().copied())
            .map(|e| e.to_lowercase())
            .filter(|e| !exclude_av || !is_audio_video_ext(e))
            .collect();
        exts.sort();
        exts.dedup();
        exts
    }

    pub fn scan_sibling_files(&self, file_path: &str, exclude_av: bool) -> Vec<String> {
        let exts = self.all_extensions(exclude_av);
        let path = Path::new(file_path);
        let parent = match path.parent() {
            Some(p) => p,
            None => return vec![file_path.to_string()],
        };

        let mut files: Vec<String> = match std::fs::read_dir(parent) {
            Ok(entries) => entries
                .flatten()
                .filter(|e| e.path().is_file())
                .map(|e| e.path().to_string_lossy().to_string())
                .filter(|p| {
                    Path::new(p)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| {
                            exts.iter()
                                .any(|supported| e.eq_ignore_ascii_case(supported))
                        })
                        .unwrap_or(false)
                })
                .collect(),
            Err(_) => return vec![file_path.to_string()],
        };

        files.sort_by_key(|a| a.to_lowercase());
        if files.is_empty() {
            vec![file_path.to_string()]
        } else {
            files
        }
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn verify_path_exists(path: &Path) -> Result<(), ParseError> {
    if path.exists() {
        return Ok(());
    }
    log_error!("ParserRegistry: File not found: {}", path.to_string_lossy());
    Err(ParseError::FileNotFound)
}

fn extract_extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default()
}

fn enforce_size_limit(path: &Path, ext: &str) -> Result<(), ParseError> {
    let limit = size_limit(ext);
    let metadata = path.metadata().map_err(|e| {
        log_error!(
            "ParserRegistry: Failed to read metadata for {}: {}",
            path.to_string_lossy(),
            e
        );
        ParseError::PermissionDenied
    })?;

    if metadata.len() <= limit {
        return Ok(());
    }

    log_error!(
        "ParserRegistry: File too large. Size: {}, Limit: {} for extension: {}",
        human_size(metadata.len()),
        human_size(limit),
        ext
    );
    notify_too_large(path, metadata.len(), limit);
    Err(ParseError::TooLarge)
}

fn size_limit(ext: &str) -> u64 {
    match ext {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "mp3" | "wav" | "flac" | "ogg" | "aac"
        | "m4a" => 10 * 1024 * 1024 * 1024,
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => 2 * 1024 * 1024 * 1024,
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
        | "epub" => 500 * 1024 * 1024,
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "ttf" | "otf"
        | "woff" | "woff2" => 100 * 1024 * 1024,
        _ => 20 * 1024 * 1024,
    }
}

fn is_audio_video_ext(ext: &str) -> bool {
    matches!(
        ext,
        "mp3"
            | "wav"
            | "flac"
            | "ogg"
            | "aac"
            | "m4a"
            | "opus"
            | "mp4"
            | "mkv"
            | "avi"
            | "mov"
            | "wmv"
            | "webm"
            | "flv"
            | "m4v"
    )
}

fn notify_too_large(path: &Path, size: u64, limit: u64) {
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
                human_size(size),
                human_size(limit)
            ),
        ])
        .status();
}

fn try_fallback_text(
    path: &Path,
) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
    log_debug!("ParserRegistry: Falling back to plain text read");
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let line_count = content.lines().count();
            log_info!(
                "ParserRegistry: Read file as plain text ({} lines)",
                line_count
            );
            Ok(Box::new(TextContent {
                content,
                language: "Plain Text".into(),
                line_count,
                highlighted_html: None,
            }))
        }
        Err(_) => {
            log_error!(
                "ParserRegistry: Unsupported format and cannot be read as plain text: {}",
                path.to_string_lossy()
            );
            Err(ParseError::UnsupportedFormat)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::ParseError;
    use crate::parsers::types::PreviewParser;
    use std::path::Path;

    struct MockParser;

    impl PreviewParser<crate::app::Message> for MockParser {
        fn supported_extensions(&self) -> &[&str] {
            &["mock"]
        }
        fn is_supported(&self, _path: &Path) -> bool {
            true
        }
        fn parse(
            &self,
            _path: &Path,
        ) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
            Ok(Box::new(TextContent {
                content: "mock".into(),
                language: "plaintext".into(),
                line_count: 1,
                highlighted_html: None,
            }))
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
