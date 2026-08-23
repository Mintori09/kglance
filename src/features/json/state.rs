use crate::core::types::{JsonState, KglanceState};
use crate::features::json::JsonNode;
use std::collections::HashSet;

pub fn populate_state(
    state: &mut KglanceState,
    nodes: &[JsonNode],
    pretty: &str,
    has_parse_error: bool,
) {
    let old_scroll = state.json.scroll_y;
    let old_tree_mode = state.json.tree_mode;

    let mut expanded = HashSet::new();
    for (i, node) in nodes.iter().enumerate() {
        if node.children_count > 0 {
            expanded.insert(i);
        }
    }

    let minified = serde_json::from_str::<serde_json::Value>(pretty)
        .ok()
        .and_then(|v| serde_json::to_string(&v).ok())
        .unwrap_or_else(|| pretty.to_string());

    state.json = JsonState {
        nodes: nodes.to_vec(),
        expanded,
        raw_content: pretty.to_string(),
        pretty_content: pretty.to_string(),
        tree_mode: old_tree_mode,
        scroll_y: old_scroll,
        has_parse_error,
        raw_editor: iced::widget::text_editor::Content::with_text(pretty),
        search_visible: false,
        search_query: String::new(),
        search_matches: Vec::new(),
        search_match_index: 0,
        search_info: String::new(),
        minified_content: minified,
        raw_pretty: true,
        active_node: None,
    };
    state.file_type_text = "JSON Document".to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_populate_state_expands_all_containers() {
        let mut state = KglanceState::default();
        let nodes = vec![
            JsonNode {
                key: None,
                value_type: "Object",
                value_preview: "{ 2 items }".to_string(),
                children_count: 2,
                skip_count: 2,
                depth: 0,
                parent_index: None,
            },
            JsonNode {
                key: Some("arr".to_string()),
                value_type: "Array",
                value_preview: "[ 1 item ]".to_string(),
                children_count: 1,
                skip_count: 1,
                depth: 1,
                parent_index: Some(0),
            },
            JsonNode {
                key: Some("[0]".to_string()),
                value_type: "Number",
                value_preview: "42".to_string(),
                children_count: 0,
                skip_count: 0,
                depth: 2,
                parent_index: Some(1),
            },
        ];

        populate_state(&mut state, &nodes, "{}", false);
        assert!(state.json.expanded.contains(&0));
        assert!(state.json.expanded.contains(&1));
        assert!(!state.json.expanded.contains(&2));
    }
}
