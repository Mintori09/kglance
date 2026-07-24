use crate::core::types::KglanceState;
use crate::features::document::folder_content::FolderContent;
use crate::features::document::spreadsheet_content::SpreadsheetContent;
use crate::features::document::types::DirEntry;
use crate::features::image::content::ImageContent;
use crate::features::image::types::{ExifData, ImageFormat};
use crate::features::markdown::content::MarkdownContent;
use crate::features::pdf::content::PdfContent;
use crate::features::text::content::TextContent;
use crate::features::video::content::MediaContent;
use iced::{Element, Task};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Image,
    Markdown,
    Pdf,
    Folder,
    Spreadsheet,
    Video,
    Audio,
    Font,
    Archive,
    Error,
}

pub trait PreviewContent<Message>: Send + Sync + 'static {
    fn populate_state(&self, state: &mut KglanceState);
    fn view<'a>(&'a self, state: &'a KglanceState) -> Element<'a, Message>;
    fn content_type(&self) -> ContentType;
    fn is_media(&self) -> bool {
        false
    }
    fn needs_media_player(&self) -> bool {
        false
    }
    fn supports_text_operations(&self) -> bool {
        false
    }
    fn supports_zoom(&self) -> bool {
        false
    }
    fn supports_toc(&self) -> bool {
        false
    }
    fn is_folder_view(&self) -> bool {
        false
    }
    fn on_loaded(&self, _state: &KglanceState, _path: &str) -> Task<Message> {
        Task::none()
    }
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
        blocks: Vec<crate::features::markdown::Block>,
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

pub fn preview_data_to_content(data: PreviewData) -> Box<dyn PreviewContent<crate::app::Message>> {
    match data {
        PreviewData::Text {
            content, language, ..
        } => {
            let line_count = content.lines().count();
            Box::new(TextContent {
                content,
                language,
                line_count,
                highlighted_html: None,
            }) as Box<dyn PreviewContent<crate::app::Message>>
        }
        PreviewData::Image {
            data,
            width,
            height,
            format_info,
            exif_content,
        } => {
            let format = if format_info.contains("Png") {
                ImageFormat::Png
            } else if format_info.contains("Jpeg") {
                ImageFormat::Jpeg
            } else if format_info.contains("WebP") {
                ImageFormat::WebP
            } else if format_info.contains("Gif") {
                ImageFormat::Gif
            } else if format_info.contains("Bmp") {
                ImageFormat::Bmp
            } else {
                ImageFormat::Png
            };
            let exif = exif_content.map(|_| {
                Box::new(ExifData {
                    camera_make: None,
                    camera_model: None,
                    date_taken: None,
                    gps_lat: None,
                    gps_lon: None,
                    exposure: None,
                    f_number: None,
                    iso: None,
                    focal_length: None,
                })
            });
            Box::new(ImageContent {
                data,
                width,
                height,
                format,
                exif,
            }) as Box<dyn PreviewContent<crate::app::Message>>
        }
        PreviewData::Markdown { blocks } => Box::new(MarkdownContent {
            content: String::new(),
            images: Vec::new(),
            blocks,
        })
            as Box<dyn PreviewContent<crate::app::Message>>,
        PreviewData::Pdf {
            page_count,
            data,
            width,
            height,
            ..
        } => Box::new(PdfContent {
            page_count: page_count as u32,
            first_page: crate::features::pdf::types::PageData {
                data,
                width,
                height,
            },
        }) as Box<dyn PreviewContent<crate::app::Message>>,
        PreviewData::Media {
            url,
            metadata,
            thumbnail_or_waveform,
            width,
            height,
        } => {
            let is_video = metadata.contains("Video");
            Box::new(MediaContent {
                path: url,
                duration: 0.0,
                thumbnail: if is_video {
                    thumbnail_or_waveform.clone()
                } else {
                    Vec::new()
                },
                metadata,
                waveform: if !is_video {
                    thumbnail_or_waveform
                } else {
                    Vec::new()
                },
                waveform_width: width,
                waveform_height: height,
                is_video,
            }) as Box<dyn PreviewContent<crate::app::Message>>
        }
        PreviewData::Folder { rows, .. } => {
            let entries: Vec<DirEntry> = rows
                .into_iter()
                .map(|r| DirEntry {
                    name: r.name,
                    is_dir: r.is_dir,
                    size: r.raw_size,
                    modified: r.modified,
                    raw_modified: r.raw_modified,
                })
                .collect();
            Box::new(FolderContent { entries }) as Box<dyn PreviewContent<crate::app::Message>>
        }
        PreviewData::Spreadsheet { sheets, .. } => {
            let sheets_data = sheets
                .into_iter()
                .map(|s| crate::features::document::types::SheetData {
                    name: s.name,
                    headers: s.headers,
                    rows: s.rows,
                })
                .collect();
            Box::new(SpreadsheetContent {
                sheets: sheets_data,
            }) as Box<dyn PreviewContent<crate::app::Message>>
        }
        PreviewData::Error(_) => Box::new(TextContent {
            content: String::new(),
            language: "Error".into(),
            line_count: 0,
            highlighted_html: None,
        }) as Box<dyn PreviewContent<crate::app::Message>>,
    }
}

pub fn content_to_preview_data(
    content: &dyn PreviewContent<crate::app::Message>,
    state: &KglanceState,
) -> PreviewData {
    match content.content_type() {
        ContentType::Text => PreviewData::Text {
            content: state.text.content.text().to_string(),
            line_numbers: state.text.line_numbers.clone(),
            language: state.text.extension.clone(),
        },
        ContentType::Image => PreviewData::Image {
            data: state.image.image_bytes.clone(),
            width: state.image.width,
            height: state.image.height,
            format_info: state.image.format_info.clone(),
            exif_content: if state.image.exif_content.is_empty() {
                None
            } else {
                Some(state.image.exif_content.clone())
            },
        },
        ContentType::Markdown => {
            PreviewData::Markdown {
                blocks: Vec::new(), // Populated during parse, stored in content struct
            }
        }
        ContentType::Pdf => {
            let first = state.pdf.pages.first().and_then(|p| p.as_ref());
            PreviewData::Pdf {
                page_count: state.pdf.page_count,
                current_page: state.pdf.current_page,
                data: first.map(|p| p.data.clone()).unwrap_or_default(),
                width: first.map(|p| p.width).unwrap_or(0),
                height: first.map(|p| p.height).unwrap_or(0),
            }
        }
        ContentType::Folder | ContentType::Archive => PreviewData::Folder {
            rows: state.table.rows.clone(),
            total_size: state.table.total_size,
        },
        ContentType::Spreadsheet => PreviewData::Spreadsheet {
            sheets: state.spreadsheet.sheets.clone(),
            active_sheet: state.spreadsheet.active_sheet,
        },
        ContentType::Video | ContentType::Audio => PreviewData::Media {
            url: state.file_name.clone(),
            metadata: state.media.metadata.clone(),
            thumbnail_or_waveform: Vec::new(),
            width: 0,
            height: 0,
        },
        ContentType::Font => PreviewData::Text {
            content: state.text.content.text().to_string(),
            line_numbers: state.text.line_numbers.clone(),
            language: state.text.extension.clone(),
        },
        ContentType::Error => PreviewData::Error("Preview error".into()),
    }
}
