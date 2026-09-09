use super::style::{STYLE, divider_line_style, heading_layout};
use crate::app::Message;
use crate::features::markdown::view::components::render_inlines;
use crate::parsers::markdown::Inline;
use crate::ui::types::RenderContext;
use iced::widget::{column, container, text};
use iced::{Element, Length, Padding};

pub(crate) fn render_heading<'a>(
    level: u8,
    content: &'a [Inline],
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let layout = heading_layout(level, ctx.font_size);
    let heading_content = render_inlines(content, layout.font_size, ctx);
    let heading = container(heading_content)
        .padding(Padding {
            top: layout.padding_top,
            right: 0.0,
            bottom: layout.padding_bottom,
            left: 0.0,
        })
        .width(Length::Fill);

    if level == 1 || level == 2 {
        let theme = ctx.theme;
        let divider = container(text(""))
            .style(move |_: &iced::Theme| divider_line_style(theme))
            .height(STYLE.general.divider_height)
            .width(Length::Fill);
        column![heading, divider]
            .spacing(STYLE.general.section_spacing)
            .into()
    } else {
        heading.into()
    }
}
