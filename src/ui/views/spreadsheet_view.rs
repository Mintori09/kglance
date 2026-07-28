use crate::app::Message;
use crate::core::SpreadsheetState;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::{breeze_button, glass_card, glass_scrollable};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length, Theme};

fn even_row_style(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);
    container::Style {
        background: Some(
            (if is_dark {
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.03)
            } else {
                iced::Color::from_rgba(0.0, 0.0, 0.0, 0.03)
            })
            .into(),
        ),
        ..container::Style::default()
    }
}

fn sort_indicator(col: usize, sort_col: Option<usize>, ascending: Option<bool>) -> &'static str {
    if Some(col) == sort_col {
        match ascending {
            Some(true) => " ↑",
            Some(false) => " ↓",
            None => "",
        }
    } else {
        ""
    }
}

pub fn view_spreadsheet<'a>(state: &'a SpreadsheetState) -> Element<'a, Message> {
    let active = state.sheets.get(state.active_sheet);

    let sheet_tabs: Element<'a, Message> = if state.sheets.len() > 1 {
        let mut tabs = row![].spacing(4);
        for (i, sheet) in state.sheets.iter().enumerate() {
            let is_active = i == state.active_sheet;
            let btn = button(text(&sheet.name))
                .on_press(Message::SheetTabClicked(i))
                .style(if is_active {
                    crate::ui::theme::glass_button_primary
                } else {
                    breeze_button
                });
            tabs = tabs.push(btn);
        }
        container(tabs).style(glass_card).padding([4, 8]).into()
    } else {
        container(row![]).into()
    };

    let body: Element<'a, Message> = if let Some(sheet) = active {
        let sort_col = state.sort_col;
        let sort_asc = state.sort_ascending;

        // Filter rows by search query (case-insensitive, across all columns)
        let filtered: Vec<&Vec<String>> = if state.search_query.is_empty() {
            sheet.rows.iter().collect()
        } else {
            let q = state.search_query.to_lowercase();
            sheet
                .rows
                .iter()
                .filter(|row| row.iter().any(|cell| cell.to_lowercase().contains(&q)))
                .collect()
        };

        let mut header_row = row![].spacing(8);
        for (ci, h) in sheet.headers.iter().enumerate() {
            let label = format!("{}{}", h, sort_indicator(ci, sort_col, sort_asc));
            let btn = button(text(label))
                .on_press(Message::SpreadsheetColumnClicked(ci))
                .style(breeze_button)
                .width(Length::FillPortion(1));
            header_row = header_row.push(btn);
        }

        let header = container(header_row.padding(5))
            .style(glass_card)
            .padding([4, 8]);

        let display_rows = if let Some(sc) = sort_col {
            let ascending = sort_asc == Some(true);
            let mut sorted: Vec<(usize, &Vec<String>)> =
                filtered.iter().enumerate().map(|(i, r)| (i, *r)).collect();
            sorted.sort_by(|(_, a), (_, b)| {
                let va = a.get(sc).map(|s| s.as_str()).unwrap_or("");
                let vb = b.get(sc).map(|s| s.as_str()).unwrap_or("");
                if ascending { va.cmp(vb) } else { vb.cmp(va) }
            });
            sorted.into_iter().map(|(_, r)| r).collect::<Vec<_>>()
        } else {
            filtered
        };

        let mut rows_list = column![].spacing(2);
        for (idx, row_data) in display_rows.iter().enumerate() {
            let mut row_widget = row![].spacing(8).padding(3);
            let max_cols = row_data.len().max(sheet.headers.len());
            for ci in 0..max_cols {
                let cell_text = row_data.get(ci).map(|s| s.as_str()).unwrap_or("");
                row_widget = row_widget.push(
                    container(text(cell_text).size(14).width(Length::Fill))
                        .width(Length::FillPortion(1)),
                );
            }

            let row_style: fn(&Theme) -> container::Style = if idx % 2 == 0 {
                even_row_style
            } else {
                |_| container::Style::default()
            };
            rows_list = rows_list.push(container(row_widget).style(row_style));
        }

        let scroll_area = scrollable(rows_list)
            .style(glass_scrollable)
            .height(Length::Fill);

        let mut col = column![].spacing(6);
        if state.search_visible {
            col = col.push(search_bar(
                SearchKind::Spreadsheet,
                &state.search_query,
                None,
                "Search spreadsheet...",
                "ss_search_input",
            ));
        }
        col = col.push(header);
        col = col.push(scroll_area);
        col.into()
    } else {
        text("No data").size(18).into()
    };

    column![sheet_tabs, body].spacing(6).into()
}
