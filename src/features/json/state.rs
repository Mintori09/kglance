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
        if node.depth == 0 {
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
        minified_content: minified,
        raw_pretty: true,
        active_node: None,
        editing_node: None,
        edit_value: String::new(),
        schema_visible: false,
        schema_info: String::new(),
    };
    state.file_type_text = "JSON Document".to_string();
}
