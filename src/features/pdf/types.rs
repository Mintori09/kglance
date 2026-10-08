use crate::core::scroll::ScrollController;
use crate::core::types::SmoothScrollState;
use crate::features::pdf::PdfDiskCache;
use crate::features::pdf::selection::{PdfPageText, PdfPosition, PdfSelection};
use crate::parsers::pdf::PdfTocEntry;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Clone)]
pub struct PageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Page dimensions in PDF points (1 point = 1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageDimensions {
    pub width_pts: f32,
    pub height_pts: f32,
}

impl PageDimensions {
    /// aspect ratio (width / height). Returns 1.0 for zero-height pages
    pub fn aspect_ratio(&self) -> f32 {
        if self.height_pts > 0.0 {
            self.width_pts / self.height_pts
        } else {
            1.0
        }
    }

    /// pixel height when scaled to a given display width
    pub fn display_height(&self, display_width: f32) -> f32 {
        display_width / self.aspect_ratio()
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
    pub const MAX_COUNT: usize = 16;
    pub const MAX_BYTES: usize = 192 * 1024 * 1024; // 192 MiB logical budget

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
    pub visible_page: Arc<AtomicUsize>,
    pub window_start: usize,
    pub window_end: usize,
    pub preload_end: usize,
    pub sidebar_visible: bool,
    pub sidebar_mode: PdfSidebarMode,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub generation_id: Arc<AtomicUsize>,
    pub sidebar_drag_start_width: f32,
    pub outline: Vec<PdfTocEntry>,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub display_width: f32,
    pub desired_width: f32,
    pub page_dimensions: Vec<PageDimensions>,
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
    pub visible_thumb_page: Arc<AtomicUsize>,
    /// Atomic generation ID specifically for background thumbnail worker.
    pub thumb_generation_id: Arc<AtomicUsize>,
    /// Tier 1 session disk cache for compressed PDF page files.
    pub disk_cache: Option<Arc<PdfDiskCache>>,
    pub scroll_controller: ScrollController,
    pub smooth_scroll: SmoothScrollState,
    pub page_texts: Vec<Option<PdfPageText>>,
    pub selection: Option<PdfSelection>,
    pub selected_text: Option<String>,
    pub selected_html: Option<String>,
    pub is_selecting: bool,
    pub selection_drag_start: Option<PdfPosition>,
    pub auto_scroll_delta: Option<f32>,
    pub drag_last_x: f32,
    pub drag_last_y: f32,
}

impl PdfState {
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.active_page_tasks > 0
    }

    #[inline]
    pub fn selected_text(&self) -> Option<String> {
        self.selected_text.clone()
    }

    #[inline]
    pub fn selected_html(&self) -> Option<String> {
        self.selected_html.clone()
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selected_text = None;
        self.selected_html = None;
        self.is_selecting = false;
        self.selection_drag_start = None;
        self.auto_scroll_delta = None;
    }
}

impl Default for PdfState {
    fn default() -> Self {
        Self {
            pages: PageCache::default(),
            thumbnails: ThumbnailCache::default(),
            page_count: 0,
            active_page_tasks: 0,
            generation_id: Arc::new(AtomicUsize::new(0)),
            thumb_generation_id: Arc::new(AtomicUsize::new(0)),
            current_page: 0,
            visible_page: Arc::new(AtomicUsize::new(0)),
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
            visible_thumb_page: Arc::new(AtomicUsize::new(0)),
            disk_cache: None,
            scroll_controller: ScrollController::default(),
            smooth_scroll: SmoothScrollState::default(),
            page_texts: Vec::new(),
            selection: None,
            selected_text: None,
            selected_html: None,
            is_selecting: false,
            selection_drag_start: None,
            auto_scroll_delta: None,
            drag_last_x: 0.0,
            drag_last_y: 0.0,
        }
    }
}
