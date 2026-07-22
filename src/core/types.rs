#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Kind,
    Modified,
    Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortState {
    pub field: SortField,
    pub ascending: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            ascending: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableRowState {
    pub name: String,
    pub kind: String,
    pub size: String,
    pub modified: String,
    pub path: String,
    pub is_dir: bool,
    pub icon: &'static str,
}
use crate::preview::image::{Camera, ImageLoadState};
use iced::widget::image;

#[derive(Debug, Clone)]
pub struct ImageState {
    pub exif_content: String,
    pub image_bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub handle: Option<image::Handle>,
    pub format_info: String,
    pub camera: Camera,
    pub load_state: ImageLoadState,
}

impl Default for ImageState {
    fn default() -> Self {
        Self {
            exif_content: String::new(),
            image_bytes: Vec::new(),
            width: 0,
            height: 0,
            handle: None,
            format_info: String::new(),
            camera: Camera::new(),
            load_state: ImageLoadState::Loading,
        }
    }
}

#[derive(Debug)]
pub struct TextState {
    pub content: iced::widget::text_editor::Content,
    pub extension: String,
    pub line_numbers: String,
    pub wrap: bool,
    pub search_visible: bool,
    pub search_query: String,
    pub search_matches: Vec<(usize, usize)>,
    pub search_match_index: usize,
    pub search_info: String,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            content: iced::widget::text_editor::Content::new(),
            extension: String::new(),
            line_numbers: String::new(),
            wrap: true,
            search_visible: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_index: 0,
            search_info: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PdfState {
    pub pages: Vec<Option<(Vec<u8>, u32, u32)>>,
    pub cached_handles: Vec<Option<iced::widget::image::Handle>>,
    pub page_count: usize,
    pub zoom: f32,
    pub loading: bool,
    pub current_page: usize,
    pub visible_page: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl Default for PdfState {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            cached_handles: Vec::new(),
            page_count: 0,
            zoom: 1.0,
            loading: false,
            current_page: 0,
            visible_page: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableState {
    pub rows: Vec<TableRowState>,
    pub sort_state: SortState,
    pub selected_index: Option<usize>,
    pub total_size: u64,
    pub folder_path: String,
}

#[derive(Debug, Clone)]
pub struct SheetInfo {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct SpreadsheetState {
    pub sheets: Vec<SheetInfo>,
    pub active_sheet: usize,
    pub sort_col: Option<usize>,
    pub sort_ascending: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct MediaState {
    pub playing: bool,
    pub time: String,
    pub progress: f32,
    pub metadata: String,
    pub has_video: bool,
    pub show_controls: bool,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub frame_data: Vec<u8>,
    pub frame_width: u32,
    pub frame_height: u32,
    pub video_handle: Option<image::Handle>,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryState {
    pub history: Vec<String>,
    pub current_index: isize,
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownState {
    pub cached_mermaid_handles: std::collections::HashMap<usize, iced::widget::image::Handle>,
    pub cached_image_handles: std::collections::HashMap<usize, iced::widget::image::Handle>,
    pub cached_image_sizes: std::collections::HashMap<usize, (u32, u32)>,
}

#[derive(Debug, Clone, Default)]
pub struct DirState {
    pub files: Vec<String>,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ToastInfo {
    pub id: u64,
    pub message: String,
}

#[derive(Debug)]
pub struct KglanceState {
    pub file_name: String,
    pub title_text: String,
    pub status_text: String,
    pub file_size_text: String,
    pub file_modified_text: String,
    pub file_type_text: String,
    pub show_file_info: bool,
    pub content_ready: bool,
    pub show_back_button: bool,
    pub back_target: Option<String>,

    pub image: ImageState,
    pub text: TextState,
    pub pdf: PdfState,
    pub table: TableState,
    pub spreadsheet: SpreadsheetState,
    pub media: MediaState,
    pub history: HistoryState,
    pub dir: DirState,
    pub markdown: MarkdownState,

    pub font_size: f32,

    pub theme_dark: bool,

    pub toasts: Vec<ToastInfo>,
    pub next_toast_id: u64,
}

impl Default for KglanceState {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            title_text: String::new(),
            status_text: String::new(),
            file_size_text: String::new(),
            file_modified_text: String::new(),
            file_type_text: String::new(),
            show_file_info: false,
            content_ready: true,
            show_back_button: false,
            back_target: None,
            image: ImageState::default(),
            text: TextState::default(),
            pdf: PdfState::default(),
            table: TableState::default(),
            spreadsheet: SpreadsheetState::default(),
            media: MediaState::default(),
            history: HistoryState::default(),
            dir: DirState::default(),
            markdown: MarkdownState::default(),
            font_size: 14.0,
            theme_dark: true,
            toasts: Vec::new(),
            next_toast_id: 0,
        }
    }
}
