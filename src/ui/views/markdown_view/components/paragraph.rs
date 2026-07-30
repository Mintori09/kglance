use super::style::STYLE;
use crate::app::Message;
use crate::parsers::markdown::Inline;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::markdown_view::components::render_inlines;
use iced::widget::container;
use iced::{Element, Length};

pub(crate) fn render_paragraph<'a>(
    content: &'a [Inline],
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let rich = render_inlines(content, ctx.font_size, ctx);
    container(rich)
        .padding(STYLE.paragraph.padding)
        .width(Length::Fill)
        .into()
}
