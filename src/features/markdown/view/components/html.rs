use crate::app::Message;
use crate::features::markdown::view::build_selectable;
use crate::features::markdown::view::components::style::STYLE;
use crate::parsers::markdown::is_empty_anchor_html;
use crate::ui::types::RenderContext;
use iced::widget::container;
use iced::{Element, Length};

pub(crate) fn render_html<'a>(html: &'a str, ctx: &RenderContext<'_>) -> Element<'a, Message> {
    let trimmed = html.trim();

    if is_empty_anchor_html(trimmed) || (trimmed.starts_with("<!--") && trimmed.ends_with("-->")) {
        return iced::widget::Space::new().into();
    }

    let default_text_color = ctx.theme.palette().base.text;
    let span = iced::widget::text::Span::new(html.to_string()).color(default_text_color);
    let blk = ctx.alloc_block_index();
    let selectable = build_selectable(
        vec![span],
        ctx.font_size,
        blk,
        Length::Fill,
        default_text_color,
        ctx.selection_range,
        ctx.drag_active,
    );
    container(selectable)
        .padding(STYLE.paragraph.padding)
        .width(Length::Fill)
        .into()
}
