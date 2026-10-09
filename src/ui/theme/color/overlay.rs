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
}
