

pub mod archive;
pub mod image;
pub mod pdf;
pub mod svg;
pub mod text;
pub mod folder;

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

pub trait PreviewParser: Send + Sync {
    fn supported_extensions(&self) -> &[&str];
    fn is_supported(&self, path: &Path) -> bool;
    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError>;
}

#[derive(Debug)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
}

#[derive(Debug)]
pub struct ArchiveEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Debug)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
}

#[derive(Debug)]
pub enum ParsedContent {
    Text {
        content: String,
        language: String,
        line_count: usize,
    },
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
        format: ImageFormat,
    },
    Pdf {
        page_count: u32,
    },
    Archive {
        entries: Vec<ArchiveEntry>,
        total_files: usize,
    },
    Folder {
        entries: Vec<DirEntry>,
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
        if !path.exists() {
            return Err(ParseError::FileNotFound);
        }
        if path.is_dir() {
            for parser in &self.parsers {
                if parser.is_supported(path) {
                    return parser.parse(path);
                }
            }
            return Err(ParseError::UnsupportedFormat);
        }
        let metadata = path.metadata().map_err(|_| ParseError::PermissionDenied)?;
        if metadata.len() > 100 * 1024 * 1024 {
            return Err(ParseError::TooLarge);
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        for parser in &self.parsers {
            if parser.supported_extensions().contains(&ext.as_str()) {
                return parser.parse(path);
            }
        }

        for parser in &self.parsers {
            if parser.is_supported(path) {
                return parser.parse(path);
            }
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let line_count = content.lines().count();
            return Ok(ParsedContent::Text {
                content,
                language: "Plain Text".into(),
                line_count,
            });
        }

        Err(ParseError::UnsupportedFormat)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
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
