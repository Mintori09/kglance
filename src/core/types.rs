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
    pub symbols: Vec<crate::features::text::CodeSymbol>,
    pub outline_visible: bool,
    pub sidebar_width: f32,
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
            symbols: Vec::new(),
            outline_visible: false,
            sidebar_width: 220.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Inserted,
    RejectedOversized,
    InvalidIndex,
}

#[derive(Debug, Clone)]
pub struct PageCacheEntry {
    pub width: u32,
    pub height: u32,
    pub handle: iced::widget::image::Handle,
}

impl PageCacheEntry {
    #[inline]
    pub fn decoded_bytes(&self) -> usize {
        (self.width as usize) * (self.height as usize) * 4
    }
}

#[derive(Debug, Clone, Default)]
pub struct PageCache {
    entries: Vec<Option<PageCacheEntry>>,
    accounted_decoded_bytes: usize,
}

impl PageCache {
    pub const MAX_COUNT: usize = 8;
    pub const MAX_BYTES: usize = 48 * 1024 * 1024; // 48 MiB logical budget

    pub fn new(page_count: usize) -> Self {
        Self {
            entries: vec![None; page_count],
            accounted_decoded_bytes: 0,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&PageCacheEntry> {
        self.entries.get(index).and_then(|p| p.as_ref())
    }

    #[inline]
    pub fn is_cached(&self, index: usize) -> bool {
        self.get(index).is_some()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.entries.iter().filter(|p| p.is_some()).count()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn accounted_decoded_bytes(&self) -> usize {
        self.accounted_decoded_bytes
    }

    pub fn compute_actual_decoded_bytes(&self) -> usize {
        self.entries
            .iter()
            .filter_map(|p| p.as_ref())
            .map(|e| e.decoded_bytes())
            .sum()
    }

    pub fn insert(
        &mut self,
        page_index: usize,
        entry: PageCacheEntry,
        anchor_page: usize,
    ) -> InsertResult {
        if page_index >= self.entries.len() {
            return InsertResult::InvalidIndex;
        }
        let entry_bytes = entry.decoded_bytes();
        if entry_bytes > Self::MAX_BYTES {
            crate::log_debug!(
                "[PDF_CACHE] Page {page_index} exceeds total RAM cache budget ({entry_bytes} bytes), keeping disk-backed only"
            );
            return InsertResult::RejectedOversized;
        }

        if let Some(old) = self.entries[page_index].take() {
            self.accounted_decoded_bytes = self
                .accounted_decoded_bytes
                .saturating_sub(old.decoded_bytes());
        }

        self.accounted_decoded_bytes += entry_bytes;
        self.entries[page_index] = Some(entry);
        self.evict(anchor_page);
        InsertResult::Inserted
    }

    pub fn evict(&mut self, anchor_page: usize) {
        let mut cached_indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(idx, page)| page.is_some().then_some(idx))
            .collect();

        if cached_indices.len() > Self::MAX_COUNT || self.accounted_decoded_bytes > Self::MAX_BYTES
        {
            cached_indices.sort_by_key(|&idx| {
                let dist = (idx as isize - anchor_page as isize).abs();
                (std::cmp::Reverse(dist), std::cmp::Reverse(idx))
            });

            for &idx in &cached_indices {
                if self.count() <= Self::MAX_COUNT
                    && self.accounted_decoded_bytes <= Self::MAX_BYTES
                {
                    break;
                }
                if let Some(entry) = self.entries[idx].take() {
                    self.accounted_decoded_bytes = self
                        .accounted_decoded_bytes
                        .saturating_sub(entry.decoded_bytes());
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.entries {
            *slot = None;
        }
        self.accounted_decoded_bytes = 0;
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThumbnailCache {
    entries: Vec<Option<PageCacheEntry>>,
    accounted_decoded_bytes: usize,
}

impl ThumbnailCache {
    pub const MAX_COUNT: usize = 2000;
    pub const MAX_BYTES: usize = 64 * 1024 * 1024; // 64 MiB logical budget

    pub fn new(page_count: usize) -> Self {
        Self {
            entries: vec![None; page_count],
            accounted_decoded_bytes: 0,
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&PageCacheEntry> {
        self.entries.get(index).and_then(|p| p.as_ref())
    }

    #[inline]
    pub fn is_cached(&self, index: usize) -> bool {
        self.get(index).is_some()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.entries.iter().filter(|p| p.is_some()).count()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn accounted_decoded_bytes(&self) -> usize {
        self.accounted_decoded_bytes
    }

    pub fn insert(
        &mut self,
        page_index: usize,
        entry: PageCacheEntry,
        anchor_page: usize,
    ) -> InsertResult {
        if page_index >= self.entries.len() {
            return InsertResult::InvalidIndex;
        }
        let entry_bytes = entry.decoded_bytes();
        if entry_bytes > Self::MAX_BYTES {
            return InsertResult::RejectedOversized;
        }

        if let Some(old) = self.entries[page_index].take() {
            self.accounted_decoded_bytes = self
                .accounted_decoded_bytes
                .saturating_sub(old.decoded_bytes());
        }

        self.accounted_decoded_bytes += entry_bytes;
        self.entries[page_index] = Some(entry);
        self.evict(anchor_page);
        InsertResult::Inserted
    }

    pub fn evict(&mut self, anchor_page: usize) {
        let mut cached_indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(idx, page)| page.is_some().then_some(idx))
            .collect();

        if cached_indices.len() > Self::MAX_COUNT || self.accounted_decoded_bytes > Self::MAX_BYTES
        {
            cached_indices.sort_by_key(|&idx| {
                let dist = (idx as isize - anchor_page as isize).abs();
                (std::cmp::Reverse(dist), std::cmp::Reverse(idx))
            });

            for &idx in &cached_indices {
                if self.count() <= Self::MAX_COUNT
                    && self.accounted_decoded_bytes <= Self::MAX_BYTES
                {
                    break;
                }
                if let Some(entry) = self.entries[idx].take() {
                    self.accounted_decoded_bytes = self
                        .accounted_decoded_bytes
                        .saturating_sub(entry.decoded_bytes());
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.entries {
            *slot = None;
        }
        self.accounted_decoded_bytes = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfSidebarMode {
    #[default]
    Thumbnails,
    Toc,
}

#[derive(Debug, Clone)]
pub struct PdfState {
    pub pages: PageCache,
    pub thumbnails: ThumbnailCache,
    pub page_count: usize,
    pub active_page_tasks: usize,
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
    pub generation_id: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    pub sidebar_drag_start_width: f32,
    pub outline: Vec<crate::parsers::pdf::PdfTocEntry>,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub display_width: f32,
    pub desired_width: f32,
    pub page_dimensions: Vec<crate::features::pdf::types::PageDimensions>,
    /// Cumulative Y offset for each page (pixels). `page_y_offsets[i]` = Y start of page i.
    pub page_y_offsets: Vec<f32>,
    /// Cumulative Y bottom edge for each page (pixels). `page_ends[i]` = Y end of page i.
    pub page_ends: Vec<f32>,
    /// Total estimated height of all pages + spacing (pixels).
    pub total_content_height: f32,
    /// Cumulative Y offset for each thumbnail in sidebar (pixels).
    pub thumbnail_y_offsets: Vec<f32>,
    /// Cumulative Y bottom edge for each thumbnail in sidebar (pixels).
    pub thumbnail_ends: Vec<f32>,
    /// Total estimated height of all thumbnails + spacing in sidebar (pixels).
    pub total_thumbnail_height: f32,
    /// Current Y scroll offset of the sidebar thumbnails.
    pub sidebar_scroll_y: f32,
    /// Current viewport height of the sidebar.
    pub sidebar_viewport_height: f32,
    /// Atomic index of top visible thumbnail for prioritizing background thumbnail loading.
    pub visible_thumb_page: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    /// Atomic generation ID specifically for background thumbnail worker.
    pub thumb_generation_id: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    /// Tier 1 session disk cache for compressed PDF page files.
    pub disk_cache: Option<std::sync::Arc<crate::features::pdf::PdfDiskCache>>,
}

impl PdfState {
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.active_page_tasks > 0
    }
}

impl Default for PdfState {
    fn default() -> Self {
        Self {
            pages: PageCache::default(),
            thumbnails: ThumbnailCache::default(),
            page_count: 0,
            active_page_tasks: 0,
            generation_id: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            thumb_generation_id: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
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
            scroll_y: 0.0,
            viewport_height: 800.0,
            display_width: 800.0,
            desired_width: 800.0,
            page_dimensions: Vec::new(),
            page_y_offsets: Vec::new(),
            page_ends: Vec::new(),
            total_content_height: 0.0,
            thumbnail_y_offsets: Vec::new(),
            thumbnail_ends: Vec::new(),
            total_thumbnail_height: 0.0,
            sidebar_scroll_y: 0.0,
            sidebar_viewport_height: 800.0,
            visible_thumb_page: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            disk_cache: None,
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
            tree_mode: false,
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
    /// Set when video/audio loading fails; cleared on new file load.
    pub error: Option<String>,
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
    pub is_mouse_held: bool,
    pub auto_scroll_delta: Option<f32>,
    pub drag_last_y: f32,
    /// `block_y_offsets[i]` is the pixel Y where block `i` starts.
    pub block_y_offsets: Vec<f32>,
    /// Total estimated height of all content (sum of all block heights).
    pub total_content_height: f32,
    /// Height of the visible scroll viewport; updated on scroll events.
    pub viewport_height: f32,
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
            is_mouse_held: false,
            auto_scroll_delta: None,
            drag_last_y: 0.0,
            block_y_offsets: Vec::new(),
            total_content_height: 0.0,
            viewport_height: 800.0,
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
    pub word_wrap: bool,
    pub json_tree_view: bool,
    pub current_window_size: iced::Size,

    pub read_positions: crate::core::ReadPositions,
    pub read_positions_dirty: bool,
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
            word_wrap: false,
            json_tree_view: false,
            current_window_size: iced::Size::new(1024.0, 768.0),

            read_positions: crate::core::ReadPositions::default(),
            read_positions_dirty: false,
        }
    }
}
