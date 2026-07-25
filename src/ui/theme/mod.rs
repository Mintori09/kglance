pub mod glass;
pub mod icon_theme;

pub use glass::{
    DARK_BG, DARK_BORDER, DARK_BORDER_FOCUS, DARK_RULE, DARK_SHADOW, DARK_SURFACE,
    DARK_SURFACE_RAISED, DARK_TEXT, DARK_TEXT_DIM, LIGHT_BG, LIGHT_BORDER, LIGHT_BORDER_FOCUS,
    LIGHT_RULE, LIGHT_SHADOW, LIGHT_SURFACE, LIGHT_SURFACE_RAISED, LIGHT_TEXT, LIGHT_TEXT_DIM,
};
pub use glass::{
    glass_button, glass_button_primary, glass_card, glass_checkbox, glass_inset, glass_pick_list,
    glass_raised, glass_root, glass_row_button, glass_rule, glass_scrollable, glass_slider,
    glass_text_input,
};

// ── Backward-compat aliases (kept so existing call-sites compile without change) ──

use iced::Theme;
use iced::widget::{button, container, text_input};

/// Header / toolbar panel — maps to `glass_raised`.
pub fn breeze_header_container(theme: &Theme) -> container::Style {
    glass_raised(theme)
}

/// Root background container — maps to `glass_root`.
pub fn breeze_container(theme: &Theme) -> container::Style {
    glass_root(theme)
}

/// Standard button — maps to `glass_button`.
pub fn breeze_button(theme: &Theme, status: button::Status) -> button::Style {
    glass_button(theme, status)
}

/// Text input — maps to `glass_text_input`.
pub fn breeze_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    glass_text_input(theme, status)
}
