use crate::features::text::TextState;

#[derive(Debug, Clone, Default)]
pub struct TypstState {
    pub source_text: TextState,
    pub show_source: bool,
    pub error: Option<String>,
}
