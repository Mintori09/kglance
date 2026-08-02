use iced::widget::{button, container, mouse_area, text};
use iced::{Background, Border, Color, Element, Length, Shadow, mouse};

use crate::ui::theme::AppTheme;

const BORDER_RADIUS: f32 = 4.0;
const ARROW_FONT_SIZE: f32 = 9.0;
const DRAG_HANDLE_WIDTH: f32 = 6.0;

pub const INDENT_PER_LEVEL: f32 = 12.0;
pub const SIDEBAR_BORDER_WIDTH: f32 = 1.0;
pub const SIDEBAR_ITEM_SPACING: f32 = 2.0;
pub const SIDEBAR_ENTRY_PADDING_V: f32 = 4.0;
pub const SIDEBAR_ENTRY_FONT_SIZE: f32 = 12.0;

const ICON_EXPANDED: &str = "▼ ";
const ICON_COLLAPSED: &str = "▶ ";

pub fn sidebar_entry_style(
    _theme: &iced::Theme,
    status: button::Status,
    is_active: bool,
    app_theme: AppTheme,
) -> button::Style {
    button::Style {
        background: determine_sidebar_entry_background(status, is_active, app_theme),
        text_color: determine_sidebar_entry_text_color(is_active, app_theme),
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
    app_theme: AppTheme,
    on_press: Message,
) -> Element<'a, Message> {
    let background_color = determine_drag_handle_color(is_resizing, app_theme);

    mouse_area(
        container(text(""))
            .width(DRAG_HANDLE_WIDTH)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(background_color.into()),
                ..Default::default()
            }),
    )
    .on_press(on_press)
    .interaction(mouse::Interaction::ResizingHorizontally)
    .into()
}

pub fn collapse_arrow<'a, Message: 'static + Clone>(
    is_collapsed: bool,
    app_theme: AppTheme,
    on_press: Message,
) -> Element<'a, Message> {
    let arrow_icon = if is_collapsed {
        ICON_COLLAPSED
    } else {
        ICON_EXPANDED
    };
    let text_color = app_theme.palette().sidebar.arrow_text;

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
    app_theme: AppTheme,
) -> Option<Background> {
    let c = &app_theme.palette().sidebar;
    match status {
        button::Status::Hovered | button::Status::Pressed => Some(c.hover_press.into()),
        _ if is_active => Some(c.active_bg.into()),
        _ => None,
    }
}

fn determine_sidebar_entry_text_color(is_active: bool, app_theme: AppTheme) -> Color {
    let c = &app_theme.palette().sidebar;
    if is_active {
        c.active_text
    } else {
        c.inactive_text
    }
}

fn determine_drag_handle_color(is_resizing: bool, app_theme: AppTheme) -> Color {
    let c = &app_theme.palette().sidebar;
    if is_resizing {
        c.resizing
    } else {
        c.normal_drag
    }
}
