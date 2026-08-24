use super::style::{
    STYLE, table_border_style, table_header_style, table_row_background_style,
    table_separator_style,
};
use crate::app::Message;
use crate::features::markdown::view::components::render_inlines;
use crate::parsers::markdown::TableBlock;
use crate::ui::theme::scale_size;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub(crate) fn render_table<'a>(
    table: &'a TableBlock,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let header_size = scale_size(STYLE.table.header_font_size, ctx.font_size);
    let cell_size = scale_size(STYLE.table.cell_font_size, ctx.font_size);

    let get_column_width = |index: usize| -> Length {
        table
            .column_weights
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
            let cell_ctx = RenderContext {
                block_index: ctx.block_index + i + 1,
                ..*ctx
            };
            let cell = render_inlines(&header.content, header_size, &cell_ctx);
            container(cell)
                .padding(STYLE.table.header_padding)
                .width(get_column_width(i))
                .into()
        })
        .collect();

    let theme = ctx.theme;
    let header_row = container(row(header_cells).spacing(0))
        .style(move |_: &iced::Theme| table_header_style(theme));

    let mut children: Vec<Element<'a, Message>> = vec![header_row.into()];

    if !table.rows.is_empty() {
        let separator = container(text(""))
            .style(move |_: &iced::Theme| table_separator_style(theme))
            .height(STYLE.general.divider_height)
            .width(Length::Fill);
        children.push(separator.into());
    }

    let col_count = if table.headers.is_empty() {
        table.rows.first().map_or(1, |r| r.len())
    } else {
        table.headers.len()
    };
    let num_cols = if col_count == 0 { 1 } else { col_count };

    for (row_index, row_data) in table.rows.iter().enumerate() {
        let cells: Vec<Element<'a, Message>> = row_data
            .iter()
            .enumerate()
            .map(|(j, cell)| {
                let cell_ctx = RenderContext {
                    block_index: ctx.block_index + num_cols + row_index * num_cols + j + 1,
                    ..*ctx
                };
                let cell_content = render_inlines(&cell.content, cell_size, &cell_ctx);
                container(cell_content)
                    .padding(STYLE.table.cell_padding)
                    .width(get_column_width(j))
                    .into()
            })
            .collect();

        let row_widget = row(cells).spacing(0);
        children.push(
            container(row_widget)
                .style(move |_: &iced::Theme| table_row_background_style(theme, row_index))
                .into(),
        );
    }

    let table_content = column(children).spacing(0);
    container(table_content)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| table_border_style(theme))
        .into()
}
