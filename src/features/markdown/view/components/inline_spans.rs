use std::cell::Cell;

use crate::features::markdown::view::components::style::STYLE;
use crate::parsers::markdown::Inline;
use crate::ui::theme::font::{get_code_font, get_main_font};
use iced::font::Weight;
use iced::widget::text::Span;
use iced::{Color, Font};

use crate::ui::theme::AppTheme;

pub(crate) struct SpanCtx<'a> {
    pub font_family: Option<&'a str>,
    pub font_family_mono: Option<&'a str>,
    pub search_query: &'a str,
    pub active_match: usize,
    pub counter: &'a Cell<usize>,
    pub theme: AppTheme,
}

fn search_highlight_color(is_active: bool, theme: AppTheme) -> Color {
    let mp = theme.palette().markdown;
    if is_active {
        mp.search_active_bg
    } else {
        mp.search_inactive_bg
    }
}

fn highlight_search_in_text<'a>(
    text: &'a str,
    span_ctx: &SpanCtx,
    font: Font,
    normal_color: Option<Color>,
) -> Vec<Span<'a, (), Font>> {
    let mut spans = Vec::new();
    let lower = text.to_lowercase();
    let query_lower = span_ctx.search_query.to_lowercase();
    let mut pos = 0;

    while let Some(match_pos) = lower[pos..].find(&query_lower) {
        let abs_pos = pos + match_pos;
        let end_pos = abs_pos + query_lower.len();

        if abs_pos > pos {
            let mut span = Span::new(&text[pos..abs_pos]).font(font);
            if let Some(color) = normal_color {
                span = span.color(color);
            }
            spans.push(span);
        }

        let bg = search_highlight_color(
            span_ctx.counter.get() == span_ctx.active_match,
            span_ctx.theme,
        );
        spans.push(Span::new(&text[abs_pos..end_pos]).font(font).background(bg));

        span_ctx.counter.set(span_ctx.counter.get() + 1);
        pos = end_pos;
    }

    if pos < text.len() {
        let mut span = Span::new(&text[pos..]).font(font);
        if let Some(color) = normal_color {
            span = span.color(color);
        }
        spans.push(span);
    }

    spans
}

fn inlines_to_spans_core<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
) -> Vec<Span<'a, (), Font>> {
    let main_font = get_main_font(span_ctx.font_family);
    let code_font = get_code_font(span_ctx.font_family_mono);
    let mut spans = Vec::new();
    let search_query = span_ctx.search_query;

    for inline in children {
        match inline {
            Inline::Text(t) => {
                if search_query.is_empty() {
                    spans.push(Span::new(t.as_str()).font(main_font));
                } else {
                    spans.extend(highlight_search_in_text(t, span_ctx, main_font, None));
                }
            }
            Inline::Bold(children) => {
                spans.extend(apply_style_to_children(
                    children,
                    span_ctx,
                    main_font,
                    |f| Font {
                        weight: Weight::Bold,
                        ..f
                    },
                ));
            }
            Inline::Italic(children) => {
                spans.extend(apply_style_to_children(
                    children,
                    span_ctx,
                    main_font,
                    |f| Font {
                        style: iced::font::Style::Italic,
                        ..f
                    },
                ));
            }
            Inline::Strikethrough(children) => {
                for s in inlines_to_spans_core(children, span_ctx) {
                    spans.push(s.font(main_font).strikethrough(true));
                }
            }
            Inline::Code(code) => {
                if search_query.is_empty() {
                    spans.push(
                        Span::new(code.as_str())
                            .font(code_font)
                            .color(STYLE.inline.inline_code_color),
                    );
                } else {
                    spans.extend(highlight_search_in_text(
                        code,
                        span_ctx,
                        code_font,
                        Some(STYLE.inline.inline_code_color),
                    ));
                }
            }
            Inline::Link {
                text: link_text, ..
            } => {
                let link_color = span_ctx.theme.palette().roles.link;
                for s in inlines_to_spans_core(link_text, span_ctx) {
                    spans.push(s.color(link_color).underline(true));
                }
            }
            Inline::SoftBreak => {
                spans.push(Span::new(" ").font(main_font));
            }
            Inline::Image { alt, .. } => {
                spans.push(
                    Span::new(format!("[{alt}]"))
                        .font(main_font)
                        .color(STYLE.inline.image_alt_color),
                );
            }
            Inline::InlineMath(latex) | Inline::DisplayMath(latex) => {
                spans.push(
                    Span::new(latex.as_str())
                        .font(main_font)
                        .color(STYLE.inline.math_color),
                );
            }
        }
    }

    spans
}

fn apply_style_to_children<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
    main_font: Font,
    style: fn(Font) -> Font,
) -> Vec<Span<'a, (), Font>> {
    inlines_to_spans_core(children, span_ctx)
        .into_iter()
        .map(|span| span.font(style(main_font)))
        .collect()
}

pub(crate) fn inlines_to_spans<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
) -> Vec<Span<'a, (), Font>> {
    inlines_to_spans_core(children, span_ctx)
}
