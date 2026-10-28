use super::style::STYLE;
use crate::app::Message;
use crate::features::epub::parser::css::BlockStyle;
use crate::features::markdown::view::components::inline::render_inlines_aligned_with_prefix;
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

    let starts_with_bullet = content.first().is_some_and(|inline| match inline {
        Inline::Text(t) => {
            let s = t.trim_start();
            s.chars().next().is_some_and(|c| {
                crate::features::common::parser::html::CUSTOM_BULLET_CHARS.contains(&c)
            })
        }
        _ => false,
    });

    let prefix_storage: String;
    let prefix_text = if indent > 0.0 && !content.is_empty() && !starts_with_bullet {
        let num_spaces = if indent >= 40.0 {
            3
        } else if indent >= 20.0 {
            2
        } else {
            1
        };
        prefix_storage = "\u{2003}".repeat(num_spaces);
        Some(prefix_storage.as_str())
    } else {
        None
    };

    let rich = render_inlines_aligned_with_prefix(
        content,
        ctx.font_size,
        weight,
        None,
        align_x,
        prefix_text,
        ctx,
    );

    let [pad_v, pad_h] = STYLE.paragraph.padding;
    let (top_padding, bottom_padding) = if is_bold {
        (pad_v as f32 + 10.0, pad_v as f32 + 4.0)
    } else {
        (pad_v as f32, pad_v as f32)
    };

    let padding = Padding {
        top: top_padding,
        right: pad_h as f32,
        bottom: bottom_padding,
        left: pad_h as f32,
    };

    container(rich).padding(padding).width(Length::Fill).into()
}
