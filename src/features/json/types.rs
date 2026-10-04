use crate::core::scroll::ScrollController;
use crate::core::types::SmoothScrollState;
use crate::features::json::JsonNode;
use crate::features::text::TextState;
use rustc_hash::FxHashSet;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type JsonParsedCache = Arc<Mutex<HashMap<String, (Vec<JsonNode>, String, bool)>>>;

#[derive(Debug, Clone)]
pub struct JsonState {
    pub nodes: Vec<JsonNode>,
    pub expanded: FxHashSet<usize>,
    pub raw_content: String,
    pub pretty_content: String,
    pub tree_mode: bool,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub total_content_height: f32,
    pub scroll_controller: ScrollController,
    pub smooth_scroll: SmoothScrollState,
    pub has_parse_error: bool,
    pub raw_text: TextState,
    pub search_visible: bool,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub search_match_index: usize,
    pub search_info: String,
    pub minified_content: String,
    pub raw_pretty: bool,
    pub active_node: Option<usize>,
    pub parsed_cache: JsonParsedCache,
}

impl Default for JsonState {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            expanded: FxHashSet::default(),
            raw_content: String::new(),
            pretty_content: String::new(),
            tree_mode: false,
            scroll_y: 0.0,
            viewport_height: 0.0,
            total_content_height: 0.0,
            scroll_controller: ScrollController::default(),
            smooth_scroll: SmoothScrollState::default(),
            has_parse_error: false,
            raw_text: TextState::default(),
            search_visible: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_index: 0,
            search_info: String::new(),
            minified_content: String::new(),
            raw_pretty: true,
            active_node: None,
            parsed_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
