use std::cell::Cell;
use std::sync::{Mutex, OnceLock};

use crate::features::markdown::view::components::style::STYLE;
use crate::parsers::markdown::Inline;
use crate::ui::theme::font::{get_code_font, get_main_font};
use iced::font::Weight;
use iced::widget::text::Span;
use iced::{Color, Font};
use lru::LruCache;
use rustc_hash::FxHasher;
use std::hash::Hasher;

use crate::ui::theme::AppTheme;

pub(crate) use super::latex::render_latex_to_text;

pub struct SpanCtx<'a> {
    pub font_family: Option<&'a str>,
    pub font_family_mono: Option<&'a str>,
    pub search_query: &'a str,
    pub active_match: usize,
    pub counter: &'a Cell<usize>,
    pub theme: AppTheme,
}

#[derive(Clone, Debug)]
pub(crate) struct CachedSpan {
    pub text: String,
    pub font: Font,
    pub color: Option<Color>,
    pub strikethrough: bool,
    pub underline: bool,
}

type SpanCacheKey = (u64, AppTheme, Option<String>, Option<String>);

fn span_cache() -> &'static Mutex<LruCache<SpanCacheKey, Vec<CachedSpan>>> {
    static CACHE: OnceLock<Mutex<LruCache<SpanCacheKey, Vec<CachedSpan>>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(1024).expect("1024 is non-zero"),
        ))
    })
}

fn hash_inlines(inlines: &[Inline]) -> u64 {
    let mut hasher = FxHasher::default();
    hash_inlines_recursive(inlines, &mut hasher);
    hasher.finish()
}

fn hash_inlines_recursive(inlines: &[Inline], hasher: &mut FxHasher) {
    for inline in inlines {
        match inline {
            Inline::Text(t) => {
                hasher.write_u8(1);
                hasher.write(t.as_bytes());
            }
            Inline::Bold(c) => {
                hasher.write_u8(2);
                hash_inlines_recursive(c, hasher);
            }
            Inline::Italic(c) => {
                hasher.write_u8(3);
                hash_inlines_recursive(c, hasher);
            }
            Inline::Strikethrough(c) => {
                hasher.write_u8(4);
                hash_inlines_recursive(c, hasher);
            }
            Inline::Code(code) => {
                hasher.write_u8(5);
                hasher.write(code.as_bytes());
            }
            Inline::Link { text, url } => {
                hasher.write_u8(6);
                hasher.write(url.as_bytes());
                hash_inlines_recursive(text, hasher);
            }
            Inline::SoftBreak => {
                hasher.write_u8(7);
            }
            Inline::Image { alt, url } => {
                hasher.write_u8(8);
                hasher.write(alt.as_bytes());
                hasher.write(url.as_bytes());
            }
            Inline::InlineMath(m) => {
                hasher.write_u8(9);
                hasher.write(m.as_bytes());
            }
            Inline::DisplayMath(m) => {
                hasher.write_u8(10);
                hasher.write(m.as_bytes());
            }
            Inline::FootnoteReference(label) => {
                hasher.write_u8(11);
                hasher.write(label.as_bytes());
            }
        }
    }
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
                let display_text = super::latex::render_latex_to_text(latex);
                let math_color = span_ctx.theme.palette().markdown.math;
                spans.push(Span::new(display_text).font(main_font).color(math_color));
            }
            Inline::FootnoteReference(label) => {
                let link_color = span_ctx.theme.palette().roles.link;
                spans.push(
                    Span::new(format!("[{label}]"))
                        .font(main_font)
                        .color(link_color),
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

pub fn inlines_to_spans<'a>(children: &'a [Inline], span_ctx: &SpanCtx) -> Vec<Span<'a, (), Font>> {
    if !span_ctx.search_query.is_empty() {
        return inlines_to_spans_core(children, span_ctx);
    }

    let inline_hash = hash_inlines(children);
    let key: SpanCacheKey = (
        inline_hash,
        span_ctx.theme,
        span_ctx.font_family.map(ToString::to_string),
        span_ctx.font_family_mono.map(ToString::to_string),
    );

    if let Ok(mut cache) = span_cache().lock()
        && let Some(cached) = cache.get(&key)
    {
        return cached
            .iter()
            .map(|cs| {
                let mut span = Span::new(cs.text.clone()).font(cs.font);
                if let Some(c) = cs.color {
                    span = span.color(c);
                }
                if cs.strikethrough {
                    span = span.strikethrough(true);
                }
                if cs.underline {
                    span = span.underline(true);
                }
                span
            })
            .collect();
    }

    let spans = inlines_to_spans_core(children, span_ctx);

    let cached_list: Vec<CachedSpan> = spans
        .iter()
        .map(|s| CachedSpan {
            text: s.text.to_string(),
            font: s.font.unwrap_or(Font::DEFAULT),
            color: s.color,
            strikethrough: s.strikethrough,
            underline: s.underline,
        })
        .collect();

    if let Ok(mut cache) = span_cache().lock() {
        cache.put(key, cached_list);
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inlines_to_spans_caching() {
        let inlines = vec![
            Inline::Text("Hello world".to_string()),
            Inline::Bold(vec![Inline::Text("bold text".to_string())]),
            Inline::Code("code_block()".to_string()),
        ];
        let counter = Cell::new(0);
        let ctx = SpanCtx {
            font_family: None,
            font_family_mono: None,
            search_query: "",
            active_match: 0,
            counter: &counter,
            theme: AppTheme::Dark,
        };

        let spans1 = inlines_to_spans(&inlines, &ctx);
        assert_eq!(spans1.len(), 3);
        assert_eq!(spans1[0].text, "Hello world");
        assert_eq!(spans1[1].text, "bold text");
        assert_eq!(spans1[2].text, "code_block()");

        // Second call should hit the cache and return identical content
        let spans2 = inlines_to_spans(&inlines, &ctx);
        assert_eq!(spans2.len(), 3);
        assert_eq!(spans2[0].text, "Hello world");
        assert_eq!(spans2[1].text, "bold text");
        assert_eq!(spans2[2].text, "code_block()");
    }

    #[test]
    fn test_inlines_to_spans_search_query_bypasses_cache() {
        let inlines = vec![Inline::Text("Search keyword test".to_string())];
        let counter = Cell::new(0);
        let ctx = SpanCtx {
            font_family: None,
            font_family_mono: None,
            search_query: "keyword",
            active_match: 0,
            counter: &counter,
            theme: AppTheme::Dark,
        };

        let spans = inlines_to_spans(&inlines, &ctx);
        assert!(spans.len() >= 2);
        assert_eq!(counter.get(), 1);
    }
}
