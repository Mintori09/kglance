use std::cell::Cell;

use crate::app::Message;
use crate::parsers::markdown::Block;
use crate::ui::views::markdown_view::components::style::STYLE;
use crate::ui::views::markdown_view::components::{
    render_code_block, render_heading, render_horizontal_rule, render_html, render_inline_image,
    render_list, render_mermaid, render_paragraph, render_quote, render_table,
};
use iced::Element;

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
