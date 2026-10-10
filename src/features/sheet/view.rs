use crate::app::Message;
use crate::core::SpreadsheetState;
use crate::features::sheet::types::{ColumnMeta, ColumnType, SheetInfo};
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::AppTheme;
use crate::ui::theme::color::primitive;
use crate::ui::theme::tokens::{radius, spacing, typography};
use iced::alignment;
use iced::widget::{button, column, container, mouse_area, row, text};
use iced::{Border, Color, Element, Length, Shadow, Theme};

use crate::features::sheet::font::{
    bottom_bar_height, cell_padding_x, cell_padding_y, cell_text_size, empty_state_text_size,
    header_height, header_text_size, line_height, min_row_height, row_num_text_size,
    row_number_col_width, scale_col_width, scale_ratio, tab_text_size,
};

const SORT_ASCENDING_INDICATOR: &str = " ▲";
const SORT_DESCENDING_INDICATOR: &str = " ▼";
const SORT_NONE_INDICATOR: &str = "";

pub const CELL_TEXT_SIZE: f32 = typography::BODY;
pub const ROW_NUM_TEXT_SIZE: f32 = typography::CAPTION;
pub const HEADER_TEXT_SIZE: f32 = 11.5;
pub const TAB_TEXT_SIZE: f32 = typography::BODY;
pub const EMPTY_STATE_TEXT_SIZE: f32 = typography::BODY_LG;

pub const COL_SPACING: f32 = 0.0;
pub const ROWS_LIST_SPACING: f32 = 0.0;

pub const ROW_NUMBER_COL_WIDTH: f32 = 48.0;
pub const MIN_ROW_HEIGHT: f32 = 26.0;
pub const ROW_HEIGHT: f32 = MIN_ROW_HEIGHT;
pub const HEADER_HEIGHT: f32 = 26.0;
pub const BOTTOM_BAR_HEIGHT: f32 = 32.0;
pub const ROW_STEP: f32 = MIN_ROW_HEIGHT + ROWS_LIST_SPACING;
pub const LINE_HEIGHT: f32 = 17.5;
pub const CELL_PADDING_Y: f32 = 6.0;
pub const CELL_PADDING_X: f32 = 6.0;

/// Estimates row height accurately based on realistic word-wrapping and line bounds.
pub fn estimate_row_height(row_data: &[String], columns: &[ColumnMeta], font_size: f32) -> f32 {
    let min_row_h = min_row_height(font_size);
    let line_h = line_height(font_size);
    let pad_y = cell_padding_y(font_size);
    let pad_x = cell_padding_x(font_size);
    let approx_char_w = 6.8 * scale_ratio(font_size);

    let mut max_lines = 1;
    for (col_idx, col_meta) in columns.iter().enumerate() {
        if let Some(cell) = row_data.get(col_idx) {
            let trimmed = cell.trim();
            if trimmed.is_empty() {
                continue;
            }
            let col_w = scale_col_width(col_meta.width, font_size);
            let inner_width = (col_w - (pad_x * 2.0)).max(30.0);
            let chars_per_line = ((inner_width / approx_char_w).floor() as usize).max(1);

            let mut cell_lines = 0;
            for raw_line in trimmed.split('\n') {
                let mut cur_line_len = 0;
                let mut seg_lines = 1;
                for word in raw_line.split_whitespace() {
                    let w_len = word.chars().count();
                    if cur_line_len == 0 {
                        cur_line_len = w_len;
                    } else if cur_line_len + 1 + w_len <= chars_per_line {
                        cur_line_len += 1 + w_len;
                    } else {
                        seg_lines += 1;
                        cur_line_len = w_len;
                    }
                }
                cell_lines += seg_lines;
            }
            if cell_lines > max_lines {
                max_lines = cell_lines;
            }
        }
    }
    (max_lines as f32 * line_h + pad_y).clamp(min_row_h, 500.0)
}

pub fn view_spreadsheet<'a>(
    state: &'a SpreadsheetState,
    font_size: f32,
    theme: AppTheme,
    font_family: Option<&str>,
) -> Element<'a, Message> {
    let main_font = crate::ui::theme::font::get_main_font(font_family);
    let active_sheet = state.sheets.get(state.active_sheet);

    let content_body = match active_sheet {
        Some(sheet) => render_spreadsheet_body(state, sheet, theme, main_font, font_size),
        None => render_empty_state("No spreadsheet data loaded", theme, main_font),
    };

    if state.sheets.len() > 1 {
        let bottom_bar = render_bottom_sheet_bar(
            &state.sheets,
            state.active_sheet,
            theme,
            main_font,
            font_size,
        );
        column![content_body, bottom_bar].into()
    } else {
        content_body
    }
}

fn render_bottom_sheet_bar<'a>(
    sheets: &'a [SheetInfo],
    active_sheet_index: usize,
    theme: AppTheme,
    main_font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    if sheets.len() <= 1 {
        return container(row![]).height(Length::Shrink).into();
    }

    let mut tabs_row = row![]
        .spacing(spacing::XS)
        .align_y(alignment::Vertical::Center);

    for (index, sheet) in sheets.iter().enumerate() {
        let is_active = index == active_sheet_index;
        let tab_button = button(
            text(&sheet.name)
                .size(tab_text_size(font_size))
                .font(main_font)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(crate::app::messages::SpreadsheetMsg::SheetTabClicked(index).into())
        .style(sheet_tab_button_style(theme, is_active))
        .padding([2, 10]);

        tabs_row = tabs_row.push(tab_button);
    }

    container(
        scroll_pane("sheet_tabs_scroll", tabs_row)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::new()
                    .width(0)
                    .scroller_width(0)
                    .margin(0),
            ))
            .build(),
    )
    .width(Length::Fill)
    .height(Length::Fixed(bottom_bar_height(font_size)))
    .padding([2, 6])
    .align_y(alignment::Vertical::Center)
    .style(move |_: &Theme| bottom_bar_style(theme))
    .into()
}

fn render_spreadsheet_body<'a>(
    state: &'a SpreadsheetState,
    sheet: &'a SheetInfo,
    theme: AppTheme,
    main_font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    if sheet.rows.is_empty() {
        return render_empty_state("Sheet is empty", theme, main_font);
    }

    let mut layout = column![];

    if state.search_visible {
        layout = layout.push(
            container(search_bar(
                SearchKind::Spreadsheet,
                &state.search_query,
                None,
            ))
            .padding([3, 6]),
        );
    }

    let header = render_table_header(&sheet.columns, state, theme, main_font, font_size);

    if state.display_indices.is_empty() {
        layout = layout.push(header);
        layout = layout.push(
            container(
                text("No matching rows found")
                    .size(empty_state_text_size(font_size))
                    .font(main_font)
                    .style(move |_: &Theme| text::Style {
                        color: Some(theme.palette().base.text_dim),
                    }),
            )
            .padding(spacing::M as u16),
        );
        return layout.into();
    }

    let rows_effective_scroll_y =
        (state.scroll_y - (header_height(font_size) + ROWS_LIST_SPACING)).max(0.0);
    let rows_list = render_table_rows(
        state,
        sheet,
        theme,
        rows_effective_scroll_y,
        main_font,
        font_size,
    );

    let table_content = column![header, rows_list].spacing(ROWS_LIST_SPACING);

    let scrollable_area = scroll_pane("content_scroll", table_content)
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
    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_table_header<'a>(
    columns: &'a [ColumnMeta],
    state: &'a SpreadsheetState,
    theme: AppTheme,
    main_font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    let row_num_w = row_number_col_width(font_size);
    let header_h = header_height(font_size);
    let header_text_sz = header_text_size(font_size);
    let col_window = compute_csv_column_window(
        &state.prefix_widths,
        state.scroll_x,
        state.viewport_width,
        row_num_w,
    );
    let sort_column = state.sort_col;
    let sort_ascending = state.sort_ascending;
    let mut header_row = row![].spacing(COL_SPACING);

    // Fixed corner cell above row numbers
    let row_num_header = container(
        text("")
            .size(header_text_sz)
            .font(main_font)
            .align_x(alignment::Horizontal::Center)
            .wrapping(text::Wrapping::None)
            .width(Length::Fill),
    )
    .clip(true)
    .width(Length::Fixed(row_num_w))
    .height(Length::Fixed(header_h))
    .align_y(alignment::Vertical::Center)
    .style(move |_: &Theme| grid_header_cell_style(theme));

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
        let name = &col_meta.name;
        let button_label = format!("{name}{indicator}");
        let is_sorted = sort_column == Some(column_index);
        let col_w = scale_col_width(col_meta.width, font_size);

        let header_button = button(
            text(button_label)
                .size(header_text_sz)
                .font(main_font)
                .align_x(alignment::Horizontal::Center)
                .wrapping(text::Wrapping::None)
                .width(Length::Fill),
        )
        .on_press(crate::app::messages::SpreadsheetMsg::ColumnClicked(column_index).into())
        .style(grid_header_button_style(theme, is_sorted))
        .clip(true)
        .padding([0, 4])
        .width(Length::Fixed(col_w))
        .height(Length::Fixed(header_h));

        header_row = header_row.push(header_button);
    }

    if col_window.visible_end < columns.len() && col_window.right_spacer_width > 0.0 {
        header_row =
            header_row.push(iced::widget::Space::new().width(col_window.right_spacer_width));
    }

    container(header_row).height(Length::Fixed(header_h)).into()
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
    row_number_col_width: f32,
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
    let col_scroll_x = (scroll_x - (row_number_col_width + COL_SPACING))
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
    main_font: iced::Font,
    font_size: f32,
) -> Element<'a, Message> {
    let window = compute_csv_virtual_window(&state.prefix_heights, scroll_y, state.viewport_height);
    if window.visible_end == 0 {
        return column![].into();
    }
    let row_num_w = row_number_col_width(font_size);
    let min_row_h = min_row_height(font_size);
    let col_win = compute_csv_column_window(
        &state.prefix_widths,
        state.scroll_x,
        state.viewport_width,
        row_num_w,
    );

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
        let row_h = state.row_heights.get(row_idx).copied().unwrap_or(min_row_h);

        let mut row_widget = row![].spacing(COL_SPACING);
        let is_multiline = row_h > (min_row_h + 2.0);
        let is_row_selected = state.selection.is_some_and(|s| {
            let (min_r, max_r) = s.row_range();
            row_idx >= min_r && row_idx <= max_r
        });

        // Row number column (#) - clickable to select row
        let row_num = original_row_idx + 1;
        let row_num_text = format!("{row_num}");
        let row_num_content = container(
            text(row_num_text)
                .size(row_num_text_size(font_size))
                .font(main_font)
                .align_x(alignment::Horizontal::Center)
                .wrapping(text::Wrapping::None)
                .width(Length::Fill),
        )
        .align_y(if is_multiline {
            alignment::Vertical::Top
        } else {
            alignment::Vertical::Center
        })
        .height(Length::Fill);

        let row_num_container = container(row_num_content)
            .style(move |_: &Theme| grid_row_num_style(theme, is_row_selected))
            .clip(true)
            .width(Length::Fixed(row_num_w))
            .height(Length::Fixed(row_h))
            .padding([cell_padding_y(font_size) / 2.0, 2.0]);

        let row_num_widget = mouse_area(row_num_container)
            .on_press(crate::app::messages::SpreadsheetMsg::RowHeaderPressed(row_idx).into())
            .on_enter(crate::app::messages::SpreadsheetMsg::RowHeaderEntered(row_idx).into())
            .on_release(crate::app::messages::SpreadsheetMsg::CellReleased.into());

        row_widget = row_widget.push(row_num_widget);

        if col_win.visible_start > 0 && col_win.left_spacer_width > 0.0 {
            row_widget =
                row_widget.push(iced::widget::Space::new().width(col_win.left_spacer_width));
        }

        for (col_offset, col_meta) in sheet.columns[col_win.visible_start..col_win.visible_end]
            .iter()
            .enumerate()
        {
            let col_idx = col_win.visible_start + col_offset;
            let cell_text = row_data.get(col_idx).map(String::as_str).unwrap_or("");
            let is_selected = state
                .selection
                .is_some_and(|s| s.contains(row_idx, col_idx));
            let is_primary = state
                .selection
                .is_some_and(|s| s.start.row == row_idx && s.start.col == col_idx);

            let text_align = match col_meta.col_type {
                ColumnType::Integer | ColumnType::Float => alignment::Horizontal::Right,
                ColumnType::Date => alignment::Horizontal::Center,
                ColumnType::Text | ColumnType::Empty => alignment::Horizontal::Left,
            };

            let col_w = scale_col_width(col_meta.width, font_size);
            let cell_content = container(
                text(cell_text)
                    .size(cell_text_size(font_size))
                    .font(main_font)
                    .align_x(text_align)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .width(Length::Fill),
            )
            .align_y(if is_multiline {
                alignment::Vertical::Top
            } else {
                alignment::Vertical::Center
            })
            .height(Length::Fill);

            let cell_container = container(cell_content)
                .style(move |_: &Theme| grid_data_cell_style(theme, is_selected, is_primary))
                .clip(true)
                .width(Length::Fixed(col_w))
                .height(Length::Fixed(row_h))
                .padding([cell_padding_y(font_size) / 2.0, cell_padding_x(font_size)]);

            let cell_widget = mouse_area(cell_container)
                .on_press(
                    crate::app::messages::SpreadsheetMsg::CellPressed {
                        row: row_idx,
                        col: col_idx,
                    }
                    .into(),
                )
                .on_enter(
                    crate::app::messages::SpreadsheetMsg::CellEntered {
                        row: row_idx,
                        col: col_idx,
                    }
                    .into(),
                )
                .on_release(crate::app::messages::SpreadsheetMsg::CellReleased.into());

            row_widget = row_widget.push(cell_widget);
        }

        if col_win.visible_end < sheet.columns.len() && col_win.right_spacer_width > 0.0 {
            row_widget =
                row_widget.push(iced::widget::Space::new().width(col_win.right_spacer_width));
        }

        let row_container = container(row_widget).height(Length::Fixed(row_h));

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

fn grid_header_cell_style(theme: AppTheme) -> container::Style {
    container::Style {
        background: Some(theme.palette().base.surface_raised.into()),
        border: Border {
            color: theme.palette().base.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

fn grid_header_button_style(
    theme: AppTheme,
    is_sorted: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let p = theme.palette().base;
        let (bg, text_color) = match status {
            button::Status::Hovered => (Some(primitive::WHITE_010.into()), p.text),
            button::Status::Pressed => (Some(primitive::WHITE_015.into()), p.text),
            _ => (
                Some(p.surface_raised.into()),
                if is_sorted { p.text } else { p.text_dim },
            ),
        };

        button::Style {
            background: bg,
            text_color,
            border: Border {
                color: p.border,
                width: 1.0,
                radius: radius::NONE.into(),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

fn grid_row_num_style(theme: AppTheme, is_selected: bool) -> container::Style {
    let p = theme.palette().base;
    let role = theme.palette().roles;

    let (bg, border_color) = if is_selected {
        let mut active_bg = role.accent;
        active_bg.a = 0.25;
        (Some(active_bg.into()), role.accent)
    } else {
        (Some(p.surface_raised.into()), p.border)
    };

    container::Style {
        background: bg,
        text_color: Some(if is_selected { p.text } else { p.text_dim }),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::NONE.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn grid_data_cell_style(theme: AppTheme, is_selected: bool, is_primary: bool) -> container::Style {
    let p = theme.palette().base;
    let role = theme.palette().roles;

    let (bg, border_color, border_width) = if is_selected {
        let mut active_bg = role.accent;
        active_bg.a = if is_primary { 0.28 } else { 0.18 };
        (Some(active_bg.into()), role.accent, 1.5)
    } else {
        (Some(p.bg.into()), p.border, 1.0)
    };

    container::Style {
        background: bg,
        text_color: Some(p.text),
        border: Border {
            color: border_color,
            width: border_width,
            radius: radius::NONE.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn bottom_bar_style(theme: AppTheme) -> container::Style {
    container::Style {
        background: Some(theme.palette().base.surface.into()),
        border: Border {
            color: theme.palette().base.border,
            width: 1.0,
            radius: radius::NONE.into(),
        },
        ..container::Style::default()
    }
}

fn sheet_tab_button_style(
    theme: AppTheme,
    is_active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let p = theme.palette().base;
        if is_active {
            button::Style {
                background: Some(p.surface_raised.into()),
                text_color: p.text,
                border: Border {
                    color: p.border_focus,
                    width: 1.0,
                    radius: radius::MD.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        } else {
            let (bg, text_color) = match status {
                button::Status::Hovered => (Some(primitive::WHITE_008.into()), p.text),
                button::Status::Pressed => (Some(primitive::WHITE_015.into()), p.text),
                _ => (None, p.text_dim),
            };

            button::Style {
                background: bg,
                text_color,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 1.0,
                    radius: radius::MD.into(),
                },
                shadow: Shadow::default(),
                snap: false,
            }
        }
    }
}

fn render_empty_state<'a>(
    message: &'a str,
    theme: AppTheme,
    main_font: iced::Font,
) -> Element<'a, Message> {
    container(
        text(message)
            .size(EMPTY_STATE_TEXT_SIZE)
            .font(main_font)
            .style(move |_: &Theme| text::Style {
                color: Some(theme.palette().base.text_dim),
            }),
    )
    .padding(spacing::M as u16)
    .into()
}
