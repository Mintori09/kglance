use crate::core::scroll::{ScrollController, TouchpadGestureTracker};
use crate::core::types::SmoothScrollState;
use crate::parsers::markdown::BlockLayout;
use iced::widget::image;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

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
    pub cached_mermaid_handles: HashMap<usize, image::Handle>,
    pub cached_image_handles: HashMap<usize, image::Handle>,
    pub cached_image_sizes: HashMap<usize, (u32, u32)>,
    pub toc: Vec<TocEntry>,
    pub toc_visible: bool,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
    pub collapsed_headings: HashSet<usize>,
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
    /// `block_layouts[i]` stores the estimated and measured height for block `i`.
    pub block_layouts: Vec<BlockLayout>,
    /// `block_y_offsets[i]` is the pixel Y where block `i` starts.
    pub block_y_offsets: Vec<f32>,
    /// Total estimated height of all content (sum of all block heights).
    pub total_content_height: f32,
    /// Height of the visible scroll viewport; updated on scroll events.
    pub viewport_height: f32,
    /// Content width used when current block layouts were computed.
    pub layout_content_width: f32,
    /// Font size used when current block layouts were computed.
    pub layout_font_size: f32,
    /// Atomic generation ID specifically for background markdown tasks (mermaid, images).
    pub generation_id: Arc<AtomicUsize>,
    pub smooth_scroll: SmoothScrollState,
    pub touchpad_tracker: TouchpadGestureTracker,
    pub scroll_controller: ScrollController,
}

impl Default for MarkdownState {
    fn default() -> Self {
        Self {
            cached_mermaid_handles: HashMap::new(),
            cached_image_handles: HashMap::new(),
            cached_image_sizes: HashMap::new(),
            toc: Vec::new(),
            toc_visible: false,
            sidebar_width: 250.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 250.0,
            collapsed_headings: HashSet::new(),
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
            block_layouts: Vec::new(),
            block_y_offsets: Vec::new(),
            total_content_height: 0.0,
            viewport_height: 800.0,
            layout_content_width: 0.0,
            layout_font_size: 0.0,
            generation_id: Arc::new(AtomicUsize::new(0)),
            smooth_scroll: SmoothScrollState::default(),
            touchpad_tracker: TouchpadGestureTracker::default(),
            scroll_controller: ScrollController::default(),
        }
    }
}
