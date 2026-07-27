mod flatten;
mod layout;
mod mermaid;
mod parser;
mod types;

#[cfg(test)]
mod tests;

pub use flatten::{flatten_inlines, flatten_inlines_toc};
pub use layout::{estimated_block_height, extract_toc};
pub use mermaid::render_mermaid_to_png;
pub use parser::parse_to_blocks;
pub use types::{Block, Inline, ListItem, TableBlock, TableCell};

use std::path::Path;

use crate::parsers::{ParseError, ParsedContent, PreviewParser};

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

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
        let parent = path.parent().unwrap_or(Path::new("."));
        let raw =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let images = parser::extract_images(&raw, parent);
        let blocks = parse_to_blocks(&raw);

        Ok(ParsedContent::Markdown {
            content: raw,
            images,
            blocks,
        })
    }
}
