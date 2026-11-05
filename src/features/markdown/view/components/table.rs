use super::style::{
    STYLE, copy_button_style, table_border_style, table_header_style, table_row_background_style,
    table_separator_style, table_toolbar_badge_style,
};
use crate::app::Message;
use crate::features::markdown::inlines_metrics;
use crate::features::markdown::view::components::render_inlines_aligned;
use crate::parsers::markdown::TableBlock;
use crate::ui::theme::font::get_code_font;
use crate::ui::theme::scale_size;
use crate::ui::types::RenderContext;
use iced::alignment::Horizontal;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

fn map_alignment(align: Option<&pulldown_cmark::Alignment>) -> Horizontal {
    match align {
        Some(pulldown_cmark::Alignment::Center) => Horizontal::Center,
        Some(pulldown_cmark::Alignment::Right) => Horizontal::Right,
        _ => Horizontal::Left,
    }
}

fn format_table_as_markdown(table: &TableBlock) -> String {
    let mut out = String::new();
    if !table.headers.is_empty() {
        out.push('|');
        for h in &table.headers {
            out.push(' ');
            out.push_str(&crate::features::markdown::flatten_inlines(&h.content));
            out.push_str(" |");
        }
        out.push('\n');
        out.push('|');
        for (i, _) in table.headers.iter().enumerate() {
            let align = table.alignments.get(i);
            match align {
                Some(pulldown_cmark::Alignment::Center) => out.push_str(" :---: |"),
                Some(pulldown_cmark::Alignment::Right) => out.push_str(" ---: |"),
                _ => out.push_str(" --- |"),
            }
        }
        out.push('\n');
    }
    for row in &table.rows {
        out.push('|');
        for cell in row {
            out.push(' ');
            out.push_str(&crate::features::markdown::flatten_inlines(&cell.content));
            out.push_str(" |");
        }
        out.push('\n');
    }
    out
}

pub(crate) fn table_scroll_id(block_index: usize) -> iced::widget::Id {
    iced::widget::Id::from(format!("table_scroll_{block_index}"))
}

pub fn estimate_table_total_width(table: &TableBlock, font_size: f32) -> f32 {
    let col_count = if table.headers.is_empty() {
        table.rows.first().map_or(1, |r| r.len())
    } else {
        table.headers.len()
    };
    let num_cols = if col_count == 0 { 1 } else { col_count };

    let char_w = font_size * 0.55;
    let pad_h = (STYLE.table.cell_padding[1] * 2) as f32; // 24px padding per cell
    let min_col_w =
        (scale_size(crate::ui::theme::tokens::tables::ROW_HEIGHT, font_size) * 2.2).max(85.0);

    let mut col_max_chars = vec![0usize; num_cols];
    for (i, h) in table.headers.iter().enumerate().take(num_cols) {
        col_max_chars[i] = col_max_chars[i].max(inlines_metrics(&h.content).0);
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate().take(num_cols) {
            col_max_chars[i] = col_max_chars[i].max(inlines_metrics(&cell.content).0);
        }
    }

    col_max_chars
        .iter()
        .map(|&units| {
            let estimated = units as f32 * char_w + pad_h + 16.0;
            estimated.max(min_col_w)
        })
        .sum()
}

pub(crate) fn render_table<'a>(
    block_index: usize,
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

    // Calculate maximum visual units per column across header and rows
    let char_w = ctx.font_size * 0.55;
    let pad_h = (STYLE.table.cell_padding[1] * 2) as f32; // 24px padding per cell
    let min_col_w =
        (scale_size(crate::ui::theme::tokens::tables::ROW_HEIGHT, ctx.font_size) * 2.2).max(85.0);

    let mut col_max_chars = vec![0usize; num_cols];
    for (i, h) in table.headers.iter().enumerate().take(num_cols) {
        col_max_chars[i] = col_max_chars[i].max(inlines_metrics(&h.content).0);
    }
    for row in &table.rows {
        for (i, cell) in row.iter().enumerate().take(num_cols) {
            col_max_chars[i] = col_max_chars[i].max(inlines_metrics(&cell.content).0);
        }
    }

    let col_fixed_widths: Vec<f32> = col_max_chars
        .iter()
        .map(|&units| {
            let estimated = units as f32 * char_w + pad_h + 16.0;
            estimated.max(min_col_w)
        })
        .collect();

    let total_table_w: f32 = col_fixed_widths.iter().sum::<f32>();

    let theme = ctx.theme;
    let header_text_color = theme.palette().markdown.table_header_text;

    // Header cells
    let header_count = table.headers.len();
    let header_cells: Vec<Element<'a, Message>> = table
        .headers
        .iter()
        .enumerate()
        .map(|(i, header)| {
            let align_x = map_alignment(table.alignments.get(i));
            let cell = render_inlines_aligned(
                &header.content,
                header_size,
                Some(iced::font::Weight::Semibold),
                Some(header_text_color),
                align_x,
                ctx,
            );
            let width = Length::Fixed(col_fixed_widths.get(i).copied().unwrap_or(min_col_w));

            let is_not_last = i < header_count - 1;
            let sep_color = theme.palette().markdown.table_separator;
            container(cell)
                .padding(STYLE.table.header_padding)
                .width(width)
                .align_x(align_x)
                .style(move |_: &iced::Theme| {
                    if is_not_last {
                        container::Style {
                            border: iced::Border {
                                color: sep_color,
                                width: STYLE.general.divider_height,
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        }
                    } else {
                        container::Style::default()
                    }
                })
                .into()
        })
        .collect();

    let has_rows = !table.rows.is_empty();
    let header_row_container = container(row(header_cells).spacing(0))
        .width(Length::Fixed(total_table_w))
        .style(move |_: &iced::Theme| table_header_style(theme, has_rows));

    let mut children: Vec<Element<'a, Message>> = vec![header_row_container.into()];

    if has_rows {
        let separator = container(text(""))
            .style(move |_: &iced::Theme| table_separator_style(theme))
            .height(STYLE.general.divider_height)
            .width(Length::Fixed(total_table_w));
        children.push(separator.into());
    }

    let total_rows = table.rows.len();
    for (row_index, row_data) in table.rows.iter().enumerate() {
        if row_index > 0 {
            let separator = container(text(""))
                .style(move |_: &iced::Theme| table_separator_style(theme))
                .height(STYLE.general.divider_height)
                .width(Length::Fixed(total_table_w));
            children.push(separator.into());
        }

        let row_cell_count = row_data.len();
        let row_cells: Vec<Element<'a, Message>> = row_data
            .iter()
            .enumerate()
            .map(|(j, cell)| {
                let align_x = map_alignment(table.alignments.get(j));
                let cell_content =
                    render_inlines_aligned(&cell.content, cell_size, None, None, align_x, ctx);
                let width = Length::Fixed(col_fixed_widths.get(j).copied().unwrap_or(min_col_w));

                let is_not_last = j < row_cell_count - 1;
                let sep_color = theme.palette().markdown.table_separator;
                container(cell_content)
                    .padding(STYLE.table.cell_padding)
                    .width(width)
                    .align_x(align_x)
                    .style(move |_: &iced::Theme| {
                        if is_not_last {
                            container::Style {
                                border: iced::Border {
                                    color: sep_color,
                                    width: STYLE.general.divider_height,
                                    radius: 0.0.into(),
                                },
                                ..Default::default()
                            }
                        } else {
                            container::Style::default()
                        }
                    })
                    .into()
            })
            .collect();

        let is_alternate = row_index % 2 == 1;
        let is_last = row_index == total_rows - 1;
        let row_container = container(row(row_cells).spacing(0))
            .width(Length::Fixed(total_table_w))
            .style(move |_: &iced::Theme| table_row_background_style(theme, is_alternate, is_last));

        children.push(row_container.into());
    }

    let table_content = column(children).spacing(0);

    let table_border_box = container(table_content)
        .width(Length::Fixed(total_table_w))
        .style(move |_: &iced::Theme| table_border_style(theme));

    // Scrollable horizontal container
    let table_view: Element<'a, Message> = scrollable(table_border_box)
        .id(table_scroll_id(block_index))
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::new()
                .width(6)
                .scroller_width(6)
                .margin(2),
        ))
        .on_scroll(move |vp| {
            crate::app::messages::MarkdownMsg::TableScrolled {
                block_index,
                x: vp.absolute_offset().x,
            }
            .into()
        })
        .width(Length::Shrink)
        .into();

    // Table Toolbar
    let code_font = get_code_font(ctx.font_family_mono);
    let dim_text = format!("{num_cols} cols × {total_rows} rows");
    let copy_content = format_table_as_markdown(table);

    let badge_label = container(
        text(dim_text)
            .font(code_font)
            .size(scale_size(STYLE.code.label_button_font_size, ctx.font_size)),
    )
    .padding(STYLE.code.top_bar_padding)
    .style(move |_: &iced::Theme| table_toolbar_badge_style(theme));

    let copy_btn = button(
        text("⎘")
            .font(code_font)
            .size(scale_size(STYLE.code.label_button_font_size, ctx.font_size)),
    )
    .on_press(crate::app::messages::ActionMsg::CopyText(copy_content).into())
    .style(move |_: &iced::Theme, status| copy_button_style(theme, status))
    .padding(STYLE.code.button_padding);

    let top_bar = row![
        badge_label,
        container(copy_btn)
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .width(Length::Fill)
    .align_y(iced::alignment::Vertical::Center);

    column![top_bar, table_view]
        .spacing(STYLE.general.section_spacing)
        .width(Length::Fill)
        .into()
}
