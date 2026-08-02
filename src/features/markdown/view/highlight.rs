use std::sync::OnceLock;

use iced::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::ui::theme::color::primitive::{MD_DARK_CODE_FG, MD_LIGHT_CODE_FG};

fn syntax_set() -> &'static SyntaxSet {
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    SS.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme_set() -> &'static ThemeSet {
    static TS: OnceLock<ThemeSet> = OnceLock::new();
    TS.get_or_init(ThemeSet::load_defaults)
}

fn syntect_to_iced_color(c: syntect::highlighting::Color) -> Color {
    Color::from_rgba(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}

use crate::ui::theme::AppTheme;

pub(crate) fn highlight_code<'a>(
    lang: &Option<String>,
    code: &'a str,
    app_theme: AppTheme,
) -> Vec<Vec<(Color, &'a str)>> {
    let ss = syntax_set();
    let ts = theme_set();

    let syntax = lang
        .as_deref()
        .and_then(|l| ss.find_syntax_by_token(l))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme_name = if app_theme.is_dark() {
        "base16-eighties.dark"
    } else {
        "InspiredGitHub"
    };

    let syntect_theme = ts
        .themes
        .get(theme_name)
        .unwrap_or_else(|| &ts.themes["base16-ocean.dark"]);

    let mut highlighter = HighlightLines::new(syntax, syntect_theme);
    let mut result = Vec::new();

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter
            .highlight_line(line, ss)
            .unwrap_or_else(|_| vec![]);

        if ranges.is_empty() {
            let fg = syntect_theme
                .settings
                .foreground
                .map(syntect_to_iced_color)
                .unwrap_or_else(|| {
                    if app_theme.is_dark() {
                        MD_DARK_CODE_FG
                    } else {
                        MD_LIGHT_CODE_FG
                    }
                });
            let t = line.strip_suffix('\n').unwrap_or(line);
            result.push(vec![(fg, t)]);
            continue;
        }

        let line_spans: Vec<(Color, &'a str)> = ranges
            .iter()
            .map(|(style, text)| {
                let t = text.strip_suffix('\n').unwrap_or(text);
                (syntect_to_iced_color(style.foreground), t)
            })
            .collect();
        result.push(line_spans);
    }

    result
}
