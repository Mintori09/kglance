use super::style::STYLE;
use crate::app::Message;
use crate::features::markdown::view::blocks::render_block;
use crate::parsers::markdown::Block;
use crate::ui::types::RenderContext;
use iced::widget::{Space, column, container, row};
use iced::{Border, Element, Length};

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

    let mp = ctx.theme.palette().markdown;

    let content = container(inner)
        .padding(STYLE.quote.content_padding)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(mp.quote_bg.into()),
            border: Border {
                radius: iced::border::Radius {
                    top_left: 0.0,
                    top_right: 4.0,
                    bottom_right: 4.0,
                    bottom_left: 0.0,
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .width(Length::Fill);

    let bar = container(
        Space::new()
            .width(STYLE.quote.bar_width)
            .height(Length::Fill),
    )
    .width(STYLE.quote.bar_width)
    .height(Length::Fill)
    .style(move |_: &iced::Theme| container::Style {
        background: Some(mp.quote_accent.into()),
        border: Border {
            radius: 1.5.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    row![bar, content].spacing(0).into()
}
