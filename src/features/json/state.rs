use rustc_hash::FxHashSet;

use crate::core::types::{JsonState, KglanceState};
use crate::features::json::JsonNode;

pub fn populate_state(
    state: &mut KglanceState,
    nodes: &[JsonNode],
    pretty: &str,
    has_parse_error: bool,
) {
    let old_scroll = state.json.scroll_y;
    let old_tree_mode = state.json.tree_mode;

    let mut expanded = FxHashSet::with_capacity_and_hasher(nodes.len() / 2, Default::default());
    for (i, node) in nodes.iter().enumerate() {
        if node.children_count > 0 {
            expanded.insert(i);
        }
    }

    let parsed_cache_hit = state.json.parsed_cache.clone();

    let win_w = if state.current_window_size.width > 0.0 {
        state.current_window_size.width
    } else if state.window_width > 0.0 {
        state.window_width
    } else {
        1024.0
    };

    let raw_text = crate::features::text::create_text_state(
        pretty.to_string(),
        "json",
        state.font_size,
        state.word_wrap,
        state.app_theme,
        win_w,
    );

    let mut scroll_ctrl = crate::core::scroll::ScrollController::default();
    scroll_ctrl.set_position_y(old_scroll);

    state.json = JsonState {
        nodes: nodes.to_vec(),
        expanded,
        raw_content: pretty.to_string(),
        pretty_content: pretty.to_string(),
        tree_mode: old_tree_mode,
        scroll_y: old_scroll,
        viewport_height: 0.0,
        total_content_height: 0.0,
        scroll_controller: scroll_ctrl,
        smooth_scroll: crate::core::types::SmoothScrollState::default(),
        has_parse_error,
        raw_text,
        search_visible: false,
        search_query: String::new(),
        search_matches: Vec::new(),
        search_match_index: 0,
        search_info: String::new(),
        minified_content: String::new(),
        raw_pretty: true,
        active_node: None,
        parsed_cache: parsed_cache_hit,
    };
    state
        .json
        .scroll_controller
        .apply_config(&state.scroll_config);
    state.json.smooth_scroll.apply_config(&state.scroll_config);
    state
        .json
        .raw_text
        .scroll_controller
        .apply_config(&state.scroll_config);
    state
        .json
        .raw_text
        .smooth_scroll
        .apply_config(&state.scroll_config);

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

    #[test]
    fn test_populate_state_raw_text_highlight() {
        let mut state = KglanceState::default();
        let json_str = "{\n  \"version\": 1,\n  \"name\": \"kglance\"\n}";
        populate_state(&mut state, &[], json_str, false);

        assert_eq!(state.json.raw_text.document.total_lines(), 4);
        assert_eq!(state.json.raw_text.cached_tokens.len(), 4);
        assert!(!state.json.raw_text.cached_tokens[1].is_empty());
    }
}
