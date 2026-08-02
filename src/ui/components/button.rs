use crate::ui::theme::color::base::BaseColors;
use crate::ui::theme::color::{primitive, roles};
pub use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Theme};

pub fn breeze(theme: &Theme, status: Status) -> Style {
    let p = BaseColors::palette(theme);
    let role = roles::RoleColors::DARK;
    let (bg, border_color, text_color) = match status {
        Status::Active => (p.surface_raised, p.border, p.text),
        Status::Hovered => (role.accent_hover, role.accent_hover, Color::WHITE),
        Status::Pressed => (role.accent_pressed, role.accent_pressed, Color::WHITE),
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

pub fn breeze_primary(theme: &Theme, status: Status) -> Style {
    let p = BaseColors::palette(theme);
    let role = roles::RoleColors::DARK;
    let bg = match status {
        Status::Hovered => role.accent_hover,
        Status::Pressed => role.accent_pressed,
        Status::Disabled => p.surface,
        _ => role.accent,
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

pub fn breeze_tool(theme: &Theme, status: Status) -> Style {
    let p = BaseColors::palette(theme);
    let bg = match status {
        Status::Hovered | Status::Pressed => Some(
            if p.bg.r > 0.5 {
                primitive::BLACK_006
            } else {
                primitive::WHITE_010
            }
            .into(),
        ),
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

pub fn transparent(_theme: &Theme, _status: Status) -> Style {
    Style {
        background: None,
        text_color: Color::TRANSPARENT,
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn breeze_close(theme: &Theme, status: Status) -> Style {
    let p = BaseColors::palette(theme);
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
            background: Some(Background::Color(primitive::DANGER)),
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
            background: Some(Background::Color(primitive::DANGER_PRESSED)),
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
