use crate::parser::ParsedContent;
use std::sync::Arc;

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
}

#[derive(Debug, Clone)]
pub struct ImageState {
    pub zoom: f32,
    pub rotation: i32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub show_exif: bool,
    pub exif_content: String,
}

impl Default for ImageState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            rotation: 0,
            pan_x: 0.0,
            pan_y: 0.0,
            show_exif: false,
            exif_content: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextState {
    pub content: String,
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
            content: String::new(),
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
    pub current_page: usize,
    pub page_count: usize,
    pub zoom: f32,
    pub thumbnails: Vec<(Vec<u8>, u32, u32)>, // RGB/RGBA bytes, width, height
    pub show_nav: bool,
}

impl Default for PdfState {
    fn default() -> Self {
        Self {
            current_page: 0,
            page_count: 0,
            zoom: 1.0,
            thumbnails: Vec::new(),
            show_nav: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableState {
    pub rows: Vec<TableRowState>,
    pub sort_state: SortState,
}

#[derive(Debug, Clone, Default)]
pub struct MediaState {
    pub playing: bool,
    pub time: String,
    pub progress: f32,
    pub metadata: String,
    pub has_video: bool,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryState {
    pub history: Vec<String>,
    pub current_index: isize,
}

#[derive(Debug, Clone, Default)]
pub struct DirState {
    pub files: Vec<String>,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone)]
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
    pub media: MediaState,
    pub history: HistoryState,
    pub dir: DirState,

    pub theme_dark: bool,
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
            media: MediaState::default(),
            history: HistoryState::default(),
            dir: DirState::default(),
            theme_dark: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Basic actions
    OpenClicked,
    CopyPathClicked,
    BackClicked,
    CloseRequested,
    FileClicked(usize),

    // Navigation
    PrevPageClicked,
    NextPageClicked,
    PrevFileClicked,
    NextFileClicked,
    HistoryBack,
    HistoryForward,

    // Image viewer
    ImageZoomIn,
    ImageZoomOut,
    ImageRotateLeft,
    ImageRotateRight,
    ImageReset,
    ImageDragMoved {
        dx: f32,
        dy: f32,
    },
    ImageDragStarted {
        x: f32,
        y: f32,
    },
    ImageDragFinished,
    ImageScrollZoom(f32),
    ToggleExifSidebar,

    // Text search & wrap
    SearchQueryChanged(String),
    TextSearchNext,
    TextSearchPrev,
    TextSearchClosed,
    TextWrapToggled,
    CopyContentClicked,

    // Table sorting
    SortByFieldClicked(SortField),

    // Media
    PlayPauseClicked,
    SeekClicked(f32),
    SeekRelativeClicked(f32),

    // System
    ThemeToggled,
    FileLoaded {
        path: String,
        content: Arc<ParsedContent>,
    },
}
