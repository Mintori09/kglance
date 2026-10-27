use super::style::STYLE;
use crate::app::Message;
use crate::features::epub::parser::css::BlockStyle;
use crate::features::markdown::view::components::render_inlines_aligned;
use crate::parsers::markdown::Inline;
use crate::ui::types::RenderContext;
use iced::widget::container;
use iced::{Element, Length, Padding};

pub(crate) fn render_paragraph<'a>(
    content: &'a [Inline],
    style: Option<&BlockStyle>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let (align_x, indent, is_bold) = match style {
        Some(s) => (s.align.to_horizontal_align(), s.text_indent, s.is_bold),
        None => (iced::alignment::Horizontal::Left, 0.0, false),
    };

    let weight = if is_bold {
        Some(iced::font::Weight::Bold)
    } else {
        None
    };

    let rich = render_inlines_aligned(content, ctx.font_size, weight, None, align_x, ctx);

    let [pad_v, pad_h] = STYLE.paragraph.padding;
    let padding = Padding {
        top: pad_v as f32,
        right: pad_h as f32,
        bottom: pad_v as f32,
        left: pad_h as f32 + indent,
    };

    container(rich).padding(padding).width(Length::Fill).into()
}
