mod handle;
pub mod types;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::features::common::parser::traits::{ParseError, PreviewParser};
use crate::features::common::parser::types::ParsedContent;

pub struct JsonParser;

const SUPPORTED_EXTENSION: &str = "json";

impl PreviewParser for JsonParser {
    fn supported_extensions(&self) -> &[&str] {
        &[SUPPORTED_EXTENSION]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case(SUPPORTED_EXTENSION))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let content =
            fs::read_to_string(path).map_err(|err| ParseError::ParseFailed(err.to_string()))?;

        let (mut nodes, pretty, has_parse_error) = match serde_json::from_str::<Value>(&content) {
            Ok(parsed_json) => {
                let pretty =
                    serde_json::to_string_pretty(&parsed_json).unwrap_or_else(|_| content.clone());
                let nodes = Self::flatten_json(&parsed_json, None, 0);
                (nodes, pretty, false)
            }
            Err(_) => (Vec::new(), content.clone(), true),
        };

        Self::assign_parent_indices(&mut nodes);

        Ok(ParsedContent::Json {
            content,
            pretty,
            nodes,
            has_parse_error,
        })
    }
}
