use crate::core::scroll::ScrollController;
use crate::core::types::SmoothScrollState;

#[derive(Debug, Clone)]
pub struct SheetInfo {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SpreadsheetState {
    pub sheets: Vec<SheetInfo>,
    pub active_sheet: usize,
    pub sort_col: Option<usize>,
    pub sort_ascending: Option<bool>,
    pub search_visible: bool,
    pub search_query: String,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub total_content_height: f32,
    pub scroll_controller: ScrollController,
    pub smooth_scroll: SmoothScrollState,
}

impl Default for SpreadsheetState {
    fn default() -> Self {
        Self {
            sheets: Vec::new(),
            active_sheet: 0,

            sort_col: None,
            sort_ascending: None,
            search_visible: false,
            search_query: String::new(),
            scroll_y: 0.0,
            viewport_height: 800.0,
            total_content_height: 0.0,
            scroll_controller: ScrollController::default(),
            smooth_scroll: SmoothScrollState::default(),
        }
    }
}
