use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::core::preview::PreviewContent;
use crate::features::text::content::TextContent;
use crate::parsers::ParseError;

const MAX_HIGHLIGHT_LINES: usize = 200;

pub struct TextParser {
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) theme_set: ThemeSet,
}

impl TextParser {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn supported_text_extensions() -> &'static [&'static str] {
    &[
        "rs",
        "py",
        "js",
        "ts",
        "jsx",
        "tsx",
        "html",
        "css",
        "scss",
        "json",
        "md",
        "toml",
        "yml",
        "yaml",
        "xml",
        "sh",
        "bash",
        "zsh",
        "fish",
        "c",
        "h",
        "cpp",
        "hpp",
        "java",
        "kt",
        "swift",
        "go",
        "rb",
        "php",
        "pl",
        "pm",
        "lua",
        "r",
        "sql",
        "graphql",
        "proto",
        "tex",
        "bib",
        "dockerfile",
        "makefile",
        "cmake",
        "gradle",
        "cfg",
        "ini",
        "conf",
        "txt",
        "log",
        "diff",
        "patch",
        "vim",
        "ps1",
        "bat",
    ]
}

pub(crate) fn parse_text(
    parser: &TextParser,
    path: &Path,
) -> Result<Box<dyn PreviewContent<crate::app::Message>>, ParseError> {
    let content =
        std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

    let syntax = parser
        .syntax_set
        .find_syntax_for_file(path)
        .ok()
        .flatten()
        .unwrap_or_else(|| parser.syntax_set.find_syntax_plain_text());

    let language = syntax.name.clone();
    let line_count = content.lines().count();

    if line_count > MAX_HIGHLIGHT_LINES {
        return Ok(Box::new(TextContent {
            content,
            language,
            line_count,
            highlighted_html: None,
        }));
    }

    let theme = parser
        .theme_set
        .themes
        .get("InspiredGitHub")
        .unwrap_or_else(|| &parser.theme_set.themes["base16-ocean.dark"]);

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut highlighted_html = String::new();

    for line in LinesWithEndings::from(&content) {
        let ranges = highlighter
            .highlight_line(line, &parser.syntax_set)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;
        for (style, text) in &ranges {
            let color = style.foreground;
            let hex = if color.a == 0 {
                String::from("inherit")
            } else {
                format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
            };
            if hex != "inherit" {
                highlighted_html.push_str(&format!("<font color='{}'>{}</font>", hex, text));
            } else {
                highlighted_html.push_str(text);
            }
        }
    }

    Ok(Box::new(TextContent {
        content,
        language,
        line_count,
        highlighted_html: Some(highlighted_html),
    }))
}

pub(crate) fn is_supported_text(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| supported_text_extensions().contains(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_rust_file() {
        let mut tmp = tempfile::Builder::new().suffix(".rs").tempfile().unwrap();
        write!(tmp, "fn main() {{ println!(\"hello\"); }}").unwrap();
        let parser = TextParser::new();
        let result = parse_text(&parser, tmp.path()).unwrap();
        assert_eq!(result.content_type(), crate::core::ContentType::Text);
    }

    #[test]
    fn detects_language_by_extension() {
        assert!(is_supported_text(Path::new("test.rs")));
        assert!(is_supported_text(Path::new("test.py")));
        assert!(!is_supported_text(Path::new("test.zip")));
    }

    #[test]
    fn returns_error_for_binary_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[0xFF, 0xFE, 0x00]).unwrap();
        let parser = TextParser::new();
        let result = parse_text(&parser, tmp.path());
        assert!(result.is_err());
    }
}
