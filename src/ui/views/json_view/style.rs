use iced::Color;

use crate::ui::components::button;
use crate::ui::theme::color::json::JsonColors;
use crate::ui::theme::color::primitive::{
    JSON_DARK_ERROR, JSON_DARK_LINK, JSON_DARK_SELECTION, JSON_LIGHT_ERROR, JSON_LIGHT_LINK,
    JSON_LIGHT_SELECTION,
};

pub fn type_color(value_type: &str, is_dark: bool) -> Color {
    let c = JsonColors::palette_for(is_dark);
    match value_type {
        "String" => c.string,
        "Number" => c.number,
        "Bool" => c.boolean,
        "Null" => c.null,
        _ => c.object,
    }
}

pub fn text_color(is_dark: bool) -> Color {
    JsonColors::palette_for(is_dark).text
}

pub fn dim_color(is_dark: bool) -> Color {
    JsonColors::palette_for(is_dark).dim
}

pub fn string_color(is_dark: bool) -> Color {
    JsonColors::palette_for(is_dark).string
}

pub fn link_color(is_dark: bool) -> Color {
    if is_dark {
        JSON_DARK_LINK
    } else {
        JSON_LIGHT_LINK
    }
}

pub fn error_color(is_dark: bool) -> Color {
    if is_dark {
        JSON_DARK_ERROR
    } else {
        JSON_LIGHT_ERROR
    }
}

pub fn selection_color(is_dark: bool) -> Color {
    if is_dark {
        JSON_DARK_SELECTION
    } else {
        JSON_LIGHT_SELECTION
    }
}

pub fn header_button_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    button::breeze
}

pub fn small_btn_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    button::breeze_tool
}
