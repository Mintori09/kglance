//! Semantic colour tokens for overlays, floating toolbars, pills, and selection.

use iced::Color;

use super::primitive;

#[derive(Clone, Copy)]
pub struct OverlayColors {
    pub floating_bg: Color,
    pub floating_border: Color,
    pub floating_fg: Color,
    pub floating_text_dim: Color,
    pub floating_text_muted: Color,
    pub floating_shadow: Color,
    pub selection_bg: Color,
    pub backdrop_modal: Color,
}

impl OverlayColors {
    pub const DARK: OverlayColors = OverlayColors {
        floating_bg: primitive::OVERLAY_FLOATING_DARK_BG,
        floating_border: primitive::OVERLAY_FLOATING_DARK_BORDER,
        floating_fg: primitive::OVERLAY_FLOATING_DARK_FG,
        floating_text_dim: primitive::OVERLAY_FLOATING_DARK_TEXT_DIM,
        floating_text_muted: primitive::OVERLAY_FLOATING_DARK_TEXT_MUTED,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_DARK_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const LIGHT: OverlayColors = OverlayColors {
        floating_bg: primitive::OVERLAY_FLOATING_LIGHT_BG,
        floating_border: primitive::OVERLAY_FLOATING_LIGHT_BORDER,
        floating_fg: primitive::OVERLAY_FLOATING_LIGHT_FG,
        floating_text_dim: primitive::OVERLAY_FLOATING_LIGHT_TEXT_DIM,
        floating_text_muted: primitive::OVERLAY_FLOATING_LIGHT_TEXT_MUTED,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_LIGHT_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const NORD: OverlayColors = OverlayColors {
        floating_bg: primitive::NORD0,
        floating_border: primitive::NORD3,
        floating_fg: primitive::NORD4,
        floating_text_dim: primitive::NORD9,
        floating_text_muted: primitive::NORD3,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_NORD_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const CATPPUCCIN_MOCHA: OverlayColors = OverlayColors {
        floating_bg: primitive::MOCHA_BASE,
        floating_border: primitive::MOCHA_SURFACE1,
        floating_fg: primitive::MOCHA_TEXT,
        floating_text_dim: primitive::MOCHA_SUBTEXT0,
        floating_text_muted: primitive::MOCHA_OVERLAY0,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_MOCHA_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const CATPPUCCIN_LATTE: OverlayColors = OverlayColors {
        floating_bg: primitive::LATTE_BASE,
        floating_border: primitive::LATTE_SURFACE1,
        floating_fg: primitive::LATTE_TEXT,
        floating_text_dim: primitive::LATTE_SUBTEXT0,
        floating_text_muted: primitive::LATTE_SURFACE0,
        floating_shadow: primitive::LIGHT_SHADOW,
        selection_bg: primitive::SELECTION_LATTE_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const TOKYO_NIGHT: OverlayColors = OverlayColors {
        floating_bg: primitive::TOKYO_BG,
        floating_border: primitive::TOKYO_BORDER,
        floating_fg: primitive::TOKYO_TEXT,
        floating_text_dim: primitive::TOKYO_TEXT_DIM,
        floating_text_muted: primitive::TOKYO_BORDER,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_TOKYO_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const GRUVBOX_DARK: OverlayColors = OverlayColors {
        floating_bg: primitive::GRUVBOX_BG,
        floating_border: primitive::GRUVBOX_BORDER,
        floating_fg: primitive::GRUVBOX_TEXT,
        floating_text_dim: primitive::GRUVBOX_TEXT_DIM,
        floating_text_muted: primitive::GRUVBOX_BORDER,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_GRUVBOX_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };

    pub const DRACULA: OverlayColors = OverlayColors {
        floating_bg: primitive::DRACULA_BG,
        floating_border: primitive::DRACULA_BORDER,
        floating_fg: primitive::DRACULA_TEXT,
        floating_text_dim: primitive::DRACULA_TEXT_DIM,
        floating_text_muted: primitive::DRACULA_BORDER,
        floating_shadow: primitive::OVERLAY_SHADOW,
        selection_bg: primitive::SELECTION_DRACULA_BG,
        backdrop_modal: primitive::OVERLAY_BACKDROP,
    };
}
