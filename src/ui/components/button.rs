use iced::{Background, Border, Color, Shadow, Theme};

pub use iced::widget::button::{Status, Style};

use crate::ui::theme::glass;

/// Standard KDE Breeze-style button.
///
/// - Subtle surface background
/// - 1px border
/// - 4px border radius
/// - Accent highlight on hover/pressed
pub fn breeze(theme: &Theme, status: Status) -> Style {
    let p = glass::palette(theme);
    let (bg, border_color, text_color) = match status {
        Status::Active => (p.surface_raised, p.border, p.text),
        Status::Hovered => (glass::ACCENT_HOVER, glass::ACCENT_HOVER, Color::WHITE),
        Status::Pressed => (glass::ACCENT_PRESSED, glass::ACCENT_PRESSED, Color::WHITE),
        Status::Disabled => (p.surface, p.border, p.text_dim),
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
    let p = glass::palette(theme);
    let bg = match status {
        Status::Hovered => glass::ACCENT_HOVER,
        Status::Pressed => glass::ACCENT_PRESSED,
        Status::Disabled => p.surface,
        _ => glass::ACCENT,
    };
    let text_color = match status {
        Status::Disabled => p.text_dim,
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
    let p = glass::palette(theme);
    let bg = match status {
        Status::Hovered | Status::Pressed => {
            let a = if p.bg.r > 0.5 { 0.06 } else { 0.10 };
            Some(Background::Color(Color {
                a,
                ..if p.bg.r > 0.5 {
                    Color::BLACK
                } else {
                    Color::WHITE
                }
            }))
        }
        _ => None,
    };

    Style {
        background: bg,
        text_color: p.text,
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
    let p = glass::palette(theme);
    match status {
        Status::Active => Style {
            background: None,
            text_color: p.text,
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
            text_color: p.text_dim,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        },
    }
}
