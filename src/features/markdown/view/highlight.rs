use std::sync::{Mutex, OnceLock};

use iced::Color;
use lru::LruCache;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

use crate::ui::theme::AppTheme;
use crate::ui::theme::color::primitive::syntect_to_iced_color;

type HighlightCacheKey = (Option<String>, String, AppTheme);
type HighlightSpanRanges = Vec<Vec<(Color, usize, usize)>>;

fn highlight_cache() -> &'static Mutex<LruCache<HighlightCacheKey, HighlightSpanRanges>> {
    static CACHE: OnceLock<Mutex<LruCache<HighlightCacheKey, HighlightSpanRanges>>> =
        OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(LruCache::new(
            std::num::NonZeroUsize::new(256).expect("256 is non-zero"),
        ))
    })
}

pub(crate) fn highlight_code<'a>(
    lang: &Option<String>,
    code: &'a str,
    app_theme: AppTheme,
) -> Vec<Vec<(Color, &'a str)>> {
    let cache_key = (lang.clone(), code.to_string(), app_theme);

    if let Ok(mut cache) = highlight_cache().lock()
        && let Some(ranges) = cache.get(&cache_key)
    {
        return ranges
            .iter()
            .map(|line_ranges| {
                line_ranges
                    .iter()
                    .map(|(color, start, end)| (*color, &code[*start..*end]))
                    .collect()
            })
            .collect();
    }

    let ss = syntax_set();
    let ts = theme_set();

    let syntax = lang
        .as_deref()
        .and_then(|l| ss.find_syntax_by_token(l))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme_name = app_theme.syntect_theme();

    let syntect_theme = ts
        .themes
        .get(theme_name)
        .or_else(|| ts.themes.values().next())
        .expect("ThemeSet is empty");

    let mut highlighter = HighlightLines::new(syntax, syntect_theme);
    let mut result = Vec::new();
    let mut cached_ranges: HighlightSpanRanges = Vec::new();

    let base_ptr = code.as_ptr() as usize;

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter
            .highlight_line(line, ss)
            .unwrap_or_else(|_| vec![]);

        if ranges.is_empty() {
            let fg = app_theme.resolve_code_fg(syntect_theme);
            let t = line.strip_suffix('\n').unwrap_or(line);
            let start = t.as_ptr() as usize - base_ptr;
            let end = start + t.len();

            result.push(vec![(fg, t)]);
            cached_ranges.push(vec![(fg, start, end)]);
            continue;
        }

        let mut line_spans = Vec::new();
        let mut line_cached_ranges = Vec::new();
        for (style, text) in ranges {
            let t = text.strip_suffix('\n').unwrap_or(text);
            let color = syntect_to_iced_color(style.foreground);
            let start = t.as_ptr() as usize - base_ptr;
            let end = start + t.len();
            line_spans.push((color, t));
            line_cached_ranges.push((color, start, end));
        }
        result.push(line_spans);
        cached_ranges.push(line_cached_ranges);
    }

    if let Ok(mut cache) = highlight_cache().lock() {
        cache.put(cache_key, cached_ranges);
    }

    result
}

pub fn pre_highlight_blocks(blocks: &[crate::parsers::markdown::Block]) {
    for block in blocks {
        match block {
            crate::parsers::markdown::Block::CodeBlock { lang, code, .. } => {
                highlight_code(lang, code, AppTheme::Dark);
                highlight_code(lang, code, AppTheme::Light);
            }
            crate::parsers::markdown::Block::List { items, .. } => {
                for item in items {
                    pre_highlight_blocks(&item.sub_blocks);
                }
            }
            crate::parsers::markdown::Block::Quote(sub)
            | crate::parsers::markdown::Block::Alert { content: sub, .. }
            | crate::parsers::markdown::Block::FootnoteDefinition { content: sub, .. } => {
                pre_highlight_blocks(sub);
            }
            _ => {}
        }
    }
}
