use super::types::{CellCoord, CellRange, SpreadsheetState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Moves the active selection in the specified direction.
/// If `extend` is true, extends the selection range from the anchor `start`.
/// Returns an optional (new_scroll_x, new_scroll_y) if viewport scrolling is required to reveal the cell.
pub fn navigate_grid(
    state: &mut SpreadsheetState,
    direction: NavDirection,
    extend: bool,
) -> Option<iced::widget::operation::AbsoluteOffset> {
    let sheet = state.sheets.get(state.active_sheet)?;
    let total_rows = state.display_indices.len();
    let total_cols = sheet.columns.len();

    if total_rows == 0 || total_cols == 0 {
        return None;
    }

    let max_row = total_rows.saturating_sub(1);
    let max_col = total_cols.saturating_sub(1);

    // Current target cell
    let current = match state.selection {
        Some(range) => {
            if extend {
                range.end
            } else {
                range.start
            }
        }
        None => CellCoord { row: 0, col: 0 },
    };

    let new_coord = match direction {
        NavDirection::Up => CellCoord {
            row: current.row.saturating_sub(1),
            col: current.col,
        },
        NavDirection::Down => CellCoord {
            row: (current.row + 1).min(max_row),
            col: current.col,
        },
        NavDirection::Left => CellCoord {
            row: current.row,
            col: current.col.saturating_sub(1),
        },
        NavDirection::Right => CellCoord {
            row: current.row,
            col: (current.col + 1).min(max_col),
        },
        NavDirection::PageUp => CellCoord {
            row: current.row.saturating_sub(15),
            col: current.col,
        },
        NavDirection::PageDown => CellCoord {
            row: (current.row + 15).min(max_row),
            col: current.col,
        },
        NavDirection::Home => CellCoord {
            row: current.row,
            col: 0,
        },
        NavDirection::End => CellCoord {
            row: current.row,
            col: max_col,
        },
    };

    if extend {
        if let Some(existing) = state.selection {
            state.selection = Some(CellRange {
                start: existing.start,
                end: new_coord,
            });
        } else {
            state.selection = Some(CellRange::single(new_coord));
        }
    } else {
        state.selection = Some(CellRange::single(new_coord));
        state.active_cell = Some(new_coord);
    }

    compute_scroll_into_view(state, new_coord)
}

/// Computes viewport scroll adjustments to make sure `coord` is clearly visible.
pub fn compute_scroll_into_view(
    state: &mut SpreadsheetState,
    coord: CellCoord,
) -> Option<iced::widget::operation::AbsoluteOffset> {
    let mut changed = false;

    // 1. Vertical scrolling
    let cell_top = state.prefix_heights.get(coord.row).copied().unwrap_or(0.0);
    let cell_bottom = state
        .prefix_heights
        .get(coord.row + 1)
        .copied()
        .unwrap_or(cell_top + 26.0);

    let vh = if state.viewport_height > 0.0 {
        state.viewport_height
    } else {
        600.0
    };

    if cell_top < state.scroll_y {
        state.scroll_y = cell_top;
        state.smooth_scroll.stop(cell_top);
        state.scroll_controller.stop(cell_top);
        changed = true;
    } else if cell_bottom > state.scroll_y + vh {
        let new_y = (cell_bottom - vh).max(0.0);
        state.scroll_y = new_y;
        state.smooth_scroll.stop(new_y);
        state.scroll_controller.stop(new_y);
        changed = true;
    }

    // 2. Horizontal scrolling
    let cell_left = state.prefix_widths.get(coord.col).copied().unwrap_or(0.0);
    let cell_right = state
        .prefix_widths
        .get(coord.col + 1)
        .copied()
        .unwrap_or(cell_left + 80.0);

    let vw = if state.viewport_width > 0.0 {
        state.viewport_width
    } else {
        800.0
    };

    if cell_left < state.scroll_x {
        state.scroll_x = cell_left;
        state.smooth_scroll_x.stop(cell_left);
        state.scroll_controller_x.stop(cell_left);
        changed = true;
    } else if cell_right > state.scroll_x + vw {
        let new_x = (cell_right - vw).max(0.0);
        state.scroll_x = new_x;
        state.smooth_scroll_x.stop(new_x);
        state.scroll_controller_x.stop(new_x);
        changed = true;
    }

    if changed {
        Some(iced::widget::operation::AbsoluteOffset {
            x: state.scroll_x,
            y: state.scroll_y,
        })
    } else {
        None
    }
}
