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
        if !state.file_name.is_empty() {
            let path = Path::new(&state.file_name);
            if let Ok(meta) = std::fs::metadata(path) {
                state.file_size_text = crate::parsers::human_size(meta.len());
            }
        }

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
                    width: *width,
                    height: *height,
                    exif_content: exif_content.clone().unwrap_or_default(),
                    format_info: format_info.clone(),
                    load_state: crate::preview::image::ImageLoadState::Ready,
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
}
