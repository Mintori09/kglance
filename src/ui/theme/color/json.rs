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
        dim: primitive::NORD3,
        link: primitive::NORD8,
        error: primitive::NORD11,
        selection: primitive::NORD2,
    };

    pub fn palette(theme: &Theme) -> &'static JsonColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }
}
