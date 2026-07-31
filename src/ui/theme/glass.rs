//! Glassmorphic widget styles for Kglance.
//!
//! Provides a consistent frosted-glass aesthetic across all widget types,
//! adapting automatically to dark and light themes.

use iced::widget::{
    button, checkbox, container, pick_list, rule, scrollable, slider, text_editor, text_input,
};
use iced::{Border, Color, Shadow, Theme};

#[derive(Clone, Copy)]
pub struct ThemePalette {
    pub bg: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub border: Color,
    pub border_focus: Color,
    pub text: Color,
    pub text_dim: Color,
    pub shadow: Color,
    pub rule: Color,
}

pub const DARK_PALETTE: ThemePalette = ThemePalette {
    bg: Color::from_rgba(0.08, 0.09, 0.11, 1.0),
    surface: Color::from_rgba(0.14, 0.16, 0.20, 0.85),
    surface_raised: Color::from_rgba(0.18, 0.21, 0.26, 0.90),
    border: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
    border_focus: Color::from_rgba(1.0, 1.0, 1.0, 0.16),
    text: Color::from_rgb(0.93, 0.94, 0.96),
    text_dim: Color::from_rgba(0.93, 0.94, 0.96, 0.50),
    shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.40),
    rule: Color::from_rgba(1.0, 1.0, 1.0, 0.07),
};

pub const LIGHT_PALETTE: ThemePalette = ThemePalette {
    bg: Color::from_rgba(0.93, 0.94, 0.96, 1.0),
    surface: Color::from_rgba(0.98, 0.98, 1.0, 0.82),
    surface_raised: Color::from_rgba(1.0, 1.0, 1.0, 0.88),
    border: Color::from_rgba(0.0, 0.0, 0.0, 0.07),
    border_focus: Color::from_rgba(0.0, 0.0, 0.0, 0.14),
    text: Color::from_rgb(0.12, 0.13, 0.16),
    text_dim: Color::from_rgba(0.12, 0.13, 0.16, 0.50),
    shadow: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
    rule: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
};

pub fn palette(theme: &Theme) -> &'static ThemePalette {
    match theme {
        Theme::Dark => &DARK_PALETTE,
        _ => &LIGHT_PALETTE,
    }
}

pub fn palette_for(is_dark: bool) -> &'static ThemePalette {
    if is_dark {
        &DARK_PALETTE
    } else {
        &LIGHT_PALETTE
    }
}

// ── Accent ────────────────────────────────────────────────────────────────────

pub const ACCENT: Color = Color::from_rgb(0.0, 0.55, 1.0);
pub const ACCENT_HOVER: Color = Color::from_rgb(0.05, 0.62, 1.0);
pub const ACCENT_PRESSED: Color = Color::from_rgb(0.0, 0.42, 0.85);

// ── Shadow builders ───────────────────────────────────────────────────────────

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

// ── Container ─────────────────────────────────────────────────────────────────

/// Root background container — opaque base layer.
pub fn glass_root(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Frosted-glass card / panel with subtle border and shadow.
pub fn glass_card(theme: &Theme) -> container::Style {
    let p = palette(theme);
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

/// Slightly elevated surface used for toolbars, headers and sidebars.
pub fn glass_raised(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(p.surface_raised.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: subtle_shadow(p.shadow),
        snap: false,
    }
}

/// Inset well — used inside scrollable areas / text panes.
pub fn glass_inset(theme: &Theme) -> container::Style {
    let p = palette(theme);
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

// ── Button ────────────────────────────────────────────────────────────────────

/// Standard ghost-glass button.
pub fn glass_button(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);

    let (bg, border_color, text_color) = match status {
        button::Status::Hovered => (ACCENT_HOVER, ACCENT_HOVER, Color::WHITE),
        button::Status::Pressed => (ACCENT_PRESSED, ACCENT_PRESSED, Color::WHITE),
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

/// Dynamic list-item style for file rows supporting Finder-like hover and selection.
pub fn glass_row_button(theme: &Theme, status: button::Status, is_selected: bool) -> button::Style {
    let p = palette(theme);
    let text_color = p.text;

    let bg_color = match (is_selected, status) {
        (true, button::Status::Hovered) => {
            let mut c = ACCENT;
            c.a = 0.20;
            Some(c.into())
        }
        (true, _) => {
            let mut c = ACCENT;
            c.a = 0.15;
            Some(c.into())
        }
        (false, button::Status::Hovered) => {
            let mut c = if p.bg.r > 0.5 {
                Color::BLACK
            } else {
                Color::WHITE
            };
            c.a = 0.06;
            Some(c.into())
        }
        (false, _) => None,
    };

    let border = if is_selected {
        let mut bc = ACCENT;
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

/// Dynamic card style for grid item buttons supporting glassmorphism hover, border, shadow, and selection.
pub fn glass_grid_card(theme: &Theme, status: button::Status, is_selected: bool) -> button::Style {
    let p = palette(theme);
    let is_dark = matches!(theme, Theme::Dark);

    let (bg_color, border_color, border_width, shadow) = if is_selected {
        let mut active_bg = ACCENT;
        active_bg.a = if is_dark { 0.25 } else { 0.20 };

        let active_shadow = Shadow {
            color: Color { a: 0.4, ..ACCENT },
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        };
        (active_bg, ACCENT, 2.0, active_shadow)
    } else {
        match status {
            button::Status::Hovered => {
                let hover_bg = if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.12)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                };
                let hover_border = if is_dark {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.2)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
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

/// Accent-filled primary action button (always uses accent colour).
pub fn glass_button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let bg = match status {
        button::Status::Hovered => ACCENT_HOVER,
        button::Status::Pressed => ACCENT_PRESSED,
        _ => ACCENT,
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

// ── TextInput ─────────────────────────────────────────────────────────────────

/// Frosted-glass text-input field.
pub fn glass_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);

    let (bg, border_color) = match status {
        text_input::Status::Focused { .. } => (p.surface_raised, ACCENT),
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
        selection: ACCENT,
        icon: p.text,
    }
}

// ── TextEditor ────────────────────────────────────────────────────────────────

/// Transparent text-editor style for code display (no border, translucent selection).
pub fn glass_text_editor(theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    let p = palette(theme);
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
            Color::from_rgba(0.0, 0.0, 0.0, 0.15)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.15)
        },
    }
}

// ── Scrollable ────────────────────────────────────────────────────────────────

/// Minimal glass scrollbar — subtle rail/scroller, expanding and highlighting on hover/drag.
pub fn glass_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let p = palette(theme);

    let (scroller_bg, scroller_radius, rail_bg) = match status {
        scrollable::Status::Dragged { .. } => (
            ACCENT_PRESSED,
            3.0,
            Color {
                a: 0.08,
                ..p.border
            },
        ),
        scrollable::Status::Hovered { .. } => (
            ACCENT_HOVER,
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

// ── Slider ────────────────────────────────────────────────────────────────────

/// Glass-style range slider with accent track fill.
pub fn glass_slider(theme: &Theme, status: slider::Status) -> slider::Style {
    let p = palette(theme);

    let handle_color = match status {
        slider::Status::Hovered | slider::Status::Dragged => ACCENT_HOVER,
        _ => ACCENT,
    };

    let rail_color = Color {
        a: 0.15,
        ..p.border
    };

    slider::Style {
        rail: slider::Rail {
            backgrounds: (ACCENT.into(), rail_color.into()),
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

// ── Checkbox ──────────────────────────────────────────────────────────────────

/// Frosted-glass checkbox with accent check mark.
pub fn glass_checkbox(theme: &Theme, status: checkbox::Status) -> checkbox::Style {
    let p = palette(theme);

    let (bg, border_color, icon_color) = match status {
        checkbox::Status::Active { is_checked: true }
        | checkbox::Status::Disabled { is_checked: true } => (ACCENT, ACCENT, Color::WHITE),
        checkbox::Status::Hovered { is_checked: true } => {
            (ACCENT_HOVER, ACCENT_HOVER, Color::WHITE)
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

// ── Rule ──────────────────────────────────────────────────────────────────────

/// Tooltip popup — dark surface with rounded corners and subtle shadow.
pub fn glass_tooltip(theme: &Theme) -> container::Style {
    let p = palette(theme);
    let bg = if p.bg.r > 0.5 {
        Color::from_rgba(0.22, 0.24, 0.28, 0.95)
    } else {
        Color::from_rgba(0.12, 0.14, 0.18, 0.95)
    };
    let border_color = if p.bg.r > 0.5 {
        Color::from_rgba(1.0, 1.0, 1.0, 0.12)
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

/// Translucent hairline separator.
pub fn glass_rule(theme: &Theme) -> rule::Style {
    let p = palette(theme);
    rule::Style {
        color: p.rule,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

// ── PickList ──────────────────────────────────────────────────────────────────

/// Glass-style drop-down selector.
pub fn glass_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let p = palette(theme);

    let (bg, border_color) = match status {
        pick_list::Status::Opened { .. } => (p.surface_raised, ACCENT),
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
