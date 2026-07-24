//! Glassmorphic widget styles for Kglance.
//!
//! Provides a consistent frosted-glass aesthetic across all widget types,
//! adapting automatically to dark and light themes.

use iced::widget::{button, checkbox, container, pick_list, rule, scrollable, slider, text_input};
use iced::{Border, Color, Shadow, Theme};

// ── Accent ────────────────────────────────────────────────────────────────────

/// Vibrant blue accent used for interactive focus and hover states.
pub const ACCENT: Color = Color::from_rgb(0.0, 0.55, 1.0);
pub const ACCENT_HOVER: Color = Color::from_rgb(0.05, 0.62, 1.0);
pub const ACCENT_PRESSED: Color = Color::from_rgb(0.0, 0.42, 0.85);

// ── Dark palette ──────────────────────────────────────────────────────────────

pub const DARK_BG: Color = Color::from_rgba(0.08, 0.09, 0.11, 1.0);
pub const DARK_SURFACE: Color = Color::from_rgba(0.14, 0.16, 0.20, 0.85);
pub const DARK_SURFACE_RAISED: Color = Color::from_rgba(0.18, 0.21, 0.26, 0.90);
pub const DARK_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub const DARK_BORDER_FOCUS: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.16);
pub const DARK_TEXT: Color = Color::from_rgb(0.93, 0.94, 0.96);
pub const DARK_TEXT_DIM: Color = Color::from_rgba(0.93, 0.94, 0.96, 0.50);
pub const DARK_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.40);
pub const DARK_RULE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.07);

// ── Light palette ─────────────────────────────────────────────────────────────

pub const LIGHT_BG: Color = Color::from_rgba(0.93, 0.94, 0.96, 1.0);
pub const LIGHT_SURFACE: Color = Color::from_rgba(0.98, 0.98, 1.0, 0.82);
pub const LIGHT_SURFACE_RAISED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.88);
pub const LIGHT_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.07);
pub const LIGHT_BORDER_FOCUS: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.14);
pub const LIGHT_TEXT: Color = Color::from_rgb(0.12, 0.13, 0.16);
pub const LIGHT_TEXT_DIM: Color = Color::from_rgba(0.12, 0.13, 0.16, 0.50);
pub const LIGHT_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub const LIGHT_RULE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);

// ── Shared shadow builders ────────────────────────────────────────────────────

fn card_shadow(is_dark: bool) -> Shadow {
    Shadow {
        color: if is_dark { DARK_SHADOW } else { LIGHT_SHADOW },
        offset: iced::Vector::new(0.0, 4.0),
        blur_radius: 16.0,
    }
}

fn subtle_shadow(is_dark: bool) -> Shadow {
    Shadow {
        color: if is_dark { DARK_SHADOW } else { LIGHT_SHADOW },
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 6.0,
    }
}

// ── Container ─────────────────────────────────────────────────────────────────

/// Root background container — opaque base layer.
pub fn glass_root(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);
    container::Style {
        background: Some((if is_dark { DARK_BG } else { LIGHT_BG }).into()),
        text_color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Frosted-glass card / panel with subtle border and shadow.
pub fn glass_card(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);
    container::Style {
        background: Some((if is_dark { DARK_SURFACE } else { LIGHT_SURFACE }).into()),
        text_color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
        border: Border {
            color: if is_dark { DARK_BORDER } else { LIGHT_BORDER },
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: card_shadow(is_dark),
        snap: false,
    }
}

/// Slightly elevated surface used for toolbars, headers and sidebars.
pub fn glass_raised(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);
    container::Style {
        background: Some(
            (if is_dark {
                DARK_SURFACE_RAISED
            } else {
                LIGHT_SURFACE_RAISED
            })
            .into(),
        ),
        text_color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
        border: Border {
            color: if is_dark { DARK_BORDER } else { LIGHT_BORDER },
            width: 1.0,
            radius: 10.0.into(),
        },
        shadow: subtle_shadow(is_dark),
        snap: false,
    }
}

/// Inset well — used inside scrollable areas / text panes.
pub fn glass_inset(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);
    container::Style {
        background: Some((if is_dark { DARK_BG } else { LIGHT_BG }).into()),
        text_color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
        border: Border {
            color: if is_dark { DARK_BORDER } else { LIGHT_BORDER },
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
    let is_dark = matches!(theme, Theme::Dark);

    let (bg, border_color, text_color) = if is_dark {
        match status {
            button::Status::Hovered => (ACCENT_HOVER, ACCENT_HOVER, Color::WHITE),
            button::Status::Pressed => (ACCENT_PRESSED, ACCENT_PRESSED, Color::WHITE),
            _ => (DARK_SURFACE, DARK_BORDER, DARK_TEXT),
        }
    } else {
        match status {
            button::Status::Hovered => (ACCENT_HOVER, ACCENT_HOVER, Color::WHITE),
            button::Status::Pressed => (ACCENT_PRESSED, ACCENT_PRESSED, Color::WHITE),
            _ => (LIGHT_SURFACE, LIGHT_BORDER, LIGHT_TEXT),
        }
    };

    button::Style {
        background: Some(bg.into()),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        text_color,
        shadow: subtle_shadow(is_dark),
        snap: false,
    }
}

/// Dynamic list-item style for file rows supporting Finder-like hover and selection.
pub fn glass_row_button(theme: &Theme, status: button::Status, is_selected: bool) -> button::Style {
    let is_dark = matches!(theme, Theme::Dark);
    let text_color = if is_dark { DARK_TEXT } else { LIGHT_TEXT };

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
            let mut c = if is_dark { Color::WHITE } else { Color::BLACK };
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

/// Accent-filled primary action button (always uses accent colour).
pub fn glass_button_primary(theme: &Theme, status: button::Status) -> button::Style {
    let is_dark = matches!(theme, Theme::Dark);

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
        shadow: subtle_shadow(is_dark),
        snap: false,
    }
}

// ── TextInput ─────────────────────────────────────────────────────────────────

/// Frosted-glass text-input field.
pub fn glass_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let is_dark = matches!(theme, Theme::Dark);

    let (bg, border_color) = if is_dark {
        match status {
            text_input::Status::Focused { .. } => (DARK_SURFACE_RAISED, ACCENT),
            text_input::Status::Hovered => (DARK_SURFACE_RAISED, DARK_BORDER_FOCUS),
            _ => (DARK_SURFACE, DARK_BORDER),
        }
    } else {
        match status {
            text_input::Status::Focused { .. } => (LIGHT_SURFACE_RAISED, ACCENT),
            text_input::Status::Hovered => (LIGHT_SURFACE_RAISED, LIGHT_BORDER_FOCUS),
            _ => (LIGHT_SURFACE, LIGHT_BORDER),
        }
    };

    text_input::Style {
        background: bg.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        value: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
        placeholder: if is_dark {
            DARK_TEXT_DIM
        } else {
            LIGHT_TEXT_DIM
        },
        selection: ACCENT,
        icon: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
    }
}

// ── Scrollable ────────────────────────────────────────────────────────────────

/// Minimal glass scrollbar — subtle rail/scroller, expanding and highlighting on hover/drag.
pub fn glass_scrollable(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let is_dark = matches!(theme, Theme::Dark);

    let (scroller_bg, scroller_radius, rail_bg) = match status {
        scrollable::Status::Dragged { .. } => (
            ACCENT_PRESSED,
            3.0,
            if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.08)
            },
        ),
        scrollable::Status::Hovered { .. } => (
            ACCENT_HOVER,
            3.0,
            if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.05)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.05)
            },
        ),
        _ => (
            if is_dark {
                Color::from_rgba(1.0, 1.0, 1.0, 0.20)
            } else {
                Color::from_rgba(0.0, 0.0, 0.0, 0.20)
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
            background: (if is_dark { DARK_SURFACE } else { LIGHT_SURFACE }).into(),
            border: Border {
                radius: 4.0.into(),
                ..Border::default()
            },
            shadow: Shadow::default(),
            icon: if is_dark {
                DARK_TEXT_DIM
            } else {
                LIGHT_TEXT_DIM
            },
        },
    }
}

// ── Slider ────────────────────────────────────────────────────────────────────

/// Glass-style range slider with accent track fill.
pub fn glass_slider(theme: &Theme, status: slider::Status) -> slider::Style {
    let is_dark = matches!(theme, Theme::Dark);

    let handle_color = match status {
        slider::Status::Hovered | slider::Status::Dragged => ACCENT_HOVER,
        _ => ACCENT,
    };

    let rail_color = if is_dark {
        Color::from_rgba(1.0, 1.0, 1.0, 0.15)
    } else {
        Color::from_rgba(0.0, 0.0, 0.0, 0.15)
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
    let is_dark = matches!(theme, Theme::Dark);

    let (bg, border_color, icon_color) = match status {
        checkbox::Status::Active { is_checked: true }
        | checkbox::Status::Disabled { is_checked: true } => (ACCENT, ACCENT, Color::WHITE),
        checkbox::Status::Hovered { is_checked: true } => {
            (ACCENT_HOVER, ACCENT_HOVER, Color::WHITE)
        }
        checkbox::Status::Hovered { is_checked: false } => {
            if is_dark {
                (DARK_SURFACE_RAISED, DARK_BORDER_FOCUS, DARK_TEXT)
            } else {
                (LIGHT_SURFACE_RAISED, LIGHT_BORDER_FOCUS, LIGHT_TEXT)
            }
        }
        _ => {
            if is_dark {
                (DARK_SURFACE, DARK_BORDER, DARK_TEXT)
            } else {
                (LIGHT_SURFACE, LIGHT_BORDER, LIGHT_TEXT)
            }
        }
    };

    checkbox::Style {
        background: bg.into(),
        icon_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(if is_dark { DARK_TEXT } else { LIGHT_TEXT }),
    }
}

// ── Rule ──────────────────────────────────────────────────────────────────────

/// Tooltip popup — dark surface with rounded corners and subtle shadow.
pub fn glass_tooltip(theme: &Theme) -> container::Style {
    let is_dark = matches!(theme, Theme::Dark);
    container::Style {
        background: Some(
            (if is_dark {
                Color::from_rgba(0.12, 0.14, 0.18, 0.95)
            } else {
                Color::from_rgba(0.22, 0.24, 0.28, 0.95)
            })
            .into(),
        ),
        text_color: Some(if is_dark { DARK_TEXT } else { Color::WHITE }),
        border: Border {
            color: if is_dark {
                DARK_BORDER
            } else {
                Color::from_rgba(1.0, 1.0, 1.0, 0.12)
            },
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: subtle_shadow(is_dark),
        snap: false,
    }
}

/// Translucent hairline separator.
pub fn glass_rule(theme: &Theme) -> rule::Style {
    let is_dark = matches!(theme, Theme::Dark);
    rule::Style {
        color: if is_dark { DARK_RULE } else { LIGHT_RULE },
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: false,
    }
}

// ── PickList ──────────────────────────────────────────────────────────────────

/// Glass-style drop-down selector.
pub fn glass_pick_list(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let is_dark = matches!(theme, Theme::Dark);

    let (bg, border_color) = match status {
        pick_list::Status::Opened { .. } => (
            if is_dark {
                DARK_SURFACE_RAISED
            } else {
                LIGHT_SURFACE_RAISED
            },
            ACCENT,
        ),
        pick_list::Status::Hovered => (
            if is_dark {
                DARK_SURFACE_RAISED
            } else {
                LIGHT_SURFACE_RAISED
            },
            if is_dark {
                DARK_BORDER_FOCUS
            } else {
                LIGHT_BORDER_FOCUS
            },
        ),
        _ => (
            if is_dark { DARK_SURFACE } else { LIGHT_SURFACE },
            if is_dark { DARK_BORDER } else { LIGHT_BORDER },
        ),
    };

    pick_list::Style {
        text_color: if is_dark { DARK_TEXT } else { LIGHT_TEXT },
        placeholder_color: if is_dark {
            DARK_TEXT_DIM
        } else {
            LIGHT_TEXT_DIM
        },
        handle_color: if is_dark {
            DARK_TEXT_DIM
        } else {
            LIGHT_TEXT_DIM
        },
        background: bg.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
    }
}
