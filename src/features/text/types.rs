use crate::core::scroll::ScrollController;
use crate::core::types::SmoothScrollState;
use crate::features::text::{
    BackgroundIndexer, CodeDocument, CodeSymbol, DisplayMap, HighlightedSpan, PerformancePolicy,
    SyntaxCache, ViewportAnchor,
};
use crate::ui::components::code_viewer::{SelectionRange, TextPosition};
use iced::Point;

#[derive(Debug, Clone)]
pub struct TextState {
    pub document: CodeDocument,
    pub syntax_cache: SyntaxCache,
    pub cached_tokens: Vec<Vec<HighlightedSpan>>,
    pub cached_tokens_start_line: usize,
    pub cached_tokens_version: usize,
    pub extension: String,
    pub wrap: bool,
    pub selection: Option<SelectionRange>,
    pub anchor: ViewportAnchor,
    pub display_map: DisplayMap,
    pub indexer: BackgroundIndexer,
    pub policy: PerformancePolicy,
    pub search_visible: bool,
    pub search_query: String,
    pub search_matches: Vec<(usize, usize)>,
    pub search_match_index: usize,
    pub search_info: String,
    pub goto_line_visible: bool,
    pub goto_line_query: String,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub total_content_height: f32,
    pub word_count: usize,
    pub char_count: usize,
    pub reading_time_mins: usize,
    pub symbols: Vec<CodeSymbol>,
    pub outline_visible: bool,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
    pub scroll_controller: ScrollController,
    pub smooth_scroll: SmoothScrollState,
    pub is_dragging_selection: bool,
    pub drag_start: Option<TextPosition>,
    pub drag_last_cursor: Point,
    pub auto_scroll_delta: Option<f32>,
}

impl TextState {
    /// Select all text within the document.
    pub fn select_all(&mut self) {
        let total_lines = self.document.total_lines();
        if total_lines == 0 {
            self.selection = None;
            return;
        }

        let last_line_idx = total_lines.saturating_sub(1);
        let last_line = self.document.get_line(last_line_idx);
        let last_line_len = last_line.chars().count();

        self.selection = Some(SelectionRange::new(
            TextPosition::new(0, 0),
            TextPosition::new(last_line_idx, last_line_len),
        ));
    }

    /// Extract the currently selected text, if any.
    pub fn selected_text(&self) -> Option<String> {
        let sel = self.selection?;
        if sel.is_empty() {
            return None;
        }
        let (start, end) = sel.normalized();
        Some(
            self.document
                .extract_range(start.line, start.col, end.line, end.col),
        )
    }

    /// Clear the current selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            document: CodeDocument::default(),
            syntax_cache: SyntaxCache::default(),
            cached_tokens: Vec::new(),
            cached_tokens_start_line: 0,
            cached_tokens_version: 0,
            extension: String::new(),
            wrap: true,
            selection: None,
            anchor: ViewportAnchor::default(),
            display_map: DisplayMap::default(),
            indexer: BackgroundIndexer::default(),
            policy: PerformancePolicy::default(),
            search_visible: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_index: 0,
            search_info: String::new(),
            goto_line_visible: false,
            goto_line_query: String::new(),
            scroll_y: 0.0,
            viewport_height: 800.0,
            total_content_height: 0.0,
            word_count: 0,
            char_count: 0,
            reading_time_mins: 0,
            symbols: Vec::new(),
            outline_visible: false,
            sidebar_width: 250.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 250.0,
            scroll_controller: ScrollController::default(),
            smooth_scroll: SmoothScrollState::default(),
            is_dragging_selection: false,
            drag_start: None,
            drag_last_cursor: Point::ORIGIN,
            auto_scroll_delta: None,
        }
    }
}
