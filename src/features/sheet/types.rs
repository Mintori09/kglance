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
pub struct SheetData {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SheetInfo {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub columns: Vec<ColumnMeta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellCoord {
    pub row: usize, // index in display_indices
    pub col: usize, // column index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellCoord,
    pub end: CellCoord,
}

impl CellRange {
    pub fn single(coord: CellCoord) -> Self {
        Self {
            start: coord,
            end: coord,
        }
    }

    pub fn row_range(&self) -> (usize, usize) {
        (
            self.start.row.min(self.end.row),
            self.start.row.max(self.end.row),
        )
    }

    pub fn col_range(&self) -> (usize, usize) {
        (
            self.start.col.min(self.end.col),
            self.start.col.max(self.end.col),
        )
    }

    pub fn contains(&self, row: usize, col: usize) -> bool {
        let (min_r, max_r) = self.row_range();
        let (min_c, max_c) = self.col_range();
        row >= min_r && row <= max_r && col >= min_c && col <= max_c
    }
}

#[derive(Debug, Clone)]
pub struct SpreadsheetState {
    pub sheets: Vec<SheetInfo>,
    pub active_sheet: usize,
    pub display_indices: Vec<usize>,
    pub row_heights: Vec<f32>,
    pub prefix_heights: Vec<f32>,
    pub prefix_widths: Vec<f32>,
    pub selection: Option<CellRange>,
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

impl SpreadsheetState {
    pub fn selected_text(&self) -> Option<String> {
        let range = self.selection?;
        let sheet = self.sheets.get(self.active_sheet)?;
        let (min_r, max_r) = range.row_range();
        let (min_c, max_c) = range.col_range();

        let mut lines = Vec::new();
        for r in min_r..=max_r {
            if let Some(row_data) = self
                .display_indices
                .get(r)
                .and_then(|&orig_idx| sheet.rows.get(orig_idx))
            {
                let mut row_cells = Vec::new();
                for c in min_c..=max_c {
                    let cell_text = row_data.get(c).map(String::as_str).unwrap_or("");
                    row_cells.push(cell_text);
                }
                lines.push(row_cells.join("\t"));
            }
        }

        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }
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
            selection: None,
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
