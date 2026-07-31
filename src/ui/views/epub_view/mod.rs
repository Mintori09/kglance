pub mod constants;
pub mod content;
pub mod header;
pub mod helpers;
pub mod sidebar;

use std::cell::Cell;

use iced::widget::{column, row};
use iced::{Element, Length};

use crate::app::Message;
use crate::core::types::EpubState;
use crate::ui::components::sidebar::drag_handle;
use crate::ui::theme::font::get_main_font;
use crate::ui::theme::glass;
use crate::ui::views::epub_view::content::build_epub_content;
use crate::ui::views::epub_view::header::build_epub_header;
use crate::ui::views::epub_view::helpers::clamp_active_chapter;
use crate::ui::views::epub_view::sidebar::render_chapter_sidebar;
use crate::ui::views::markdown_view::blocks::RenderContext;

pub fn view_epub<'a>(
    state: &'a EpubState,
    font_size: f32,
    is_dark: bool,
    font_family: Option<&str>,
    font_family_mono: Option<&str>,
    max_text_width: Option<f32>,
) -> Element<'a, Message> {
    let main_font = get_main_font(font_family);
    let active_chapter = clamp_active_chapter(state);
    let search_counter = Cell::new(0);

    let ctx = RenderContext {
        search_query: &state.markdown_state.search_query,
        active_match: state.markdown_state.search_match_index,
        counter: &search_counter,
        is_dark,
        font_size,
        font_family,
        font_family_mono,
    };

    let palette = *glass::palette_for(is_dark);
    let header_bar = build_epub_header(
        state,
        main_font,
        font_size,
        palette.text,
        palette.text_dim,
        palette.bg,
        palette.border,
    );
    let main_content = build_epub_content(state, active_chapter, &ctx, max_text_width);
    let main_view = column![header_bar, main_content].height(Length::Fill);

    if state.sidebar_visible && !state.chapters.is_empty() {
        let sidebar = render_chapter_sidebar(state, is_dark);
        let drag = drag_handle(
            state.sidebar_resizing,
            is_dark,
            Message::SidebarDragStarted(0.0),
        );
        row![sidebar, drag, main_view]
            .spacing(0)
            .height(Length::Fill)
            .into()
    } else {
        main_view.into()
    }
}
