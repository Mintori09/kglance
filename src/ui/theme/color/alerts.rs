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
}
