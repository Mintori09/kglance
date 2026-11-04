use super::style::STYLE;
use crate::app::Message;
use crate::features::epub::parser::css::BlockStyle;
use crate::features::markdown::view::components::inline::{
    InlineStyleOptions, render_inlines_with_options,
};
use crate::parsers::markdown::Inline;
use crate::ui::types::RenderContext;
use iced::widget::container;
use iced::{Element, Length, Padding};

pub(crate) fn render_paragraph<'a>(
    content: &'a [Inline],
    style: Option<&BlockStyle>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let (align_x, indent, is_bold, is_italic, is_underline, is_strikethrough, font_scale) =
        match style {
            Some(s) => (
                s.align.to_horizontal_align(),
                s.text_indent,
                s.is_bold,
                s.is_italic,
                s.is_underline,
                s.is_strikethrough,
                s.font_scale,
            ),
            None => (
                iced::alignment::Horizontal::Left,
                0.0,
                false,
                false,
                false,
                false,
                None,
            ),
        };

    let weight = if is_bold {
        Some(iced::font::Weight::Bold)
    } else {
        None
    };

    let font_style = if is_italic {
        Some(iced::font::Style::Italic)
    } else {
        None
    };

    let font_size = if let Some(scale) = font_scale {
        (ctx.font_size * scale).round()
    } else {
        ctx.font_size
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

    let opts = InlineStyleOptions {
        base_weight: weight,
        base_style: font_style,
        base_color: None,
        align_x,
        prefix_text,
        base_underline: is_underline,
        base_strikethrough: is_strikethrough,
        line_height: style.and_then(|s| s.line_height),
        font_family: style.and_then(|s| s.font_family.as_deref()),
    };

    let rich = render_inlines_with_options(content, font_size, opts, ctx);

    let [pad_v, pad_h] = STYLE.paragraph.padding;
    let is_heading_like = is_bold
        || align_x == iced::alignment::Horizontal::Center
        || font_scale.is_some_and(|s| s > 1.05);
    let (top_padding, bottom_padding) = if is_heading_like {
        (pad_v as f32 + 10.0, pad_v as f32 + 6.0)
    } else {
        (pad_v as f32, pad_v as f32)
    };

    let (extra_ml, extra_mr, extra_mt, extra_mb) = match style {
        Some(s) => (
            s.effective_margin_left(),
            s.effective_margin_right(),
            s.effective_margin_top(),
            s.effective_margin_bottom(),
        ),
        None => (0.0, 0.0, 0.0, 0.0),
    };

    let padding = Padding {
        top: top_padding + extra_mt,
        right: pad_h as f32 + extra_mr,
        bottom: bottom_padding + extra_mb,
        left: pad_h as f32 + extra_ml,
    };

    container(rich).padding(padding).width(Length::Fill).into()
}
