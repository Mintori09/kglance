use super::style::{STYLE, divider_line_style, heading_layout};
use crate::app::Message;
use crate::features::epub::parser::css::BlockStyle;
use crate::features::markdown::view::components::inline::{
    InlineStyleOptions, render_inlines_with_options,
};
use crate::parsers::markdown::Inline;
use crate::ui::types::RenderContext;
use iced::font::Weight;
use iced::widget::{column, container, text};
use iced::{Element, Length, Padding};

pub(crate) fn render_heading<'a>(
    level: u8,
    content: &'a [Inline],
    style: Option<&BlockStyle>,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let layout = heading_layout(level, ctx.font_size);
    let (default_weight, color) = match level {
        1 => (Some(Weight::Bold), Some(ctx.theme.palette().base.text)),
        2 => (Some(Weight::Bold), Some(ctx.theme.palette().base.text)),
        3 => (Some(Weight::Semibold), Some(ctx.theme.palette().base.text)),
        _ => (Some(Weight::Medium), Some(ctx.theme.palette().base.text)),
    };
    let weight = if style.is_some_and(|s| s.is_bold) {
        Some(Weight::Bold)
    } else {
        default_weight
    };
    let align_x = style
        .map(|s| s.align.to_horizontal_align())
        .unwrap_or(iced::alignment::Horizontal::Left);

    let uppercase_storage: Vec<Inline>;
    let effective_content = if style.is_some_and(|s| s.is_uppercase) {
        uppercase_storage = content
            .iter()
            .map(|inline| match inline {
                Inline::Text(t) => Inline::Text(t.to_uppercase()),
                other => other.clone(),
            })
            .collect();
        &uppercase_storage
    } else {
        content
    };

    let (is_italic, is_underline, is_strikethrough) = style
        .map(|s| (s.is_italic, s.is_underline, s.is_strikethrough))
        .unwrap_or((false, false, false));

    let base_style = if is_italic {
        Some(iced::font::Style::Italic)
    } else {
        None
    };

    let opts = InlineStyleOptions {
        base_weight: weight,
        base_style,
        base_color: color,
        align_x,
        prefix_text: None,
        base_underline: is_underline,
        base_strikethrough: is_strikethrough,
        line_height: style.and_then(|s| s.line_height),
        font_family: style.and_then(|s| s.font_family.as_deref()),
    };

    let heading_content =
        render_inlines_with_options(effective_content, layout.font_size, opts, ctx);
    let (extra_ml, extra_mr, extra_mt, extra_mb) = match style {
        Some(s) => (
            s.effective_margin_left(),
            s.effective_margin_right(),
            s.effective_margin_top(),
            s.effective_margin_bottom(),
        ),
        None => (0.0, 0.0, 0.0, 0.0),
    };

    let heading = container(heading_content)
        .padding(Padding {
            top: layout.padding_top + extra_mt,
            right: extra_mr,
            bottom: layout.padding_bottom + extra_mb,
            left: extra_ml,
        })
        .width(Length::Fill);

    if (level == 1 || level == 2) && ctx.show_heading_divider {
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
