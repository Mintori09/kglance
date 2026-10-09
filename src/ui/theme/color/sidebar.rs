//! Semantic colour tokens for the sidebar / tree components.

use iced::{Color, Theme};

use super::primitive;

#[derive(Clone, Copy)]
pub struct SidebarColors {
    pub hover_press: Color,
    pub active_bg: Color,
    pub active_text: Color,
    pub inactive_text: Color,
    pub resizing: Color,
    pub normal_drag: Color,
    pub arrow_text: Color,
}

impl SidebarColors {
    pub const DARK: SidebarColors = SidebarColors {
        hover_press: primitive::DARK_BORDER,
        active_bg: primitive::SIDEBAR_DARK_ACTIVE_BG,
        active_text: primitive::SIDEBAR_DARK_ACTIVE_TEXT,
        inactive_text: primitive::SIDEBAR_DARK_INACTIVE_TEXT,
        resizing: primitive::SIDEBAR_DARK_RESIZING,
        normal_drag: primitive::WHITE_005,
        arrow_text: primitive::SIDEBAR_DARK_ARROW_TEXT,
    };

    pub const LIGHT: SidebarColors = SidebarColors {
        hover_press: primitive::BLACK_006,
        active_bg: primitive::SIDEBAR_LIGHT_ACTIVE_BG,
        active_text: primitive::SIDEBAR_LIGHT_ACTIVE_TEXT,
        inactive_text: primitive::SIDEBAR_LIGHT_INACTIVE_TEXT,
        resizing: primitive::SIDEBAR_LIGHT_RESIZING,
        normal_drag: primitive::BLACK_005,
        arrow_text: primitive::SIDEBAR_LIGHT_ARROW_TEXT,
    };

    pub const NORD: SidebarColors = SidebarColors {
        hover_press: primitive::NORD2,
        active_bg: primitive::NORD2,
        active_text: primitive::NORD8,
        inactive_text: primitive::NORD4,
        resizing: primitive::NORD8,
        normal_drag: primitive::NORD1,
        arrow_text: primitive::NORD9,
    };

    pub const CATPPUCCIN_MOCHA: SidebarColors = SidebarColors {
        hover_press: primitive::MOCHA_SURFACE0,
        active_bg: primitive::MOCHA_SURFACE1,
        active_text: primitive::MOCHA_BLUE,
        inactive_text: primitive::MOCHA_TEXT,
        resizing: primitive::MOCHA_BLUE,
        normal_drag: primitive::MOCHA_SURFACE0,
        arrow_text: primitive::MOCHA_SUBTEXT0,
    };

    pub const CATPPUCCIN_LATTE: SidebarColors = SidebarColors {
        hover_press: primitive::LATTE_SURFACE0,
        active_bg: primitive::LATTE_SURFACE1,
        active_text: primitive::LATTE_BLUE,
        inactive_text: primitive::LATTE_TEXT,
        resizing: primitive::LATTE_BLUE,
        normal_drag: primitive::LATTE_SURFACE0,
        arrow_text: primitive::LATTE_SUBTEXT0,
    };

    pub const TOKYO_NIGHT: SidebarColors = SidebarColors {
        hover_press: primitive::TOKYO_SURFACE,
        active_bg: primitive::TOKYO_SURFACE_RAISED,
        active_text: primitive::TOKYO_BLUE,
        inactive_text: primitive::TOKYO_TEXT,
        resizing: primitive::TOKYO_BLUE,
        normal_drag: primitive::TOKYO_SURFACE,
        arrow_text: primitive::TOKYO_TEXT_DIM,
    };

    pub const GRUVBOX_DARK: SidebarColors = SidebarColors {
        hover_press: primitive::GRUVBOX_SURFACE,
        active_bg: primitive::GRUVBOX_SURFACE_RAISED,
        active_text: primitive::GRUVBOX_ORANGE,
        inactive_text: primitive::GRUVBOX_TEXT,
        resizing: primitive::GRUVBOX_ORANGE,
        normal_drag: primitive::GRUVBOX_SURFACE,
        arrow_text: primitive::GRUVBOX_TEXT_DIM,
    };

    pub const DRACULA: SidebarColors = SidebarColors {
        hover_press: primitive::DRACULA_SURFACE,
        active_bg: primitive::DRACULA_SURFACE_RAISED,
        active_text: primitive::DRACULA_PURPLE,
        inactive_text: primitive::DRACULA_TEXT,
        resizing: primitive::DRACULA_PURPLE,
        normal_drag: primitive::DRACULA_SURFACE,
        arrow_text: primitive::DRACULA_TEXT_DIM,
    };

    pub fn palette(theme: &Theme) -> &'static SidebarColors {
        let app_theme = crate::ui::theme::AppTheme::from(theme);
        &app_theme.palette().sidebar
    }
}
