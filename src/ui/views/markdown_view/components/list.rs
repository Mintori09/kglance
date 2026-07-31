use super::style::{STYLE, md_palette_for};
use crate::app::Message;
use crate::parsers::markdown::ListItem;
use crate::ui::theme::font::get_code_font;
use crate::ui::views::markdown_view::blocks::{RenderContext, render_block};
use crate::ui::views::markdown_view::components::render_inlines;
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Length, Padding};

pub(crate) fn render_list<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let item_elements = items.iter().enumerate().map(|(index, item)| {
        let prefix = create_list_prefix(ordered, start_number, index, item, ctx);

        let content = render_inlines(&item.content, ctx.font_size, ctx);

        let mut children: Vec<Element<'a, Message>> = vec![
            row![prefix, content]
                .spacing(STYLE.list.item_spacing)
                .into(),
        ];

        for (sub_index, sub_block) in item.sub_blocks.iter().enumerate() {
            let sub_element = render_block(index * 1000 + sub_index, sub_block, state, ctx);
            children.push(
                container(sub_element)
                    .padding(Padding {
                        top: STYLE.list.item_padding,
                        right: 0.0,
                        bottom: STYLE.list.item_padding,
                        left: STYLE.list.sub_block_left_padding,
                    })
                    .width(Length::Fill)
                    .into(),
            );
        }

        container(column(children).spacing(STYLE.general.item_spacing_small))
            .padding(Padding {
                top: STYLE.list.item_padding,
                right: 0.0,
                bottom: STYLE.list.item_padding,
                left: 0.0,
            })
            .width(Length::Fill)
            .into()
    });

    column(item_elements)
        .spacing(STYLE.general.section_spacing)
        .into()
}

fn create_list_prefix<'a>(
    ordered: bool,
    start_number: u64,
    index: usize,
    item: &ListItem,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    if let Some(checked) = item.is_task {
        let symbol = if checked { "[x] " } else { "[ ] " };
        let color: Color = if checked {
            md_palette_for(ctx.is_dark).task_checked
        } else {
            crate::ui::theme::glass::palette_for(ctx.is_dark).text_dim
        };
        text(symbol)
            .font(get_code_font(ctx.font_family_mono))
            .size(ctx.font_size)
            .color(color)
            .into()
    } else if ordered {
        text(format!("{}. ", start_number + index as u64))
            .size(ctx.font_size)
            .color(STYLE.list.bullet_color)
            .into()
    } else {
        text("• ")
            .size(ctx.font_size)
            .color(STYLE.list.bullet_color)
            .into()
    }
}
