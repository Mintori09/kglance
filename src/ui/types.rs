use std::cell::Cell;

use crate::ui::theme::AppTheme;

pub(crate) struct RenderContext<'a> {
    pub(crate) next_block_index: &'a Cell<usize>,
    pub(crate) selection_range: Option<crate::core::SelectionRange>,
    pub(crate) drag_active: bool,
    pub(crate) search_query: &'a str,
    pub(crate) active_match: usize,
    pub(crate) counter: &'a Cell<usize>,
    pub(crate) theme: AppTheme,
    pub(crate) font_size: f32,
    pub(crate) font_family: Option<&'a str>,
    pub(crate) font_family_mono: Option<&'a str>,
    pub(crate) ruby_display_mode: crate::core::config::RubyDisplayMode,
    pub(crate) footnotes: Option<&'a std::collections::HashMap<String, String>>,
    pub(crate) result_popup: bool,
    pub(crate) show_heading_divider: bool,
}

impl<'a> RenderContext<'a> {
    #[inline]
    pub(crate) fn alloc_block_index(&self) -> Option<usize> {
        let idx = self.next_block_index.get();
        self.next_block_index.set(idx + 1);
        Some(idx)
    }
}
