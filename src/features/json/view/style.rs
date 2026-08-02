use iced::Color;

use crate::ui::components::button;

use crate::ui::theme::AppTheme;

pub fn type_color(value_type: &str, theme: AppTheme) -> Color {
    let c = &theme.palette().json;
    match value_type {
        "String" => c.string,
        "Number" => c.number,
        "Bool" => c.boolean,
        "Null" => c.null,
        _ => c.object,
    }
}

pub fn text_color(theme: AppTheme) -> Color {
    theme.palette().json.text
}

pub fn dim_color(theme: AppTheme) -> Color {
    theme.palette().json.dim
}

pub fn string_color(theme: AppTheme) -> Color {
    theme.palette().json.string
}

pub fn link_color(theme: AppTheme) -> Color {
    theme.palette().json.link
}

pub fn error_color(theme: AppTheme) -> Color {
    theme.palette().json.error
}

pub fn selection_color(theme: AppTheme) -> Color {
    theme.palette().json.selection
}

pub fn header_button_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    button::breeze
}

pub fn small_btn_style() -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    button::breeze_tool
}
