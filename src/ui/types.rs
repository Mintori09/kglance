use std::cell::Cell;

pub(crate) struct RenderContext<'a> {
    pub(crate) block_index: usize,
    pub(crate) selection_range: Option<crate::core::SelectionRange>,
    pub(crate) drag_active: bool,
    pub(crate) search_query: &'a str,
    pub(crate) active_match: usize,
    pub(crate) counter: &'a Cell<usize>,
    pub(crate) is_dark: bool,
    pub(crate) font_size: f32,
    pub(crate) font_family: Option<&'a str>,
    pub(crate) font_family_mono: Option<&'a str>,
}
