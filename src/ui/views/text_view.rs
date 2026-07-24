use crate::app::Message;
use crate::core::TextState;
use crate::ui::theme::{breeze_button, breeze_text_input, glass_scrollable};
use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Length};

pub fn view_text<'a>(
    state: &'a TextState,
    is_dark: bool,
    font_size: f32,
    font_family_mono: Option<&str>,
) -> Element<'a, Message> {
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

    let font = match font_family_mono {
        Some(name) => iced::Font::with_name(Box::leak(name.to_string().into_boxed_str())),
        None => iced::Font::MONOSPACE,
    };

    let text_widget = iced::widget::text_editor(&state.content)
        .highlight(extension, theme)
        .font(font)
        .size(font_size)
        .on_action(Message::TextEdit);

    let content_scroll = scrollable(text_widget)
        .id("content_scroll")
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(4).margin(2),
        ))
        .style(glass_scrollable)
        .height(Length::Fill)
        .width(Length::Fill)
        .on_scroll(|v| Message::TextScrolled(v.absolute_offset().y));

    main_content.push(content_scroll).into()
}
