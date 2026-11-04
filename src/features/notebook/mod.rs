pub mod parser;
pub mod types;
pub mod view;

pub use view::view_notebook;

use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;

pub struct NotebookParser;

const SUPPORTED_EXTENSIONS: &[&str] = &["ipynb"];

impl PreviewParser for NotebookParser {
    fn supported_extensions(&self) -> &[&str] {
        SUPPORTED_EXTENSIONS
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                SUPPORTED_EXTENSIONS
                    .iter()
                    .any(|&ext| extension.eq_ignore_ascii_case(ext))
            })
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let mmap = crate::core::MmapFile::open(path)
            .map_err(|err| ParseError::ParseFailed(err.to_string()))?;

        let content = mmap
            .as_str()
            .map_err(|e| ParseError::ParseFailed(format!("Invalid UTF-8: {e}")))?
            .to_string();

        match parser::parse_notebook(&content) {
            Ok(data) => Ok(ParsedContent::Notebook(data)),
            Err(_) => {
                // Fallback to JSON parser if notebook parsing fails
                let (mut nodes, pretty, has_parse_error) =
                    crate::features::json::JsonParser::parse_content(&content, "json");
                crate::features::json::JsonParser::assign_parent_indices(&mut nodes);

                Ok(ParsedContent::Json {
                    content,
                    pretty,
                    nodes,
                    has_parse_error,
                })
            }
        }
    }
}
