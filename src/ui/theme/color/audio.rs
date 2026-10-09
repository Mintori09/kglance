//! Semantic colour tokens for the audio player component.

use iced::Color;

use super::primitive;

#[derive(Clone, Copy)]
pub struct AudioColors {
    pub vinyl_bg: Color,
    pub vinyl_border: Color,
    pub vinyl_shadow: Color,
}

impl AudioColors {
    pub const DARK: AudioColors = AudioColors {
        vinyl_bg: primitive::VINYL_DARK_BG,
        vinyl_border: primitive::VINYL_DARK_BORDER,
        vinyl_shadow: primitive::VINYL_DARK_SHADOW,
    };

    pub const LIGHT: AudioColors = AudioColors {
        vinyl_bg: primitive::VINYL_LIGHT_BG,
        vinyl_border: primitive::VINYL_LIGHT_BORDER,
        vinyl_shadow: primitive::VINYL_LIGHT_SHADOW,
    };

    pub const NORD: AudioColors = AudioColors {
        vinyl_bg: primitive::NORD0,
        vinyl_border: primitive::NORD3,
        vinyl_shadow: primitive::OVERLAY_SHADOW,
    };
}
