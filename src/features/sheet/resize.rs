use super::font::{scale_col_width, scale_ratio};
use super::parser::{MAX_COLUMN_WIDTH, MIN_COLUMN_WIDTH};
use super::types::{ColumnResizeState, SpreadsheetState};
use super::update::{compute_prefix_widths, compute_total_content_width};

/// Starts column resizing when dragging the right divider of a column header.
pub fn start_column_resize(
    state: &mut SpreadsheetState,
    col_idx: usize,
    cursor_x: f32,
    font_size: f32,
) {
    let sheet = match state.sheets.get(state.active_sheet) {
        Some(s) => s,
        None => return,
    };
    let current_width = match sheet.columns.get(col_idx) {
        Some(c) => scale_col_width(c.width, font_size),
        None => return,
    };

    state.resizing_col = Some(ColumnResizeState {
        col_idx,
        start_x: cursor_x,
        initial_width: current_width,
    });
}

/// Updates column width while dragging.
pub fn update_column_resize(state: &mut SpreadsheetState, cursor_x: f32, font_size: f32) -> bool {
    let resize_state = match state.resizing_col {
        Some(s) => s,
        None => return false,
    };

    let delta_x = cursor_x - resize_state.start_x;
    let ratio = scale_ratio(font_size);
    let min_scaled = MIN_COLUMN_WIDTH * ratio;
    let max_scaled = (MAX_COLUMN_WIDTH * 3.0) * ratio; // Allow dragging up to 1500px

    let new_scaled_width = (resize_state.initial_width + delta_x).clamp(min_scaled, max_scaled);
    let new_base_width = (new_scaled_width / ratio).round();

    let sheet = match state.sheets.get_mut(state.active_sheet) {
        Some(s) => s,
        None => return false,
    };

    if let Some(col_meta) = sheet.columns.get_mut(resize_state.col_idx)
        && (col_meta.width - new_base_width).abs() > f32::EPSILON
    {
        col_meta.width = new_base_width;
        state
            .custom_col_widths
            .insert(resize_state.col_idx, new_base_width);

        // Recompute geometry structures
        let prefix_widths = compute_prefix_widths(&sheet.columns, font_size);
        state.prefix_widths = prefix_widths;
        state.total_content_width = compute_total_content_width(sheet, font_size);
        return true;
    }

    false
}

/// Ends column resizing.
pub fn end_column_resize(state: &mut SpreadsheetState) {
    state.resizing_col = None;
}

/// Auto-fits a column width to fit its longest content and header text.
pub fn auto_fit_column(state: &mut SpreadsheetState, col_idx: usize, font_size: f32) -> bool {
    let sheet = match state.sheets.get_mut(state.active_sheet) {
        Some(s) => s,
        None => return false,
    };

    let col_meta = match sheet.columns.get(col_idx) {
        Some(c) => c,
        None => return false,
    };

    let header_len = col_meta.name.chars().count();
    let mut max_char_len = header_len;

    // Scan up to 500 rows for performance
    for row in sheet.rows.iter().take(500) {
        if let Some(cell) = row.get(col_idx) {
            for line in cell.lines() {
                let char_len = line.trim().chars().count();
                if char_len > max_char_len {
                    max_char_len = char_len;
                }
            }
        }
    }

    // Heuristic: char count * 8.5px + 28px padding
    let ideal_width =
        ((max_char_len as f32 * 8.5) + 28.0).clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH * 2.0);

    if let Some(col_meta) = sheet.columns.get_mut(col_idx) {
        col_meta.width = ideal_width;
        state.custom_col_widths.insert(col_idx, ideal_width);

        let prefix_widths = compute_prefix_widths(&sheet.columns, font_size);
        state.prefix_widths = prefix_widths;
        state.total_content_width = compute_total_content_width(sheet, font_size);
        return true;
    }

    false
}
