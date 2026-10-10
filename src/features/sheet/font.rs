use iced::Task;

use crate::app::{KglanceApp, Message};
use crate::core::SpreadsheetState;
use crate::features::sheet::view::{
    BOTTOM_BAR_HEIGHT, CELL_PADDING_X, CELL_PADDING_Y, CELL_TEXT_SIZE, EMPTY_STATE_TEXT_SIZE,
    HEADER_HEIGHT, HEADER_TEXT_SIZE, LINE_HEIGHT, MIN_ROW_HEIGHT, ROW_NUM_TEXT_SIZE,
    ROW_NUMBER_COL_WIDTH, ROWS_LIST_SPACING, TAB_TEXT_SIZE,
};
use crate::features::text::update::{FONT_MAX, FONT_MIN};

#[inline]
pub fn scale_ratio(user_font_size: f32) -> f32 {
    user_font_size / CELL_TEXT_SIZE
}

#[inline]
pub fn scale_size(design_size: f32, user_font_size: f32) -> f32 {
    (design_size * scale_ratio(user_font_size))
        .round()
        .max(FONT_MIN)
}

#[inline]
pub fn cell_text_size(font_size: f32) -> f32 {
    scale_size(CELL_TEXT_SIZE, font_size)
}

#[inline]
pub fn row_num_text_size(font_size: f32) -> f32 {
    scale_size(ROW_NUM_TEXT_SIZE, font_size)
}

#[inline]
pub fn header_text_size(font_size: f32) -> f32 {
    scale_size(HEADER_TEXT_SIZE, font_size)
}

#[inline]
pub fn tab_text_size(font_size: f32) -> f32 {
    scale_size(TAB_TEXT_SIZE, font_size)
}

#[inline]
pub fn empty_state_text_size(font_size: f32) -> f32 {
    scale_size(EMPTY_STATE_TEXT_SIZE, font_size)
}

#[inline]
pub fn header_height(font_size: f32) -> f32 {
    (HEADER_HEIGHT * scale_ratio(font_size)).round()
}

#[inline]
pub fn min_row_height(font_size: f32) -> f32 {
    (MIN_ROW_HEIGHT * scale_ratio(font_size)).round()
}

#[inline]
pub fn row_number_col_width(font_size: f32) -> f32 {
    (ROW_NUMBER_COL_WIDTH * scale_ratio(font_size)).round()
}

#[inline]
pub fn bottom_bar_height(font_size: f32) -> f32 {
    (BOTTOM_BAR_HEIGHT * scale_ratio(font_size)).round()
}

#[inline]
pub fn line_height(font_size: f32) -> f32 {
    LINE_HEIGHT * scale_ratio(font_size)
}

#[inline]
pub fn cell_padding_y(font_size: f32) -> f32 {
    (CELL_PADDING_Y * scale_ratio(font_size)).round()
}

#[inline]
pub fn cell_padding_x(font_size: f32) -> f32 {
    (CELL_PADDING_X * scale_ratio(font_size)).round()
}

#[inline]
pub fn scale_col_width(base_width: f32, font_size: f32) -> f32 {
    (base_width * scale_ratio(font_size)).round()
}

pub fn rescale_spreadsheet_geometry(
    state: &mut SpreadsheetState,
    old_size: f32,
    new_size: f32,
) -> f32 {
    if let Some(sheet) = state.sheets.get(state.active_sheet) {
        let (row_heights, prefix_heights, total_h) =
            super::update::compute_height_structures(sheet, &state.display_indices, new_size);
        let prefix_widths = super::update::compute_prefix_widths(&sheet.columns, new_size);
        state.row_heights = row_heights;
        state.prefix_heights = prefix_heights;
        state.prefix_widths = prefix_widths;
        state.total_content_height = total_h + header_height(new_size) + ROWS_LIST_SPACING;
        state.total_content_width = super::update::compute_total_content_width(sheet, new_size);
    }
    let ratio = if old_size > 0.0 {
        new_size / old_size
    } else {
        1.0
    };
    let new_scroll_y = (state.scroll_y * ratio).max(0.0);
    let new_scroll_x = (state.scroll_x * ratio).max(0.0);
    state.scroll_y = new_scroll_y;
    state.scroll_x = new_scroll_x;
    state.smooth_scroll.stop(new_scroll_y);
    state.scroll_controller.stop(new_scroll_y);
    state.smooth_scroll_x.stop(new_scroll_x);
    state.scroll_controller_x.stop(new_scroll_x);
    new_scroll_y
}

pub fn zoom_spreadsheet_font(app: &mut KglanceApp, direction: f32) -> Option<Task<Message>> {
    let target = app.state.font_size + direction;
    rescale_spreadsheet_font(app, target)
}

pub fn rescale_spreadsheet_font(app: &mut KglanceApp, new_size: f32) -> Option<Task<Message>> {
    let old_size = app.state.font_size;
    let new_size = new_size.clamp(FONT_MIN, FONT_MAX);
    if (new_size - old_size).abs() < f32::EPSILON {
        return Some(Task::none());
    }
    app.state.font_size = new_size;

    let mut config = app.load_config();
    config.ui.font_size = new_size;
    let _ = app.save_config(&config);

    app.state
        .toasts
        .retain(|t| !t.message.starts_with("Font Size:"));
    let toast = app.show_toast(format!("Font Size: {new_size:.0}px"));

    rescale_spreadsheet_geometry(&mut app.state.spreadsheet, old_size, new_size);

    let scroll_task = iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: app.state.spreadsheet.scroll_x,
            y: app.state.spreadsheet.scroll_y,
        },
    );
    Some(Task::batch([scroll_task, toast]))
}
