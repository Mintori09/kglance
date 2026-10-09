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

    pub const CATPPUCCIN_MOCHA: AudioColors = AudioColors {
        vinyl_bg: primitive::MOCHA_CRUST,
        vinyl_border: primitive::MOCHA_SURFACE1,
        vinyl_shadow: primitive::OVERLAY_SHADOW,
    };

    pub const CATPPUCCIN_LATTE: AudioColors = AudioColors {
        vinyl_bg: primitive::LATTE_CRUST,
        vinyl_border: primitive::LATTE_SURFACE1,
        vinyl_shadow: primitive::LIGHT_SHADOW,
    };

    pub const TOKYO_NIGHT: AudioColors = AudioColors {
        vinyl_bg: primitive::TOKYO_BG,
        vinyl_border: primitive::TOKYO_BORDER,
        vinyl_shadow: primitive::OVERLAY_SHADOW,
    };

    pub const GRUVBOX_DARK: AudioColors = AudioColors {
        vinyl_bg: primitive::GRUVBOX_BG,
        vinyl_border: primitive::GRUVBOX_BORDER,
        vinyl_shadow: primitive::OVERLAY_SHADOW,
    };

    pub const DRACULA: AudioColors = AudioColors {
        vinyl_bg: primitive::DRACULA_BG,
        vinyl_border: primitive::DRACULA_BORDER,
        vinyl_shadow: primitive::OVERLAY_SHADOW,
    };
}
