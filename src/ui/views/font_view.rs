use crate::app::Message;
use iced::widget::{column, container, text};
use iced::{Alignment, Color, Element, Length};

pub fn view_font<'a>(name: &'a str, metadata: &'a str, is_dark: bool) -> Element<'a, Message> {
    let title_color = if is_dark {
        Color::from_rgb(0.9, 0.9, 0.95)
    } else {
        Color::from_rgb(0.1, 0.1, 0.15)
    };

    let meta_color = if is_dark {
        Color::from_rgb(0.65, 0.68, 0.72)
    } else {
        Color::from_rgb(0.35, 0.38, 0.42)
    };

    let sample_color = if is_dark {
        Color::from_rgb(0.85, 0.88, 0.92)
    } else {
        Color::from_rgb(0.15, 0.18, 0.22)
    };

    let custom_font = iced::Font::with_name(Box::leak(name.to_string().into_boxed_str()));

    let font_title = text(format!("Font: {}", name))
        .size(24)
        .font(custom_font)
        .color(title_color);

    let sample_large = text("The quick brown fox jumps over the lazy dog")
        .size(32)
        .font(custom_font)
        .color(sample_color);

    let sample_digits = text("ABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\n0123456789 !@#$%^&*()_+-=[]{}|;:'\",.<>/?")
        .size(20)
        .font(custom_font)
        .color(sample_color);

    let meta_text = text(metadata).size(14).color(meta_color);

    let card = column![font_title, sample_large, sample_digits, meta_text,]
        .spacing(24)
        .align_x(Alignment::Start);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(32)
        .into()
}
