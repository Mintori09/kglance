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

    let syntax = find_syntax(ss, lang.as_deref(), code);

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

pub(crate) fn find_syntax<'a>(
    ss: &'a SyntaxSet,
    lang: Option<&str>,
    code: &str,
) -> &'a syntect::parsing::SyntaxReference {
    if let Some(l) = lang {
        let token = l.trim();
        if !token.is_empty() {
            if let Some(syntax) = ss
                .find_syntax_by_token(token)
                .or_else(|| ss.find_syntax_by_extension(token))
                .or_else(|| ss.find_syntax_by_name(token))
            {
                return syntax;
            }
            let fallback_alias = match token.to_lowercase().as_str() {
                "ts" | "typescript" | "tsx" => Some("js"),
                "rs" | "rust" => Some("rs"),
                "py" | "python" | "python3" => Some("py"),
                "sh" | "bash" | "zsh" | "shell" => Some("sh"),
                "yml" | "yaml" => Some("yaml"),
                _ => None,
            };
            if let Some(alias) = fallback_alias
                && let Some(syntax) = ss.find_syntax_by_token(alias)
            {
                return syntax;
            }
        }
    }

    if let Some(detected) = detect_code_language(code) {
        if let Some(syntax) = ss
            .find_syntax_by_token(detected)
            .or_else(|| ss.find_syntax_by_extension(detected))
            .or_else(|| ss.find_syntax_by_name(detected))
        {
            return syntax;
        }
        if matches!(detected, "ts" | "typescript" | "tsx")
            && let Some(syntax) = ss.find_syntax_by_token("js")
        {
            return syntax;
        }
    }

    if let Some(first_line) = code.lines().next()
        && let Some(syntax) = ss.find_syntax_by_first_line(first_line)
    {
        return syntax;
    }

    ss.find_syntax_plain_text()
}

pub(crate) fn detect_code_language(code: &str) -> Option<&'static str> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return None;
    }

    // TypeScript / JavaScript
    let has_ts_types = trimmed.contains(": Promise<")
        || trimmed.contains(": Result<")
        || trimmed.contains("): Promise")
        || trimmed.contains("): Result")
        || trimmed.contains("interface ")
        || (trimmed.contains("type ")
            && (trimmed.contains(" = {")
                || trimmed.contains(" = string")
                || trimmed.contains(" = number")))
        || (trimmed.contains("class ")
            && (trimmed.contains("implements ")
                || trimmed.contains("private ")
                || trimmed.contains("public ")
                || trimmed.contains("protected ")))
        || trimmed.contains("as const")
        || trimmed.contains(": string")
        || trimmed.contains(": number")
        || trimmed.contains(": boolean")
        || trimmed.contains(": any")
        || trimmed.contains("<T>");

    if has_ts_types {
        return Some("ts");
    }

    let has_js_keywords = trimmed.contains("const ")
        || trimmed.contains("let ")
        || trimmed.contains("var ")
        || trimmed.contains("function ")
        || trimmed.contains("async ")
        || trimmed.contains("await ")
        || trimmed.contains("console.log")
        || trimmed.contains("=> {")
        || (trimmed.contains("import ")
            && (trimmed.contains("from '") || trimmed.contains("from \"")));

    if has_js_keywords {
        return Some("js");
    }

    // Rust
    let has_rust = trimmed.contains("fn ")
        || trimmed.contains("let mut ")
        || trimmed.contains("pub fn ")
        || trimmed.contains("pub struct ")
        || trimmed.contains("impl ")
        || trimmed.contains("println!")
        || trimmed.contains("eprintln!")
        || trimmed.contains("use std::")
        || trimmed.contains("#[derive(")
        || (trimmed.contains("match ") && trimmed.contains(" => "));

    if has_rust {
        return Some("rs");
    }

    // Python
    let has_python = trimmed.contains("def ")
        || trimmed.contains("elif ")
        || (trimmed.contains("import ") && !trimmed.contains("from '"))
        || (trimmed.contains("from ") && trimmed.contains(" import "))
        || trimmed.contains("self.")
        || (trimmed.contains("class ") && trimmed.contains(":"));

    if has_python {
        return Some("py");
    }

    // C / C++
    let has_cpp = trimmed.contains("#include <")
        || trimmed.contains("#include \"")
        || trimmed.contains("std::")
        || trimmed.contains("cout <<")
        || trimmed.contains("int main(");

    if has_cpp {
        return Some("cpp");
    }

    // Go
    let has_go = trimmed.contains("package ")
        || trimmed.contains("func ")
        || trimmed.contains("fmt.Println")
        || trimmed.contains("fmt.Printf");

    if has_go {
        return Some("go");
    }

    // HTML / XML
    if trimmed.starts_with("<!DOCTYPE html")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<?xml")
    {
        return Some("html");
    }

    // SQL
    let upper = trimmed.to_uppercase();
    if upper.starts_with("SELECT ")
        || upper.starts_with("INSERT INTO ")
        || upper.starts_with("CREATE TABLE ")
        || upper.starts_with("UPDATE ")
    {
        return Some("sql");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_typescript_code() {
        let code = r#"
// Secure user authentication with result types
class AuthenticationService {
  async authenticateUser(credentials: LoginCredentials): Promise<Result<AuthenticatedUser, AuthenticationError>> {
    try {
      const validationResult = this.validateCredentials(credentials);
      if (!validationResult.success) {
        return failure(new ValidationError('Invalid credentials format', 'credentials', credentials.username));
      }
    } catch (error) {
      return failure(new SystemError('Authentication system error', { originalError: error }));
    }
  }
}
"#;
        assert_eq!(detect_code_language(code), Some("ts"));
    }

    #[test]
    fn test_highlight_code_with_user_snippet() {
        let code = r#"
// Secure user authentication with result types
class AuthenticationService {
  async authenticateUser(credentials: LoginCredentials): Promise<Result<AuthenticatedUser, AuthenticationError>> {
    try {
      const validationResult = this.validateCredentials(credentials);
    } catch (error) {
      return failure(error);
    }
  }
}
"#;
        let highlighted = highlight_code(&None, code, AppTheme::Dark);
        assert!(!highlighted.is_empty());
        // Verify that different tokens have distinct colors
        let mut distinct_colors = std::collections::HashSet::new();
        for line in &highlighted {
            for (color, _) in line {
                distinct_colors.insert(format!("{color:?}"));
            }
        }
        assert!(
            distinct_colors.len() > 1,
            "Syntax highlighting should produce multiple distinct colors"
        );
    }
}
