use crate::app::Message;
use crate::core::TextState;
use crate::ui::components::code_editor::code_editor;
use crate::ui::components::scroll_pane::scroll_pane;
use crate::ui::components::search_bar::{SearchKind, search_bar};
use iced::Element;
use iced::widget::column;

pub fn view_text<'a>(
    state: &'a TextState,
    is_dark: bool,
    font_size: f32,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
    let mut main_content = column![].spacing(5);

    if state.search_visible {
        main_content = main_content.push(search_bar(
            SearchKind::Text,
            &state.search_query,
            Some(&state.search_info),
            "Search...",
            "txt_search_input",
        ));
    }

    let font = match font_family_mono {
        Some(name) => iced::Font::with_name(Box::leak(name.to_string().into_boxed_str())),
        None => iced::Font::MONOSPACE,
    };

    let editor_row = code_editor(
        &state.content,
        state.extension.as_str(),
        is_dark,
        font_size,
        font,
        Message::TextEdit,
    );

    main_content
        .push(
            scroll_pane("content_scroll", editor_row)
                .container_padding(4)
                .on_scroll(|v| Message::TextScrolled(v.absolute_offset().y))
                .build(),
        )
        .into()
}
