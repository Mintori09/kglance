pub mod outline;

use crate::app::Message;
use crate::core::TextState;
use crate::features::text::view::outline::render_outline_sidebar;
use crate::ui::components::code_editor::code_editor;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::font::get_code_font;
use iced::Element;
use iced::widget::{column, row};

use crate::ui::theme::tokens::spacing;

const MAIN_CONTENT_SPACING: f32 = spacing::XS;
const SCROLL_PANE_PADDING: f32 = spacing::XS;

const SCROLL_PANE_ID: &str = "content_scroll";

pub fn view_text<'a>(
    state: &'a TextState,
    theme: crate::ui::theme::AppTheme,
    font_size: f32,
    font_family_mono: Option<&str>,
    word_wrap: bool,
) -> Element<'a, Message> {
    let font = get_code_font(font_family_mono);
    let mut main_content = column![].spacing(MAIN_CONTENT_SPACING);

    if state.search_visible {
        main_content = main_content.push(search_bar(
            SearchKind::Text,
            &state.search_query,
            Some(&state.search_info),
        ));
    }

    let editor_element = code_editor(
        &state.content,
        state.extension.as_str(),
        theme,
        font_size,
        font,
        word_wrap,
        |action| crate::app::messages::TextMsg::Edit(action).into(),
    );

    let scrollable_editor = scroll_pane(SCROLL_PANE_ID, editor_element)
        .container_padding(SCROLL_PANE_PADDING)
        .on_scroll(|viewport| {
            crate::app::messages::TextMsg::Scrolled(viewport.absolute_offset().y).into()
        })
        .build();

    main_content = main_content.push(scrollable_editor);

    if state.outline_visible && !state.symbols.is_empty() {
        let sidebar = render_outline_sidebar(
            &state.symbols,
            theme,
            state.sidebar_width,
            state.scroll_y,
            font_size,
        );
        row![sidebar, main_content].into()
    } else {
        main_content.into()
    }
}
