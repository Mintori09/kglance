use crate::app::Message;
use crate::core::SpreadsheetState;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::{
    AppTheme, default_button, default_button_primary, default_card, default_scrollable,
};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length, Theme};

use crate::ui::theme::tokens::spacing;

const SORT_ASCENDING_INDICATOR: &str = " ↑";
const SORT_DESCENDING_INDICATOR: &str = " ↓";
const SORT_NONE_INDICATOR: &str = "";

const CELL_TEXT_SIZE: f32 = 14.0;
const EMPTY_STATE_TEXT_SIZE: f32 = 18.0;

const TAB_SPACING: f32 = spacing::XS;
const MAIN_SPACING: f32 = spacing::S;
const HEADER_ROW_SPACING: f32 = spacing::S;
const DATA_ROW_SPACING: f32 = spacing::S;
const ROWS_LIST_SPACING: f32 = spacing::XXS;

const CARD_PADDING: [u16; 2] = [spacing::XS as u16, spacing::S as u16];
const HEADER_INNER_PADDING: u16 = 5;
const DATA_ROW_INNER_PADDING: u16 = 3;

const EMPTY_STATE_MESSAGE: &str = "No data";

pub fn view_spreadsheet<'a>(state: &'a SpreadsheetState, theme: AppTheme) -> Element<'a, Message> {
    let active_sheet = state.sheets.get(state.active_sheet);

    let tabs_bar = render_sheet_tabs(&state.sheets, state.active_sheet);
    let content_body = match active_sheet {
        Some(sheet) => render_spreadsheet_body(state, sheet, theme),
        None => render_empty_state(),
    };

    column![tabs_bar, content_body].spacing(MAIN_SPACING).into()
}

fn render_sheet_tabs<'a>(
    sheets: &'a [crate::core::SheetInfo],
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
    sheet: &'a crate::core::SheetInfo,
    theme: AppTheme,
) -> Element<'a, Message> {
    let filtered_rows = filter_rows(&sheet.rows, &state.search_query);
    let sorted_rows = sort_rows(filtered_rows, state.sort_col, state.sort_ascending);

    let header = render_table_header(&sheet.headers, state.sort_col, state.sort_ascending);
    let rows_list = render_table_rows(&sorted_rows, sheet.headers.len(), theme);

    let scrollable_area = scrollable(rows_list)
        .style(default_scrollable)
        .height(Length::Fill);

    let mut layout = column![].spacing(MAIN_SPACING);

    if state.search_visible {
        layout = layout.push(search_bar(
            SearchKind::Spreadsheet,
            &state.search_query,
            None,
        ));
    }

    layout = layout.push(header);
    layout = layout.push(scrollable_area);

    layout.into()
}

fn render_table_header<'a>(
    headers: &'a [String],
    sort_column: Option<usize>,
    sort_ascending: Option<bool>,
) -> Element<'a, Message> {
    let mut header_row = row![].spacing(HEADER_ROW_SPACING);

    for (column_index, header_title) in headers.iter().enumerate() {
        let indicator = get_sort_indicator(column_index, sort_column, sort_ascending);
        let button_label = format!("{}{}", header_title, indicator);

        let header_button = button(text(button_label))
            .on_press(crate::app::messages::SpreadsheetMsg::ColumnClicked(column_index).into())
            .style(default_button)
            .width(Length::FillPortion(1));

        header_row = header_row.push(header_button);
    }

    container(header_row.padding(HEADER_INNER_PADDING))
        .style(default_card)
        .padding(CARD_PADDING)
        .into()
}

fn render_table_rows<'a>(
    rows: &[&'a Vec<String>],
    column_count: usize,
    theme: AppTheme,
) -> Element<'a, Message> {
    let mut rows_list = column![].spacing(ROWS_LIST_SPACING);

    for (row_index, row_data) in rows.iter().enumerate() {
        let mut row_widget = row![]
            .spacing(DATA_ROW_SPACING)
            .padding(DATA_ROW_INNER_PADDING);
        let total_columns = row_data.len().max(column_count);

        for column_index in 0..total_columns {
            let cell_text = row_data
                .get(column_index)
                .map(|text_val| text_val.as_str())
                .unwrap_or("");

            let cell_container =
                container(text(cell_text).size(CELL_TEXT_SIZE).width(Length::Fill))
                    .width(Length::FillPortion(1));

            row_widget = row_widget.push(cell_container);
        }

        let is_even_row = row_index % 2 == 0;
        let row_container = container(row_widget).style(move |_: &Theme| {
            if is_even_row {
                apply_even_row_style(theme)
            } else {
                container::Style::default()
            }
        });

        rows_list = rows_list.push(row_container);
    }

    rows_list.into()
}

fn filter_rows<'a>(rows: &'a [Vec<String>], search_query: &str) -> Vec<&'a Vec<String>> {
    if search_query.is_empty() {
        rows.iter().collect()
    } else {
        let query_lowercase = search_query.to_lowercase();
        rows.iter()
            .filter(|row| {
                row.iter()
                    .any(|cell| cell.to_lowercase().contains(&query_lowercase))
            })
            .collect()
    }
}

fn sort_rows<'a>(
    rows: Vec<&'a Vec<String>>,
    sort_column: Option<usize>,
    sort_ascending: Option<bool>,
) -> Vec<&'a Vec<String>> {
    let Some(column_index) = sort_column else {
        return rows;
    };

    let is_ascending = sort_ascending == Some(true);
    let mut indexed_rows: Vec<(usize, &'a Vec<String>)> = rows.into_iter().enumerate().collect();

    indexed_rows.sort_by(|(_, row_a), (_, row_b)| {
        let val_a = row_a.get(column_index).map(|s| s.as_str()).unwrap_or("");
        let val_b = row_b.get(column_index).map(|s| s.as_str()).unwrap_or("");

        if is_ascending {
            val_a.cmp(val_b)
        } else {
            val_b.cmp(val_a)
        }
    });

    indexed_rows.into_iter().map(|(_, row)| row).collect()
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

fn render_empty_state<'a>() -> Element<'a, Message> {
    text(EMPTY_STATE_MESSAGE).size(EMPTY_STATE_TEXT_SIZE).into()
}
