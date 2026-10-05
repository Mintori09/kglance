use crate::app::Message;
use iced::widget::{column, container, scrollable, text};
use iced::{Alignment, Color, Element, Font, Length};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::ui::theme::tokens::font_view as font_tokens;

const TITLE_FONT_SIZE: f32 = font_tokens::PREVIEW_TITLE_SIZE;
const LARGE_SAMPLE_FONT_SIZE: f32 = font_tokens::SAMPLE_TEXT_SIZE;
const CHARSET_FONT_SIZE: f32 = 18.0;
const METADATA_FONT_SIZE: f32 = font_tokens::PREVIEW_BODY_SIZE;

const CARD_SPACING: f32 = font_tokens::ELEMENT_SPACING;
const CARD_PADDING: f32 = font_tokens::CARD_PADDING;

const SAMPLE_SENTENCE: &str = "The quick brown fox jumps over the lazy dog";
const SAMPLE_CHARACTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\n0123456789 !@#$%^&*()_+-=[]{}|;:'\",.<>/?";

use crate::ui::theme::AppTheme;

struct ThemeColors {
    title: Color,
    meta: Color,
    sample: Color,
}

impl ThemeColors {
    fn new(theme: AppTheme) -> Self {
        let p = theme.palette().base;
        Self {
            title: p.text,
            meta: p.text_dim,
            sample: p.text,
        }
    }
}

fn intern_font_name(name: &str) -> &'static str {
    static CACHE: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = cache.lock().unwrap();
    if let Some(&interned) = guard.get(name) {
        return interned;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    guard.insert(leaked);
    leaked
}

fn map_weight(weight: u16) -> iced::font::Weight {
    match weight {
        0..=150 => iced::font::Weight::Thin,
        151..=250 => iced::font::Weight::ExtraLight,
        251..=350 => iced::font::Weight::Light,
        351..=450 => iced::font::Weight::Normal,
        451..=550 => iced::font::Weight::Medium,
        551..=650 => iced::font::Weight::Semibold,
        651..=750 => iced::font::Weight::Bold,
        751..=850 => iced::font::Weight::ExtraBold,
        _ => iced::font::Weight::Black,
    }
}

pub fn view_font<'a>(
    name: &'a str,
    family: &'a str,
    _post_script_name: Option<&'a str>,
    weight: u16,
    is_italic: bool,
    metadata: &'a str,
    theme: AppTheme,
) -> Element<'a, Message> {
    let colors = ThemeColors::new(theme);
    let custom_font = Font {
        family: iced::font::Family::Name(intern_font_name(family)),
        weight: map_weight(weight),
        style: if is_italic {
            iced::font::Style::Italic
        } else {
            iced::font::Style::Normal
        },
        stretch: iced::font::Stretch::Normal,
    };

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

    container(scrollable(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(CARD_PADDING)
        .into()
}
