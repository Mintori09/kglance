use iced::Color;

use crate::ui::components::button;

pub fn type_color(value_type: &str, is_dark: bool) -> Color {
    match value_type {
        "String" => {
            if is_dark {
                Color::from_rgb(0.6, 0.9, 0.4)
            } else {
                Color::from_rgb(0.2, 0.6, 0.1)
            }
        }
        "Number" => {
            if is_dark {
                Color::from_rgb(0.8, 0.6, 0.3)
            } else {
                Color::from_rgb(0.7, 0.4, 0.0)
            }
        }
        "Bool" => {
            if is_dark {
                Color::from_rgb(0.4, 0.7, 1.0)
            } else {
                Color::from_rgb(0.0, 0.3, 0.8)
            }
        }
        "Null" => {
            if is_dark {
                Color::from_rgb(0.6, 0.6, 0.6)
            } else {
                Color::from_rgb(0.5, 0.5, 0.5)
            }
        }
        _ => {
            if is_dark {
                Color::from_rgb(0.7, 0.7, 0.9)
            } else {
                Color::from_rgb(0.3, 0.3, 0.6)
            }
        }
    }
}

pub fn text_color(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb(0.85, 0.85, 0.85)
    } else {
        Color::from_rgb(0.15, 0.15, 0.15)
    }
}

pub fn dim_color(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb(0.5, 0.5, 0.5)
    } else {
        Color::from_rgb(0.6, 0.6, 0.6)
    }
}

pub fn header_button_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    button::breeze
}

pub fn small_btn_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    button::breeze_tool
}
