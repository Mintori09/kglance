use super::style::STYLE;
use crate::app::Message;
use crate::ui::theme::color::markdown::MarkdownColors;
use crate::ui::views::markdown_view::blocks::RenderContext;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_html<'a>(html: &'a str, ctx: &RenderContext<'_>) -> Element<'a, Message> {
    let mp = MarkdownColors::palette_for(ctx.is_dark);
    let preview = html
        .chars()
        .take(STYLE.html.preview_truncate)
        .collect::<String>();
    container(
        text(format!("[HTML: {}]", preview))
            .size(STYLE.html.font_size)
            .color(mp.html_fg),
    )
    .padding(STYLE.paragraph.padding)
    .width(Length::Fill)
    .into()
}
