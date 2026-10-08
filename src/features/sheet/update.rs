use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::types::KglanceState;
use crate::features::sheet::parser::try_parse_date_or_datetime;
use crate::features::sheet::types::{ColumnMeta, ColumnType, SheetInfo};
use crate::features::sheet::view::{
    COL_SPACING, HEADER_HEIGHT, ROW_HEIGHT, ROW_NUMBER_COL_WIDTH, ROWS_LIST_SPACING,
    estimate_row_height,
};
use iced::Task;

const CONTENT_SCROLL_ID: &str = "content_scroll";

pub fn compute_prefix_widths(columns: &[ColumnMeta]) -> Vec<f32> {
    let mut prefix = Vec::with_capacity(columns.len() + 1);
    prefix.push(0.0);
    let mut acc = 0.0;
    for col in columns {
        acc += col.width + COL_SPACING;
        prefix.push(acc);
    }
    prefix
}

pub fn compute_total_content_width(sheet: &SheetInfo) -> f32 {
    let mut total = ROW_NUMBER_COL_WIDTH + COL_SPACING;
    for col in &sheet.columns {
        total += col.width + COL_SPACING;
    }
    total
}

pub fn populate_state(state: &mut KglanceState, sheets: &[SheetInfo], active_sheet: usize) {
    state.spreadsheet.sheets = sheets.to_vec();
    state.spreadsheet.active_sheet = active_sheet;
    state.spreadsheet.sort_col = None;
    state.spreadsheet.sort_ascending = None;
    state.spreadsheet.selection = None;
    state.spreadsheet.is_dragging = false;
    state.spreadsheet.is_dragging_row_headers = false;
    state.spreadsheet.auto_scroll_delta_y = None;
    state.spreadsheet.auto_scroll_delta_x = None;
    state.spreadsheet.drag_last_cursor = iced::Point::ORIGIN;
    state.spreadsheet.search_query.clear();
    state.spreadsheet.search_visible = false;
    state.spreadsheet.scroll_x = 0.0;
    state.spreadsheet.scroll_y = 0.0;
    state.spreadsheet.smooth_scroll.stop(0.0);
    state.spreadsheet.scroll_controller.stop(0.0);
    state.spreadsheet.smooth_scroll_x.stop(0.0);
    state.spreadsheet.scroll_controller_x.stop(0.0);

    if let Some(sheet) = state.spreadsheet.sheets.get(active_sheet) {
        let indices = recompute_display_indices(sheet, "", None, None);
        let (row_heights, prefix_heights, total_h) = compute_height_structures(sheet, &indices);
        let prefix_widths = compute_prefix_widths(&sheet.columns);
        state.spreadsheet.display_indices = indices;
        state.spreadsheet.row_heights = row_heights;
        state.spreadsheet.prefix_heights = prefix_heights;
        state.spreadsheet.prefix_widths = prefix_widths;
        state.spreadsheet.total_content_height = total_h + HEADER_HEIGHT + ROWS_LIST_SPACING;
        state.spreadsheet.total_content_width = compute_total_content_width(sheet);
    } else {
        state.spreadsheet.display_indices.clear();
        state.spreadsheet.row_heights.clear();
        state.spreadsheet.prefix_heights = vec![0.0];
        state.spreadsheet.prefix_widths = vec![0.0];
        state.spreadsheet.total_content_height = 0.0;
        state.spreadsheet.total_content_width = 0.0;
    }

    state
        .spreadsheet
        .scroll_controller
        .apply_config(&state.scroll_config);
    state
        .spreadsheet
        .scroll_controller_x
        .apply_config(&state.scroll_config);
    state
        .spreadsheet
        .smooth_scroll
        .apply_config(&state.scroll_config);
    state
        .spreadsheet
        .smooth_scroll_x
        .apply_config(&state.scroll_config);

    state.file_type_text = "Spreadsheet".to_string();
}

pub fn compute_height_structures(
    sheet: &SheetInfo,
    display_indices: &[usize],
) -> (Vec<f32>, Vec<f32>, f32) {
    let mut row_heights = Vec::with_capacity(display_indices.len());
    let mut prefix_heights = Vec::with_capacity(display_indices.len() + 1);
    prefix_heights.push(0.0);

    let mut acc = 0.0;
    for &orig_idx in display_indices {
        let h = if let Some(row_data) = sheet.rows.get(orig_idx) {
            estimate_row_height(row_data, &sheet.columns)
        } else {
            ROW_HEIGHT
        };
        row_heights.push(h);
        acc += h + ROWS_LIST_SPACING;
        prefix_heights.push(acc);
    }

    (row_heights, prefix_heights, acc)
}

pub fn recompute_display_indices(
    sheet: &SheetInfo,
    search_query: &str,
    sort_col: Option<usize>,
    sort_ascending: Option<bool>,
) -> Vec<usize> {
    let trimmed_query = search_query.trim();
    let mut indices: Vec<usize> = if trimmed_query.is_empty() {
        (0..sheet.rows.len()).collect()
    } else {
        let query_lower = trimmed_query.to_lowercase();
        (0..sheet.rows.len())
            .filter(|&idx| {
                if let Some(row) = sheet.rows.get(idx) {
                    row.iter()
                        .any(|cell| cell.to_lowercase().contains(&query_lower))
                } else {
                    false
                }
            })
            .collect()
    };

    if let (Some(col), Some(ascending)) = (sort_col, sort_ascending) {
        let col_type = sheet
            .columns
            .get(col)
            .map(|c| c.col_type)
            .unwrap_or(ColumnType::Text);

        indices.sort_by(|&idx_a, &idx_b| {
            let row_a = sheet.rows.get(idx_a);
            let row_b = sheet.rows.get(idx_b);
            let val_a = row_a
                .and_then(|r| r.get(col))
                .map(|s| s.trim())
                .unwrap_or("");
            let val_b = row_b
                .and_then(|r| r.get(col))
                .map(|s| s.trim())
                .unwrap_or("");

            // Empty values always sort to the end
            if val_a.is_empty() && !val_b.is_empty() {
                return std::cmp::Ordering::Greater;
            }
            if !val_a.is_empty() && val_b.is_empty() {
                return std::cmp::Ordering::Less;
            }
            if val_a.is_empty() && val_b.is_empty() {
                return std::cmp::Ordering::Equal;
            }

            let ord = match col_type {
                ColumnType::Integer => {
                    let num_a = val_a.parse::<i64>().ok();
                    let num_b = val_b.parse::<i64>().ok();
                    match (num_a, num_b) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => val_a.cmp(val_b),
                    }
                }
                ColumnType::Float => {
                    let num_a = val_a.parse::<f64>().ok();
                    let num_b = val_b.parse::<f64>().ok();
                    match (num_a, num_b) {
                        (Some(a), Some(b)) => a.total_cmp(&b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => val_a.cmp(val_b),
                    }
                }
                ColumnType::Date => {
                    let date_a = try_parse_date_or_datetime(val_a);
                    let date_b = try_parse_date_or_datetime(val_b);
                    match (date_a, date_b) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => val_a.cmp(val_b),
                    }
                }
                ColumnType::Text | ColumnType::Empty => {
                    // Try numeric parsing if both values are valid floats
                    if let (Ok(num_a), Ok(num_b)) = (val_a.parse::<f64>(), val_b.parse::<f64>()) {
                        num_a.total_cmp(&num_b)
                    } else {
                        val_a.cmp(val_b)
                    }
                }
            };

            if ascending { ord } else { ord.reverse() }
        });
    }

    indices
}

pub fn handle_sheet_tab_clicked(app: &mut KglanceApp, index: usize) -> Task<Message> {
    if index < app.state.spreadsheet.sheets.len() {
        app.state.spreadsheet.active_sheet = index;
        app.state.spreadsheet.sort_col = None;
        app.state.spreadsheet.sort_ascending = None;
        app.state.spreadsheet.selection = None;
        app.state.spreadsheet.scroll_x = 0.0;
        app.state.spreadsheet.scroll_y = 0.0;
        app.state.spreadsheet.smooth_scroll.stop(0.0);
        app.state.spreadsheet.scroll_controller.stop(0.0);
        app.state.spreadsheet.smooth_scroll_x.stop(0.0);
        app.state.spreadsheet.scroll_controller_x.stop(0.0);

        if let Some(sheet) = app.state.spreadsheet.sheets.get(index) {
            let indices =
                recompute_display_indices(sheet, &app.state.spreadsheet.search_query, None, None);
            let (row_heights, prefix_heights, total_h) = compute_height_structures(sheet, &indices);
            let prefix_widths = compute_prefix_widths(&sheet.columns);
            app.state.spreadsheet.display_indices = indices;
            app.state.spreadsheet.row_heights = row_heights;
            app.state.spreadsheet.prefix_heights = prefix_heights;
            app.state.spreadsheet.prefix_widths = prefix_widths;
            app.state.spreadsheet.total_content_height =
                total_h + HEADER_HEIGHT + ROWS_LIST_SPACING;
            app.state.spreadsheet.total_content_width = compute_total_content_width(sheet);
        }
    }
    Task::none()
}

pub fn handle_cell_pressed(app: &mut KglanceApp, row: usize, col: usize) -> Task<Message> {
    let target = crate::features::sheet::CellCoord { row, col };
    let is_shift = app.shift_held;
    let sheet_state = &mut app.state.spreadsheet;
    sheet_state.is_dragging = true;
    sheet_state.is_dragging_row_headers = false;

    if is_shift {
        if let Some(existing) = sheet_state.selection {
            sheet_state.selection = Some(crate::features::sheet::CellRange {
                start: existing.start,
                end: target,
            });
        } else {
            sheet_state.selection = Some(crate::features::sheet::CellRange::single(target));
        }
    } else {
        sheet_state.selection = Some(crate::features::sheet::CellRange::single(target));
    }
    Task::none()
}

pub fn handle_cell_entered(app: &mut KglanceApp, row: usize, col: usize) -> Task<Message> {
    let sheet_state = &mut app.state.spreadsheet;
    if sheet_state.is_dragging
        && let Some(ref mut selection) = sheet_state.selection
    {
        selection.end = crate::features::sheet::CellCoord { row, col };
    }
    Task::none()
}

pub fn handle_cell_released(app: &mut KglanceApp) -> Task<Message> {
    let sheet_state = &mut app.state.spreadsheet;
    sheet_state.is_dragging = false;
    sheet_state.is_dragging_row_headers = false;
    sheet_state.auto_scroll_delta_y = None;
    sheet_state.auto_scroll_delta_x = None;
    Task::none()
}

pub fn handle_row_header_pressed(app: &mut KglanceApp, row: usize) -> Task<Message> {
    let is_shift = app.shift_held;
    let sheet_state = &mut app.state.spreadsheet;
    let col_count = sheet_state
        .sheets
        .get(sheet_state.active_sheet)
        .map_or(0, |s| s.columns.len());

    if col_count > 0 {
        let max_col = col_count.saturating_sub(1);
        sheet_state.is_dragging = false;
        sheet_state.is_dragging_row_headers = true;
        if is_shift {
            if let Some(existing) = sheet_state.selection {
                sheet_state.selection = Some(crate::features::sheet::CellRange {
                    start: crate::features::sheet::CellCoord {
                        row: existing.start.row,
                        col: 0,
                    },
                    end: crate::features::sheet::CellCoord { row, col: max_col },
                });
            } else {
                sheet_state.selection = Some(crate::features::sheet::CellRange {
                    start: crate::features::sheet::CellCoord { row, col: 0 },
                    end: crate::features::sheet::CellCoord { row, col: max_col },
                });
            }
        } else {
            sheet_state.selection = Some(crate::features::sheet::CellRange {
                start: crate::features::sheet::CellCoord { row, col: 0 },
                end: crate::features::sheet::CellCoord { row, col: max_col },
            });
        }
    }
    Task::none()
}

pub fn handle_row_header_entered(app: &mut KglanceApp, row: usize) -> Task<Message> {
    let sheet_state = &mut app.state.spreadsheet;
    if sheet_state.is_dragging_row_headers {
        let col_count = sheet_state
            .sheets
            .get(sheet_state.active_sheet)
            .map_or(0, |s| s.columns.len());
        if col_count > 0 {
            let max_col = col_count.saturating_sub(1);
            if let Some(ref mut selection) = sheet_state.selection {
                selection.end = crate::features::sheet::CellCoord { row, col: max_col };
            }
        }
    }
    Task::none()
}

pub fn handle_copy_selection(app: &mut KglanceApp) -> Task<Message> {
    if let Some(text) = app
        .state
        .spreadsheet
        .selected_text()
        .filter(|t| !t.is_empty())
    {
        let toast = app.show_toast("Copied selected!");
        return Task::batch(vec![
            crate::core::clipboard::copy_to_clipboard(text, None),
            toast,
        ]);
    }
    Task::none()
}

pub fn handle_column_clicked(app: &mut KglanceApp, col: usize) -> Task<Message> {
    let sort = &mut app.state.spreadsheet;
    if sort.sort_col == Some(col) {
        sort.sort_ascending = match sort.sort_ascending {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        };
        if sort.sort_ascending.is_none() {
            sort.sort_col = None;
        }
    } else {
        sort.sort_col = Some(col);
        sort.sort_ascending = Some(true);
    }

    let active_idx = sort.active_sheet;
    let query = sort.search_query.clone();
    let sort_col = sort.sort_col;
    let sort_ascending = sort.sort_ascending;

    if let Some(sheet) = sort.sheets.get(active_idx) {
        let indices = recompute_display_indices(sheet, &query, sort_col, sort_ascending);
        let (row_heights, prefix_heights, total_h) = compute_height_structures(sheet, &indices);
        sort.display_indices = indices;
        sort.row_heights = row_heights;
        sort.prefix_heights = prefix_heights;
        sort.total_content_height = total_h + HEADER_HEIGHT + ROWS_LIST_SPACING;
        sort.total_content_width = compute_total_content_width(sheet);
    }

    Task::none()
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    let sort = &mut app.state.spreadsheet;
    sort.search_query = query.clone();
    sort.scroll_y = 0.0;
    sort.smooth_scroll.stop(0.0);
    sort.scroll_controller.stop(0.0);

    let active_idx = sort.active_sheet;
    let sort_col = sort.sort_col;
    let sort_ascending = sort.sort_ascending;

    if let Some(sheet) = sort.sheets.get(active_idx) {
        let indices = recompute_display_indices(sheet, &query, sort_col, sort_ascending);
        let (row_heights, prefix_heights, total_h) = compute_height_structures(sheet, &indices);
        sort.display_indices = indices;
        sort.row_heights = row_heights;
        sort.prefix_heights = prefix_heights;
        sort.total_content_height = total_h + HEADER_HEIGHT + ROWS_LIST_SPACING;
        sort.total_content_width = compute_total_content_width(sheet);
    }

    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    let sort = &mut app.state.spreadsheet;
    sort.search_visible = false;
    sort.search_query.clear();
    sort.scroll_y = 0.0;
    sort.smooth_scroll.stop(0.0);
    sort.scroll_controller.stop(0.0);

    let active_idx = sort.active_sheet;
    let sort_col = sort.sort_col;
    let sort_ascending = sort.sort_ascending;

    if let Some(sheet) = sort.sheets.get(active_idx) {
        let indices = recompute_display_indices(sheet, "", sort_col, sort_ascending);
        let (row_heights, prefix_heights, total_h) = compute_height_structures(sheet, &indices);
        sort.display_indices = indices;
        sort.row_heights = row_heights;
        sort.prefix_heights = prefix_heights;
        sort.total_content_height = total_h + HEADER_HEIGHT + ROWS_LIST_SPACING;
        sort.total_content_width = compute_total_content_width(sheet);
    }

    Task::none()
}

pub fn handle_spreadsheet_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let x = viewport.absolute_offset().x;
    let y = viewport.absolute_offset().y;
    let vw = viewport.bounds().width;
    let vh = viewport.bounds().height;
    let total_w = viewport.content_bounds().width;
    let total_h = viewport.content_bounds().height;

    let state = &mut app.state.spreadsheet;
    let delta_vh = (state.viewport_height - vh).abs();
    if vw > 0.0 {
        state.viewport_width = vw;
    }
    if vh > 0.0 {
        state.viewport_height = vh;
    }
    if total_w > 0.0 && state.total_content_width <= 0.0 {
        state.total_content_width = total_w;
    }
    if total_h > 0.0 && state.total_content_height <= 0.0 {
        state.total_content_height = total_h;
    }

    let is_animating_y = state.smooth_scroll.is_animating()
        || state.scroll_controller.is_animating()
        || state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging;

    let is_animating_x = state.smooth_scroll_x.is_animating()
        || state.scroll_controller_x.is_animating()
        || state.scroll_controller_x.state() == crate::core::scroll::GestureState::Dragging;

    if is_animating_y || is_animating_x {
        return Task::none();
    }

    let delta_x = (state.scroll_x - x).abs();
    let delta_y = (state.scroll_y - y).abs();
    if delta_x < 4.0 && delta_y < 4.0 && delta_vh < 1.0 {
        return Task::none();
    }

    state.smooth_scroll.stop(y);
    state.scroll_controller.stop(y);
    state.scroll_y = y;

    state.smooth_scroll_x.stop(x);
    state.scroll_controller_x.stop(x);
    state.scroll_x = x;
    Task::none()
}

pub fn handle_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let state = &mut app.state.spreadsheet;
    if state.viewport_height <= 0.0 {
        return Task::none();
    }
    let vw = state.viewport_width;
    let vh = state.viewport_height;
    let extent_y = crate::core::scroll::ViewportExtent {
        content_height: state.total_content_height,
        viewport_height: vh,
    };
    let extent_x = crate::core::scroll::ViewportExtent {
        content_height: state.total_content_width,
        viewport_height: vw,
    };
    let max_y = extent_y.max_scroll_y();
    let max_x = extent_x.max_scroll_y();

    match delta {
        iced::mouse::ScrollDelta::Lines { x, y } => {
            if app.shift_held {
                // Shift + Wheel vertical scroll -> Redirect to horizontal scroll
                let delta_h = if y.abs() > f32::EPSILON { y } else { x };
                if delta_h.abs() > f32::EPSILON {
                    let col_width = (vw * 0.15).clamp(80.0, 250.0);
                    let step = -delta_h * col_width;
                    state
                        .smooth_scroll_x
                        .start_interactive(state.scroll_x, step, max_x);
                }
            } else {
                if y.abs() > f32::EPSILON {
                    let line_height = (vh * crate::core::scroll::WHEEL_SCROLL_VIEWPORT_FRACTION)
                        .clamp(50.0, 240.0);
                    let step = -y * line_height;
                    state
                        .smooth_scroll
                        .start_interactive(state.scroll_y, step, max_y);
                }
                if x.abs() > f32::EPSILON {
                    let col_width = (vw * 0.15).clamp(80.0, 250.0);
                    let step = -x * col_width;
                    state
                        .smooth_scroll_x
                        .start_interactive(state.scroll_x, step, max_x);
                }
            }
            if !state.smooth_scroll.is_animating() && !state.smooth_scroll_x.is_animating() {
                state.scroll_y = state.smooth_scroll.position_y();
                state.scroll_x = state.smooth_scroll_x.position_y();
                return iced::widget::operation::scroll_to(
                    CONTENT_SCROLL_ID,
                    iced::widget::operation::AbsoluteOffset {
                        x: state.scroll_x,
                        y: state.scroll_y,
                    },
                );
            }
            Task::none()
        }
        iced::mouse::ScrollDelta::Pixels { x, y } => {
            let now = std::time::Instant::now();
            let (dx, dy) = if app.shift_held && x.abs() <= f32::EPSILON && y.abs() > f32::EPSILON {
                // Shift + vertical pixel scroll -> redirect to horizontal
                (y, 0.0)
            } else {
                (x, y)
            };

            let scaled_delta_y = -dy * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;
            let scaled_delta_x = -dx * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;

            if scaled_delta_y.abs() > f32::EPSILON {
                state.scroll_controller.set_position_y(state.scroll_y);
                state.scroll_controller.handle_input(
                    crate::core::scroll::ScrollInput::Motion {
                        delta_x: scaled_delta_x,
                        delta_y: scaled_delta_y,
                        time: now,
                    },
                    extent_y,
                );
                let new_y = state.scroll_controller.position_y();
                state.scroll_y = new_y;
                state.smooth_scroll.stop(new_y);
            }

            if scaled_delta_x.abs() > f32::EPSILON {
                state.scroll_controller_x.set_position_y(state.scroll_x);
                state.scroll_controller_x.handle_input(
                    crate::core::scroll::ScrollInput::Motion {
                        delta_x: 0.0,
                        delta_y: scaled_delta_x,
                        time: now,
                    },
                    extent_x,
                );
                let new_x = state.scroll_controller_x.position_y();
                state.scroll_x = new_x;
                state.smooth_scroll_x.stop(new_x);
            }

            iced::widget::operation::scroll_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::AbsoluteOffset {
                    x: state.scroll_x,
                    y: state.scroll_y,
                },
            )
        }
    }
}

pub fn handle_smooth_scroll_tick(app: &mut KglanceApp, now: std::time::Instant) -> Task<Message> {
    let state = &mut app.state.spreadsheet;
    let extent_y = crate::core::scroll::ViewportExtent {
        content_height: state.total_content_height,
        viewport_height: state.viewport_height,
    };
    let extent_x = crate::core::scroll::ViewportExtent {
        content_height: state.total_content_width,
        viewport_height: state.viewport_width,
    };
    let max_y = extent_y.max_scroll_y();
    let max_x = extent_x.max_scroll_y();

    let mut changed = false;

    // 1. Vertical axis: Check ScrollController (touchpad kinetic fling & inertia)
    if (state.scroll_controller.is_animating()
        || state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging)
        && let Some(next_y) = state.scroll_controller.update(now, extent_y)
    {
        state.scroll_y = next_y;
        state.smooth_scroll.stop(next_y);
        changed = true;
    } else if let Some(next_y) = state.smooth_scroll.tick(state.scroll_y, now, max_y) {
        // 2. Vertical axis: Check SmoothScroller (smooth mouse wheel lines)
        state.scroll_y = next_y;
        state.scroll_controller.set_position_y(next_y);
        changed = true;
    }

    // 3. Horizontal axis: Check ScrollController (touchpad kinetic fling & inertia)
    if (state.scroll_controller_x.is_animating()
        || state.scroll_controller_x.state() == crate::core::scroll::GestureState::Dragging)
        && let Some(next_x) = state.scroll_controller_x.update(now, extent_x)
    {
        state.scroll_x = next_x;
        state.smooth_scroll_x.stop(next_x);
        changed = true;
    } else if let Some(next_x) = state.smooth_scroll_x.tick(state.scroll_x, now, max_x) {
        // 4. Horizontal axis: Check SmoothScroller (smooth shift+wheel lines)
        state.scroll_x = next_x;
        state.scroll_controller_x.set_position_y(next_x);
        changed = true;
    }

    if changed {
        iced::widget::operation::scroll_to(
            CONTENT_SCROLL_ID,
            iced::widget::operation::AbsoluteOffset {
                x: state.scroll_x,
                y: state.scroll_y,
            },
        )
    } else {
        Task::none()
    }
}

pub fn handle_auto_scroll_tick(app: &mut KglanceApp) -> Task<Message> {
    let state = &mut app.state.spreadsheet;
    if !state.is_dragging && !state.is_dragging_row_headers {
        state.auto_scroll_delta_y = None;
        state.auto_scroll_delta_x = None;
        return Task::none();
    }

    let mut changed = false;

    // 1. Vertical auto-scroll
    if let Some(dy) = state.auto_scroll_delta_y {
        let extent_y = crate::core::scroll::ViewportExtent {
            content_height: state.total_content_height,
            viewport_height: state.viewport_height,
        };
        let max_y = extent_y.max_scroll_y();
        let next_y = (state.scroll_y + dy).clamp(0.0, max_y);
        if (next_y - state.scroll_y).abs() > f32::EPSILON {
            state.scroll_y = next_y;
            state.smooth_scroll.stop(next_y);
            state.scroll_controller.stop(next_y);
            changed = true;
        }
    }

    // 2. Horizontal auto-scroll
    if let Some(dx) = state.auto_scroll_delta_x {
        let extent_x = crate::core::scroll::ViewportExtent {
            content_height: state.total_content_width,
            viewport_height: state.viewport_width,
        };
        let max_x = extent_x.max_scroll_y();
        let next_x = (state.scroll_x + dx).clamp(0.0, max_x);
        if (next_x - state.scroll_x).abs() > f32::EPSILON {
            state.scroll_x = next_x;
            state.smooth_scroll_x.stop(next_x);
            state.scroll_controller_x.stop(next_x);
            changed = true;
        }
    }

    // 3. Update selection endpoint based on current scroll position and cursor
    if let Some(ref mut selection) = state.selection {
        let target_y = (state.scroll_y + state.drag_last_cursor.y).max(0.0);
        let row_idx = state
            .prefix_heights
            .partition_point(|&h| h <= target_y)
            .saturating_sub(1)
            .min(state.display_indices.len().saturating_sub(1));

        if state.is_dragging_row_headers {
            let max_col = state
                .sheets
                .get(state.active_sheet)
                .map_or(0, |s| s.columns.len().saturating_sub(1));
            selection.end = crate::features::sheet::CellCoord {
                row: row_idx,
                col: max_col,
            };
        } else if state.is_dragging {
            let target_x = (state.scroll_x + state.drag_last_cursor.x).max(0.0);
            let col_count = state
                .sheets
                .get(state.active_sheet)
                .map_or(0, |s| s.columns.len());
            let max_col = col_count.saturating_sub(1);
            let col_idx = state
                .prefix_widths
                .partition_point(|&w| w <= target_x)
                .saturating_sub(1)
                .min(max_col);

            selection.end = crate::features::sheet::CellCoord {
                row: row_idx,
                col: col_idx,
            };
        }
    }

    if changed {
        iced::widget::operation::scroll_to(
            CONTENT_SCROLL_ID,
            iced::widget::operation::AbsoluteOffset {
                x: state.scroll_x,
                y: state.scroll_y,
            },
        )
    } else {
        Task::none()
    }
}
