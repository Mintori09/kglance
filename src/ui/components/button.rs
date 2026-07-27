use iced::{Background, Border, Color, Shadow, Theme};

pub use iced::widget::button::{Status, Style};

use crate::ui::theme::glass::{ACCENT, ACCENT_HOVER, ACCENT_PRESSED};
use crate::ui::theme::{
    DARK_BORDER, DARK_SURFACE, DARK_SURFACE_RAISED, DARK_TEXT, DARK_TEXT_DIM, LIGHT_BORDER,
    LIGHT_SURFACE, LIGHT_SURFACE_RAISED, LIGHT_TEXT, LIGHT_TEXT_DIM,
};

fn is_dark(theme: &Theme) -> bool {
    matches!(theme, Theme::Dark)
}

/// Standard KDE Breeze-style button.
///
/// - Subtle surface background
/// - 1px border
/// - 4px border radius
/// - Accent highlight on hover/pressed
pub fn breeze(theme: &Theme, status: Status) -> Style {
    let dark = is_dark(theme);
    let (bg, border_color, text_color) = match status {
        Status::Active => (
            if dark {
                DARK_SURFACE_RAISED
            } else {
                LIGHT_SURFACE_RAISED
            },
            if dark { DARK_BORDER } else { LIGHT_BORDER },
            if dark { DARK_TEXT } else { LIGHT_TEXT },
        ),
        Status::Hovered => (ACCENT_HOVER, ACCENT_HOVER, Color::WHITE),
        Status::Pressed => (ACCENT_PRESSED, ACCENT_PRESSED, Color::WHITE),
        Status::Disabled => (
            if dark { DARK_SURFACE } else { LIGHT_SURFACE },
            if dark { DARK_BORDER } else { LIGHT_BORDER },
            if dark { DARK_TEXT_DIM } else { LIGHT_TEXT_DIM },
        ),
    };

    Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: if matches!(status, Status::Disabled) {
                0.0
            } else {
                1.0
            },
            color: border_color,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Primary action button filled with accent color.
pub fn breeze_primary(theme: &Theme, status: Status) -> Style {
    let dark = is_dark(theme);
    let bg = match status {
        Status::Hovered => ACCENT_HOVER,
        Status::Pressed => ACCENT_PRESSED,
        Status::Disabled => {
            if dark {
                DARK_SURFACE
            } else {
                LIGHT_SURFACE
            }
        }
        _ => ACCENT,
    };
    let text_color = match status {
        Status::Disabled => {
            if dark {
                DARK_TEXT_DIM
            } else {
                LIGHT_TEXT_DIM
            }
        }
        _ => Color::WHITE,
    };

    Style {
        background: Some(Background::Color(bg)),
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Flat toolbar-style button with hover-only background.
pub fn breeze_tool(theme: &Theme, status: Status) -> Style {
    let dark = is_dark(theme);
    let bg = match status {
        Status::Hovered | Status::Pressed => {
            let a = if dark { 0.10 } else { 0.06 };
            Some(Background::Color(Color {
                a,
                ..if dark { Color::WHITE } else { Color::BLACK }
            }))
        }
        _ => None,
    };

    Style {
        background: bg,
        text_color: if dark { DARK_TEXT } else { LIGHT_TEXT },
        border: Border {
            radius: 3.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Destructive close button (red hover/pressed).
pub fn breeze_close(theme: &Theme, status: Status) -> Style {
    let dark = is_dark(theme);
    match status {
        Status::Active => Style {
            background: None,
            text_color: if dark { DARK_TEXT } else { LIGHT_TEXT },
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Hovered => Style {
            background: Some(Background::Color(Color::from_rgb8(232, 17, 35))),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Pressed => Style {
            background: Some(Background::Color(Color::from_rgb8(180, 0, 0))),
            text_color: Color::WHITE,
            border: Border {
                radius: 4.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: false,
        },

        Status::Disabled => Style {
            background: None,
            text_color: if dark { DARK_TEXT_DIM } else { LIGHT_TEXT_DIM },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}
