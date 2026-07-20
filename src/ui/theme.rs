use iced::{Color, Border, Shadow, Theme};
use iced::widget::{button, container, text_input};

// KDE Breeze Colors
pub const BREEZE_BLUE: Color = Color::from_rgb(0.24, 0.68, 0.91); // #3daee9
pub const BREEZE_BLUE_DARK: Color = Color::from_rgb(0.16, 0.48, 0.64);

// Dark Mode Colors (Breeze Dark)
pub const DARK_BG: Color = Color::from_rgb(0.16, 0.18, 0.20); // #2a2e32
pub const DARK_CONTAINER: Color = Color::from_rgb(0.19, 0.21, 0.23); // #31363b
pub const DARK_BORDER: Color = Color::from_rgb(0.30, 0.31, 0.32); // #4d5052
pub const DARK_TEXT: Color = Color::from_rgb(0.94, 0.94, 0.95); // #eff0f1

// Light Mode Colors (Breeze Light)
pub const LIGHT_BG: Color = Color::from_rgb(0.99, 0.99, 0.99); // #fcfcfc
pub const LIGHT_CONTAINER: Color = Color::from_rgb(0.94, 0.94, 0.95); // #eff0f1
pub const LIGHT_BORDER: Color = Color::from_rgb(0.73, 0.74, 0.75); // #babdbf
pub const LIGHT_TEXT: Color = Color::from_rgb(0.19, 0.21, 0.23); // #31363b

pub fn breeze_button(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = matches!(theme, Theme::Dark);

    let (bg, border, text_color) = if is_dark {
        match status {
            button::Status::Hovered => (
                BREEZE_BLUE,
                BREEZE_BLUE,
                Color::WHITE,
            ),
            button::Status::Pressed => (
                BREEZE_BLUE_DARK,
                BREEZE_BLUE_DARK,
                Color::WHITE,
            ),
            _ => (
                DARK_CONTAINER,
                DARK_BORDER,
                DARK_TEXT,
            )
        }
    } else {
        match status {
            button::Status::Hovered => (
                BREEZE_BLUE,
                BREEZE_BLUE,
                Color::WHITE,
            ),
            button::Status::Pressed => (
                BREEZE_BLUE_DARK,
                BREEZE_BLUE_DARK,
                Color::WHITE,
            ),
            _ => (
                LIGHT_CONTAINER,
                LIGHT_BORDER,
                LIGHT_TEXT,
            )
        }
    };

    button::Style {
        background: Some(bg.into()),
        border: Border {
            color: border,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color,
        shadow: Shadow::default(),
    }
}

pub fn breeze_container(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);

    if is_dark {
        container::Style {
            background: Some(DARK_BG.into()),
            text_color: Some(DARK_TEXT),
            border: Border {
                color: DARK_BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
        }
    } else {
        container::Style {
            background: Some(LIGHT_BG.into()),
            text_color: Some(LIGHT_TEXT),
            border: Border {
                color: LIGHT_BORDER,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn breeze_header_container(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);

    if is_dark {
        container::Style {
            background: Some(DARK_CONTAINER.into()),
            text_color: Some(DARK_TEXT),
            border: Border {
                color: DARK_BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
        }
    } else {
        container::Style {
            background: Some(LIGHT_CONTAINER.into()),
            text_color: Some(LIGHT_TEXT),
            border: Border {
                color: LIGHT_BORDER,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn breeze_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let is_dark = matches!(theme, Theme::Dark);

    let (bg, border, text_color) = if is_dark {
        match status {
            text_input::Status::Focused => (DARK_BG, BREEZE_BLUE, DARK_TEXT),
            _ => (DARK_BG, DARK_BORDER, DARK_TEXT),
        }
    } else {
        match status {
            text_input::Status::Focused => (LIGHT_BG, BREEZE_BLUE, LIGHT_TEXT),
            _ => (LIGHT_BG, LIGHT_BORDER, LIGHT_TEXT),
        }
    };

    text_input::Style {
        background: bg.into(),
        border: Border {
            color: border,
            width: 1.0,
            radius: 4.0.into(),
        },
        value: text_color,
        placeholder: Color::from_rgb(0.5, 0.5, 0.5),
        selection: BREEZE_BLUE,
        icon: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
    }
}
