//! Base colour tokens shared across the whole UI.

use iced::{Color, Theme};

use super::primitive;

#[derive(Clone, Copy)]
pub struct BaseColors {
    pub bg: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub border: Color,
    pub border_focus: Color,
    pub text: Color,
    pub text_dim: Color,
    pub shadow: Color,
    pub rule: Color,
}

impl BaseColors {
    pub const DARK: BaseColors = BaseColors {
        bg: primitive::DARK_BG,
        surface: primitive::DARK_SURFACE,
        surface_raised: primitive::DARK_SURFACE_RAISED,
        border: primitive::DARK_BORDER,
        border_focus: primitive::DARK_BORDER_FOCUS,
        text: primitive::DARK_TEXT,
        text_dim: primitive::DARK_TEXT_DIM,
        shadow: primitive::DARK_SHADOW,
        rule: primitive::DARK_RULE,
    };

    pub const LIGHT: BaseColors = BaseColors {
        bg: primitive::LIGHT_BG,
        surface: primitive::LIGHT_SURFACE,
        surface_raised: primitive::LIGHT_SURFACE_RAISED,
        border: primitive::LIGHT_BORDER,
        border_focus: primitive::LIGHT_BORDER_FOCUS,
        text: primitive::LIGHT_TEXT,
        text_dim: primitive::LIGHT_TEXT_DIM,
        shadow: primitive::LIGHT_SHADOW,
        rule: primitive::LIGHT_RULE,
    };

    pub fn palette(theme: &Theme) -> &'static BaseColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }

    pub fn palette_for(is_dark: bool) -> &'static BaseColors {
        if is_dark { &Self::DARK } else { &Self::LIGHT }
    }
}
