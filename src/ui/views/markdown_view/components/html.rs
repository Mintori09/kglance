use super::style::{STYLE, md_palette_for};
use crate::app::Message;
use crate::ui::views::markdown_view::blocks::RenderContext;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_html<'a>(html: &'a str, ctx: &RenderContext<'_>) -> Element<'a, Message> {
    let mp = md_palette_for(ctx.is_dark);
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
