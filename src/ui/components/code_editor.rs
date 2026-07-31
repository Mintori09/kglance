use iced::alignment::{Horizontal, Vertical};
use iced::highlighter::Theme as HighlightTheme;
use iced::widget::text_editor::{Action, Content};
use iced::widget::{container, row, text, text_editor};
use iced::{Alignment, Element, Font, Length};

use crate::app::Message;
use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::default_text_editor;

const CHARACTER_WIDTH_RATIO: f32 = 0.65;
const GUTTER_PADDING: f32 = 16.0;
const GUTTER_SPACING: f32 = 6.0;

fn calculate_digit_count(line_count: usize) -> usize {
    if line_count > 0 {
        (line_count as f32).log10() as usize + 1
    } else {
        1
    }
}

pub fn line_number_width(line_count: usize, font_size: f32) -> Length {
    let digit_count = calculate_digit_count(line_count);
    let calculated_width =
        (digit_count as f32) * font_size * CHARACTER_WIDTH_RATIO + GUTTER_PADDING;
    Length::Fixed(calculated_width)
}

fn select_highlight_theme(is_dark: bool) -> HighlightTheme {
    if is_dark {
        HighlightTheme::Base16Mocha
    } else {
        HighlightTheme::InspiredGitHub
    }
}

fn generate_line_numbers_text(total_lines: usize) -> String {
    (1..=total_lines)
        .map(|number| number.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn code_editor<'a>(
    content: &'a Content,
    extension: &str,
    is_dark: bool,
    font_size: f32,
    font: Font,
    on_action: fn(Action) -> Message,
) -> Element<'a, Message> {
    let highlight_theme = select_highlight_theme(is_dark);
    let line_count = content.text().lines().count().max(1);

    let line_numbers = generate_line_numbers_text(line_count);
    let gutter_width = line_number_width(line_count, font_size);
    let gutter_color = BaseColors::palette_for(is_dark).text_dim;

    let line_numbers_widget = container(
        text(line_numbers)
            .font(font)
            .size(font_size)
            .width(gutter_width)
            .color(gutter_color)
            .align_x(Horizontal::Right),
    )
    .padding(iced::Padding {
        top: 5.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    });

    let editor_widget = text_editor(content)
        .highlight(extension, highlight_theme)
        .font(font)
        .size(font_size)
        .wrapping(iced::widget::text::Wrapping::None)
        .on_action(on_action)
        .style(default_text_editor);

    row![
        container(line_numbers_widget).align_y(Vertical::Top),
        editor_widget
    ]
    .spacing(GUTTER_SPACING)
    .align_y(Alignment::Start)
    .into()
}
