pub mod constants;
pub mod content;
pub mod helpers;
pub mod sidebar;

use std::cell::Cell;

use iced::widget::{column, row};
use iced::{Element, Length};

use crate::app::Message;
use crate::core::types::EpubState;
use crate::features::epub::view::content::build_epub_content;
use crate::features::epub::view::helpers::clamp_active_chapter;
use crate::features::epub::view::sidebar::render_chapter_sidebar;
use crate::ui::components::sidebar::drag_handle;
use crate::ui::types::RenderContext;

pub fn view_epub<'a>(
    state: &'a EpubState,
    font_size: f32,
    theme: crate::ui::theme::AppTheme,
    font_family: Option<&str>,
    font_family_mono: Option<&str>,
    max_text_width: Option<f32>,
    ruby_display_mode: crate::core::config::RubyDisplayMode,
) -> Element<'a, Message> {
    let active_chapter = clamp_active_chapter(state);
    let search_counter = Cell::new(0);
    let block_counter = Cell::new(0);

    let ctx = RenderContext {
        next_block_index: &block_counter,
        selection_range: state.markdown_state.selection_range,
        drag_active: state.markdown_state.is_dragging_selection
            || state.markdown_state.is_mouse_held,
        search_query: &state.markdown_state.search_query,
        active_match: state.markdown_state.search_match_index,
        counter: &search_counter,
        theme,
        font_size,
        font_family,
        font_family_mono,
        ruby_display_mode,
        footnotes: None,
        result_popup: false,
        show_heading_divider: false,
    };

    let main_content = build_epub_content(state, active_chapter, &ctx, max_text_width);
    let main_view = column![main_content].height(Length::Fill);

    if state.sidebar_visible && !state.chapters.is_empty() {
        let sidebar = render_chapter_sidebar(state, theme, font_family);
        let drag = drag_handle(state.sidebar_resizing, theme, Message::SidebarDragStarted);
        row![sidebar, drag, main_view]
            .spacing(0)
            .height(Length::Fill)
            .into()
    } else {
        main_view.into()
    }
}
