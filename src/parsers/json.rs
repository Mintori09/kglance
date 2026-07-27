use std::path::Path;

use serde_json::Value;

use crate::parsers::{ParseError, ParsedContent, PreviewParser};

const MAX_STRING_PREVIEW_LEN: usize = 100;
const TRUNCATED_STRING_TAKE_LEN: usize = 97;
const SUPPORTED_EXTENSION: &str = "json";

#[derive(Debug, Clone)]
pub struct JsonNode {
    pub key: Option<String>,
    pub value_type: &'static str,
    pub value_preview: String,
    pub children_count: usize,
    pub skip_count: usize,
    pub depth: usize,
    pub parent_index: Option<usize>,
}

pub struct JsonParser;

impl JsonParser {
    fn format_string_preview(s: &str) -> String {
        if s.chars().count() > MAX_STRING_PREVIEW_LEN {
            let truncated: String = s.chars().take(TRUNCATED_STRING_TAKE_LEN).collect();
            format!("\"{truncated}\"...")
        } else {
            format!("\"{s}\"")
        }
    }

    fn flatten_json(value: &Value, key: Option<String>, depth: usize) -> Vec<JsonNode> {
        let (value_type, value_preview, children) = match value {
            Value::Null => ("Null", "null".into(), vec![]),
            Value::Bool(b) => ("Bool", b.to_string(), vec![]),
            Value::Number(n) => ("Number", n.to_string(), vec![]),
            Value::String(s) => ("String", Self::format_string_preview(s), vec![]),
            Value::Array(arr) => {
                let children = arr
                    .iter()
                    .enumerate()
                    .flat_map(|(i, v)| Self::flatten_json(v, Some(format!("[{i}]")), depth + 1))
                    .collect();
                ("Array", format!("Array[{}]", arr.len()), children)
            }
            Value::Object(map) => {
                let children = map
                    .iter()
                    .flat_map(|(k, v)| Self::flatten_json(v, Some(k.clone()), depth + 1))
                    .collect();
                ("Object", format!("Object{{{}}}", map.len()), children)
            }
        };

        let children_count = children.len();
        let node = JsonNode {
            key,
            value_type,
            value_preview,
            children_count,
            skip_count: children_count,
            depth,
            parent_index: None,
        };

        let mut result = Vec::with_capacity(1 + children_count);
        result.push(node);
        result.extend(children);
        result
    }

    fn assign_parent_indices(nodes: &mut [JsonNode]) {
        let mut stack: Vec<usize> = Vec::new();
        for i in 0..nodes.len() {
            while stack
                .last()
                .is_some_and(|&parent_idx| nodes[parent_idx].depth >= nodes[i].depth)
            {
                stack.pop();
            }
            nodes[i].parent_index = stack.last().copied();
            stack.push(i);
        }
    }
}

impl PreviewParser for JsonParser {
    fn supported_extensions(&self) -> &[&str] {
        &[SUPPORTED_EXTENSION]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case(SUPPORTED_EXTENSION))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let content = std::fs::read_to_string(path)
            .map_err(|err| ParseError::ParseFailed(err.to_string()))?;

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
