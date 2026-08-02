//! Shared role colour tokens: accent, link, success, warning, danger.

use iced::{Color, Theme};

use super::primitive;

#[derive(Clone, Copy)]
pub struct RoleColors {
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub link: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl RoleColors {
    pub const DARK: RoleColors = RoleColors {
        accent: primitive::ACCENT,
        accent_hover: primitive::ACCENT_HOVER,
        accent_pressed: primitive::ACCENT_PRESSED,
        link: primitive::MD_DARK_LINK,
        success: primitive::MD_DARK_TASK_CHECKED,
        warning: primitive::MD_DARK_SEARCH_ACTIVE,
        danger: primitive::DANGER,
    };

    pub const LIGHT: RoleColors = RoleColors {
        accent: primitive::ACCENT,
        accent_hover: primitive::ACCENT_HOVER,
        accent_pressed: primitive::ACCENT_PRESSED,
        link: primitive::MD_LIGHT_LINK,
        success: primitive::MD_LIGHT_TASK_CHECKED,
        warning: primitive::MD_LIGHT_SEARCH_ACTIVE,
        danger: primitive::DANGER,
    };

    pub const NORD: RoleColors = RoleColors {
        accent: primitive::NORD8,
        accent_hover: primitive::NORD7,
        accent_pressed: primitive::NORD9,
        link: primitive::NORD8,
        success: primitive::NORD14,
        warning: primitive::NORD13,
        danger: primitive::NORD11,
    };

    pub fn palette(theme: &Theme) -> &'static RoleColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }
}

pub fn palette(theme: &Theme) -> &'static RoleColors {
    RoleColors::palette(theme)
}
