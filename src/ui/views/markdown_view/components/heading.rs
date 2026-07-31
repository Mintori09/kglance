use super::style::{STYLE, divider_line_style, heading_layout};
use crate::app::Message;
use crate::parsers::markdown::Inline;
use crate::ui::theme::scale_size;
use crate::ui::views::markdown_view::blocks::RenderContext;
use crate::ui::views::markdown_view::components::render_inlines;
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
        let divider = container(text(""))
            .style(divider_line_style)
            .height(STYLE.general.divider_height)
            .width(Length::Fill);
        column![heading, divider]
            .spacing(STYLE.general.section_spacing)
            .into()
    } else {
        heading.into()
    }
}
