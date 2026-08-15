use crate::features::{
    archive::types::ArchiveEntry,
    folder::types::DirEntry,
    image::types::{ExifData, ImageFormat, ImageRef},
    json::parser::types::JsonNode,
    markdown::parser::Block,
    office::types::SheetData,
    pdf::{parser::PdfTocEntry, types::PageData},
};

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
        exif: Option<Box<ExifData>>,
    },
    Pdf {
        page_count: u32,
        first_page: PageData,
        outline: Vec<PdfTocEntry>,
    },
    Typst {
        source: String,
        page_count: u32,
        first_page: PageData,
        error: Option<String>,
        outline: Vec<PdfTocEntry>,
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
        blocks: Vec<Block>,
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
        nodes: Vec<JsonNode>,
        has_parse_error: bool,
    },
    Epub {
        title: String,
        author: String,
        chapters: Vec<(String, u8, Option<String>, Vec<Block>)>,
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
