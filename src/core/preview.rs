use crate::core::types::{EpubChapterInfo, FolderRowState, KglanceState, SheetInfo};
use crate::features::common::parser::traits::ParseError;
use crate::features::json::JsonNode;
use crate::features::pdf::PdfTocEntry;
use crate::features::pdf::types::PageDimensions;
use crate::parsers::markdown::Block;
use std::collections::HashMap;
use std::path::Path;

#[doc(hidden)]
pub use crate::features::markdown::compute_block_y_offsets;

pub fn compute_pdf_page_offsets(
    dims: &[PageDimensions],
    display_width: f32,
    spacing: f32,
) -> (Vec<f32>, Vec<f32>, f32) {
    let (offsets, ends, _, total_h) =
        crate::features::pdf::geometry::compute_pdf_page_offsets(dims, display_width, spacing);
    (offsets, ends, total_h)
}

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
        blocks: Vec<Block>,
        raw_text: String,
    },
    Pdf {
        page_count: usize,
        current_page: usize,
        data: Vec<u8>,
        width: u32,
        height: u32,
        outline: Vec<PdfTocEntry>,
        page_dimensions: Vec<PageDimensions>,
    },
    Typst {
        page_count: usize,
        current_page: usize,
        data: Vec<u8>,
        width: u32,
        height: u32,
        source: String,
        error: Option<String>,
        outline: Vec<PdfTocEntry>,
        page_dimensions: Vec<PageDimensions>,
    },
    Media {
        url: String,
        metadata: String,
        thumbnail_or_waveform: Vec<u8>,
        width: u32,
        height: u32,
    },
    Folder {
        rows: Vec<FolderRowState>,
        total_size: u64,
    },
    Spreadsheet {
        sheets: Vec<SheetInfo>,
        active_sheet: usize,
    },
    Json {
        nodes: Vec<JsonNode>,
        content: String,
        pretty: String,
        has_parse_error: bool,
    },
    Epub {
        title: String,
        author: String,
        chapters: Vec<EpubChapterInfo>,
        active_chapter: usize,
        images: HashMap<String, Vec<u8>>,
    },
    Font {
        name: String,
        metadata: String,
        sample: Vec<u8>,
        sample_width: u32,
        sample_height: u32,
    },
    Error(String),
}

pub trait FilePreviewer {
    fn parse(&self, path: &Path) -> Result<PreviewData, ParseError>;
}

fn update_file_metadata(state: &mut KglanceState) {
    if !state.file_name.is_empty() {
        let path = Path::new(&state.file_name);
        if let Ok(meta) = std::fs::metadata(path) {
            state.file_size_text = crate::parsers::human_size(meta.len());
            if let Ok(modified) = meta.modified() {
                state.file_modified_text = crate::parsers::human_time(modified);
            }
        }
    }
}

impl PreviewData {
    pub fn populate_state(&self, state: &mut KglanceState) {
        update_file_metadata(state);

        match self {
            PreviewData::Text {
                content,
                line_numbers,
                language,
            } => {
                crate::features::text::populate_state(
                    state,
                    content.clone(),
                    line_numbers.clone(),
                    language,
                );
            }
            PreviewData::Image {
                data,
                format_info,
                exif_content,
                ..
            } => {
                crate::features::image::populate_image_state(
                    state,
                    data,
                    format_info,
                    exif_content.as_deref(),
                );
            }
            PreviewData::Pdf {
                page_count,
                outline,
                page_dimensions,
                ..
            } => {
                crate::features::pdf::populate_state(
                    state,
                    *page_count,
                    outline.clone(),
                    page_dimensions.clone(),
                );
            }
            PreviewData::Typst {
                page_count,
                source,
                error,
                outline,
                page_dimensions,
                ..
            } => {
                crate::features::typst::populate_state(
                    state,
                    *page_count,
                    source,
                    error.clone(),
                    outline,
                    page_dimensions,
                );
            }
            PreviewData::Folder { rows, total_size } => {
                state.folder.rows = rows.clone();
                state.folder.total_size = *total_size;
                state.folder.folder_path = state.file_name.clone();
                state.folder.selected_index = None;
                state.file_type_text = "Folder / Archive".to_string();
                state.file_size_text.clear();
            }
            PreviewData::Markdown { blocks, .. } => {
                crate::features::markdown::populate_state(state, blocks);
            }
            PreviewData::Spreadsheet {
                sheets,
                active_sheet,
            } => {
                state.spreadsheet.sheets = sheets.clone();
                state.spreadsheet.active_sheet = *active_sheet;
                state.file_type_text = "Spreadsheet".to_string();
            }
            PreviewData::Epub {
                title,
                author,
                chapters,
                active_chapter,
                images,
            } => {
                crate::features::epub::populate_state(
                    state,
                    title,
                    author,
                    chapters,
                    *active_chapter,
                    images,
                );
            }
            PreviewData::Json {
                nodes,
                pretty,
                has_parse_error,
                ..
            } => {
                crate::features::json::populate_state(state, nodes, pretty, *has_parse_error);
            }
            PreviewData::Font {
                name,
                metadata,
                sample,
                sample_width,
                sample_height,
            } => {
                crate::features::font::populate_state(
                    state,
                    name,
                    metadata,
                    sample,
                    *sample_width,
                    *sample_height,
                );
            }
            PreviewData::Media { metadata, .. } => {
                state.media = crate::core::MediaState::default();
                state.media.metadata = metadata.clone();
                state.file_type_text = if metadata.contains("Video") {
                    "Video File"
                } else {
                    "Audio File"
                }
                .to_string();
            }
            PreviewData::Error(err) => {
                state.file_type_text = format!("Error: {}", err);
            }
        }
    }

    pub fn initial_window_size(&self, state: &KglanceState) -> iced::Size {
        match self {
            PreviewData::Image { width, height, .. } => {
                let max_w = if state.window_width > 0.0 {
                    state.window_width
                } else {
                    state.window_default_size.width
                };
                let max_h = if state.window_height > 0.0 {
                    state.window_height
                } else {
                    state.window_default_size.height
                };
                crate::features::image::view::calculate_window_size(max_w, max_h, *width, *height)
            }
            PreviewData::Media { .. } => iced::Size::new(
                850.0f32.min(state.window_default_size.width),
                550.0f32.min(state.window_default_size.height),
            ),
            _ => state.window_default_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_state_metadata() {
        let temp_dir = std::env::temp_dir().join("kglance-meta-test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("sample.txt");
        let test_content = b"Hello, metadata test!";
        std::fs::write(&test_file, test_content).unwrap();

        let mut state = KglanceState {
            file_name: test_file.to_string_lossy().to_string(),
            ..Default::default()
        };

        let preview_data = PreviewData::Text {
            content: "Hello, metadata test!".to_string(),
            line_numbers: "1".to_string(),
            language: "Plain Text".to_string(),
        };

        preview_data.populate_state(&mut state);

        assert_eq!(state.file_type_text, "Plain Text");
        assert!(
            !state.file_size_text.is_empty(),
            "file_size_text should be populated"
        );
        assert!(
            state.file_size_text.contains("B"),
            "file_size_text should display bytes"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_font_preview_populate_state() {
        let temp_dir = std::env::temp_dir().join("kglance-font-populate-test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("font.ttf");
        std::fs::write(&test_file, b"dummy").unwrap();

        let mut state = KglanceState {
            file_name: test_file.to_string_lossy().to_string(),
            ..Default::default()
        };

        let sample = vec![128u8; 60 * 30 * 4];
        let preview_data = PreviewData::Font {
            name: "TestFont".to_string(),
            metadata: "Name: TestFont".to_string(),
            sample: sample.clone(),
            sample_width: 60,
            sample_height: 30,
        };

        preview_data.populate_state(&mut state);

        assert_eq!(state.file_type_text, "Font");
        assert!(state.image.format_info.contains("TestFont"));
        assert_eq!(state.image.exif_content, "Name: TestFont");
        assert!(state.image.handle.is_some());
        assert_eq!(state.image.image_bytes, sample);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
