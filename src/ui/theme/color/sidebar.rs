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

    pub fn palette(theme: &Theme) -> &'static SidebarColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }
}
