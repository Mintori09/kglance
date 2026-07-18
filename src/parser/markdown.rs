use std::path::Path;

use crate::parser::{ImageRef, ParsedContent, ParseError, PreviewParser};

pub struct MarkdownParser;

impl PreviewParser for MarkdownParser {
    fn supported_extensions(&self) -> &[&str] {
        &["md", "markdown", "mdown", "mdwn", "mkd", "mkdn"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| self.supported_extensions().contains(&e))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        Ok(ParsedContent::Markdown {
            content,
            images: Vec::new(),
        })
    }
}
