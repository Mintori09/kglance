//! Semantic colour tokens for alerts (Markdown callouts, notification toasts, etc.).

use iced::Color;

use super::primitive;

#[derive(Clone, Copy)]
pub struct AlertGroup {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
}

#[derive(Clone, Copy)]
pub struct AlertColors {
    pub note: AlertGroup,
    pub tip: AlertGroup,
    pub info: AlertGroup,
    pub warning: AlertGroup,
    pub caution: AlertGroup,
}

impl AlertColors {
    pub const DARK: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::ALERT_NOTE_FG,
            bg: primitive::ALERT_NOTE_BG,
            border: primitive::ALERT_NOTE_FG,
        },
        tip: AlertGroup {
            fg: primitive::ALERT_TIP_FG,
            bg: primitive::ALERT_TIP_BG,
            border: primitive::ALERT_TIP_FG,
        },
        info: AlertGroup {
            fg: primitive::ALERT_INFO_FG,
            bg: primitive::ALERT_INFO_BG,
            border: primitive::ALERT_INFO_FG,
        },
        warning: AlertGroup {
            fg: primitive::ALERT_WARNING_FG,
            bg: primitive::ALERT_WARNING_BG,
            border: primitive::ALERT_WARNING_FG,
        },
        caution: AlertGroup {
            fg: primitive::ALERT_CAUTION_FG,
            bg: primitive::ALERT_CAUTION_BG,
            border: primitive::ALERT_CAUTION_FG,
        },
    };

    pub const LIGHT: AlertColors = Self::DARK;

    pub const NORD: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::NORD8,
            bg: primitive::WHITE_005,
            border: primitive::NORD8,
        },
        tip: AlertGroup {
            fg: primitive::NORD14,
            bg: primitive::WHITE_005,
            border: primitive::NORD14,
        },
        info: AlertGroup {
            fg: primitive::NORD9,
            bg: primitive::WHITE_005,
            border: primitive::NORD9,
        },
        warning: AlertGroup {
            fg: primitive::NORD13,
            bg: primitive::WHITE_005,
            border: primitive::NORD13,
        },
        caution: AlertGroup {
            fg: primitive::NORD11,
            bg: primitive::WHITE_005,
            border: primitive::NORD11,
        },
    };

    pub const CATPPUCCIN_MOCHA: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::MOCHA_BLUE,
            bg: primitive::MOCHA_SURFACE0,
            border: primitive::MOCHA_BLUE,
        },
        tip: AlertGroup {
            fg: primitive::MOCHA_GREEN,
            bg: primitive::MOCHA_SURFACE0,
            border: primitive::MOCHA_GREEN,
        },
        info: AlertGroup {
            fg: primitive::MOCHA_MAUVE,
            bg: primitive::MOCHA_SURFACE0,
            border: primitive::MOCHA_MAUVE,
        },
        warning: AlertGroup {
            fg: primitive::MOCHA_YELLOW,
            bg: primitive::MOCHA_SURFACE0,
            border: primitive::MOCHA_YELLOW,
        },
        caution: AlertGroup {
            fg: primitive::MOCHA_RED,
            bg: primitive::MOCHA_SURFACE0,
            border: primitive::MOCHA_RED,
        },
    };

    pub const CATPPUCCIN_LATTE: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::LATTE_BLUE,
            bg: primitive::LATTE_SURFACE0,
            border: primitive::LATTE_BLUE,
        },
        tip: AlertGroup {
            fg: primitive::LATTE_GREEN,
            bg: primitive::LATTE_SURFACE0,
            border: primitive::LATTE_GREEN,
        },
        info: AlertGroup {
            fg: primitive::LATTE_LAVENDER,
            bg: primitive::LATTE_SURFACE0,
            border: primitive::LATTE_LAVENDER,
        },
        warning: AlertGroup {
            fg: primitive::LATTE_YELLOW,
            bg: primitive::LATTE_SURFACE0,
            border: primitive::LATTE_YELLOW,
        },
        caution: AlertGroup {
            fg: primitive::LATTE_RED,
            bg: primitive::LATTE_SURFACE0,
            border: primitive::LATTE_RED,
        },
    };

    pub const TOKYO_NIGHT: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::TOKYO_BLUE,
            bg: primitive::TOKYO_SURFACE,
            border: primitive::TOKYO_BLUE,
        },
        tip: AlertGroup {
            fg: primitive::TOKYO_GREEN,
            bg: primitive::TOKYO_SURFACE,
            border: primitive::TOKYO_GREEN,
        },
        info: AlertGroup {
            fg: primitive::TOKYO_PURPLE,
            bg: primitive::TOKYO_SURFACE,
            border: primitive::TOKYO_PURPLE,
        },
        warning: AlertGroup {
            fg: primitive::TOKYO_YELLOW,
            bg: primitive::TOKYO_SURFACE,
            border: primitive::TOKYO_YELLOW,
        },
        caution: AlertGroup {
            fg: primitive::TOKYO_RED,
            bg: primitive::TOKYO_SURFACE,
            border: primitive::TOKYO_RED,
        },
    };

    pub const GRUVBOX_DARK: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::GRUVBOX_AQUA,
            bg: primitive::GRUVBOX_SURFACE,
            border: primitive::GRUVBOX_AQUA,
        },
        tip: AlertGroup {
            fg: primitive::GRUVBOX_GREEN,
            bg: primitive::GRUVBOX_SURFACE,
            border: primitive::GRUVBOX_GREEN,
        },
        info: AlertGroup {
            fg: primitive::GRUVBOX_PURPLE,
            bg: primitive::GRUVBOX_SURFACE,
            border: primitive::GRUVBOX_PURPLE,
        },
        warning: AlertGroup {
            fg: primitive::GRUVBOX_YELLOW,
            bg: primitive::GRUVBOX_SURFACE,
            border: primitive::GRUVBOX_YELLOW,
        },
        caution: AlertGroup {
            fg: primitive::GRUVBOX_RED,
            bg: primitive::GRUVBOX_SURFACE,
            border: primitive::GRUVBOX_RED,
        },
    };

    pub const DRACULA: AlertColors = AlertColors {
        note: AlertGroup {
            fg: primitive::DRACULA_CYAN,
            bg: primitive::DRACULA_SURFACE,
            border: primitive::DRACULA_CYAN,
        },
        tip: AlertGroup {
            fg: primitive::DRACULA_GREEN,
            bg: primitive::DRACULA_SURFACE,
            border: primitive::DRACULA_GREEN,
        },
        info: AlertGroup {
            fg: primitive::DRACULA_PURPLE,
            bg: primitive::DRACULA_SURFACE,
            border: primitive::DRACULA_PURPLE,
        },
        warning: AlertGroup {
            fg: primitive::DRACULA_YELLOW,
            bg: primitive::DRACULA_SURFACE,
            border: primitive::DRACULA_YELLOW,
        },
        caution: AlertGroup {
            fg: primitive::DRACULA_RED,
            bg: primitive::DRACULA_SURFACE,
            border: primitive::DRACULA_RED,
        },
    };
}
