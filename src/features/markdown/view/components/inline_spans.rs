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
    pub base_weight: Option<Weight>,
    pub base_color: Option<Color>,
    pub font_size: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct CachedSpan {
    pub text: String,
    pub font: Font,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub size: Option<f32>,
    pub strikethrough: bool,
    pub underline: bool,
}

impl CachedSpan {
    pub fn to_span<'a>(&self) -> Span<'a, (), Font> {
        let mut span = Span::new(self.text.clone()).font(self.font);
        if let Some(c) = self.color {
            span = span.color(c);
        }
        if let Some(bg) = self.background {
            span = span.background(bg);
        }
        if let Some(sz) = self.size {
            span = span.size(sz);
        }
        if self.strikethrough {
            span = span.strikethrough(true);
        }
        if self.underline {
            span = span.underline(true);
        }
        span
    }
}

type SpanCacheKey = (
    u64,
    AppTheme,
    Option<String>,
    Option<String>,
    Option<u16>,
    Option<u32>,
    u32,
);

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

fn highlight_search_in_text(
    text: &str,
    span_ctx: &SpanCtx,
    font: Font,
    normal_color: Option<Color>,
) -> Vec<CachedSpan> {
    let mut spans = Vec::new();
    let lower = text.to_lowercase();
    let query_lower = span_ctx.search_query.to_lowercase();
    let mut pos = 0;

    while let Some(match_pos) = lower[pos..].find(&query_lower) {
        let abs_pos = pos + match_pos;
        let end_pos = abs_pos + query_lower.len();

        if abs_pos > pos {
            spans.push(CachedSpan {
                text: text[pos..abs_pos].to_string(),
                font,
                color: normal_color,
                background: None,
                size: None,
                strikethrough: false,
                underline: false,
            });
        }

        let bg = search_highlight_color(
            span_ctx.counter.get() == span_ctx.active_match,
            span_ctx.theme,
        );
        spans.push(CachedSpan {
            text: text[abs_pos..end_pos].to_string(),
            font,
            color: normal_color,
            background: Some(bg),
            size: None,
            strikethrough: false,
            underline: false,
        });

        span_ctx.counter.set(span_ctx.counter.get() + 1);
        pos = end_pos;
    }

    if pos < text.len() {
        spans.push(CachedSpan {
            text: text[pos..].to_string(),
            font,
            color: normal_color,
            background: None,
            size: None,
            strikethrough: false,
            underline: false,
        });
    }

    spans
}

fn inlines_to_spans_core(children: &[Inline], span_ctx: &SpanCtx) -> Vec<CachedSpan> {
    let mut main_font = get_main_font(span_ctx.font_family);
    if let Some(weight) = span_ctx.base_weight {
        main_font.weight = weight;
    }
    let code_font = get_code_font(span_ctx.font_family_mono);
    let mut spans = Vec::new();
    let search_query = span_ctx.search_query;
    let mp = span_ctx.theme.palette().markdown;
    let inline_code_fg = mp.inline_code_fg;
    let inline_code_bg = mp.inline_code_bg;
    let inline_code_size = (span_ctx.font_size * 0.90).round();

    for inline in children {
        match inline {
            Inline::Text(t) => {
                let stripped = crate::parsers::markdown::strip_anchor_tags_cow(t);
                let s = stripped.as_ref();
                if !s.is_empty() {
                    if search_query.is_empty() {
                        spans.push(CachedSpan {
                            text: s.to_string(),
                            font: main_font,
                            color: span_ctx.base_color,
                            background: None,
                            size: None,
                            strikethrough: false,
                            underline: false,
                        });
                    } else {
                        spans.extend(highlight_search_in_text(
                            s,
                            span_ctx,
                            main_font,
                            span_ctx.base_color,
                        ));
                    }
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
                for mut s in inlines_to_spans_core(children, span_ctx) {
                    s.font = main_font;
                    s.strikethrough = true;
                    spans.push(s);
                }
            }
            Inline::Code(code) => {
                if search_query.is_empty() {
                    spans.push(CachedSpan {
                        text: code.clone(),
                        font: code_font,
                        color: Some(inline_code_fg),
                        background: Some(inline_code_bg),
                        size: if inline_code_size > 0.0 {
                            Some(inline_code_size)
                        } else {
                            None
                        },
                        strikethrough: false,
                        underline: false,
                    });
                } else {
                    spans.extend(highlight_search_in_text(
                        code,
                        span_ctx,
                        code_font,
                        Some(inline_code_fg),
                    ));
                }
            }
            Inline::Link {
                text: link_text, ..
            } => {
                let link_color = span_ctx.theme.palette().roles.link;
                for mut s in inlines_to_spans_core(link_text, span_ctx) {
                    s.color = Some(link_color);
                    s.underline = true;
                    spans.push(s);
                }
            }
            Inline::SoftBreak => {
                spans.push(CachedSpan {
                    text: " ".to_string(),
                    font: main_font,
                    color: None,
                    background: None,
                    size: None,
                    strikethrough: false,
                    underline: false,
                });
            }
            Inline::Image { alt, .. } => {
                spans.push(CachedSpan {
                    text: format!("[{alt}]"),
                    font: main_font,
                    color: Some(STYLE.inline.image_alt_color),
                    background: None,
                    size: None,
                    strikethrough: false,
                    underline: false,
                });
            }
            Inline::InlineMath(latex) | Inline::DisplayMath(latex) => {
                let display_text = super::latex::render_latex_to_text(latex);
                let math_color = span_ctx.theme.palette().markdown.math;
                spans.push(CachedSpan {
                    text: display_text,
                    font: main_font,
                    color: Some(math_color),
                    background: None,
                    size: None,
                    strikethrough: false,
                    underline: false,
                });
            }
            Inline::FootnoteReference(label) => {
                let link_color = span_ctx.theme.palette().roles.link;
                spans.push(CachedSpan {
                    text: format!("[{label}]"),
                    font: main_font,
                    color: Some(link_color),
                    background: None,
                    size: None,
                    strikethrough: false,
                    underline: false,
                });
            }
        }
    }

    spans
}

fn apply_style_to_children(
    children: &[Inline],
    span_ctx: &SpanCtx,
    main_font: Font,
    style: fn(Font) -> Font,
) -> Vec<CachedSpan> {
    inlines_to_spans_core(children, span_ctx)
        .into_iter()
        .map(|mut span| {
            span.font = style(main_font);
            span
        })
        .collect()
}

pub fn inlines_to_spans<'a>(children: &'a [Inline], span_ctx: &SpanCtx) -> Vec<Span<'a, (), Font>> {
    if !span_ctx.search_query.is_empty() {
        return inlines_to_spans_core(children, span_ctx)
            .into_iter()
            .map(|cs| cs.to_span())
            .collect();
    }

    let inline_hash = hash_inlines(children);
    let base_weight_val = span_ctx.base_weight.map(|w| match w {
        Weight::Thin => 100,
        Weight::ExtraLight => 200,
        Weight::Light => 300,
        Weight::Normal => 400,
        Weight::Medium => 500,
        Weight::Semibold => 600,
        Weight::Bold => 700,
        Weight::ExtraBold => 800,
        Weight::Black => 900,
    });
    let base_color_val = span_ctx.base_color.map(|c| {
        let [r, g, b, a] = c.into_rgba8();
        u32::from_be_bytes([r, g, b, a])
    });
    let font_size_bits = span_ctx.font_size.to_bits();

    let key: SpanCacheKey = (
        inline_hash,
        span_ctx.theme,
        span_ctx.font_family.map(ToString::to_string),
        span_ctx.font_family_mono.map(ToString::to_string),
        base_weight_val,
        base_color_val,
        font_size_bits,
    );

    if let Ok(mut cache) = span_cache().lock()
        && let Some(cached) = cache.get(&key)
    {
        return cached.iter().map(CachedSpan::to_span).collect();
    }

    let cached_list = inlines_to_spans_core(children, span_ctx);
    let result = cached_list.iter().map(CachedSpan::to_span).collect();

    if let Ok(mut cache) = span_cache().lock() {
        cache.put(key, cached_list);
    }

    result
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
            base_weight: None,
            base_color: None,
            font_size: 14.0,
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
            base_weight: None,
            base_color: None,
            font_size: 14.0,
        };

        let spans = inlines_to_spans(&inlines, &ctx);
        assert!(spans.len() >= 2);
        assert_eq!(counter.get(), 1);
    }
}
