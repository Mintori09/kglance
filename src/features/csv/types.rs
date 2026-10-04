use crate::core::scroll::ScrollController;
use crate::core::types::SmoothScrollState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnType {
    #[default]
    Text,
    Integer,
    Float,
    Date,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnMeta {
    pub name: String,
    pub col_type: ColumnType,
    pub width: f32,
}

#[derive(Debug, Clone)]
pub struct SheetInfo {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub columns: Vec<ColumnMeta>,
}

#[derive(Debug, Clone)]
pub struct SpreadsheetState {
    pub sheets: Vec<SheetInfo>,
    pub active_sheet: usize,
    pub display_indices: Vec<usize>,
    pub row_heights: Vec<f32>,
    pub prefix_heights: Vec<f32>,
    pub prefix_widths: Vec<f32>,
    pub sort_col: Option<usize>,
    pub sort_ascending: Option<bool>,
    pub search_visible: bool,
    pub search_query: String,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub total_content_width: f32,
    pub total_content_height: f32,
    pub scroll_controller: ScrollController,
    pub smooth_scroll: SmoothScrollState,
    pub scroll_controller_x: ScrollController,
    pub smooth_scroll_x: SmoothScrollState,
}

impl Default for SpreadsheetState {
    fn default() -> Self {
        Self {
            sheets: Vec::new(),
            active_sheet: 0,
            display_indices: Vec::new(),
            row_heights: Vec::new(),
            prefix_heights: vec![0.0],
            prefix_widths: vec![0.0],
            sort_col: None,
            sort_ascending: None,
            search_visible: false,
            search_query: String::new(),
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_width: 1000.0,
            viewport_height: 800.0,
            total_content_width: 0.0,
            total_content_height: 0.0,
            scroll_controller: ScrollController::default(),
            smooth_scroll: SmoothScrollState::default(),
            scroll_controller_x: ScrollController::default(),
            smooth_scroll_x: SmoothScrollState::default(),
        }
    }
}
