use super::style::{
    STYLE, table_border_style, table_header_style, table_row_background_style,
    table_separator_style,
};
use crate::app::Message;
use crate::features::markdown::view::components::{render_inlines, render_inlines_styled};
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

    let col_count = if table.headers.is_empty() {
        table.rows.first().map_or(1, |r| r.len())
    } else {
        table.headers.len()
    };
    let num_cols = if col_count == 0 { 1 } else { col_count };

    let col_widths: Vec<Length> = (0..num_cols)
        .map(|i| {
            table
                .column_weights
                .get(i)
                .map_or(Length::FillPortion(1), |&weight| {
                    Length::FillPortion(weight)
                })
        })
        .collect();

    let theme = ctx.theme;
    let header_text_color = theme.palette().markdown.table_header_text;

    let header_cells: Vec<Element<'a, Message>> = table
        .headers
        .iter()
        .enumerate()
        .map(|(i, header)| {
            let cell_ctx = RenderContext {
                block_index: ctx.block_index + i + 1,
                ..*ctx
            };
            let cell = render_inlines_styled(
                &header.content,
                header_size,
                Some(iced::font::Weight::Semibold),
                Some(header_text_color),
                &cell_ctx,
            );
            let width = col_widths.get(i).copied().unwrap_or(Length::FillPortion(1));
            container(cell)
                .padding(STYLE.table.header_padding)
                .width(width)
                .into()
        })
        .collect();

    let has_rows = !table.rows.is_empty();
    let header_row = container(row(header_cells).spacing(0))
        .width(Length::Fill)
        .style(move |_: &iced::Theme| table_header_style(theme, has_rows));

    let mut children: Vec<Element<'a, Message>> = vec![header_row.into()];

    if has_rows {
        let separator = container(text(""))
            .style(move |_: &iced::Theme| table_separator_style(theme))
            .height(STYLE.general.divider_height)
            .width(Length::Fill);
        children.push(separator.into());
    }

    for (row_index, row_data) in table.rows.iter().enumerate() {
        if row_index > 0 {
            let separator = container(text(""))
                .style(move |_: &iced::Theme| table_separator_style(theme))
                .height(STYLE.general.divider_height)
                .width(Length::Fill);
            children.push(separator.into());
        }

        let cells: Vec<Element<'a, Message>> = row_data
            .iter()
            .enumerate()
            .map(|(j, cell)| {
                let cell_ctx = RenderContext {
                    block_index: ctx.block_index + num_cols + row_index * num_cols + j + 1,
                    ..*ctx
                };
                let cell_content = render_inlines(&cell.content, cell_size, &cell_ctx);
                let width = col_widths.get(j).copied().unwrap_or(Length::FillPortion(1));
                container(cell_content)
                    .padding(STYLE.table.cell_padding)
                    .width(width)
                    .into()
            })
            .collect();

        let row_widget = row(cells).spacing(0);
        children.push(
            container(row_widget)
                .width(Length::Fill)
                .style(move |_: &iced::Theme| table_row_background_style(theme))
                .into(),
        );
    }

    let table_content = column(children).spacing(0);
    container(table_content)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| table_border_style(theme))
        .into()
}
