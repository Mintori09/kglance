use std::{fmt, path::Path};

use crate::features::common::parser::types::ParsedContent;

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
    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .map(|ext| self.supported_extensions().contains(&ext.as_str()))
            .unwrap_or(false)
    }
    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError>;
}

pub struct ParserRegistry {
    pub parsers: Vec<Box<dyn PreviewParser>>,
}
