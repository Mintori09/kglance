use crate::app::Message;
use crate::core::TextState;
use crate::ui::theme::{breeze_button, breeze_text_input, glass_inset, glass_scrollable};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

fn on_scrolled(viewport: iced::widget::scrollable::Viewport) -> Message {
    Message::ContentScrolled(viewport.absolute_offset().y)
}

pub fn view_text<'a>(state: &'a TextState, is_dark: bool, font_size: f32) -> Element<'a, Message> {
    let mut main_content = column![].spacing(5);

    if state.search_visible {
        let search_bar = row![
            text_input("Search...", &state.search_query)
                .on_input(Message::SearchQueryChanged)
                .style(breeze_text_input)
                .width(Length::FillPortion(3)),
            button(text("Next"))
                .on_press(Message::TextSearchNext)
                .style(breeze_button),
            button(text("Prev"))
                .on_press(Message::TextSearchPrev)
                .style(breeze_button),
            text(&state.search_info).size(14),
            button(text("Close"))
                .on_press(Message::TextSearchClosed)
                .style(breeze_button),
        ]
        .spacing(10)
        .padding(5);
        main_content = main_content.push(search_bar);
    }

    // In Iced 0.14, highlight takes (extension, theme)
    let extension = state.extension.as_str();
    let theme = if is_dark {
        iced::highlighter::Theme::Base16Mocha
    } else {
        iced::highlighter::Theme::InspiredGitHub
    };

    let text_widget = iced::widget::text_editor(&state.content)
        .highlight(extension, theme)
        .size(font_size);

    let content_scroll = scrollable(container(text_widget).padding(15).style(glass_inset))
        .id("content_scroll")
        .on_scroll(on_scrolled)
        .style(glass_scrollable)
        .height(Length::Fill)
        .width(Length::Fill);

    main_content.push(content_scroll).into()
}
