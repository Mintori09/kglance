use super::style::STYLE;
use crate::app::Message;
use crate::features::markdown::view::blocks::render_block;
use crate::features::markdown::view::components::render_inlines;
use crate::parsers::markdown::ListItem;
use crate::ui::theme::font::get_code_font;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Length, Padding};

pub(crate) fn render_list<'a>(
    ordered: bool,
    start_number: u64,
    items: &'a [ListItem],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let mut item_elements = Vec::with_capacity(items.len());
    let mut current_idx = ctx.block_index + 1;

    for (_index, item) in items.iter().enumerate() {
        let prefix = create_list_prefix(ordered, start_number, _index, item, ctx);

        let item_block_index = current_idx;
        current_idx += 1;

        let item_ctx = RenderContext {
            block_index: item_block_index,
            ..*ctx
        };

        let content = render_inlines(&item.content, ctx.font_size, &item_ctx);

        let mut children: Vec<Element<'a, Message>> = vec![
            row![prefix, content]
                .spacing(STYLE.list.item_spacing)
                .into(),
        ];

        for sub_block in item.sub_blocks.iter() {
            let sub_ctx = RenderContext {
                block_index: current_idx,
                ..*ctx
            };
            let sub_element = render_block(current_idx, sub_block, state, &sub_ctx);
            current_idx += 10;
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

        item_elements.push(
            container(column(children).spacing(STYLE.general.item_spacing_small))
                .padding(Padding {
                    top: STYLE.list.item_padding,
                    right: 0.0,
                    bottom: STYLE.list.item_padding,
                    left: 0.0,
                })
                .width(Length::Fill)
                .into(),
        );
    }

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
            ctx.theme.palette().roles.success
        } else {
            ctx.theme.palette().base.text_dim
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
