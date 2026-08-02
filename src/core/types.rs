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
    pub active: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            ascending: true,
            active: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FolderRowState {
    pub name: String,
    pub kind: String,
    pub size: String,
    pub raw_size: u64,
    pub modified: String,
    pub raw_modified: i64,
    pub path: String,
    pub is_dir: bool,
    pub icon: &'static str,
}
use crate::features::image::{Camera, ImageLoadState};
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
    pub scroll_y: f32,
    pub word_count: usize,
    pub char_count: usize,
    pub reading_time_mins: usize,
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
            scroll_y: 0.0,
            word_count: 0,
            char_count: 0,
            reading_time_mins: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageCacheEntry {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub handle: iced::widget::image::Handle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfSidebarMode {
    #[default]
    Thumbnails,
    Toc,
}

#[derive(Debug, Clone)]
pub struct PdfState {
    pub pages: Vec<Option<PageCacheEntry>>,
    pub thumbnails: Vec<Option<PageCacheEntry>>,
    pub page_count: usize,
    pub loading: bool,
    pub current_page: usize,
    pub visible_page: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    pub window_start: usize,
    pub window_end: usize,
    pub preload_end: usize,
    pub sidebar_visible: bool,
    pub sidebar_mode: PdfSidebarMode,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
    pub outline: Vec<crate::parsers::pdf::PdfTocEntry>,
}

impl Default for PdfState {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            thumbnails: Vec::new(),
            page_count: 0,
            loading: false,
            current_page: 0,
            visible_page: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            window_start: 0,
            window_end: 0,
            preload_end: 0,
            sidebar_visible: false,
            sidebar_mode: PdfSidebarMode::Thumbnails,
            sidebar_width: 220.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 220.0,
            outline: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypstState {
    pub pdf: PdfState,
    pub source_content: iced::widget::text_editor::Content,
    pub show_source: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FolderState {
    pub rows: Vec<FolderRowState>,
    pub sort_state: SortState,
    pub selected_index: Option<usize>,
    pub total_size: u64,
    pub folder_path: String,
}

pub fn sort_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
    if !sort.active {
        return;
    }
    match sort.field {
        SortField::Name => {
            rows.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortField::Kind => {
            rows.sort_by(|a, b| a.kind.cmp(&b.kind));
        }
        SortField::Size => {
            rows.sort_by_key(|a| a.raw_size);
        }
        SortField::Modified => {
            rows.sort_by_key(|a| a.raw_modified);
        }
    }
    if !sort.ascending {
        rows.reverse();
    }
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
    pub search_visible: bool,
    pub search_query: String,
}

#[derive(Debug, Clone)]
pub struct EpubChapterInfo {
    pub title: String,
    pub level: u8,
    pub anchor: Option<String>,
    pub blocks: Vec<crate::parsers::markdown::Block>,
}

#[derive(Debug, Clone)]
pub struct JsonState {
    pub nodes: Vec<crate::parsers::json::JsonNode>,
    pub expanded: std::collections::HashSet<usize>,
    pub raw_content: String,
    pub pretty_content: String,
    pub tree_mode: bool,
    pub scroll_y: f32,
    pub has_parse_error: bool,
    pub raw_editor: iced::widget::text_editor::Content,
    pub search_visible: bool,
    pub search_query: String,
    pub minified_content: String,
    pub raw_pretty: bool,
    pub active_node: Option<usize>,
    pub editing_node: Option<usize>,
    pub edit_value: String,
    pub schema_visible: bool,
    pub schema_info: String,
}

impl Default for JsonState {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            expanded: std::collections::HashSet::new(),
            raw_content: String::new(),
            pretty_content: String::new(),
            tree_mode: true,
            scroll_y: 0.0,
            has_parse_error: false,
            raw_editor: iced::widget::text_editor::Content::new(),
            search_visible: false,
            search_query: String::new(),
            minified_content: String::new(),
            raw_pretty: true,
            active_node: None,
            editing_node: None,
            edit_value: String::new(),
            schema_visible: false,
            schema_info: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpubState {
    pub title: String,
    pub author: String,
    pub chapters: Vec<EpubChapterInfo>,
    pub active_chapter: usize,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
    pub scroll_y: f32,
    pub collapsed_chapters: std::collections::HashSet<usize>,
    pub markdown_state: crate::core::MarkdownState,
}

impl Default for EpubState {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            chapters: Vec::new(),
            active_chapter: 0,
            sidebar_visible: false,
            sidebar_width: 240.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 240.0,
            scroll_y: 0.0,
            collapsed_chapters: std::collections::HashSet::new(),
            markdown_state: crate::core::MarkdownState::default(),
        }
    }
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
}

#[derive(Debug, Clone, Default)]
pub struct HistoryState {
    pub history: Vec<String>,
    pub current_index: isize,
}

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub block_index: usize,
    pub y_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionPoint {
    pub block: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRange {
    pub start: SelectionPoint,
    pub end: SelectionPoint,
}

#[derive(Debug, Clone)]
pub struct MarkdownState {
    pub cached_mermaid_handles: std::collections::HashMap<usize, iced::widget::image::Handle>,
    pub cached_image_handles: std::collections::HashMap<usize, iced::widget::image::Handle>,
    pub cached_image_sizes: std::collections::HashMap<usize, (u32, u32)>,
    pub toc: Vec<TocEntry>,
    pub toc_visible: bool,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
    pub collapsed_headings: std::collections::HashSet<usize>,
    pub scroll_y: f32,
    pub word_count: usize,
    pub char_count: usize,
    pub reading_time_mins: usize,
    pub search_visible: bool,
    pub search_query: String,
    pub search_match_count: usize,
    pub search_match_index: usize,
    pub search_match_blocks: Vec<usize>,
    pub search_info: String,
    pub selected_text: Option<String>,
    pub selection_range: Option<SelectionRange>,
    pub is_dragging_selection: bool,
    pub auto_scroll_delta: Option<f32>,
    pub drag_last_y: f32,
}

impl Default for MarkdownState {
    fn default() -> Self {
        Self {
            cached_mermaid_handles: std::collections::HashMap::new(),
            cached_image_handles: std::collections::HashMap::new(),
            cached_image_sizes: std::collections::HashMap::new(),
            toc: Vec::new(),
            toc_visible: false,
            sidebar_width: 220.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 220.0,
            collapsed_headings: std::collections::HashSet::new(),
            scroll_y: 0.0,
            word_count: 0,
            char_count: 0,
            reading_time_mins: 0,
            search_visible: false,
            search_query: String::new(),
            search_match_count: 0,
            search_match_index: 0,
            search_match_blocks: Vec::new(),
            search_info: String::new(),
            selected_text: None,
            selection_range: None,
            is_dragging_selection: false,
            auto_scroll_delta: None,
            drag_last_y: 0.0,
        }
    }
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

use crate::core::preview::PreviewData;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Detail,
    Grid(Vec<GridThumbnail>),
    Settings,
}

pub const GRID_ITEM_WIDTH: f32 = 150.0;
pub const GRID_GAP: f32 = 12.0;
pub const GRID_ROW_HEIGHT: f32 = 140.0;

#[derive(Debug, Clone, PartialEq)]
pub struct GridThumbnail {
    pub path: String,
    pub name: String,
    pub thumbnail_handle: Option<iced::widget::image::Handle>,
    pub is_loading: bool,
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

    pub playlist: Vec<String>,
    pub current_index: usize,
    pub view_mode: ViewMode,
    pub cache: LruCache<String, Arc<PreviewData>>,
    pub pending_preloads: std::collections::HashSet<String>,

    pub image: ImageState,
    pub text: TextState,
    pub pdf: PdfState,
    pub typst: TypstState,
    pub folder: FolderState,
    pub spreadsheet: SpreadsheetState,
    pub media: MediaState,
    pub history: HistoryState,
    pub dir: DirState,
    pub markdown: MarkdownState,
    pub epub: EpubState,
    pub json: JsonState,

    pub grid_cols: usize,
    pub window_width: f32,
    pub grid_scale: f32,
    pub grid_search_visible: bool,
    pub grid_search_query: String,

    pub font_size: f32,
    pub default_font_size: f32,
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    pub epub_font_family: Option<String>,
    pub max_text_width: Option<f32>,

    pub window_default_size: iced::Size,
    pub window_min_size: iced::Size,

    pub app_theme: crate::ui::theme::AppTheme,
    pub theme_setting: String,

    pub toasts: Vec<ToastInfo>,
    pub next_toast_id: u64,
    pub prefer_mermaid_cli: bool,
    pub current_window_size: iced::Size,
}

const CACHE_CAPACITY: NonZeroUsize = match NonZeroUsize::new(7) {
    Some(cap) => cap,
    None => unreachable!(),
};

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
            playlist: Vec::new(),
            current_index: 0,
            view_mode: ViewMode::Detail,
            cache: LruCache::new(CACHE_CAPACITY),
            pending_preloads: std::collections::HashSet::new(),

            image: ImageState::default(),
            text: TextState::default(),
            pdf: PdfState::default(),
            typst: TypstState::default(),
            folder: FolderState::default(),
            spreadsheet: SpreadsheetState::default(),
            media: MediaState::default(),
            history: HistoryState::default(),
            dir: DirState::default(),
            markdown: MarkdownState::default(),
            epub: EpubState::default(),
            json: JsonState::default(),
            grid_cols: 5,
            window_width: 0.0,
            grid_scale: 1.0,
            grid_search_visible: false,
            grid_search_query: String::new(),
            font_size: 14.0,
            default_font_size: 14.0,
            font_family: None,
            font_family_mono: None,
            epub_font_family: None,
            max_text_width: None,
            window_default_size: iced::Size::new(1024.0, 768.0),
            window_min_size: iced::Size::new(800.0, 600.0),
            app_theme: crate::ui::theme::AppTheme::Dark,
            theme_setting: "Auto".to_string(),
            toasts: Vec::new(),
            next_toast_id: 0,
            prefer_mermaid_cli: false,
            current_window_size: iced::Size::new(1024.0, 768.0),
        }
    }
}
