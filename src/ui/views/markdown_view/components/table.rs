use super::style::{
    STYLE, table_border_style, table_header_style, table_row_background_style,
    table_separator_style,
};
use crate::app::Message;
use crate::parsers::markdown::{TableBlock, TableCell, flatten_inlines};
use crate::ui::theme::scale_size;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::markdown_view::components::render_inlines;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

fn calculate_column_weights(headers: &[TableCell], rows: &[Vec<TableCell>]) -> Vec<u16> {
    let n = headers.len();
    if n == 0 {
        return vec![];
    }

    let mut max_lens = vec![0usize; n];
    for (i, header) in headers.iter().enumerate() {
        max_lens[i] = flatten_inlines(&header.content).len();
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(n) {
            max_lens[i] = max_lens[i].max(flatten_inlines(&cell.content).len());
        }
    }

    let total: usize = max_lens.iter().sum();
    if total == 0 {
        return vec![1; n];
    }

    max_lens
        .iter()
        .map(|&length| {
            ((length as f32 / total as f32) * 100.0).max(STYLE.table.min_column_weight) as u16
        })
        .collect()
}

pub(crate) fn render_table<'a>(
    table: &'a TableBlock,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let header_size = scale_size(STYLE.table.header_font_size, ctx.font_size);
    let cell_size = scale_size(STYLE.table.cell_font_size, ctx.font_size);
    let column_weights = calculate_column_weights(&table.headers, &table.rows);

    let get_column_width = |index: usize| -> Length {
        column_weights
            .get(index)
            .map_or(Length::FillPortion(1), |&weight| {
                Length::FillPortion(weight)
            })
    };

    let header_cells: Vec<Element<'a, Message>> = table
        .headers
        .iter()
        .enumerate()
        .map(|(i, header)| {
            let cell = render_inlines(&header.content, header_size, ctx);
            container(cell)
                .padding(STYLE.table.header_padding)
                .width(get_column_width(i))
                .into()
        })
        .collect();

    let header_row = container(row(header_cells).spacing(0)).style(table_header_style);

    let mut children: Vec<Element<'a, Message>> = vec![header_row.into()];

    if !table.rows.is_empty() {
        let separator = container(text(""))
            .style(table_separator_style)
            .height(STYLE.general.divider_height)
            .width(Length::Fill);
        children.push(separator.into());
    }

    for (row_index, row_data) in table.rows.iter().enumerate() {
        let cells: Vec<Element<'a, Message>> = row_data
            .iter()
            .enumerate()
            .map(|(j, cell)| {
                let cell_content = render_inlines(&cell.content, cell_size, ctx);
                container(cell_content)
                    .padding(STYLE.table.cell_padding)
                    .width(get_column_width(j))
                    .into()
            })
            .collect();

        let row_widget = row(cells).spacing(0);
        children.push(
            container(row_widget)
                .style(move |theme| table_row_background_style(theme, row_index))
                .into(),
        );
    }

    let table_content = column(children).spacing(0);
    container(table_content)
        .width(Length::Fill)
        .style(table_border_style)
        .into()
}
