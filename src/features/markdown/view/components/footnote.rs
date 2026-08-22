use super::style::STYLE;
use crate::app::Message;
use crate::features::markdown::view::blocks::render_block;
use crate::parsers::markdown::Block;
use crate::ui::types::RenderContext;
use iced::font::Weight;
use iced::widget::{column, container, row, text};
use iced::{Element, Font, Length, Padding};

pub(crate) fn render_footnote_definition<'a>(
    label: &'a str,
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let base_color = ctx.theme.palette().base;
    let label_str = format!("[^{label}]:");
    let label_widget = text(label_str)
        .size(ctx.font_size * 0.85)
        .font(Font {
            weight: Weight::Bold,
            ..Default::default()
        })
        .color(base_color.text_dim);

    let base_block_index = ctx.block_index;
    let inner: Element<'a, Message> = column(blocks.iter().enumerate().map(|(i, block)| {
        let fn_ctx = RenderContext {
            block_index: base_block_index + i + 1,
            ..*ctx
        };
        render_block(i, block, state, &fn_ctx)
    }))
    .spacing(STYLE.general.section_spacing)
    .into();

    let row_content = row![label_widget, inner].spacing(8.0);

    container(row_content)
        .padding(Padding {
            top: 4.0,
            right: 0.0,
            bottom: 4.0,
            left: 0.0,
        })
        .width(Length::Fill)
        .into()
}
