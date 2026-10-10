//! Colour token system.
//!
//! Layered design:
//! - [`primitive`] — raw named colours, the single source of truth.
//! - [`base`], [`roles`] — shared semantic tokens for the whole UI.
//! - [`markdown`], [`json`], [`sidebar`], [`alerts`], [`symbols`], [`audio`], [`overlay`] — view/component semantic tokens.

use crate::ui::theme::color::primitive::{
    MD_DARK_CODE_FG, MD_LIGHT_CODE_FG, syntect_to_iced_color,
};
pub mod alerts;
pub mod audio;
pub mod base;
pub mod json;
pub mod markdown;
pub mod overlay;
pub mod primitive;
pub mod roles;
pub mod sidebar;
pub mod symbols;

pub use alerts::{AlertColors, AlertGroup};
pub use audio::AudioColors;
pub use base::BaseColors;
pub use json::JsonColors;
pub use markdown::MarkdownColors;
pub use overlay::OverlayColors;
pub use roles::RoleColors;
pub use sidebar::SidebarColors;
pub use symbols::SymbolColors;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
    CatppuccinMocha,
    CatppuccinLatte,
    TokyoNight,
    GruvboxDark,
    Nord,
    Dracula,
}

impl AppTheme {
    pub const ALL: [AppTheme; 8] = [
        AppTheme::Dark,
        AppTheme::Light,
        AppTheme::CatppuccinMocha,
        AppTheme::CatppuccinLatte,
        AppTheme::TokyoNight,
        AppTheme::GruvboxDark,
        AppTheme::Nord,
        AppTheme::Dracula,
    ];

    #[inline]
    pub const fn is_dark(self) -> bool {
        match self {
            AppTheme::Dark
            | AppTheme::CatppuccinMocha
            | AppTheme::TokyoNight
            | AppTheme::GruvboxDark
            | AppTheme::Nord
            | AppTheme::Dracula => true,
            AppTheme::Light | AppTheme::CatppuccinLatte => false,
        }
    }

    pub fn palette(self) -> &'static ColorPalette {
        match self {
            AppTheme::Dark => &DARK_PALETTE,
            AppTheme::Light => &LIGHT_PALETTE,
            AppTheme::CatppuccinMocha => &CATPPUCCIN_MOCHA_PALETTE,
            AppTheme::CatppuccinLatte => &CATPPUCCIN_LATTE_PALETTE,
            AppTheme::TokyoNight => &TOKYO_NIGHT_PALETTE,
            AppTheme::GruvboxDark => &GRUVBOX_DARK_PALETTE,
            AppTheme::Nord => &NORD_PALETTE,
            AppTheme::Dracula => &DRACULA_PALETTE,
        }
    }

    #[inline]
    pub const fn syntect_theme(self) -> &'static str {
        self.name()
    }

    #[inline]
    pub const fn iced_highlighter_theme(self) -> iced::highlighter::Theme {
        match self {
            AppTheme::Dark => iced::highlighter::Theme::Base16Eighties,
            AppTheme::Light => iced::highlighter::Theme::InspiredGitHub,
            AppTheme::CatppuccinMocha => iced::highlighter::Theme::Base16Mocha,
            AppTheme::CatppuccinLatte => iced::highlighter::Theme::InspiredGitHub,
            AppTheme::TokyoNight => iced::highlighter::Theme::SolarizedDark,
            AppTheme::GruvboxDark => iced::highlighter::Theme::Base16Eighties,
            AppTheme::Nord => iced::highlighter::Theme::Base16Ocean,
            AppTheme::Dracula => iced::highlighter::Theme::Base16Eighties,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            AppTheme::Dark => "dark",
            AppTheme::Light => "light",
            AppTheme::CatppuccinMocha => "catppuccin-mocha",
            AppTheme::CatppuccinLatte => "catppuccin-latte",
            AppTheme::TokyoNight => "tokyo-night",
            AppTheme::GruvboxDark => "gruvbox-dark",
            AppTheme::Nord => "nord",
            AppTheme::Dracula => "dracula",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
            AppTheme::CatppuccinMocha => "Catppuccin Mocha",
            AppTheme::CatppuccinLatte => "Catppuccin Latte",
            AppTheme::TokyoNight => "Tokyo Night",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::Nord => "Nord",
            AppTheme::Dracula => "Dracula",
        }
    }

    pub const fn next_theme(self) -> Self {
        match self {
            AppTheme::Dark => AppTheme::Light,
            AppTheme::Light => AppTheme::CatppuccinMocha,
            AppTheme::CatppuccinMocha => AppTheme::CatppuccinLatte,
            AppTheme::CatppuccinLatte => AppTheme::TokyoNight,
            AppTheme::TokyoNight => AppTheme::GruvboxDark,
            AppTheme::GruvboxDark => AppTheme::Nord,
            AppTheme::Nord => AppTheme::Dracula,
            AppTheme::Dracula => AppTheme::Dark,
        }
    }

    fn code_fg(&self) -> iced::Color {
        match self {
            AppTheme::Dark
            | AppTheme::CatppuccinMocha
            | AppTheme::TokyoNight
            | AppTheme::GruvboxDark
            | AppTheme::Nord
            | AppTheme::Dracula => MD_DARK_CODE_FG,
            AppTheme::Light | AppTheme::CatppuccinLatte => MD_LIGHT_CODE_FG,
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

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for AppTheme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace(['_', ' '], "-");
        match normalized.as_str() {
            "dark" | "breeze-dark" => Ok(AppTheme::Dark),
            "light" | "breeze-light" => Ok(AppTheme::Light),
            "catppuccin" | "catppuccin-mocha" | "mocha" => Ok(AppTheme::CatppuccinMocha),
            "catppuccin-latte" | "latte" => Ok(AppTheme::CatppuccinLatte),
            "tokyo-night" | "tokyonight" | "tokyo" => Ok(AppTheme::TokyoNight),
            "gruvbox" | "gruvbox-dark" => Ok(AppTheme::GruvboxDark),
            "nord" => Ok(AppTheme::Nord),
            "dracula" => Ok(AppTheme::Dracula),
            _ => Err(()),
        }
    }
}

impl From<AppTheme> for iced::Theme {
    fn from(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => iced::Theme::Dark,
            AppTheme::Light => iced::Theme::Light,
            AppTheme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            AppTheme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            AppTheme::TokyoNight => iced::Theme::TokyoNight,
            AppTheme::GruvboxDark => iced::Theme::GruvboxDark,
            AppTheme::Nord => iced::Theme::Nord,
            AppTheme::Dracula => iced::Theme::Dracula,
        }
    }
}

impl From<&iced::Theme> for AppTheme {
    fn from(theme: &iced::Theme) -> Self {
        match theme {
            iced::Theme::Dark => AppTheme::Dark,
            iced::Theme::Light => AppTheme::Light,
            iced::Theme::CatppuccinMocha => AppTheme::CatppuccinMocha,
            iced::Theme::CatppuccinLatte => AppTheme::CatppuccinLatte,
            iced::Theme::TokyoNight => AppTheme::TokyoNight,
            iced::Theme::GruvboxDark => AppTheme::GruvboxDark,
            iced::Theme::Nord => AppTheme::Nord,
            iced::Theme::Dracula => AppTheme::Dracula,
            _ => AppTheme::Dark,
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
    pub alerts: AlertColors,
    pub symbols: SymbolColors,
    pub audio: AudioColors,
    pub overlay: OverlayColors,
}

pub static DARK_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::DARK,
    roles: RoleColors::DARK,
    sidebar: SidebarColors::DARK,
    json: JsonColors::DARK,
    markdown: MarkdownColors::DARK,
    alerts: AlertColors::DARK,
    symbols: SymbolColors::DARK,
    audio: AudioColors::DARK,
    overlay: OverlayColors::DARK,
};

pub static LIGHT_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::LIGHT,
    roles: RoleColors::LIGHT,
    sidebar: SidebarColors::LIGHT,
    json: JsonColors::LIGHT,
    markdown: MarkdownColors::LIGHT,
    alerts: AlertColors::LIGHT,
    symbols: SymbolColors::LIGHT,
    audio: AudioColors::LIGHT,
    overlay: OverlayColors::LIGHT,
};

pub static NORD_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::NORD,
    roles: RoleColors::NORD,
    sidebar: SidebarColors::NORD,
    json: JsonColors::NORD,
    markdown: MarkdownColors::NORD,
    alerts: AlertColors::NORD,
    symbols: SymbolColors::NORD,
    audio: AudioColors::NORD,
    overlay: OverlayColors::NORD,
};

pub static CATPPUCCIN_MOCHA_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::CATPPUCCIN_MOCHA,
    roles: RoleColors::CATPPUCCIN_MOCHA,
    sidebar: SidebarColors::CATPPUCCIN_MOCHA,
    json: JsonColors::CATPPUCCIN_MOCHA,
    markdown: MarkdownColors::CATPPUCCIN_MOCHA,
    alerts: AlertColors::CATPPUCCIN_MOCHA,
    symbols: SymbolColors::CATPPUCCIN_MOCHA,
    audio: AudioColors::CATPPUCCIN_MOCHA,
    overlay: OverlayColors::CATPPUCCIN_MOCHA,
};

pub static CATPPUCCIN_LATTE_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::CATPPUCCIN_LATTE,
    roles: RoleColors::CATPPUCCIN_LATTE,
    sidebar: SidebarColors::CATPPUCCIN_LATTE,
    json: JsonColors::CATPPUCCIN_LATTE,
    markdown: MarkdownColors::CATPPUCCIN_LATTE,
    alerts: AlertColors::CATPPUCCIN_LATTE,
    symbols: SymbolColors::CATPPUCCIN_LATTE,
    audio: AudioColors::CATPPUCCIN_LATTE,
    overlay: OverlayColors::CATPPUCCIN_LATTE,
};

pub static TOKYO_NIGHT_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::TOKYO_NIGHT,
    roles: RoleColors::TOKYO_NIGHT,
    sidebar: SidebarColors::TOKYO_NIGHT,
    json: JsonColors::TOKYO_NIGHT,
    markdown: MarkdownColors::TOKYO_NIGHT,
    alerts: AlertColors::TOKYO_NIGHT,
    symbols: SymbolColors::TOKYO_NIGHT,
    audio: AudioColors::TOKYO_NIGHT,
    overlay: OverlayColors::TOKYO_NIGHT,
};

pub static GRUVBOX_DARK_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::GRUVBOX_DARK,
    roles: RoleColors::GRUVBOX_DARK,
    sidebar: SidebarColors::GRUVBOX_DARK,
    json: JsonColors::GRUVBOX_DARK,
    markdown: MarkdownColors::GRUVBOX_DARK,
    alerts: AlertColors::GRUVBOX_DARK,
    symbols: SymbolColors::GRUVBOX_DARK,
    audio: AudioColors::GRUVBOX_DARK,
    overlay: OverlayColors::GRUVBOX_DARK,
};

pub static DRACULA_PALETTE: ColorPalette = ColorPalette {
    base: BaseColors::DRACULA,
    roles: RoleColors::DRACULA,
    sidebar: SidebarColors::DRACULA,
    json: JsonColors::DRACULA,
    markdown: MarkdownColors::DRACULA,
    alerts: AlertColors::DRACULA,
    symbols: SymbolColors::DRACULA,
    audio: AudioColors::DRACULA,
    overlay: OverlayColors::DRACULA,
};
