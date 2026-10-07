use crate::app::Message;
use crate::core::SpreadsheetState;
use crate::features::sheet::types::{ColumnMeta, ColumnType, SheetInfo};
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::tokens::spacing;
use crate::ui::theme::{
    AppTheme, default_button, default_button_primary, default_card, default_tooltip,
};
use iced::alignment;
use iced::widget::{button, column, container, row, text, tooltip};
use iced::{Element, Length, Theme};

const SORT_ASCENDING_INDICATOR: &str = " ▲";
const SORT_DESCENDING_INDICATOR: &str = " ▼";
const SORT_NONE_INDICATOR: &str = "";

const CELL_TEXT_SIZE: f32 = 13.0;
const ROW_NUM_TEXT_SIZE: f32 = 12.0;
const EMPTY_STATE_TEXT_SIZE: f32 = 16.0;

const TAB_SPACING: f32 = spacing::XS;
const MAIN_SPACING: f32 = spacing::XS;
pub const COL_SPACING: f32 = spacing::XXS;
pub const ROWS_LIST_SPACING: f32 = 2.0;

const CARD_PADDING: [u16; 2] = [spacing::XS as u16, spacing::S as u16];
pub const ROW_NUMBER_COL_WIDTH: f32 = 52.0;

pub const MIN_ROW_HEIGHT: f32 = 28.0;
pub const ROW_HEIGHT: f32 = MIN_ROW_HEIGHT;
pub const HEADER_HEIGHT: f32 = 30.0;
pub const ROW_STEP: f32 = MIN_ROW_HEIGHT + ROWS_LIST_SPACING;
pub const LINE_HEIGHT: f32 = 18.0;
pub const CELL_PADDING_Y: f32 = 8.0;

/// Estimates dynamic row height based on cell text lengths and column widths.
pub fn estimate_row_height(row_data: &[String], columns: &[ColumnMeta]) -> f32 {
    let mut max_lines = 1;
    for (col_idx, col_meta) in columns.iter().enumerate() {
        if let Some(cell) = row_data.get(col_idx) {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }
            let inner_width = (col_meta.width - 12.0).max(40.0);
            let chars_per_line = ((inner_width / 7.5).floor() as usize).max(1);

            let mut cell_lines = 0;
            for line in trimmed.split('\n') {
                let char_count = line.chars().count();
                let wrapped = char_count.div_ceil(chars_per_line);
                cell_lines += wrapped.max(1);
            }
            if cell_lines > max_lines {
                max_lines = cell_lines;
            }
        }
    }
    (max_lines as f32 * LINE_HEIGHT + CELL_PADDING_Y).clamp(MIN_ROW_HEIGHT, 300.0)
}

pub fn view_spreadsheet<'a>(state: &'a SpreadsheetState, theme: AppTheme) -> Element<'a, Message> {
    let active_sheet = state.sheets.get(state.active_sheet);

    let tabs_bar = render_sheet_tabs(&state.sheets, state.active_sheet);
    let content_body = match active_sheet {
        Some(sheet) => render_spreadsheet_body(state, sheet, theme),
        None => render_empty_state("No spreadsheet data loaded"),
    };

    column![tabs_bar, content_body].spacing(MAIN_SPACING).into()
}

fn render_sheet_tabs<'a>(
    sheets: &'a [SheetInfo],
    active_sheet_index: usize,
) -> Element<'a, Message> {
    if sheets.len() <= 1 {
        return container(row![]).into();
    }

    let mut tabs_row = row![].spacing(TAB_SPACING);

    for (index, sheet) in sheets.iter().enumerate() {
        let is_active = index == active_sheet_index;
        let tab_button = button(text(&sheet.name))
            .on_press(crate::app::messages::SpreadsheetMsg::SheetTabClicked(index).into())
            .style(if is_active {
                default_button_primary
            } else {
                default_button
            });

        tabs_row = tabs_row.push(tab_button);
    }

    container(tabs_row)
        .style(default_card)
        .padding(CARD_PADDING)
        .into()
}

fn render_spreadsheet_body<'a>(
    state: &'a SpreadsheetState,
    sheet: &'a SheetInfo,
    theme: AppTheme,
) -> Element<'a, Message> {
    if sheet.rows.is_empty() {
        return render_empty_state("Sheet is empty");
    }

    let mut layout = column![].spacing(MAIN_SPACING);

    if state.search_visible {
        layout = layout.push(search_bar(
            SearchKind::Spreadsheet,
            &state.search_query,
            None,
        ));
    }

    let header = render_table_header(&sheet.columns, state, theme);

    if state.display_indices.is_empty() {
        layout = layout.push(header);
        layout = layout.push(
            container(
                text("No matching rows found")
                    .size(EMPTY_STATE_TEXT_SIZE)
                    .style(move |_: &Theme| text::Style {
                        color: Some(theme.palette().base.text_dim),
                    }),
            )
            .padding(spacing::M as u16),
        );
        return layout.into();
    }

    let rows_effective_scroll_y = (state.scroll_y - (HEADER_HEIGHT + ROWS_LIST_SPACING)).max(0.0);
    let rows_list = render_table_rows(state, sheet, theme, rows_effective_scroll_y);

    let table_content = column![header, rows_list].spacing(ROWS_LIST_SPACING);

    let scrollable_area =
        crate::ui::components::scroll_pane::scroll_pane("content_scroll", table_content)
            .direction(iced::widget::scrollable::Direction::Both {
                vertical: iced::widget::scrollable::Scrollbar::new()
                    .width(4)
                    .scroller_width(4)
                    .margin(2),
                horizontal: iced::widget::scrollable::Scrollbar::new()
                    .width(4)
                    .scroller_width(4)
                    .margin(2),
            })
            .filter_wheel(true)
            .on_scroll(|vp| crate::app::messages::SpreadsheetMsg::Scrolled(vp).into())
            .on_wheel(|delta| crate::app::messages::SpreadsheetMsg::WheelScrolled(delta).into())
            .build();

    layout = layout.push(scrollable_area);
    layout.into()
}

fn render_table_header<'a>(
    columns: &'a [ColumnMeta],
    state: &'a SpreadsheetState,
    theme: AppTheme,
) -> Element<'a, Message> {
    let col_window =
        compute_csv_column_window(&state.prefix_widths, state.scroll_x, state.viewport_width);
    let sort_column = state.sort_col;
    let sort_ascending = state.sort_ascending;
    let mut header_row = row![].spacing(COL_SPACING);

    // Fixed row number header column
    let row_num_header = container(
        text("#")
            .size(ROW_NUM_TEXT_SIZE)
            .align_x(alignment::Horizontal::Center)
            .wrapping(text::Wrapping::None)
            .width(Length::Fill),
    )
    .clip(true)
    .width(Length::Fixed(ROW_NUMBER_COL_WIDTH))
    .height(Length::Fixed(HEADER_HEIGHT))
    .align_y(alignment::Vertical::Center);

    header_row = header_row.push(row_num_header);

    if col_window.visible_start > 0 && col_window.left_spacer_width > 0.0 {
        header_row =
            header_row.push(iced::widget::Space::new().width(col_window.left_spacer_width));
    }

    for (col_offset, col_meta) in columns[col_window.visible_start..col_window.visible_end]
        .iter()
        .enumerate()
    {
        let column_index = col_window.visible_start + col_offset;
        let indicator = get_sort_indicator(column_index, sort_column, sort_ascending);
        let button_label = format!("{}{}", col_meta.name, indicator);

        let header_button = button(
            text(button_label)
                .size(CELL_TEXT_SIZE)
                .align_x(alignment::Horizontal::Left)
                .wrapping(text::Wrapping::None)
                .width(Length::Fill),
        )
        .on_press(crate::app::messages::SpreadsheetMsg::ColumnClicked(column_index).into())
        .style(if sort_column == Some(column_index) {
            default_button_primary
        } else {
            default_button
        })
        .clip(true)
        .width(Length::Fixed(col_meta.width))
        .height(Length::Fixed(HEADER_HEIGHT));

        let tooltip_content =
            container(
                text(&col_meta.name)
                    .size(12.0)
                    .style(move |_: &Theme| text::Style {
                        color: Some(theme.palette().base.text),
                    }),
            )
            .padding([4, 8])
            .style(default_tooltip);

        let header_widget = tooltip(header_button, tooltip_content, tooltip::Position::Bottom)
            .gap(4.0)
            .snap_within_viewport(true);

        header_row = header_row.push(header_widget);
    }

    if col_window.visible_end < columns.len() && col_window.right_spacer_width > 0.0 {
        header_row =
            header_row.push(iced::widget::Space::new().width(col_window.right_spacer_width));
    }

    container(header_row)
        .height(Length::Fixed(HEADER_HEIGHT))
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CsvColumnWindow {
    pub visible_start: usize,
    pub visible_end: usize,
    pub left_spacer_width: f32,
    pub right_spacer_width: f32,
}

pub fn compute_csv_column_window(
    prefix_widths: &[f32],
    scroll_x: f32,
    viewport_width: f32,
) -> CsvColumnWindow {
    let total_cols = prefix_widths.len().saturating_sub(1);
    if total_cols == 0 {
        return CsvColumnWindow {
            visible_start: 0,
            visible_end: 0,
            left_spacer_width: 0.0,
            right_spacer_width: 0.0,
        };
    }

    let vw = if viewport_width > 0.0 {
        viewport_width
    } else {
        1000.0
    };

    let total_width = *prefix_widths.last().unwrap_or(&0.0);
    let col_scroll_x = (scroll_x - (ROW_NUMBER_COL_WIDTH + COL_SPACING))
        .max(0.0)
        .min(total_width);

    let start_idx = match prefix_widths.binary_search_by(|&w| {
        if w <= col_scroll_x {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };

    let col_overscan = 2;
    let visible_start = start_idx.saturating_sub(col_overscan);

    let end_x = col_scroll_x + vw;
    let end_idx = match prefix_widths.binary_search_by(|&w| {
        if w <= end_x {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }) {
        Ok(idx) => idx + col_overscan,
        Err(idx) => idx + col_overscan,
    };

    let visible_end = end_idx.clamp(visible_start + 1, total_cols);

    let left_spacer_width = if visible_start > 0 {
        (prefix_widths[visible_start] - COL_SPACING).max(0.0)
    } else {
        0.0
    };

    let right_spacer_width = if visible_end < total_cols {
        (total_width - prefix_widths[visible_end] - COL_SPACING).max(0.0)
    } else {
        0.0
    };

    CsvColumnWindow {
        visible_start,
        visible_end,
        left_spacer_width,
        right_spacer_width,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CsvVirtualWindow {
    pub visible_start: usize,
    pub visible_end: usize,
    pub top_spacer_height: f32,
    pub bottom_spacer_height: f32,
}

pub fn compute_csv_virtual_window(
    prefix_heights: &[f32],
    scroll_y: f32,
    viewport_height: f32,
) -> CsvVirtualWindow {
    let total_rows = prefix_heights.len().saturating_sub(1);
    if total_rows == 0 {
        return CsvVirtualWindow {
            visible_start: 0,
            visible_end: 0,
            top_spacer_height: 0.0,
            bottom_spacer_height: 0.0,
        };
    }

    let vh = if viewport_height > 0.0 {
        viewport_height
    } else {
        800.0
    };

    let total_height = *prefix_heights.last().unwrap_or(&0.0);
    let clamped_scroll_y = scroll_y.clamp(0.0, total_height);

    // Binary search for visible start
    let start_idx = match prefix_heights.binary_search_by(|&h| {
        if h <= clamped_scroll_y {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };

    let overscan = 4;
    let visible_start = start_idx.saturating_sub(overscan);

    let end_y = clamped_scroll_y + vh;
    let end_idx = match prefix_heights.binary_search_by(|&h| {
        if h <= end_y {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }) {
        Ok(idx) => idx + overscan,
        Err(idx) => idx + overscan,
    };

    let visible_end = end_idx.min(total_rows);

    let top_spacer_height = prefix_heights[visible_start];
    let bottom_spacer_height = (total_height - prefix_heights[visible_end]).max(0.0);

    CsvVirtualWindow {
        visible_start,
        visible_end,
        top_spacer_height,
        bottom_spacer_height,
    }
}

fn render_table_rows<'a>(
    state: &'a SpreadsheetState,
    sheet: &'a SheetInfo,
    theme: AppTheme,
    scroll_y: f32,
) -> Element<'a, Message> {
    let window = compute_csv_virtual_window(&state.prefix_heights, scroll_y, state.viewport_height);
    if window.visible_end == 0 {
        return column![].into();
    }
    let col_win =
        compute_csv_column_window(&state.prefix_widths, state.scroll_x, state.viewport_width);

    let mut rows_list = column![].spacing(ROWS_LIST_SPACING);

    if window.visible_start > 0 && window.top_spacer_height > 0.0 {
        let adjusted_top = (window.top_spacer_height - ROWS_LIST_SPACING).max(0.0);
        rows_list = rows_list.push(iced::widget::Space::new().height(adjusted_top));
    }

    for (display_offset, &original_row_idx) in state.display_indices
        [window.visible_start..window.visible_end]
        .iter()
        .enumerate()
    {
        let Some(row_data) = sheet.rows.get(original_row_idx) else {
            continue;
        };

        let row_idx = window.visible_start + display_offset;
        let row_h = state
            .row_heights
            .get(row_idx)
            .copied()
            .unwrap_or(MIN_ROW_HEIGHT);

        let mut row_widget = row![].spacing(COL_SPACING);

        // Row number column (#) - exactly matching row_h
        let row_num_text = format!("{}", original_row_idx + 1);
        let row_num_container = container(
            text(row_num_text)
                .size(ROW_NUM_TEXT_SIZE)
                .style(move |_: &Theme| text::Style {
                    color: Some(theme.palette().base.text_dim),
                })
                .align_x(alignment::Horizontal::Center)
                .wrapping(text::Wrapping::None)
                .width(Length::Fill),
        )
        .width(Length::Fixed(ROW_NUMBER_COL_WIDTH))
        .height(Length::Fixed(row_h))
        .align_y(alignment::Vertical::Center);

        row_widget = row_widget.push(row_num_container);

        if col_win.visible_start > 0 && col_win.left_spacer_width > 0.0 {
            row_widget =
                row_widget.push(iced::widget::Space::new().width(col_win.left_spacer_width));
        }

        let is_single_line = row_h <= MIN_ROW_HEIGHT;

        for (col_offset, col_meta) in sheet.columns[col_win.visible_start..col_win.visible_end]
            .iter()
            .enumerate()
        {
            let col_idx = col_win.visible_start + col_offset;
            let cell_text = row_data.get(col_idx).map(String::as_str).unwrap_or("");

            let text_align = match col_meta.col_type {
                ColumnType::Integer | ColumnType::Float => alignment::Horizontal::Right,
                ColumnType::Date => alignment::Horizontal::Center,
                ColumnType::Text | ColumnType::Empty => alignment::Horizontal::Left,
            };

            let wrapping = if is_single_line {
                text::Wrapping::None
            } else {
                text::Wrapping::WordOrGlyph
            };

            let cell_container = container(
                text(cell_text)
                    .size(CELL_TEXT_SIZE)
                    .align_x(text_align)
                    .wrapping(wrapping)
                    .width(Length::Fill),
            )
            .width(Length::Fixed(col_meta.width))
            .height(Length::Fixed(row_h))
            .padding([2, 4])
            .align_y(alignment::Vertical::Center);

            row_widget = row_widget.push(cell_container);
        }

        if col_win.visible_end < sheet.columns.len() && col_win.right_spacer_width > 0.0 {
            row_widget =
                row_widget.push(iced::widget::Space::new().width(col_win.right_spacer_width));
        }

        let is_even_row = row_idx.is_multiple_of(2);
        let row_container =
            container(row_widget)
                .height(Length::Fixed(row_h))
                .style(move |_: &Theme| {
                    if is_even_row {
                        apply_even_row_style(theme)
                    } else {
                        container::Style::default()
                    }
                });

        rows_list = rows_list.push(row_container);
    }

    if window.bottom_spacer_height > 0.0 {
        rows_list = rows_list.push(iced::widget::Space::new().height(window.bottom_spacer_height));
    }

    rows_list.into()
}

fn get_sort_indicator(
    column_index: usize,
    sort_column: Option<usize>,
    sort_ascending: Option<bool>,
) -> &'static str {
    if Some(column_index) != sort_column {
        return SORT_NONE_INDICATOR;
    }

    match sort_ascending {
        Some(true) => SORT_ASCENDING_INDICATOR,
        Some(false) => SORT_DESCENDING_INDICATOR,
        None => SORT_NONE_INDICATOR,
    }
}

fn apply_even_row_style(theme: AppTheme) -> container::Style {
    container::Style {
        background: Some(theme.palette().base.surface.into()),
        ..container::Style::default()
    }
}

fn render_empty_state<'a>(message: &'a str) -> Element<'a, Message> {
    container(text(message).size(EMPTY_STATE_TEXT_SIZE))
        .padding(spacing::M as u16)
        .into()
}
