use super::style::STYLE;
use crate::app::Message;
use crate::ui::types::RenderContext;
use iced::widget::{container, text};
use iced::{Element, Length};

pub(crate) fn render_html<'a>(html: &'a str, ctx: &RenderContext<'_>) -> Element<'a, Message> {
    let trimmed = html.trim();

    let is_anchor =
        trimmed.starts_with("<a ") || trimmed.starts_with("<a\n") || trimmed.starts_with("<a\t");

    if is_anchor {
        if trimmed.ends_with("/>") {
            return iced::widget::Space::new().into();
        }

        if trimmed.ends_with("</a>")
            && let Some(open_end) = trimmed.find('>')
        {
            let inner = &trimmed[open_end + 1..trimmed.len() - 4];

            if inner.trim().is_empty() {
                return iced::widget::Space::new().into();
            }
        }
    }

    let mp = ctx.theme.palette().markdown;
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
