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
