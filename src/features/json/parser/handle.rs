use serde_json::Value;

use crate::features::json::parser::JsonParser;

use crate::features::json::parser::types::JsonNode;

const MAX_STRING_PREVIEW_LEN: usize = 100;
const TRUNCATED_STRING_TAKE_LEN: usize = 97;

impl JsonParser {
    pub fn format_string_preview(value: &str) -> String {
        if value.chars().count() > MAX_STRING_PREVIEW_LEN {
            let truncated: String = value.chars().take(TRUNCATED_STRING_TAKE_LEN).collect();
            format!("\"{truncated}\"...")
        } else {
            format!("\"{value}\"")
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
                ("Array", format!("Array[{}]", arr.len()), children)
            }
            Value::Object(map) => {
                let children = map
                    .iter()
                    .flat_map(|(child_key, child_value)| {
                        Self::flatten_json(child_value, Some(child_key.clone()), depth + 1)
                    })
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
}
