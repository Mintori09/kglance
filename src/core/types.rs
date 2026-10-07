use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

use iced::Size;

pub use crate::core::scroll::SmoothScroller as SmoothScrollState;
pub use crate::features::audio::AudioState;
pub use crate::features::epub::{EpubChapterInfo, EpubState};
pub use crate::features::folder::{
    FolderRowState, FolderState, SortField, SortState, sort_folder_rows,
};
pub use crate::features::image::ImageState;
pub use crate::features::json::{JsonParsedCache, JsonState};
pub use crate::features::markdown::{MarkdownState, SelectionPoint, SelectionRange, TocEntry};
pub use crate::features::pdf::{
    InsertResult, PageCache, PageCacheEntry, PdfSidebarMode, PdfState, ThumbnailCache,
};
pub use crate::features::sheet::{SheetInfo, SpreadsheetState};
pub use crate::features::text::TextState;
pub use crate::features::typst::TypstState;
pub use crate::features::video::MediaState;

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
pub struct ToastInfo {
    pub id: u64,
    pub message: String,
}

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
    pub is_loading: bool,
    pub spinner_angle: f32,
    pub show_back_button: bool,
    pub back_target: Option<String>,
    pub active_dir: Option<PathBuf>,

    pub playlist: Vec<String>,
    pub current_index: usize,
    pub view_mode: ViewMode,
    pub cache: crate::core::MemoryCache,
    pub pending_preloads: HashSet<String>,
    pub generation_id: Arc<AtomicUsize>,
    pub dir_sync_generation_id: Arc<AtomicUsize>,

    pub image: ImageState,
    pub text: TextState,
    pub pdf: PdfState,
    pub typst: TypstState,
    pub folder: FolderState,
    pub spreadsheet: SpreadsheetState,
    pub media: MediaState,
    pub audio: AudioState,
    pub history: HistoryState,
    pub dir: DirState,
    pub markdown: MarkdownState,
    pub epub: EpubState,
    pub json: JsonState,

    pub grid_cols: usize,
    pub window_width: f32,
    pub window_height: f32,
    pub grid_scale: f32,
    pub grid_search_visible: bool,
    pub grid_search_query: String,

    pub font_size: f32,
    pub default_font_size: f32,
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    pub epub_font_family: Option<String>,
    pub max_text_width: Option<f32>,

    pub window_default_size: Size,
    pub window_min_size: Size,

    pub app_theme: crate::ui::theme::AppTheme,
    pub theme_setting: String,

    pub toasts: Vec<ToastInfo>,
    pub next_toast_id: u64,
    pub prefer_mermaid_cli: bool,
    pub word_wrap: bool,
    pub json_tree_view: bool,
    pub current_window_size: Size,

    pub read_positions: crate::core::ReadPositions,
    pub read_positions_dirty: bool,
    pub last_navigated_at: Option<Instant>,
    pub is_rapid_navigating: bool,
    pub scroll_config: crate::core::config::ScrollConfigOptions,
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
            is_loading: false,
            spinner_angle: 0.0,
            show_back_button: false,
            back_target: None,
            active_dir: None,
            playlist: Vec::new(),
            current_index: 0,
            view_mode: ViewMode::Detail,
            cache: crate::core::MemoryCache::default(),
            pending_preloads: HashSet::new(),
            generation_id: Arc::new(AtomicUsize::new(0)),
            dir_sync_generation_id: Arc::new(AtomicUsize::new(0)),
            last_navigated_at: None,
            is_rapid_navigating: false,
            image: ImageState::default(),
            text: TextState::default(),
            pdf: PdfState::default(),
            typst: TypstState::default(),
            folder: FolderState::default(),
            spreadsheet: SpreadsheetState::default(),
            media: MediaState::default(),
            audio: AudioState::default(),
            history: HistoryState::default(),
            dir: DirState::default(),
            markdown: MarkdownState::default(),
            epub: EpubState::default(),
            json: JsonState::default(),
            grid_cols: 5,
            window_width: 0.0,
            window_height: 0.0,
            grid_scale: 1.0,
            grid_search_visible: false,
            grid_search_query: String::new(),
            font_size: 14.0,
            default_font_size: 14.0,
            font_family: None,
            font_family_mono: None,
            epub_font_family: None,
            max_text_width: None,
            window_default_size: Size::new(1024.0, 768.0),
            window_min_size: Size::new(800.0, 600.0),
            app_theme: crate::ui::theme::AppTheme::Dark,
            theme_setting: "Auto".to_string(),
            toasts: Vec::new(),
            next_toast_id: 0,
            prefer_mermaid_cli: false,
            word_wrap: false,
            json_tree_view: false,
            current_window_size: Size::new(1024.0, 768.0),

            read_positions: crate::core::ReadPositions::default(),
            read_positions_dirty: false,
            scroll_config: crate::core::config::ScrollConfigOptions::default(),
        }
    }
}

impl KglanceState {
    pub fn reset_content_state(&mut self) {
        self.file_name.clear();
        self.title_text.clear();
        self.status_text.clear();
        self.file_size_text.clear();
        self.file_modified_text.clear();
        self.file_type_text.clear();
        self.show_file_info = false;
        self.content_ready = false;
        self.is_loading = false;
        self.spinner_angle = 0.0;
        self.show_back_button = false;
        self.back_target = None;
        self.active_dir = None;
        self.playlist.clear();
        self.current_index = 0;
        self.view_mode = ViewMode::Detail;
        self.cache.clear();
        self.pending_preloads.clear();

        self.image = ImageState::default();
        self.text = TextState::default();
        self.pdf = PdfState::default();
        self.typst = TypstState::default();
        self.folder = FolderState::default();
        self.spreadsheet = SpreadsheetState::default();
        self.media = MediaState::default();
        self.audio = AudioState::default();
        self.dir = DirState::default();
        self.markdown = MarkdownState::default();
        self.epub = EpubState::default();
        self.json = JsonState::default();
        self.apply_scroll_controllers();
    }

    pub fn reset_content_state_for_loading(&mut self, file_name: String) {
        self.file_name = file_name;
        self.title_text.clear();
        self.status_text.clear();
        self.file_size_text.clear();
        self.file_modified_text.clear();
        self.file_type_text.clear();
        self.show_file_info = false;
        self.content_ready = false;
        self.is_loading = true;
        self.spinner_angle = 0.0;

        let path = std::path::Path::new(&self.file_name);
        if let Ok(meta) = std::fs::metadata(path) {
            self.file_size_text = crate::core::utils::human_size(meta.len());
            if let Ok(modified) = meta.modified() {
                self.file_modified_text = crate::core::utils::human_time(modified);
            }
        }
        if path.is_dir() {
            self.active_dir = Some(path.to_path_buf());
        } else if let Some(parent) = path.parent() {
            self.active_dir = Some(parent.to_path_buf());
        }

        self.image = ImageState::default();
        self.text = TextState::default();
        self.pdf = PdfState::default();
        self.typst = TypstState::default();
        self.folder = FolderState::default();
        self.spreadsheet = SpreadsheetState::default();
        self.media = MediaState::default();
        self.audio = AudioState::default();
        self.dir = DirState::default();
        self.markdown = MarkdownState::default();
        self.epub = EpubState::default();
        self.json = JsonState::default();
        self.apply_scroll_controllers();
    }

    pub fn markdown_content_width(&self) -> f32 {
        crate::features::markdown::effective_content_width(
            self.max_text_width,
            self.window_width,
            self.markdown.toc_visible,
            self.markdown.sidebar_width,
        )
    }

    pub fn epub_content_width(&self) -> f32 {
        crate::features::markdown::effective_content_width(
            self.max_text_width,
            self.window_width,
            self.epub.sidebar_visible,
            self.epub.sidebar_width,
        )
    }

    pub fn apply_scroll_config(&mut self, config: &crate::core::config::ScrollConfigOptions) {
        self.scroll_config = config.clone();
        self.apply_scroll_controllers();
    }

    pub fn apply_scroll_controllers(&mut self) {
        let config = &self.scroll_config;
        self.markdown.scroll_controller.apply_config(config);
        self.markdown.smooth_scroll.apply_config(config);
        self.pdf.scroll_controller.apply_config(config);
        self.pdf.smooth_scroll.apply_config(config);
        self.text.scroll_controller.apply_config(config);
        self.text.smooth_scroll.apply_config(config);
        self.json.scroll_controller.apply_config(config);
        self.json.smooth_scroll.apply_config(config);
        self.json.raw_text.scroll_controller.apply_config(config);
        self.json.raw_text.smooth_scroll.apply_config(config);
        self.typst
            .source_text
            .scroll_controller
            .apply_config(config);
        self.typst.source_text.smooth_scroll.apply_config(config);
        self.epub
            .markdown_state
            .scroll_controller
            .apply_config(config);
        self.epub.markdown_state.smooth_scroll.apply_config(config);
        self.spreadsheet.scroll_controller.apply_config(config);
        self.spreadsheet.scroll_controller_x.apply_config(config);
        self.spreadsheet.smooth_scroll.apply_config(config);
        self.spreadsheet.smooth_scroll_x.apply_config(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smooth_scroll_state_default() {
        let state = SmoothScrollState::default();
        assert_eq!(state.target_y, 0.0);
        assert!(!state.is_animating);
    }
}
