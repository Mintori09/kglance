//! Colour token system.
//!
//! Layered design:
//! - [`primitive`] — raw named colours, the single source of truth.
//! - [`base`], [`roles`] — shared semantic tokens for the whole UI.
//! - [`markdown`], [`json`], [`sidebar`] — view/component semantic tokens.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
    Nord,
}

impl AppTheme {
    pub fn is_dark(self) -> bool {
        matches!(self, AppTheme::Dark | AppTheme::Nord)
    }

    pub fn palette(self) -> &'static ColorPalette {
        match self {
            AppTheme::Dark => &DARK_PALETTE,
            AppTheme::Light => &LIGHT_PALETTE,
            AppTheme::Nord => &NORD_PALETTE,
        }
    }
}

impl From<bool> for AppTheme {
    fn from(is_dark: bool) -> Self {
        if is_dark {
            AppTheme::Dark
        } else {
            AppTheme::Light
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
