use crate::app::Message;
use crate::core::TextState;
use crate::ui::theme::{DARK_TEXT, DARK_TEXT_DIM, LIGHT_TEXT, LIGHT_TEXT_DIM};
use crate::ui::theme::{breeze_button, breeze_container, breeze_text_input, glass_scrollable};
use iced::widget::{button, column, container, row, scrollable, text, text_editor, text_input};
use iced::{Color, Element, Length};

fn line_number_width(line_count: usize, font_size: f32) -> Length {
    let digits = if line_count > 0 {
        (line_count as f32).log10() as usize + 1
    } else {
        1
    };
    Length::Fixed((digits as f32) * font_size * 0.65 + 16.0)
}

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

    let num_width = line_number_width(state.line_numbers.lines().count(), font_size);

    let line_numbers_widget = text(&state.line_numbers)
        .font(font)
        .size(font_size)
        .width(num_width)
        .color(if is_dark {
            DARK_TEXT_DIM
        } else {
            LIGHT_TEXT_DIM
        })
        .align_x(iced::alignment::Horizontal::Right);

    let text_widget = iced::widget::text_editor(&state.content)
        .highlight(extension, theme)
        .font(font)
        .size(font_size)
        .on_action(Message::TextEdit)
        .style(|theme: &iced::Theme, _status| {
            let is_dark = matches!(theme, iced::Theme::Dark);
            text_editor::Style {
                background: Color::TRANSPARENT.into(),
                border: iced::Border {
                    width: 0.0,
                    color: Color::TRANSPARENT,
                    radius: 0.0.into(),
                },
                placeholder: Color::TRANSPARENT,
                value: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
                selection: if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.15)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                },
            }
        });

    let editor_row = row![line_numbers_widget, text_widget]
        .spacing(6)
        .align_y(iced::Alignment::Start);

    let content_scroll = scrollable(
        container(editor_row)
            .width(Length::Fill)
            .padding(4)
            .style(breeze_container),
    )
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
