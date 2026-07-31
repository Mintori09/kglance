use crate::app::Message;
use crate::core::TextState;
use crate::ui::components::code_editor::code_editor;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use crate::ui::theme::font::get_code_font;
use iced::Element;
use iced::widget::column;

use crate::ui::theme::tokens::spacing;

const MAIN_CONTENT_SPACING: f32 = spacing::XS;
const SCROLL_PANE_PADDING: f32 = spacing::XS;

const SCROLL_PANE_ID: &str = "content_scroll";

pub fn view_text<'a>(
    state: &'a TextState,
    is_dark: bool,
    font_size: f32,
    font_family_mono: Option<&str>,
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
        is_dark,
        font_size,
        font,
        Message::TextEdit,
    );

    let scrollable_editor = scroll_pane(SCROLL_PANE_ID, editor_element)
        .container_padding(SCROLL_PANE_PADDING)
        .on_scroll(|viewport| Message::TextScrolled(viewport.absolute_offset().y))
        .build();

    main_content.push(scrollable_editor).into()
}
