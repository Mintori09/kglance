//! Semantic colour tokens for the JSON view.

use iced::{Color, Theme};

use super::primitive;

#[derive(Clone, Copy)]
pub struct JsonColors {
    pub string: Color,
    pub number: Color,
    pub boolean: Color,
    pub null: Color,
    pub object: Color,
    pub text: Color,
    pub dim: Color,
    pub link: Color,
    pub error: Color,
    pub selection: Color,
}

impl JsonColors {
    pub const DARK: JsonColors = JsonColors {
        string: primitive::JSON_DARK_STRING,
        number: primitive::JSON_DARK_NUMBER,
        boolean: primitive::JSON_DARK_BOOL,
        null: primitive::JSON_DARK_NULL,
        object: primitive::JSON_DARK_OBJECT,
        text: primitive::JSON_DARK_TEXT,
        dim: primitive::JSON_DARK_DIM,
        link: primitive::JSON_DARK_LINK,
        error: primitive::JSON_DARK_ERROR,
        selection: primitive::JSON_DARK_SELECTION,
    };

    pub const LIGHT: JsonColors = JsonColors {
        string: primitive::JSON_LIGHT_STRING,
        number: primitive::JSON_LIGHT_NUMBER,
        boolean: primitive::JSON_LIGHT_BOOL,
        null: primitive::JSON_LIGHT_NULL,
        object: primitive::JSON_LIGHT_OBJECT,
        text: primitive::JSON_LIGHT_TEXT,
        dim: primitive::JSON_LIGHT_DIM,
        link: primitive::JSON_LIGHT_LINK,
        error: primitive::JSON_LIGHT_ERROR,
        selection: primitive::JSON_LIGHT_SELECTION,
    };

    pub const NORD: JsonColors = JsonColors {
        string: primitive::NORD14,
        number: primitive::NORD15,
        boolean: primitive::NORD9,
        null: primitive::NORD11,
        object: primitive::NORD7,
        text: primitive::NORD4,
        dim: primitive::NORD_TEXT_DIM,
        link: primitive::NORD8,
        error: primitive::NORD11,
        selection: primitive::NORD2,
    };

    pub const CATPPUCCIN_MOCHA: JsonColors = JsonColors {
        string: primitive::MOCHA_GREEN,
        number: primitive::MOCHA_PEACH,
        boolean: primitive::MOCHA_MAUVE,
        null: primitive::MOCHA_RED,
        object: primitive::MOCHA_SKY,
        text: primitive::MOCHA_TEXT,
        dim: primitive::MOCHA_SUBTEXT0,
        link: primitive::MOCHA_BLUE,
        error: primitive::MOCHA_RED,
        selection: primitive::MOCHA_SURFACE0,
    };

    pub const CATPPUCCIN_LATTE: JsonColors = JsonColors {
        string: primitive::LATTE_GREEN,
        number: primitive::LATTE_YELLOW,
        boolean: primitive::LATTE_LAVENDER,
        null: primitive::LATTE_RED,
        object: primitive::LATTE_SAPPHIRE,
        text: primitive::LATTE_TEXT,
        dim: primitive::LATTE_SUBTEXT0,
        link: primitive::LATTE_BLUE,
        error: primitive::LATTE_RED,
        selection: primitive::LATTE_SURFACE0,
    };

    pub const TOKYO_NIGHT: JsonColors = JsonColors {
        string: primitive::TOKYO_GREEN,
        number: primitive::TOKYO_ORANGE,
        boolean: primitive::TOKYO_PURPLE,
        null: primitive::TOKYO_RED,
        object: primitive::TOKYO_CYAN,
        text: primitive::TOKYO_TEXT,
        dim: primitive::TOKYO_TEXT_DIM,
        link: primitive::TOKYO_BLUE,
        error: primitive::TOKYO_RED,
        selection: primitive::TOKYO_SURFACE,
    };

    pub const GRUVBOX_DARK: JsonColors = JsonColors {
        string: primitive::GRUVBOX_GREEN,
        number: primitive::GRUVBOX_PURPLE,
        boolean: primitive::GRUVBOX_ORANGE,
        null: primitive::GRUVBOX_RED,
        object: primitive::GRUVBOX_AQUA,
        text: primitive::GRUVBOX_TEXT,
        dim: primitive::GRUVBOX_TEXT_DIM,
        link: primitive::GRUVBOX_ORANGE,
        error: primitive::GRUVBOX_RED,
        selection: primitive::GRUVBOX_SURFACE,
    };

    pub const DRACULA: JsonColors = JsonColors {
        string: primitive::DRACULA_YELLOW,
        number: primitive::DRACULA_ORANGE,
        boolean: primitive::DRACULA_PURPLE,
        null: primitive::DRACULA_RED,
        object: primitive::DRACULA_CYAN,
        text: primitive::DRACULA_TEXT,
        dim: primitive::DRACULA_TEXT_DIM,
        link: primitive::DRACULA_PURPLE,
        error: primitive::DRACULA_RED,
        selection: primitive::DRACULA_SURFACE,
    };

    pub fn palette(theme: &Theme) -> &'static JsonColors {
        let app_theme = crate::ui::theme::AppTheme::from(theme);
        &app_theme.palette().json
    }
}
