#![expect(dead_code)]

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
    fn name(&self) -> &'static str;
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
    Svg,
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
    Svg {
        data: String,
    },
    Pdf {
        page_count: u32,
        pages: Vec<Vec<u8>>,
    },
    Archive {
        entries: Vec<ArchiveEntry>,
        total_files: usize,
    },
    Folder {
        entries: Vec<DirEntry>,
    },
}
