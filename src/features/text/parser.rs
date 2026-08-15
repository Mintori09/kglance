use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;

pub struct TextParser;

impl TextParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

fn extension_to_language(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "rs" => "Rust",
        "py" | "pyw" => "Python",
        "js" | "mjs" | "cjs" => "JavaScript",
        "ts" | "mts" | "cts" => "TypeScript",
        "jsx" => "JSX",
        "tsx" => "TSX",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "json" | "jsonc" => "JSON",
        "md" | "markdown" => "Markdown",
        "toml" => "TOML",
        "yml" | "yaml" => "YAML",
        "xml" | "svg" => "XML",
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "c" | "h" => "C",
        "cpp" | "hpp" | "cc" | "cxx" => "C++",
        "java" => "Java",
        "kt" => "Kotlin",
        "swift" => "Swift",
        "go" => "Go",
        "rb" => "Ruby",
        "php" => "PHP",
        "pl" | "pm" => "Perl",
        "lua" => "Lua",
        "r" => "R",
        "sql" => "SQL",
        "graphql" => "GraphQL",
        "proto" => "Protobuf",
        "tex" | "bib" => "TeX",
        "dockerfile" => "Dockerfile",
        "makefile" => "Makefile",
        "cmake" => "CMake",
        "gradle" => "Gradle",
        "cfg" | "ini" | "conf" => "Config",
        "log" => "Log",
        "diff" | "patch" => "Diff",
        "vim" => "VimL",
        "ps1" => "PowerShell",
        "bat" => "Batch",
        "txt" => "Plain Text",
        _ => "Plain Text",
    }
}

impl PreviewParser for TextParser {
    fn supported_extensions(&self) -> &[&str] {
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

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.supported_extensions().contains(&e))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let language = path
            .extension()
            .and_then(|e| e.to_str())
            .map(extension_to_language)
            .unwrap_or("Plain Text")
            .to_string();

        let line_count = content.lines().count();

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
        let mut tmp = tempfile::Builder::new().suffix(".rs").tempfile().unwrap();
        write!(tmp, "fn main() {{\n    println!(\"hello\");\n}}").unwrap();
        let parser = TextParser::new();
        let result = parser.parse(tmp.path()).unwrap();
        match result {
            ParsedContent::Text {
                content,
                language,
                line_count,
            } => {
                assert_eq!(language, "Rust");
                assert_eq!(line_count, 3);
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
