use iced::widget::{button, container, text};
use iced::{Border, Color, Element, Length, Shadow};

pub fn sidebar_entry_style(
    _theme: &iced::Theme,
    status: button::Status,
    is_active: bool,
    is_dark: bool,
) -> button::Style {
    let bg = match status {
        button::Status::Hovered | button::Status::Pressed => Some(
            (if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.06)
            })
            .into(),
        ),
        _ => {
            if is_active {
                Some(
                    (if is_dark {
                        Color::from_rgba(0.4, 0.7, 1.0, 0.15)
                    } else {
                        Color::from_rgba(0.1, 0.4, 0.8, 0.1)
                    })
                    .into(),
                )
            } else {
                None
            }
        }
    };
    let text_color = if is_active {
        if is_dark {
            Color::from_rgb(0.5, 0.8, 1.0)
        } else {
            Color::from_rgb(0.1, 0.45, 0.85)
        }
    } else if is_dark {
        Color::from_rgb(0.8, 0.82, 0.85)
    } else {
        Color::from_rgb(0.3, 0.32, 0.35)
    };

    button::Style {
        background: bg,
        text_color,
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn drag_handle<'a, Message: 'static + Clone>(
    is_resizing: bool,
    is_dark: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let (resize_col, normal_col) = if is_dark {
        (
            Color::from_rgb(0.4, 0.7, 1.0),
            Color::from_rgba(1.0, 1.0, 1.0, 0.05),
        )
    } else {
        (
            Color::from_rgb(0.1, 0.45, 0.85),
            Color::from_rgba(0.0, 0.0, 0.0, 0.05),
        )
    };
    let bg = if is_resizing { resize_col } else { normal_col };

    button(
        container(text(""))
            .width(4)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(bg.into()),
                ..Default::default()
            }),
    )
    .padding(0)
    .width(6)
    .height(Length::Fill)
    .on_press(on_press)
    .style(|_, _| button::Style {
        background: None,
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Color::TRANSPARENT,
        snap: false,
    })
    .into()
}

pub fn collapse_arrow<'a, Message: 'static + Clone>(
    is_collapsed: bool,
    is_dark: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let arrow_icon = if is_collapsed { "▶ " } else { "▼ " };
    let text_color = if is_dark {
        Color::from_rgb(0.6, 0.65, 0.7)
    } else {
        Color::from_rgb(0.5, 0.55, 0.6)
    };
    button(text(arrow_icon).size(9))
        .on_press(on_press)
        .style(move |_, _| button::Style {
            background: None,
            text_color,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: false,
        })
        .padding([2, 4])
        .into()
}
