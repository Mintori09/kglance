use super::style::{STYLE, divider_line_style, heading_layout};
use crate::app::Message;
use crate::features::markdown::view::components::render_inlines;
use crate::parsers::markdown::Inline;
use crate::ui::theme::scale_size;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, text};
use iced::{Element, Length, Padding};

pub(crate) fn render_heading<'a>(
    level: u8,
    content: &'a [Inline],
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let (raw_size, padding_top, padding_bottom) = heading_layout(level);
    let font_size = scale_size(raw_size, ctx.font_size);

    let heading_content = render_inlines(content, font_size, ctx);
    let heading = container(heading_content)
        .padding(Padding {
            top: padding_top,
            right: 0.0,
            bottom: padding_bottom,
            left: 0.0,
        })
        .width(Length::Fill);

    if level == 1 || level == 2 {
        let is_dark = ctx.is_dark;
        let divider = container(text(""))
            .style(move |_: &iced::Theme| divider_line_style(is_dark))
            .height(STYLE.general.divider_height)
            .width(Length::Fill);
        column![heading, divider]
            .spacing(STYLE.general.section_spacing)
            .into()
    } else {
        heading.into()
    }
}
