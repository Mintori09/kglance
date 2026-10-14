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
    pub ruby_display_mode: crate::core::config::RubyDisplayMode,
    pub footnotes: Option<&'a std::collections::HashMap<String, String>>,
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
    pub link_url: Option<String>,
    pub ruby: Option<String>,
    pub tooltip: Option<String>,
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
    u8,
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
            Inline::Image {
                alt,
                url,
                width,
                height,
            } => {
                hasher.write_u8(8);
                hasher.write(alt.as_bytes());
                hasher.write(url.as_bytes());
                if let Some(w) = width {
                    hasher.write_u32(*w);
                }
                if let Some(h) = height {
                    hasher.write_u32(*h);
                }
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
            Inline::Ruby { text, ruby } => {
                hasher.write_u8(12);
                hasher.write(text.as_bytes());
                hasher.write(ruby.as_bytes());
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
                link_url: None,
                ruby: None,
                tooltip: None,
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
            link_url: None,
            ruby: None,
            tooltip: None,
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
            link_url: None,
            ruby: None,
            tooltip: None,
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
                            link_url: None,
                            ruby: None,
                            tooltip: None,
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
                spans.extend(apply_italic_to_children(children, span_ctx, main_font));
            }
            Inline::Strikethrough(children) => {
                for mut s in inlines_to_spans_core(children, span_ctx) {
                    s.font = main_font;
                    s.strikethrough = true;
                    spans.push(s);
                }
            }
            Inline::Code(code) => {
                let code_tooltip = format!("`{code}`");
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
                        link_url: None,
                        ruby: None,
                        tooltip: Some(code_tooltip),
                    });
                } else {
                    let mut highlighted =
                        highlight_search_in_text(code, span_ctx, code_font, Some(inline_code_fg));
                    for s in &mut highlighted {
                        s.tooltip = Some(code_tooltip.clone());
                    }
                    spans.extend(highlighted);
                }
            }
            Inline::Link {
                text: link_text,
                url,
            } => {
                let link_color = span_ctx.theme.palette().roles.link;
                for mut s in inlines_to_spans_core(link_text, span_ctx) {
                    s.color = Some(link_color);
                    s.underline = true;
                    s.link_url = Some(url.clone());
                    s.tooltip = Some(url.clone());
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
                    link_url: None,
                    ruby: None,
                    tooltip: None,
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
                    link_url: None,
                    ruby: None,
                    tooltip: None,
                });
            }
            Inline::InlineMath(latex) | Inline::DisplayMath(latex) => {
                let display_text = super::latex::render_latex_to_text(latex);
                let math_color = span_ctx.theme.palette().markdown.math;
                let math_tooltip = match inline {
                    Inline::DisplayMath(_) => format!("$${latex}$$"),
                    _ => format!("${latex}$"),
                };
                spans.push(CachedSpan {
                    text: display_text,
                    font: main_font,
                    color: Some(math_color),
                    background: None,
                    size: None,
                    strikethrough: false,
                    underline: false,
                    link_url: None,
                    ruby: None,
                    tooltip: Some(math_tooltip),
                });
            }
            Inline::FootnoteReference(label) => {
                let link_color = span_ctx.theme.palette().roles.link;
                let footnote_tooltip = span_ctx
                    .footnotes
                    .and_then(|m| m.get(label))
                    .map(|content| format!("[^{label}]: {content}"))
                    .unwrap_or_else(|| format!("[^{label}]"));
                spans.push(CachedSpan {
                    text: format!("[{label}]"),
                    font: main_font,
                    color: Some(link_color),
                    background: None,
                    size: None,
                    strikethrough: false,
                    underline: false,
                    link_url: None,
                    ruby: None,
                    tooltip: Some(footnote_tooltip),
                });
            }
            Inline::Ruby { text, ruby } => {
                let (display_str, ruby_annotation, is_underline) = match span_ctx.ruby_display_mode
                {
                    crate::core::config::RubyDisplayMode::Parentheses => {
                        (format!("{text}({ruby})"), None, false)
                    }
                    crate::core::config::RubyDisplayMode::Hidden => (text.clone(), None, false),
                    crate::core::config::RubyDisplayMode::Tooltip => (text.clone(), None, true),
                    crate::core::config::RubyDisplayMode::Stacked => {
                        (text.clone(), Some(ruby.clone()), false)
                    }
                };
                let ruby_tooltip = match span_ctx.ruby_display_mode {
                    crate::core::config::RubyDisplayMode::Tooltip => Some(ruby.clone()),
                    _ => None,
                };
                if search_query.is_empty() {
                    spans.push(CachedSpan {
                        text: display_str,
                        font: main_font,
                        color: span_ctx.base_color,
                        background: None,
                        size: None,
                        strikethrough: false,
                        underline: is_underline,
                        link_url: None,
                        ruby: ruby_annotation,
                        tooltip: ruby_tooltip,
                    });
                } else {
                    let mut highlighted = highlight_search_in_text(
                        &display_str,
                        span_ctx,
                        main_font,
                        span_ctx.base_color,
                    );
                    if let Some(r) = ruby_annotation {
                        for s in &mut highlighted {
                            s.ruby = Some(r.clone());
                        }
                    }
                    if let Some(t) = &ruby_tooltip {
                        for s in &mut highlighted {
                            s.tooltip = Some(t.clone());
                        }
                    }
                    if is_underline {
                        for s in &mut highlighted {
                            s.underline = true;
                        }
                    }
                    spans.extend(highlighted);
                }
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

fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{2E80}'..='\u{2EFF}'   // CJK Radicals Supplement
        | '\u{2F00}'..='\u{2FDF}' // Kangxi Radicals
        | '\u{3000}'..='\u{303F}' // CJK Symbols and Punctuation
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
        | '\u{3100}'..='\u{312F}' // Bopomofo
        | '\u{31F0}'..='\u{31FF}' // Katakana Phonetic Extensions
        | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
        | '\u{4E00}'..='\u{9FFF}' // CJK Unified Ideographs
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{AC00}'..='\u{D7AF}' // Hangul Syllables
        | '\u{20000}'..='\u{2A6DF}' // CJK Extension B
        | '\u{2A700}'..='\u{2B738}' // CJK Extension C
        | '\u{2B740}'..='\u{2B81D}' // CJK Extension D
        | '\u{2B820}'..='\u{2CEAD}' // Extension E
        | '\u{2CEB0}'..='\u{2EBE0}' // Extension F
    )
}

fn apply_italic_to_children(
    children: &[Inline],
    span_ctx: &SpanCtx,
    main_font: Font,
) -> Vec<CachedSpan> {
    let mut result = Vec::new();

    for mut span in inlines_to_spans_core(children, span_ctx) {
        if !span.text.chars().any(is_cjk) {
            span.font = Font {
                style: iced::font::Style::Italic,
                ..main_font
            };
            result.push(span);
            continue;
        }

        let mut chars = span.text.chars().peekable();
        while let Some(&first_char) = chars.peek() {
            let is_current_cjk = is_cjk(first_char);
            let mut chunk = String::new();

            while let Some(&c) = chars.peek() {
                if is_cjk(c) == is_current_cjk {
                    chunk.push(c);
                    chars.next();
                } else {
                    break;
                }
            }

            let mut sub_span = span.clone();
            sub_span.text = chunk;
            sub_span.font = if is_current_cjk {
                main_font
            } else {
                Font {
                    style: iced::font::Style::Italic,
                    ..main_font
                }
            };
            result.push(sub_span);
        }
    }

    result
}

pub(crate) fn extract_links(
    cached_spans: &[CachedSpan],
) -> Vec<crate::ui::components::selectable_text::TextLink> {
    let mut links: Vec<crate::ui::components::selectable_text::TextLink> = Vec::new();
    let mut current_offset = 0;
    for span in cached_spans {
        let span_len = span.text.len();
        if let Some(url) = &span.link_url {
            let span_start = current_offset;
            let span_end = current_offset + span_len;
            if let Some(last) = links.last_mut()
                && last.url == *url
                && last.end == span_start
            {
                last.end = span_end;
            } else {
                links.push(crate::ui::components::selectable_text::TextLink {
                    start: span_start,
                    end: span_end,
                    url: url.clone(),
                });
            }
        }
        current_offset += span_len;
    }
    links
}

pub(crate) fn extract_rubies(
    cached_spans: &[CachedSpan],
) -> Vec<crate::ui::components::selectable_text::TextRuby> {
    let mut rubies: Vec<crate::ui::components::selectable_text::TextRuby> = Vec::new();
    let mut current_offset = 0;
    for span in cached_spans {
        let span_len = span.text.len();
        if let Some(ruby_str) = &span.ruby {
            let span_start = current_offset;
            let span_end = current_offset + span_len;
            if let Some(last) = rubies.last_mut()
                && last.ruby == *ruby_str
                && last.end == span_start
            {
                last.end = span_end;
            } else {
                rubies.push(crate::ui::components::selectable_text::TextRuby {
                    start: span_start,
                    end: span_end,
                    ruby: ruby_str.clone(),
                });
            }
        }
        current_offset += span_len;
    }
    rubies
}

pub(crate) fn extract_tooltips(
    cached_spans: &[CachedSpan],
) -> Vec<crate::ui::components::selectable_text::TextTooltip> {
    let mut tooltips: Vec<crate::ui::components::selectable_text::TextTooltip> = Vec::new();
    let mut current_offset = 0;
    for span in cached_spans {
        let span_len = span.text.len();
        if let Some(content) = &span.tooltip {
            let span_start = current_offset;
            let span_end = current_offset + span_len;
            if let Some(last) = tooltips.last_mut()
                && last.content == *content
                && last.end == span_start
            {
                last.end = span_end;
            } else {
                tooltips.push(crate::ui::components::selectable_text::TextTooltip {
                    start: span_start,
                    end: span_end,
                    content: content.clone(),
                });
            }
        }
        current_offset += span_len;
    }
    tooltips
}

pub type InlinesSpansResult<'a> = (
    Vec<Span<'a, (), Font>>,
    Vec<crate::ui::components::selectable_text::TextLink>,
    Vec<crate::ui::components::selectable_text::TextRuby>,
    Vec<crate::ui::components::selectable_text::TextTooltip>,
);

pub fn inlines_to_spans_with_links<'a>(
    children: &'a [Inline],
    span_ctx: &SpanCtx,
) -> InlinesSpansResult<'a> {
    let cached_list = if !span_ctx.search_query.is_empty() {
        inlines_to_spans_core(children, span_ctx)
    } else {
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
            span_ctx.ruby_display_mode as u8,
        );

        if let Ok(mut cache) = span_cache().lock()
            && let Some(cached) = cache.get(&key)
        {
            cached.clone()
        } else {
            let list = inlines_to_spans_core(children, span_ctx);
            if let Ok(mut cache) = span_cache().lock() {
                cache.put(key, list.clone());
            }
            list
        }
    };

    let links = extract_links(&cached_list);
    let rubies = extract_rubies(&cached_list);
    let tooltips = extract_tooltips(&cached_list);
    let spans = cached_list.into_iter().map(|cs| cs.to_span()).collect();
    (spans, links, rubies, tooltips)
}

pub fn inlines_to_spans<'a>(children: &'a [Inline], span_ctx: &SpanCtx) -> Vec<Span<'a, (), Font>> {
    inlines_to_spans_with_links(children, span_ctx).0
}

#[cfg(test)]
#[path = "inline_spans_test.rs"]
mod tests;
