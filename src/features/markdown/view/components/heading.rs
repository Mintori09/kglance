use super::style::{STYLE, divider_line_style, heading_layout};
use crate::app::Message;
use crate::features::epub::parser::css::BlockStyle;
use crate::features::markdown::view::components::render_inlines_aligned;
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

    let heading_content = render_inlines_aligned(
        effective_content,
        layout.font_size,
        weight,
        color,
        align_x,
        ctx,
    );
    let heading = container(heading_content)
        .padding(Padding {
            top: layout.padding_top,
            right: 0.0,
            bottom: layout.padding_bottom,
            left: 0.0,
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
