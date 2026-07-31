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
    };

    pub const LIGHT: JsonColors = JsonColors {
        string: primitive::JSON_LIGHT_STRING,
        number: primitive::JSON_LIGHT_NUMBER,
        boolean: primitive::JSON_LIGHT_BOOL,
        null: primitive::JSON_LIGHT_NULL,
        object: primitive::JSON_LIGHT_OBJECT,
        text: primitive::JSON_LIGHT_TEXT,
        dim: primitive::JSON_LIGHT_DIM,
    };

    pub fn palette(theme: &Theme) -> &'static JsonColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }

    pub fn palette_for(is_dark: bool) -> &'static JsonColors {
        if is_dark { &Self::DARK } else { &Self::LIGHT }
    }
}
