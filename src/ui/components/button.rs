use iced::{Background, Border, Color, Shadow, Theme, Vector, widget::button};

pub fn glass_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = palette.background.base.color;
    let text = palette.background.base.text;

    match status {
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color { a: 0.05, ..base })),

            text_color: Color { a: 0.35, ..text },

            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: Color { a: 0.06, ..text },
            },

            shadow: Shadow::default(),
        },

        _ => {
            let (alpha, border_alpha, border_width, shadow_blur, shadow_alpha) = match status {
                button::Status::Active => (0.10, 0.10, 1.0, 4.0, 0.06),

                button::Status::Hovered => (0.40, 0.40, 1.5, 24.0, 0.28),

                button::Status::Pressed => (0.55, 0.50, 1.5, 6.0, 0.10),

                button::Status::Disabled => unreachable!(),
            };

            button::Style {
                background: Some(Background::Color(Color { a: alpha, ..base })),

                text_color: text,

                border: Border {
                    radius: 12.0.into(),
                    width: border_width,
                    color: Color {
                        a: border_alpha,
                        ..text
                    },
                },

                shadow: Shadow {
                    offset: if matches!(status, button::Status::Pressed) {
                        Vector::new(0.0, 1.0)
                    } else {
                        Vector::new(0.0, 3.0)
                    },

                    blur_radius: shadow_blur,

                    color: Color::from_rgba(0.0, 0.0, 0.0, shadow_alpha),
                },
            }
        }
    }
}
