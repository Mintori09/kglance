use std::path::Path;

use serde_json::Value;

use crate::parsers::{ParseError, ParsedContent, PreviewParser};

#[derive(Debug, Clone)]
pub struct JsonNode {
    pub key: Option<String>,
    pub value_type: &'static str,
    pub value_preview: String,
    pub children_count: usize,
    pub skip_count: usize,
    pub depth: usize,
}

pub struct JsonParser;

impl JsonParser {
    fn flatten_json(value: &Value, key: Option<String>, depth: usize) -> Vec<JsonNode> {
        let (value_type, value_preview, children) = match value {
            Value::Null => ("Null", "null".into(), vec![]),
            Value::Bool(b) => ("Bool", b.to_string(), vec![]),
            Value::Number(n) => ("Number", n.to_string(), vec![]),
            Value::String(s) => {
                let preview = if s.len() > 100 {
                    format!("\"{}\"...", &s[..97])
                } else {
                    format!("\"{}\"", s)
                };
                ("String", preview, vec![])
            }
            Value::Array(arr) => {
                let mut children = Vec::new();
                for (i, v) in arr.iter().enumerate() {
                    children.extend(Self::flatten_json(v, Some(format!("[{}]", i)), depth + 1));
                }
                ("Array", format!("Array[{}]", arr.len()), children)
            }
            Value::Object(map) => {
                let mut children = Vec::new();
                for (k, v) in map.iter() {
                    children.extend(Self::flatten_json(v, Some(k.clone()), depth + 1));
                }
                ("Object", format!("Object{{{}}}", map.len()), children)
            }
        };

        let skip_count = children.len();
        let node = JsonNode {
            key,
            value_type,
            value_preview,
            children_count: children.len(),
            skip_count,
            depth,
        };

        let mut result = vec![node];
        result.extend(children);
        result
    }
}

impl PreviewParser for JsonParser {
    fn supported_extensions(&self) -> &[&str] {
        &["json"]
    }

    fn is_supported(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    }

    fn parse(&self, path: &Path) -> Result<ParsedContent, ParseError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ParseError::ParseFailed(e.to_string()))?;

        let (nodes, pretty, has_parse_error) = match serde_json::from_str::<Value>(&content) {
            Ok(val) => {
                let pretty = serde_json::to_string_pretty(&val).unwrap_or_else(|_| content.clone());
                let nodes = Self::flatten_json(&val, None, 0);
                (nodes, pretty, false)
            }
            Err(_) => (Vec::new(), content.clone(), true),
        };

        Ok(ParsedContent::Json {
            content,
            pretty,
            nodes,
            has_parse_error,
        })
    }
}
