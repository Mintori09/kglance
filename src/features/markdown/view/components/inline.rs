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
    start_offset: usize,
    end_offset: usize,
    font_size: f32,
    span_ctx: &SpanCtx,
    ctx: &RenderContext<'_>,
    seg_idx: usize,
) {
    if start >= end {
        return;
    }
    let segment_block_index = ctx.block_index + seg_idx + 1;
    let default_text_color = ctx.theme.palette().base.text;

    let mut spans = inlines_to_spans(&inlines[start..end], span_ctx);
    if start_offset > 0 && !spans.is_empty() {
        let text_len = spans[0].text.len();
        if start_offset < text_len {
            if let std::borrow::Cow::Borrowed(s) = spans[0].text {
                spans[0].text = std::borrow::Cow::Borrowed(&s[start_offset..]);
            }
        } else {
            spans.remove(0);
        }
    }
    if end_offset > 0 && !spans.is_empty() {
        let last_idx = spans.len() - 1;
        let text_len = spans[last_idx].text.len();
        if end_offset < text_len {
            if let std::borrow::Cow::Borrowed(s) = spans[last_idx].text {
                spans[last_idx].text = std::borrow::Cow::Borrowed(&s[..text_len - end_offset]);
            }
        } else {
            spans.pop();
        }
    }

    if !spans.is_empty() {
        elements.push(
            crate::features::markdown::view::build_selectable(
                spans,
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
}

fn build_link_button<'a>(
    prefix: Option<&str>,
    link_text: &'a [Inline],
    url: &'a str,
    suffix: Option<&str>,
    font_size: f32,
    link_color: Color,
) -> Element<'a, Message> {
    let mut display = String::new();
    if let Some(p) = prefix {
        display.push_str(p);
    }
    display.push_str(&flatten_inlines(link_text));
    if let Some(s) = suffix {
        display.push_str(s);
    }
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

fn leading_closing_punctuation_len(text: &str) -> usize {
    let mut len = 0;
    for ch in text.chars() {
        if matches!(ch, ']' | ')' | '}' | ',' | '.' | ';' | ':' | '!') {
            len += ch.len_utf8();
        } else {
            break;
        }
    }
    len
}

fn trailing_opening_punctuation_len(text: &str) -> usize {
    let mut len = 0;
    for ch in text.chars().rev() {
        if matches!(ch, '[' | '(' | '{') {
            len += ch.len_utf8();
        } else {
            break;
        }
    }
    len
}

pub fn render_inlines<'a>(
    inlines: &'a [Inline],
    font_size: f32,
    ctx: &RenderContext<'_>,
) -> Element<'a, Message> {
    let link_color = ctx.theme.palette().roles.link;
    let span_ctx = SpanCtx {
        font_family: ctx.font_family,
        font_family_mono: ctx.font_family_mono,
        search_query: ctx.search_query,
        active_match: ctx.active_match,
        counter: ctx.counter,
        theme: ctx.theme,
    };
    let has_special = inlines.iter().any(|i| {
        matches!(
            i,
            Inline::Link { .. } | Inline::InlineMath(_) | Inline::DisplayMath(_)
        )
    });

    if !has_special {
        let default_text_color = ctx.theme.palette().base.text;
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
    let mut current_start_offset = 0;

    for i in 0..inlines.len() {
        let inline = &inlines[i];
        let is_special = matches!(
            inline,
            Inline::Link { .. } | Inline::InlineMath(_) | Inline::DisplayMath(_)
        );

        if !is_special {
            continue;
        }

        let mut prefix: Option<&str> = None;
        let mut text_end_limit = i;

        if matches!(inline, Inline::Link { .. })
            && i > start
            && let Inline::Text(prev_text) = &inlines[i - 1]
        {
            let p_len = trailing_opening_punctuation_len(prev_text);
            if p_len > 0 && p_len <= prev_text.len() {
                prefix = Some(&prev_text[prev_text.len() - p_len..]);
                if i - 1 == start {
                    text_end_limit = i - 1;
                }
            }
        }

        flush_text_segment(
            &mut elements,
            inlines,
            start,
            text_end_limit,
            current_start_offset,
            prefix.map(|p| p.len()).unwrap_or(0),
            font_size,
            &span_ctx,
            ctx,
            i,
        );
        current_start_offset = 0;

        match inline {
            Inline::Link {
                text: link_text,
                url,
            } => {
                let mut suffix: Option<&str> = None;
                if i + 1 < inlines.len()
                    && let Inline::Text(next_text) = &inlines[i + 1]
                {
                    let p_len = leading_closing_punctuation_len(next_text);
                    if p_len > 0 {
                        suffix = Some(&next_text[..p_len]);
                        current_start_offset = p_len;
                    }
                }
                elements.push(build_link_button(
                    prefix, link_text, url, suffix, font_size, link_color,
                ));
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
        current_start_offset,
        0,
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
