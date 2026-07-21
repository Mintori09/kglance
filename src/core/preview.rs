use crate::core::types::KglanceState;
use crate::parsers::ParseError;
use crate::parsers::markdown::Block;
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
        total_size: u64,
    },
    Spreadsheet {
        sheets: Vec<crate::core::types::SheetInfo>,
        active_sheet: usize,
    },
    Error(String),
}

pub trait FilePreviewer {
    fn parse(&self, path: &Path) -> Result<PreviewData, ParseError>;
}

impl PreviewData {
    pub fn populate_state(&self, state: &mut KglanceState) {
        match self {
            PreviewData::Text {
                content,
                line_numbers,
                language,
            } => {
                state.text.content = iced::widget::text_editor::Content::with_text(content);
                state.text.extension = language.clone();
                state.text.line_numbers.clone_from(line_numbers);
                state.file_type_text = language.clone();
            }
            PreviewData::Image {
                data,
                width,
                height,
                format_info,
                exif_content,
            } => {
                state.image = crate::core::ImageState {
                    handle: Some(iced::widget::image::Handle::from_bytes(data.clone())),
                    image_bytes: data.clone(),
                    image_width: *width,
                    image_height: *height,
                    exif_content: exif_content.clone().unwrap_or_default(),
                    ..Default::default()
                };
                state.file_type_text.clone_from(format_info);
            }
            PreviewData::Pdf { page_count, .. } => {
                state.pdf = crate::core::PdfState::default();
                state.pdf.page_count = *page_count;
                state.pdf.pages = vec![None; *page_count];
                state.pdf.cached_handles = vec![None; *page_count];
                state.file_type_text = "PDF Document".to_string();
            }
            PreviewData::Folder { rows, total_size } => {
                state.table.rows = rows.clone();
                state.table.total_size = *total_size;
                state.table.folder_path = state.file_name.clone();
                state.table.selected_index = None;
                state.file_type_text = "Folder / Archive".to_string();
            }
            PreviewData::Markdown { blocks } => {
                state.markdown = crate::core::types::MarkdownState::default();
                for (i, block) in blocks.iter().enumerate() {
                    if let Block::Mermaid {
                        rendered: Some(png),
                        ..
                    } = block
                    {
                        let handle = match image::load_from_memory(png) {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let (w, h) = rgba.dimensions();
                                iced::widget::image::Handle::from_rgba(w, h, rgba.into_raw())
                            }
                            Err(_) => iced::widget::image::Handle::from_bytes(png.clone()),
                        };
                        state.markdown.cached_mermaid_handles.insert(i, handle);
                    }
                }
                state.file_type_text = "Markdown Document".to_string();
            }
            PreviewData::Spreadsheet {
                sheets,
                active_sheet,
            } => {
                state.spreadsheet.sheets = sheets.clone();
                state.spreadsheet.active_sheet = *active_sheet;
                state.file_type_text = "Spreadsheet".to_string();
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
}
