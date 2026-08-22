mod flatten;
mod handle;
mod layout;
mod mermaid;
mod types;

#[cfg(test)]
mod tests;

pub use flatten::{
    flatten_inlines, flatten_inlines_plain, flatten_inlines_toc, flatten_inlines_visual,
};
pub use handle::parse_to_blocks;
pub use layout::{estimated_block_height, extract_toc, slugify};
pub use mermaid::render_mermaid_to_png;
pub use types::{AlertKind, Block, Inline, ListItem, TableBlock, TableCell};

use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;

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

        let images = handle::extract_images(&raw, parent);
        let blocks = parse_to_blocks(&raw);

        Ok(ParsedContent::Markdown {
            content: raw,
            images,
            blocks,
        })
    }
}
