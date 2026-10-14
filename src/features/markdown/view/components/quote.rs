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
    let inner: Element<'a, Message> = column(
        blocks
            .iter()
            .enumerate()
            .map(|(i, block)| render_block(i, block, state, ctx)),
    )
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
