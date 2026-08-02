use crate::app::Message;
use crate::ui::theme::color::primitive::{
    FONT_META_DARK, FONT_META_LIGHT, FONT_SAMPLE_DARK, FONT_SAMPLE_LIGHT, FONT_TITLE_DARK,
    FONT_TITLE_LIGHT,
};
use iced::widget::{column, container, text};
use iced::{Alignment, Color, Element, Font, Length};

use crate::ui::theme::tokens::font_view as font_tokens;

const TITLE_FONT_SIZE: f32 = font_tokens::PREVIEW_TITLE_SIZE;
const LARGE_SAMPLE_FONT_SIZE: f32 = font_tokens::SAMPLE_TEXT_SIZE;
const CHARSET_FONT_SIZE: f32 = 18.0;
const METADATA_FONT_SIZE: f32 = font_tokens::PREVIEW_BODY_SIZE;

const CARD_SPACING: f32 = font_tokens::ELEMENT_SPACING;
const CARD_PADDING: f32 = font_tokens::CARD_PADDING;

const SAMPLE_SENTENCE: &str = "The quick brown fox jumps over the lazy dog";
const SAMPLE_CHARACTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\n0123456789 !@#$%^&*()_+-=[]{}|;:'\",.<>/?";

struct ThemeColors {
    title: Color,
    meta: Color,
    sample: Color,
}

impl ThemeColors {
    fn new(is_dark: bool) -> Self {
        if is_dark {
            Self {
                title: FONT_TITLE_DARK,
                meta: FONT_META_DARK,
                sample: FONT_SAMPLE_DARK,
            }
        } else {
            Self {
                title: FONT_TITLE_LIGHT,
                meta: FONT_META_LIGHT,
                sample: FONT_SAMPLE_LIGHT,
            }
        }
    }
}

pub fn view_font<'a>(name: &'a str, metadata: &'a str, is_dark: bool) -> Element<'a, Message> {
    let colors = ThemeColors::new(is_dark);
    let custom_font = Font::with_name(Box::leak(name.to_string().into_boxed_str()));

    let font_title = text(format!("Font: {name}"))
        .size(TITLE_FONT_SIZE)
        .font(custom_font)
        .color(colors.title);

    let sample_large = text(SAMPLE_SENTENCE)
        .size(LARGE_SAMPLE_FONT_SIZE)
        .font(custom_font)
        .color(colors.sample);

    let sample_digits = text(SAMPLE_CHARACTERS)
        .size(CHARSET_FONT_SIZE)
        .font(custom_font)
        .color(colors.sample);

    let meta_text = text(metadata).size(METADATA_FONT_SIZE).color(colors.meta);

    let card = column![font_title, sample_large, sample_digits, meta_text]
        .spacing(CARD_SPACING)
        .align_x(Alignment::Start);

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(CARD_PADDING)
        .into()
}
