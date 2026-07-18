use std::path::Path;
use syntect::parsing::SyntaxSet;

use crate::parser::{PreviewParser, ParsedContent, ParseError};

pub struct TextParser {
    syntax_set: SyntaxSet,
}

impl TextParser {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewParser for TextParser {
    fn supported_extensions(&self) -> &[&str] {
        &[
            "rs", "py", "js", "ts", "jsx", "tsx", "html", "css", "scss", "json",
            "md", "toml", "yml", "yaml", "xml", "sh", "bash", "zsh", "fish",
            "c", "h", "cpp", "hpp", "java", "kt", "swift", "go", "rb", "php",
            "pl", "pm", "lua", "r", "sql", "graphql", "proto", "tex", "bib",
            "dockerfile", "makefile", "cmake", "gradle", "cfg", "ini", "conf",
            "txt", "log", "diff", "patch", "vim", "ps1", "bat",
        ]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| self.supported_extensions().contains(&e))
            .unwrap_or(false)
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let line_count = content.lines().count();
        let syntax = self
            .syntax_set
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let language = syntax.name.clone();

        Ok(ParsedContent::Text {
            content,
            language,
            line_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_rust_file() {
        let mut tmp = tempfile::Builder::new()
            .suffix(".rs")
            .tempfile()
            .unwrap();
        write!(tmp, "fn main() {{ println!(\"hello\"); }}").unwrap();
        let parser = TextParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Text { content, language, line_count } => {
                assert_eq!(language, "Rust");
                assert_eq!(line_count, 1);
                assert!(content.contains("fn main"));
            }
            _ => panic!("expected Text variant"),
        }
    }

    #[test]
    fn detects_language_by_extension() {
        let parser = TextParser::new();
        assert!(parser.is_supported(Path::new("test.rs")));
        assert!(parser.is_supported(Path::new("test.py")));
        assert!(!parser.is_supported(Path::new("test.zip")));
    }

    #[test]
    fn returns_error_for_binary_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[0xFF, 0xFE, 0x00]).unwrap();
        let parser = TextParser::new();
        let result = parser.parse(tmp.path());
        assert!(result.is_err());
    }
}
