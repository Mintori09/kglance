use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Element, Length, Shadow};

const BORDER_RADIUS: f32 = 4.0;
const ARROW_FONT_SIZE: f32 = 9.0;
const DRAG_HANDLE_CONTAINER_WIDTH: f32 = 4.0;
const DRAG_HANDLE_BUTTON_WIDTH: f32 = 6.0;

pub const INDENT_PER_LEVEL: f32 = 12.0;
pub const SIDEBAR_BORDER_WIDTH: f32 = 1.0;
pub const SIDEBAR_ITEM_SPACING: f32 = 2.0;
pub const SIDEBAR_ENTRY_PADDING_V: f32 = 4.0;
pub const SIDEBAR_ENTRY_FONT_SIZE: f32 = 12.0;

const ICON_EXPANDED: &str = "▼ ";
const ICON_COLLAPSED: &str = "▶ ";

const COLOR_DARK_HOVER_PRESS: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
const COLOR_LIGHT_HOVER_PRESS: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.06);

const COLOR_DARK_ACTIVE_BG: Color = Color::from_rgba(0.4, 0.7, 1.0, 0.15);
const COLOR_LIGHT_ACTIVE_BG: Color = Color::from_rgba(0.1, 0.4, 0.8, 0.1);

const COLOR_DARK_ACTIVE_TEXT: Color = Color::from_rgb(0.5, 0.8, 1.0);
const COLOR_LIGHT_ACTIVE_TEXT: Color = Color::from_rgb(0.1, 0.45, 0.85);

const COLOR_DARK_INACTIVE_TEXT: Color = Color::from_rgb(0.8, 0.82, 0.85);
const COLOR_LIGHT_INACTIVE_TEXT: Color = Color::from_rgb(0.3, 0.32, 0.35);

const COLOR_DARK_RESIZING: Color = Color::from_rgb(0.4, 0.7, 1.0);
const COLOR_LIGHT_RESIZING: Color = Color::from_rgb(0.1, 0.45, 0.85);

const COLOR_DARK_NORMAL_DRAG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);
const COLOR_LIGHT_NORMAL_DRAG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.05);

const COLOR_DARK_ARROW_TEXT: Color = Color::from_rgb(0.6, 0.65, 0.7);
const COLOR_LIGHT_ARROW_TEXT: Color = Color::from_rgb(0.5, 0.55, 0.6);

pub fn sidebar_entry_style(
    _theme: &iced::Theme,
    status: button::Status,
    is_active: bool,
    is_dark: bool,
) -> button::Style {
    button::Style {
        background: determine_sidebar_entry_background(status, is_active, is_dark),
        text_color: determine_sidebar_entry_text_color(is_active, is_dark),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: BORDER_RADIUS.into(),
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
    let background_color = determine_drag_handle_color(is_resizing, is_dark);

    button(
        container(text(""))
            .width(DRAG_HANDLE_CONTAINER_WIDTH)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(background_color.into()),
                ..Default::default()
            }),
    )
    .padding(0)
    .width(DRAG_HANDLE_BUTTON_WIDTH)
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
    let arrow_icon = if is_collapsed {
        ICON_COLLAPSED
    } else {
        ICON_EXPANDED
    };
    let text_color = if is_dark {
        COLOR_DARK_ARROW_TEXT
    } else {
        COLOR_LIGHT_ARROW_TEXT
    };

    button(text(arrow_icon).size(ARROW_FONT_SIZE))
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

fn determine_sidebar_entry_background(
    status: button::Status,
    is_active: bool,
    is_dark: bool,
) -> Option<Background> {
    match status {
        button::Status::Hovered | button::Status::Pressed => {
            let color = if is_dark {
                COLOR_DARK_HOVER_PRESS
            } else {
                COLOR_LIGHT_HOVER_PRESS
            };
            Some(color.into())
        }
        _ if is_active => {
            let color = if is_dark {
                COLOR_DARK_ACTIVE_BG
            } else {
                COLOR_LIGHT_ACTIVE_BG
            };
            Some(color.into())
        }
        _ => None,
    }
}

fn determine_sidebar_entry_text_color(is_active: bool, is_dark: bool) -> Color {
    match (is_active, is_dark) {
        (true, true) => COLOR_DARK_ACTIVE_TEXT,
        (true, false) => COLOR_LIGHT_ACTIVE_TEXT,
        (false, true) => COLOR_DARK_INACTIVE_TEXT,
        (false, false) => COLOR_LIGHT_INACTIVE_TEXT,
    }
}

fn determine_drag_handle_color(is_resizing: bool, is_dark: bool) -> Color {
    match (is_resizing, is_dark) {
        (true, true) => COLOR_DARK_RESIZING,
        (true, false) => COLOR_LIGHT_RESIZING,
        (false, true) => COLOR_DARK_NORMAL_DRAG,
        (false, false) => COLOR_LIGHT_NORMAL_DRAG,
    }
}
