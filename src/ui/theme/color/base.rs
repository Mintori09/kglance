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

    pub const NORD: BaseColors = BaseColors {
        bg: primitive::NORD0,
        surface: primitive::NORD1,
        surface_raised: primitive::NORD2,
        border: primitive::NORD3,
        border_focus: primitive::NORD8,
        text: primitive::NORD4,
        text_dim: primitive::NORD3,
        shadow: primitive::OVERLAY_SHADOW,
        rule: primitive::NORD3,
    };

    pub fn palette(theme: &Theme) -> &'static BaseColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }
}
