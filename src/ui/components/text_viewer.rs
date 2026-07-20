use crate::ui::types::{Message, TextState};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

pub fn view_text<'a>(state: &'a TextState) -> Element<'a, Message> {
    let mut main_content = column![].spacing(5);

    // Search bar if visible
    if state.search_visible {
        let search_bar = row![
            text_input("Search...", &state.search_query)
                .on_input(Message::SearchQueryChanged)
                .style(crate::ui::theme::breeze_text_input)
                .width(Length::FillPortion(3)),
            button(text("Next")).on_press(Message::TextSearchNext).style(crate::ui::theme::breeze_button),
            button(text("Prev")).on_press(Message::TextSearchPrev).style(crate::ui::theme::breeze_button),
            text(&state.search_info).size(14),
            button(text("Close")).on_press(Message::TextSearchClosed).style(crate::ui::theme::breeze_button),
        ]
        .spacing(10)
        .padding(5);
        main_content = main_content.push(search_bar);
    }

    // Scrollable text content
    let text_widget = if state.wrap {
        text(&state.content).width(Length::Fill)
    } else {
        text(&state.content)
    };

    let content_scroll = scrollable(container(text_widget).padding(15).style(
        crate::ui::theme::breeze_container,
    ))
    .height(Length::Fill)
    .width(Length::Fill);

    main_content.push(content_scroll).into()
}
