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
    let label_str = format!("[{label}]");
    let footnote_font_size = (ctx.font_size * 0.88).round();
    let label_widget = text(label_str)
        .size(footnote_font_size)
        .font(Font {
            weight: Weight::Bold,
            ..Default::default()
        })
        .color(ctx.theme.palette().roles.link);

    let inner_ctx = RenderContext {
        font_size: footnote_font_size,
        ..*ctx
    };

    let inner: Element<'a, Message> = column(
        blocks
            .iter()
            .enumerate()
            .map(|(i, block)| render_block(i, block, state, &inner_ctx)),
    )
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
        .style(move |_: &iced::Theme| container::Style {
            text_color: Some(base_color.text_dim),
            ..Default::default()
        })
        .into()
}
