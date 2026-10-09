use iced::widget::{button, checkbox, container, pick_list, rule, scrollable, slider, text_input};
use iced::{Border, Color, Shadow, Theme};

use crate::ui::theme::color::{AppTheme, BaseColors, primitive, roles};
use crate::ui::theme::tokens::{elevation, radius};

fn card_shadow(shadow_color: Color) -> Shadow {
    elevation::high(shadow_color)
}

fn subtle_shadow(shadow_color: Color) -> Shadow {
    elevation::low(shadow_color)
}

fn container_border(color: Color, r: f32) -> Border {
    Border {
        color,
        width: 1.0,
        radius: r.into(),
    }
}

fn default_bg_container(theme: &Theme, border: Border) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text),
        border,
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn default_root(theme: &Theme) -> container::Style {
    default_bg_container(theme, Border::default())
}

pub fn default_card(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text),
        border: container_border(p.border, radius::XL),
        shadow: card_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_raised(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface_raised.into()),
        text_color: Some(p.text),
        border: container_border(p.border, radius::NONE),
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_inset(theme: &Theme) -> container::Style {
    default_bg_container(
        theme,
        container_border(BaseColors::palette(theme).border, radius::MD),
    )
}

pub fn default_button(theme: &Theme, status: button::Status) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let (bg, border_color, text_color) = match status {
        button::Status::Hovered => (role.accent_hover, role.accent_hover, Color::WHITE),
        button::Status::Pressed => (role.accent_pressed, role.accent_pressed, Color::WHITE),
        _ => (p.surface, p.border, p.text),
    };

    button::Style {
        background: Some(bg.into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::LG.into(),
        },
        text_color,
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_row_button(
    theme: &Theme,
    status: button::Status,
    is_selected: bool,
) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);
    let text_color = p.text;
    let app_theme = AppTheme::from(theme);

    let bg_color = match (is_selected, status) {
        (true, button::Status::Hovered) => {
            let mut c = role.accent;
            c.a = 0.20;
            Some(c.into())
        }
        (true, _) => {
            let mut c = role.accent;
            c.a = 0.15;
            Some(c.into())
        }
        (false, button::Status::Hovered) => Some(
            if app_theme.is_dark() {
                primitive::WHITE_006
            } else {
                primitive::BLACK_006
            }
            .into(),
        ),
        (false, _) => None,
    };

    let border = if is_selected {
        let mut bc = role.accent;
        bc.a = 0.15;
        Border {
            color: bc,
            width: 1.0,
            radius: radius::MD.into(),
        }
    } else {
        Border::default()
    };

    button::Style {
        background: bg_color,
        text_color,
        border,
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn default_grid_card(
    theme: &Theme,
    status: button::Status,
    is_selected: bool,
) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);
    let app_theme = AppTheme::from(theme);

    let (bg_color, border_color, border_width, shadow) = if is_selected {
        let mut active_bg = role.accent;
        active_bg.a = if app_theme.is_dark() { 0.25 } else { 0.20 };

        let active_shadow = Shadow {
            color: Color {
                a: 0.4,
                ..role.accent
            },
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        };
        (active_bg, role.accent, 2.0, active_shadow)
    } else {
        match status {
            button::Status::Hovered => {
                let (hover_bg, hover_border) = if app_theme.is_dark() {
                    (primitive::WHITE_012, primitive::WHITE_020)
                } else {
                    (primitive::BLACK_008, primitive::BLACK_015)
                };
                (hover_bg, hover_border, 1.0, Shadow::default())
            }
            _ => (p.surface, p.border, 1.0, Shadow::default()),
        }
    };

    button::Style {
        background: Some(bg_color.into()),
        text_color: p.text,
        border: Border {
            color: border_color,
            width: border_width,
            radius: (radius::LG + 2.0).into(),
        },
        shadow,
        snap: false,
    }
}

pub fn default_button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);
    let bg = match status {
        button::Status::Hovered => role.accent_hover,
        button::Status::Pressed => role.accent_pressed,
        _ => role.accent,
    };
    button::Style {
        background: Some(bg.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius::LG.into(),
        },
        text_color: Color::WHITE,
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let (bg, border_color) = match status {
        text_input::Status::Focused { .. } => (p.surface_raised, role.accent),
        text_input::Status::Hovered => (p.surface_raised, p.border_focus),
        _ => (p.surface, p.border),
    };

    text_input::Style {
        background: bg.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::LG.into(),
        },
        value: p.text,
        placeholder: p.text_dim,
        selection: role.accent,
        icon: p.text,
    }
}

pub fn default_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let (scroller_bg, scroller_radius, rail_bg) = match status {
        scrollable::Status::Dragged { .. } => (
            role.accent_pressed,
            radius::XS + 1.0,
            Color {
                a: 0.12,
                ..p.border
            },
        ),
        scrollable::Status::Hovered { .. } => (
            role.accent_hover,
            radius::XS + 1.0,
            Color {
                a: 0.08,
                ..p.border
            },
        ),
        _ => (
            Color {
                a: 0.35,
                ..p.text_dim
            },
            radius::XS + 1.0,
            Color::TRANSPARENT,
        ),
    };

    let scroller = scrollable::Scroller {
        background: scroller_bg.into(),
        border: Border {
            radius: scroller_radius.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(rail_bg.into()),
            border: Border::default(),
            scroller,
        },
        horizontal_rail: scrollable::Rail {
            background: Some(rail_bg.into()),
            border: Border::default(),
            scroller,
        },
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: p.surface.into(),
            border: Border {
                radius: radius::SM.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            icon: p.text_dim,
        },
    }
}

pub fn default_slider(theme: &Theme, status: slider::Status) -> slider::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let handle_color = match status {
        slider::Status::Hovered | slider::Status::Dragged => role.accent_hover,
        _ => role.accent,
    };

    let rail_color = Color {
        a: 0.15,
        ..p.border
    };

    slider::Style {
        rail: slider::Rail {
            backgrounds: (role.accent.into(), rail_color.into()),
            width: 4.0,
            border: Border {
                radius: radius::XS.into(),
                ..Border::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 7.0 },
            background: handle_color.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

pub fn default_checkbox(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let (bg, border_color, icon_color) = match status {
        checkbox::Status::Active { is_checked: true }
        | checkbox::Status::Disabled { is_checked: true } => {
            (role.accent, role.accent, Color::WHITE)
        }
        checkbox::Status::Hovered { is_checked: true } => {
            (role.accent_hover, role.accent_hover, Color::WHITE)
        }
        checkbox::Status::Hovered { is_checked: false } => {
            (p.surface_raised, p.border_focus, p.text)
        }
        _ => (p.surface, p.border, p.text),
    };

    checkbox::Style {
        background: bg.into(),
        icon_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::SM.into(),
        },
        text_color: Some(p.text),
    }
}

pub fn default_tooltip(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    let app_theme = AppTheme::from(theme);
    let (bg, border_color) = if app_theme.is_dark() {
        (primitive::DARK_TOOLTIP, p.border)
    } else {
        (primitive::LIGHT_TOOLTIP, p.border)
    };
    container::Style {
        background: Some(bg.into()),
        text_color: Some(p.text),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::MD.into(),
        },
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_rule(theme: &Theme) -> rule::Style {
    let p = BaseColors::palette(theme);
    rule::Style {
        color: p.rule,
        radius: radius::NONE.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

pub fn default_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let (bg, border_color) = match status {
        pick_list::Status::Opened { .. } => (p.surface_raised, role.accent),
        pick_list::Status::Hovered => (p.surface_raised, p.border_focus),
        _ => (p.surface, p.border),
    };

    pick_list::Style {
        text_color: p.text,
        placeholder_color: p.text_dim,
        handle_color: p.text_dim,
        background: bg.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: radius::LG.into(),
        },
    }
}

/// Floating pill container for overlays (video/image controls, toolbars).
pub fn video_controls_pill(theme: &Theme) -> container::Style {
    let overlay = AppTheme::from(theme).palette().overlay;
    container::Style {
        background: Some(overlay.floating_bg.into()),
        text_color: Some(overlay.floating_fg),
        border: Border {
            color: overlay.floating_border,
            width: 1.0,
            radius: radius::XL.into(),
        },
        shadow: elevation::floating_pill(overlay.floating_shadow),
        snap: false,
    }
}

/// Circular icon button used inside floating overlay pills.
pub fn video_pill_button(_theme: &Theme, status: button::Status) -> button::Style {
    let (bg, text_color) = match status {
        button::Status::Hovered => (Some(primitive::WHITE_012.into()), primitive::WHITE),
        button::Status::Pressed => (Some(primitive::WHITE_020.into()), primitive::WHITE),
        _ => (None, primitive::OVERLAY_FLOATING_DARK_TEXT_DIM),
    };

    button::Style {
        background: bg,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: radius::FULL.into(),
        },
        text_color,
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Sleek slider for media players on translucent HUD overlays.
pub fn video_slider(theme: &Theme, status: slider::Status) -> slider::Style {
    let role = roles::palette(theme);

    let (handle_color, handle_radius) = match status {
        slider::Status::Hovered | slider::Status::Dragged => (primitive::WHITE, 5.5),
        _ => (role.accent, 4.5),
    };

    slider::Style {
        rail: slider::Rail {
            backgrounds: (role.accent.into(), primitive::WHITE_020.into()),
            width: 3.5,
            border: Border {
                radius: radius::XS.into(),
                ..Border::default()
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle {
                radius: handle_radius,
            },
            background: handle_color.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

/// Helper container style for small badges (PDF page counter, format badge, etc.).
pub fn badge_style(theme: &Theme, active: bool) -> container::Style {
    let palette = AppTheme::from(theme).palette();
    let (bg, text_color) = if active {
        (palette.roles.accent, primitive::WHITE)
    } else {
        (palette.base.surface_raised, palette.base.text_dim)
    };

    container::Style {
        background: Some(bg.into()),
        text_color: Some(text_color),
        border: Border {
            color: palette.base.border,
            width: 1.0,
            radius: radius::SM.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}
