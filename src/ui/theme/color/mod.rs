//! Colour token system.
//!
//! Layered design:
//! - [`primitive`] — raw named colours, the single source of truth.
//! - [`base`], [`roles`] — shared semantic tokens for the whole UI.
//! - [`markdown`], [`json`], [`sidebar`] — view/component semantic tokens.

use crate::ui::theme::color::primitive::{
    MD_DARK_CODE_FG, MD_LIGHT_CODE_FG, syntect_to_iced_color,
};
pub mod base;
pub mod json;
pub mod markdown;
pub mod primitive;
pub mod roles;
pub mod sidebar;

pub use base::BaseColors;
pub use json::JsonColors;
pub use markdown::MarkdownColors;
pub use roles::RoleColors;
pub use sidebar::SidebarColors;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
    Nord,
}

impl AppTheme {
    pub fn palette(self) -> &'static ColorPalette {
        match self {
            AppTheme::Dark => &DARK_PALETTE,
            AppTheme::Light => &LIGHT_PALETTE,
            AppTheme::Nord => &NORD_PALETTE,
        }
    }

    #[inline]
    pub const fn syntect_theme(self) -> &'static str {
        match self {
            AppTheme::Dark | AppTheme::Nord => "base16-eighties.dark",
            AppTheme::Light => "InspiredGitHub",
        }
    }

    #[inline]
    pub const fn iced_highlighter_theme(self) -> iced::highlighter::Theme {
        match self {
            AppTheme::Dark | AppTheme::Nord => iced::highlighter::Theme::Base16Mocha,
            AppTheme::Light => iced::highlighter::Theme::InspiredGitHub,
        }
    }

    fn code_fg(&self) -> iced::Color {
        match self {
            AppTheme::Dark | AppTheme::Nord => MD_DARK_CODE_FG,
            AppTheme::Light => MD_LIGHT_CODE_FG,
        }
    }

    pub fn resolve_code_fg(&self, syntect_theme: &syntect::highlighting::Theme) -> iced::Color {
        syntect_theme
            .settings
            .foreground
            .map(syntect_to_iced_color)
            .unwrap_or_else(|| self.code_fg())
    }
}

impl From<AppTheme> for iced::Theme {
    fn from(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark | AppTheme::Nord => iced::Theme::Dark,
            AppTheme::Light => iced::Theme::Light,
        }
    }
}

impl From<&iced::Theme> for AppTheme {
    fn from(theme: &iced::Theme) -> Self {
        match theme {
            iced::Theme::Dark => AppTheme::Dark,
            _ => AppTheme::Light,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ColorPalette {
    pub base: BaseColors,
    pub roles: RoleColors,
    pub sidebar: SidebarColors,
    pub json: JsonColors,
    pub markdown: MarkdownColors,
}

pub static DARK_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::DARK,
    roles: RoleColors::DARK,
    sidebar: SidebarColors::DARK,
    json: JsonColors::DARK,
    markdown: MarkdownColors::DARK,
};

pub static LIGHT_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::LIGHT,
    roles: RoleColors::LIGHT,
    sidebar: SidebarColors::LIGHT,
    json: JsonColors::LIGHT,
    markdown: MarkdownColors::LIGHT,
};

pub static NORD_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::NORD,
    roles: RoleColors::NORD,
    sidebar: SidebarColors::NORD,
    json: JsonColors::NORD,
    markdown: MarkdownColors::NORD,
};
