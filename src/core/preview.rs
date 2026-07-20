use crate::parsers::ParseError;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum PreviewData {
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
        format_info: String,
        exif_content: Option<String>,
    },
    Text {
        content: String,
        line_numbers: String,
        language: String,
    },
    Markdown {
        blocks: Vec<crate::parsers::markdown::Block>,
    },
    Pdf {
        page_count: usize,
        current_page: usize,
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Media {
        url: String,
        metadata: String,
        thumbnail_or_waveform: Vec<u8>,
        width: u32,
        height: u32,
    },
    Folder {
        rows: Vec<crate::core::types::TableRowState>,
    },
    Error(String),
}

pub trait FilePreviewer {
    fn parse(&self, path: &Path) -> Result<PreviewData, ParseError>;
}
