use crate::app::Message;
use crate::features::markdown::view::components::style::STYLE;
use crate::parsers::markdown::is_empty_anchor_html;
use crate::ui::types::RenderContext;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_html<'a>(html: &'a str, ctx: &RenderContext<'_>) -> Element<'a, Message> {
    let trimmed = html.trim();

    if is_empty_anchor_html(trimmed) || (trimmed.starts_with("<!--") && trimmed.ends_with("-->")) {
        return iced::widget::Space::new().into();
    }

    let default_text_color = ctx.theme.palette().base.text;
    container(text(html).size(ctx.font_size).color(default_text_color))
        .padding(STYLE.paragraph.padding)
        .width(Length::Fill)
        .into()
}
