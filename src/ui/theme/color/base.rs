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
        text_dim: primitive::NORD_TEXT_DIM,
        shadow: primitive::OVERLAY_SHADOW,
        rule: primitive::NORD3,
    };

    pub const CATPPUCCIN_MOCHA: BaseColors = BaseColors {
        bg: primitive::MOCHA_BASE,
        surface: primitive::MOCHA_MANTLE,
        surface_raised: primitive::MOCHA_SURFACE0,
        border: primitive::MOCHA_SURFACE1,
        border_focus: primitive::MOCHA_BLUE,
        text: primitive::MOCHA_TEXT,
        text_dim: primitive::MOCHA_SUBTEXT0,
        shadow: primitive::OVERLAY_SHADOW,
        rule: primitive::MOCHA_SURFACE1,
    };

    pub const CATPPUCCIN_LATTE: BaseColors = BaseColors {
        bg: primitive::LATTE_BASE,
        surface: primitive::LATTE_MANTLE,
        surface_raised: primitive::LATTE_SURFACE0,
        border: primitive::LATTE_SURFACE1,
        border_focus: primitive::LATTE_BLUE,
        text: primitive::LATTE_TEXT,
        text_dim: primitive::LATTE_SUBTEXT0,
        shadow: primitive::LIGHT_SHADOW,
        rule: primitive::LATTE_SURFACE1,
    };

    pub const TOKYO_NIGHT: BaseColors = BaseColors {
        bg: primitive::TOKYO_BG,
        surface: primitive::TOKYO_SURFACE,
        surface_raised: primitive::TOKYO_SURFACE_RAISED,
        border: primitive::TOKYO_BORDER,
        border_focus: primitive::TOKYO_BLUE,
        text: primitive::TOKYO_TEXT,
        text_dim: primitive::TOKYO_TEXT_DIM,
        shadow: primitive::OVERLAY_SHADOW,
        rule: primitive::TOKYO_BORDER,
    };

    pub const GRUVBOX_DARK: BaseColors = BaseColors {
        bg: primitive::GRUVBOX_BG,
        surface: primitive::GRUVBOX_SURFACE,
        surface_raised: primitive::GRUVBOX_SURFACE_RAISED,
        border: primitive::GRUVBOX_BORDER,
        border_focus: primitive::GRUVBOX_ORANGE,
        text: primitive::GRUVBOX_TEXT,
        text_dim: primitive::GRUVBOX_TEXT_DIM,
        shadow: primitive::OVERLAY_SHADOW,
        rule: primitive::GRUVBOX_BORDER,
    };

    pub const DRACULA: BaseColors = BaseColors {
        bg: primitive::DRACULA_BG,
        surface: primitive::DRACULA_SURFACE,
        surface_raised: primitive::DRACULA_SURFACE_RAISED,
        border: primitive::DRACULA_BORDER,
        border_focus: primitive::DRACULA_PURPLE,
        text: primitive::DRACULA_TEXT,
        text_dim: primitive::DRACULA_TEXT_DIM,
        shadow: primitive::OVERLAY_SHADOW,
        rule: primitive::DRACULA_BORDER,
    };

    pub fn palette(theme: &Theme) -> &'static BaseColors {
        let app_theme = crate::ui::theme::AppTheme::from(theme);
        &app_theme.palette().base
    }
}
