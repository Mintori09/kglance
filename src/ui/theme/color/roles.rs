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

    pub const CATPPUCCIN_MOCHA: RoleColors = RoleColors {
        accent: primitive::MOCHA_BLUE,
        accent_hover: primitive::MOCHA_SKY,
        accent_pressed: primitive::MOCHA_SAPPHIRE,
        link: primitive::MOCHA_LAVENDER,
        success: primitive::MOCHA_GREEN,
        warning: primitive::MOCHA_YELLOW,
        danger: primitive::MOCHA_RED,
    };

    pub const CATPPUCCIN_LATTE: RoleColors = RoleColors {
        accent: primitive::LATTE_BLUE,
        accent_hover: primitive::LATTE_SKY,
        accent_pressed: primitive::LATTE_SAPPHIRE,
        link: primitive::LATTE_BLUE,
        success: primitive::LATTE_GREEN,
        warning: primitive::LATTE_YELLOW,
        danger: primitive::LATTE_RED,
    };

    pub const TOKYO_NIGHT: RoleColors = RoleColors {
        accent: primitive::TOKYO_BLUE,
        accent_hover: primitive::TOKYO_CYAN,
        accent_pressed: primitive::TOKYO_BLUE_DARK,
        link: primitive::TOKYO_CYAN,
        success: primitive::TOKYO_GREEN,
        warning: primitive::TOKYO_YELLOW,
        danger: primitive::TOKYO_RED,
    };

    pub const GRUVBOX_DARK: RoleColors = RoleColors {
        accent: primitive::GRUVBOX_ORANGE,
        accent_hover: primitive::GRUVBOX_YELLOW,
        accent_pressed: primitive::GRUVBOX_ORANGE_DARK,
        link: primitive::GRUVBOX_AQUA,
        success: primitive::GRUVBOX_GREEN,
        warning: primitive::GRUVBOX_YELLOW,
        danger: primitive::GRUVBOX_RED,
    };

    pub const DRACULA: RoleColors = RoleColors {
        accent: primitive::DRACULA_PURPLE,
        accent_hover: primitive::DRACULA_PINK,
        accent_pressed: primitive::DRACULA_CYAN,
        link: primitive::DRACULA_CYAN,
        success: primitive::DRACULA_GREEN,
        warning: primitive::DRACULA_YELLOW,
        danger: primitive::DRACULA_RED,
    };

    pub fn palette(theme: &Theme) -> &'static RoleColors {
        let app_theme = crate::ui::theme::AppTheme::from(theme);
        &app_theme.palette().roles
    }
}

pub fn palette(theme: &Theme) -> &'static RoleColors {
    RoleColors::palette(theme)
}
