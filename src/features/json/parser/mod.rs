mod handle;
pub mod types;
use std::fs;
use std::path::Path;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;

pub struct JsonParser;

const SUPPORTED_EXTENSIONS: &[&str] = &["json", "jsonc", "jsonl", "ndjson"];

impl PreviewParser for JsonParser {
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
        let content =
            fs::read_to_string(path).map_err(|err| ParseError::ParseFailed(err.to_string()))?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let (mut nodes, pretty, has_parse_error) = Self::parse_content(&content, &ext);
        Self::assign_parent_indices(&mut nodes);

        Ok(ParsedContent::Json {
            content,
            pretty,
            nodes,
            has_parse_error,
        })
    }
}
