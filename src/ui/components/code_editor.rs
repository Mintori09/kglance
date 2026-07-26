use iced::widget::{row, text, text_editor};
use iced::{Element, Font, Length};

use crate::app::Message;
use crate::ui::theme::{DARK_TEXT_DIM, LIGHT_TEXT_DIM, glass_text_editor};

pub fn line_number_width(line_count: usize, font_size: f32) -> Length {
    let digits = if line_count > 0 {
        (line_count as f32).log10() as usize + 1
    } else {
        1
    };
    Length::Fixed((digits as f32) * font_size * 0.65 + 16.0)
}

pub fn code_editor<'a>(
    content: &'a iced::widget::text_editor::Content,
    line_numbers: &str,
    extension: &str,
    is_dark: bool,
    font_size: f32,
    font: Font,
    on_action: fn(iced::widget::text_editor::Action) -> Message,
) -> Element<'a, Message> {
    let hl_theme = if is_dark {
        iced::highlighter::Theme::Base16Mocha
    } else {
        iced::highlighter::Theme::InspiredGitHub
    };

    let num_width = line_number_width(line_numbers.lines().count(), font_size);
    let line_numbers_owned = line_numbers.to_string();

    let line_numbers_widget = text(line_numbers_owned)
        .font(font)
        .size(font_size)
        .width(num_width)
        .color(if is_dark {
            DARK_TEXT_DIM
        } else {
            LIGHT_TEXT_DIM
        })
        .align_x(iced::alignment::Horizontal::Right);

    let editor = text_editor(content)
        .highlight(extension, hl_theme)
        .font(font)
        .size(font_size)
        .on_action(on_action)
        .style(glass_text_editor);

    row![line_numbers_widget, editor]
        .spacing(6)
        .align_y(iced::Alignment::Start)
        .into()
}
