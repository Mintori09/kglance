use iced::widget::{
    button, checkbox, container, pick_list, rule, scrollable, slider, text_editor, text_input,
};
use iced::{Border, Color, Shadow, Theme};

use crate::ui::theme::color::{BaseColors, primitive, roles};

fn card_shadow(shadow_color: Color) -> Shadow {
    Shadow {
        color: shadow_color,
        offset: iced::Vector::new(0.0, 4.0),
        blur_radius: 16.0,
    }
}

fn subtle_shadow(shadow_color: Color) -> Shadow {
    Shadow {
        color: shadow_color,
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 6.0,
    }
}

pub fn default_root(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

pub fn default_card(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: card_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_raised(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.surface_raised.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 0.0.into(),
        },
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_inset(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
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
            radius: 8.0.into(),
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
            if p.bg.r > 0.5 {
                primitive::BLACK_006
            } else {
                primitive::WHITE_006
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
            radius: 6.0.into(),
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
    let is_dark = matches!(theme, Theme::Dark);

    let (bg_color, border_color, border_width, shadow) = if is_selected {
        let mut active_bg = role.accent;
        active_bg.a = if is_dark { 0.25 } else { 0.20 };

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
                let hover_bg = if is_dark {
                    primitive::WHITE_012
                } else {
                    primitive::BLACK_008
                };
                let hover_border = if is_dark {
                    primitive::WHITE_020
                } else {
                    primitive::BLACK_015
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
            radius: 10.0.into(),
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
            radius: 8.0.into(),
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
            radius: 8.0.into(),
        },
        value: p.text,
        placeholder: p.text_dim,
        selection: role.accent,
        icon: p.text,
    }
}

pub fn default_text_editor(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let p = BaseColors::palette(theme);
    text_editor::Style {
        background: Color::TRANSPARENT.into(),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        placeholder: Color::TRANSPARENT,
        value: p.text,
        selection: if p.bg.r > 0.5 {
            primitive::BLACK_015
        } else {
            primitive::WHITE_015
        },
    }
}

pub fn default_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = BaseColors::palette(theme);
    let role = roles::palette(theme);

    let (scroller_bg, scroller_radius, rail_bg) = match status {
        scrollable::Status::Dragged { .. } => (
            role.accent_pressed,
            3.0,
            Color {
                a: 0.08,
                ..p.border
            },
        ),
        scrollable::Status::Hovered { .. } => (
            role.accent_hover,
            3.0,
            Color {
                a: 0.05,
                ..p.border
            },
        ),
        _ => (
            Color {
                a: 0.20,
                ..p.border
            },
            1.5,
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
                radius: 4.0.into(),
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
                radius: 2.0.into(),
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
            radius: 4.0.into(),
        },
        text_color: Some(p.text),
    }
}

pub fn default_tooltip(theme: &Theme) -> container::Style {
    let p = BaseColors::palette(theme);
    let bg = if p.bg.r > 0.5 {
        primitive::LIGHT_TOOLTIP
    } else {
        primitive::DARK_TOOLTIP
    };
    let border_color = if p.bg.r > 0.5 {
        primitive::WHITE_012
    } else {
        p.border
    };
    container::Style {
        background: Some(bg.into()),
        text_color: Some(p.text),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

pub fn default_rule(theme: &Theme) -> rule::Style {
    let p = BaseColors::palette(theme);
    rule::Style {
        color: p.rule,
        radius: 0.0.into(),
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
            radius: 8.0.into(),
        },
    }
}
