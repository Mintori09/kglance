use super::style::STYLE;
use crate::app::Message;
use crate::parsers::markdown::Block;
use crate::ui::theme::color::markdown::MarkdownColors;
use crate::ui::views::markdown_view::blocks::{RenderContext, render_block};
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub(crate) fn render_quote<'a>(
    blocks: &'a [Block],
    state: &'a crate::core::MarkdownState,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let base_block_index = ctx.block_index;
    let inner: Element<'a, Message> = column(blocks.iter().enumerate().map(|(i, block)| {
        let quote_ctx = RenderContext {
            block_index: base_block_index + i + 1,
            ..*ctx
        };
        render_block(i, block, state, &quote_ctx)
    }))
    .spacing(STYLE.general.section_spacing)
    .into();

    let mp = MarkdownColors::palette_for(ctx.is_dark);

    let content = container(inner)
        .padding(STYLE.quote.content_padding)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(mp.quote_bg.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    let bar = container(text(""))
        .width(STYLE.quote.bar_width)
        .height(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(mp.quote_accent.into()),
            ..Default::default()
        });

    row![bar, content].spacing(0).into()
}
