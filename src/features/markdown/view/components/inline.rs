use crate::app::Message;
use crate::features::markdown::view::components::inline_spans::{SpanCtx, inlines_to_spans};
use crate::features::markdown::view::components::style::STYLE;
use crate::parsers::markdown::{Inline, flatten_inlines};
use crate::ui::theme::color::roles;
use crate::ui::theme::default_tooltip;
use crate::ui::types::RenderContext;
use iced::widget::{button, tooltip};
use iced::{Border, Color, Element, Shadow};

fn link_button_style(theme: &iced::Theme, _status: button::Status) -> button::Style {
    let link_color = roles::palette(theme).link;
    button::Style {
        background: None,
        text_color: link_color,
        border: Border {
            width: STYLE.inline.link_button_border_width,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

#[allow(clippy::too_many_arguments)]
fn flush_text_segment<'a>(
    elements: &mut Vec<Element<'a, Message>>,
    inlines: &'a [Inline],
    start: usize,
    end: usize,
    font_size: f32,
    span_ctx: &SpanCtx,
    ctx: &RenderContext<'_>,
    seg_idx: usize,
) {
    if start >= end {
        return;
    }
    let segment_block_index = ctx.block_index + seg_idx + 1;
    let default_text_color =
        crate::ui::theme::color::base::BaseColors::palette_for(ctx.is_dark).text;
    elements.push(
        crate::features::markdown::view::build_selectable(
            inlines_to_spans(&inlines[start..end], span_ctx),
            font_size,
            segment_block_index,
            iced::Length::Shrink,
            default_text_color,
            ctx.selection_range,
            ctx.drag_active,
        )
        .into(),
    );
}

fn build_link_button<'a>(
    link_text: &'a [Inline],
    url: &'a str,
    font_size: f32,
    link_color: Color,
) -> Element<'a, Message> {
    let display = flatten_inlines(link_text);
    let url_clone = url.to_string();
    let btn = button(
        iced::widget::text(display)
            .size(font_size)
            .color(link_color),
    )
    .on_press(crate::app::messages::SystemMsg::OpenLink(url_clone).into())
    .style(link_button_style)
    .padding(0);

    let tooltip_label = iced::widget::text(url)
        .size(STYLE.inline.tooltip_font_size)
        .color(Color::WHITE);
    let tooltip_wrapped = iced::widget::container(tooltip_label)
        .padding(STYLE.inline.tooltip_padding)
        .style(default_tooltip);

    tooltip(btn, tooltip_wrapped, tooltip::Position::Top)
        .gap(STYLE.inline.tooltip_gap)
        .into()
}

pub fn render_inlines<'a>(
    inlines: &'a [Inline],
    font_size: f32,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let link_color = roles::palette_for(ctx.is_dark).link;
    let span_ctx = SpanCtx {
        font_family: ctx.font_family,
        font_family_mono: ctx.font_family_mono,
        search_query: ctx.search_query,
        active_match: ctx.active_match,
        counter: ctx.counter,
        is_dark: ctx.is_dark,
    };
    let has_special = inlines.iter().any(|i| {
        matches!(
            i,
            Inline::Link { .. } | Inline::InlineMath(_) | Inline::DisplayMath(_)
        )
    });

    if !has_special {
        let default_text_color =
            crate::ui::theme::color::base::BaseColors::palette_for(ctx.is_dark).text;
        return crate::features::markdown::view::build_selectable(
            inlines_to_spans(inlines, &span_ctx),
            font_size,
            ctx.block_index,
            iced::Length::Fill,
            default_text_color,
            ctx.selection_range,
            ctx.drag_active,
        )
        .into();
    }

    let mut elements: Vec<Element<'a, Message>> = Vec::new();
    let mut start = 0;

    for (i, inline) in inlines.iter().enumerate() {
        let is_special = matches!(
            inline,
            Inline::Link { .. } | Inline::InlineMath(_) | Inline::DisplayMath(_)
        );

        if !is_special {
            continue;
        }

        flush_text_segment(
            &mut elements,
            inlines,
            start,
            i,
            font_size,
            &span_ctx,
            ctx,
            i,
        );

        match inline {
            Inline::Link {
                text: link_text,
                url,
            } => {
                elements.push(build_link_button(link_text, url, font_size, link_color));
            }
            Inline::InlineMath(latex) => {
                elements.push(iced_math::inline(latex.as_str()));
            }
            Inline::DisplayMath(latex) => {
                elements.push(iced_math::block(latex.as_str()));
            }
            _ => {}
        }

        start = i + 1;
    }

    flush_text_segment(
        &mut elements,
        inlines,
        start,
        inlines.len(),
        font_size,
        &span_ctx,
        ctx,
        inlines.len(),
    );

    let mut wrap = iced_aw::Wrap::new()
        .spacing(STYLE.inline.wrap_spacing)
        .line_spacing(STYLE.inline.wrap_line_spacing);
    for el in elements {
        wrap = wrap.push(el);
    }
    wrap.into()
}
