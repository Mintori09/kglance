use serde_json::Value;

use crate::features::json::parser::JsonParser;

use crate::features::json::parser::types::JsonNode;

const MAX_STRING_PREVIEW_LEN: usize = 100;
const TRUNCATED_STRING_TAKE_LEN: usize = 97;

impl JsonParser {
    pub fn format_string_preview(value: &str) -> String {
        let single_line: String = value
            .chars()
            .map(|c| match c {
                '\n' | '\r' => ' ',
                '\t' => ' ',
                _ => c,
            })
            .collect();
        let trimmed = single_line.trim();
        if trimmed.chars().count() > MAX_STRING_PREVIEW_LEN {
            let truncated: String = trimmed.chars().take(TRUNCATED_STRING_TAKE_LEN).collect();
            format!("\"{truncated}\"...")
        } else {
            format!("\"{trimmed}\"")
        }
    }

    pub fn strip_jsonc_comments(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        let len = chars.len();
        let mut in_string = false;
        let mut escape = false;

        while i < len {
            let c = chars[i];

            if in_string {
                result.push(c);
                if escape {
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }

            if c == '"' {
                in_string = true;
                result.push(c);
                i += 1;
                continue;
            }

            // Single line comment //
            if c == '/' && i + 1 < len && chars[i + 1] == '/' {
                i += 2;
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            // Multi line comment /* ... */
            if c == '/' && i + 1 < len && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                if i + 1 < len {
                    i += 2;
                }
                continue;
            }

            result.push(c);
            i += 1;
        }

        result
    }

    pub fn parse_content(content: &str, ext: &str) -> (Vec<JsonNode>, String, bool) {
        if ext == "jsonl" || ext == "ndjson" {
            let mut items = Vec::new();
            let mut all_valid = true;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(val) => items.push(val),
                    Err(_) => {
                        all_valid = false;
                    }
                }
            }

            if !items.is_empty() {
                let val_arr = Value::Array(items);
                let pretty =
                    serde_json::to_string_pretty(&val_arr).unwrap_or_else(|_| content.to_string());
                let nodes = Self::flatten_json(&val_arr, None, 0);
                return (nodes, pretty, !all_valid);
            }
        }

        // Try standard JSON or JSONC
        let cleaned = if ext == "jsonc" || content.contains("//") || content.contains("/*") {
            Self::strip_jsonc_comments(content)
        } else {
            content.to_string()
        };

        match serde_json::from_str::<Value>(&cleaned) {
            Ok(parsed_json) => {
                let pretty = serde_json::to_string_pretty(&parsed_json)
                    .unwrap_or_else(|_| content.to_string());
                let nodes = Self::flatten_json(&parsed_json, None, 0);
                (nodes, pretty, false)
            }
            Err(_) => (Vec::new(), content.to_string(), true),
        }
    }

    pub fn flatten_json(value: &Value, key: Option<String>, depth: usize) -> Vec<JsonNode> {
        let (value_type, value_preview, children) = match value {
            Value::Null => ("Null", "null".to_string(), vec![]),
            Value::Bool(b) => ("Bool", b.to_string(), vec![]),
            Value::Number(n) => ("Number", n.to_string(), vec![]),
            Value::String(s) => ("String", Self::format_string_preview(s), vec![]),
            Value::Array(arr) => {
                let children = arr
                    .iter()
                    .enumerate()
                    .flat_map(|(index, element)| {
                        Self::flatten_json(element, Some(format!("[{index}]")), depth + 1)
                    })
                    .collect();
                let item_str = if arr.len() == 1 { "item" } else { "items" };
                ("Array", format!("[ {} {} ]", arr.len(), item_str), children)
            }
            Value::Object(map) => {
                let children = map
                    .iter()
                    .flat_map(|(child_key, child_value)| {
                        Self::flatten_json(child_value, Some(child_key.clone()), depth + 1)
                    })
                    .collect();
                let prop_str = if map.len() == 1 { "item" } else { "items" };
                (
                    "Object",
                    format!("{{ {} {} }}", map.len(), prop_str),
                    children,
                )
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

        let mut nodes = Vec::with_capacity(1 + children_count);
        nodes.push(node);
        nodes.extend(children);
        nodes
    }

    pub fn assign_parent_indices(nodes: &mut [JsonNode]) {
        let mut ancestor_stack: Vec<usize> = Vec::new();

        for node_index in 0..nodes.len() {
            let depth = nodes[node_index].depth;

            while ancestor_stack
                .last()
                .is_some_and(|&parent_index| nodes[parent_index].depth >= depth)
            {
                ancestor_stack.pop();
            }

            nodes[node_index].parent_index = ancestor_stack.last().copied();
            ancestor_stack.push(node_index);
        }
    }

    pub fn extract_subtree_json(nodes: &[JsonNode], index: usize) -> Option<String> {
        let node = nodes.get(index)?;
        if node.children_count == 0 {
            return Some(node.value_preview.clone());
        }

        fn build_value(nodes: &[JsonNode], start: usize) -> (Value, usize) {
            let node = &nodes[start];
            match node.value_type {
                "Null" => (Value::Null, 1),
                "Bool" => {
                    let b = node.value_preview == "true";
                    (Value::Bool(b), 1)
                }
                "Number" => {
                    if let Ok(n) = node.value_preview.parse::<i64>() {
                        (Value::Number(n.into()), 1)
                    } else if let Ok(f) = node.value_preview.parse::<f64>() {
                        (
                            serde_json::Number::from_f64(f)
                                .map(Value::Number)
                                .unwrap_or(Value::Null),
                            1,
                        )
                    } else {
                        (Value::Null, 1)
                    }
                }
                "String" => {
                    let raw = &node.value_preview;
                    let trimmed = if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
                        &raw[1..raw.len() - 1]
                    } else {
                        raw.as_str()
                    };
                    (Value::String(trimmed.to_string()), 1)
                }
                "Array" => {
                    let mut arr = Vec::new();
                    let mut curr = start + 1;
                    let target_end = start + 1 + node.skip_count;
                    while curr < target_end && curr < nodes.len() {
                        let (child_val, consumed) = build_value(nodes, curr);
                        arr.push(child_val);
                        curr += consumed;
                    }
                    (Value::Array(arr), 1 + node.skip_count)
                }
                "Object" => {
                    let mut map = serde_json::Map::new();
                    let mut curr = start + 1;
                    let target_end = start + 1 + node.skip_count;
                    while curr < target_end && curr < nodes.len() {
                        let child_node = &nodes[curr];
                        let key_str = child_node.key.clone().unwrap_or_default();
                        let (child_val, consumed) = build_value(nodes, curr);
                        map.insert(key_str, child_val);
                        curr += consumed;
                    }
                    (Value::Object(map), 1 + node.skip_count)
                }
                _ => (Value::Null, 1),
            }
        }

        let (val, _) = build_value(nodes, index);
        serde_json::to_string_pretty(&val).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_jsonc_comments() {
        let jsonc = r#"{
            // This is a comment
            "name": "Kglance", /* inline comment */
            "url": "http://example.com/test//not-a-comment"
        }"#;
        let cleaned = JsonParser::strip_jsonc_comments(jsonc);
        let parsed: serde_json::Value = serde_json::from_str(&cleaned).expect("valid json");
        assert_eq!(parsed["name"], "Kglance");
        assert_eq!(parsed["url"], "http://example.com/test//not-a-comment");
    }

    #[test]
    fn test_parse_jsonl() {
        let jsonl = "{\"id\": 1, \"name\": \"a\"}\n{\"id\": 2, \"name\": \"b\"}\n";
        let (nodes, pretty, has_err) = JsonParser::parse_content(jsonl, "jsonl");
        assert!(!has_err);
        assert!(!nodes.is_empty());
        assert!(pretty.contains("\"id\": 1"));
        assert!(pretty.contains("\"id\": 2"));
    }

    #[test]
    fn test_extract_subtree_json() {
        let json_text = r#"{"user": {"name": "Alice", "age": 30}, "active": true}"#;
        let (mut nodes, _, _) = JsonParser::parse_content(json_text, "json");
        JsonParser::assign_parent_indices(&mut nodes);

        let user_idx = nodes
            .iter()
            .position(|n| n.key.as_deref() == Some("user"))
            .unwrap();
        let subtree = JsonParser::extract_subtree_json(&nodes, user_idx).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&subtree).expect("valid subtree json");
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
    }
}
