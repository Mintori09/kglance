use iced::{Element, Length};
use iced::widget::{column, row, text, scrollable, container, button, text_input};
use crate::ui::types::{TextState, Message};

pub fn view_text<'a>(state: &'a TextState) -> Element<'a, Message> {
    let mut main_content = column![].spacing(5);

    // Search bar if visible
    if state.search_visible {
        let search_bar = row![
            text_input("Search...", &state.search_query)
                .on_input(Message::SearchQueryChanged)
                .width(Length::FillPortion(3)),
            button(text("Next")).on_press(Message::TextSearchNext),
            button(text("Prev")).on_press(Message::TextSearchPrev),
            text(&state.search_info).size(14),
            button(text("Close")).on_press(Message::TextSearchClosed),
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

    let content_scroll = scrollable(
        container(text_widget)
            .padding(15)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(palette.background.weak.color.into()),
                    text_color: Some(palette.background.weak.text),
                    ..Default::default()
                }
            })
    )
    .height(Length::Fill)
    .width(Length::Fill);

    main_content.push(content_scroll).into()
}
