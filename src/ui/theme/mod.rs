pub mod color;
pub mod default;
pub mod font;
pub mod icon_theme;
pub mod tokens;

const BASE_FONT_SIZE: f32 = 14.0;
const MIN_SCALED_SIZE: f32 = 8.0;

pub(crate) fn scale_size(design_size: f32, user_font_size: f32) -> f32 {
    (design_size * user_font_size / BASE_FONT_SIZE)
        .round()
        .max(MIN_SCALED_SIZE)
}

pub use color::{AppTheme, ColorPalette, DARK_PALETTE, LIGHT_PALETTE};
pub use default::{
    default_button, default_button_primary, default_card, default_checkbox, default_grid_card,
    default_inset, default_pick_list, default_raised, default_root, default_row_button,
    default_rule, default_scrollable, default_slider, default_text_editor, default_text_input,
    default_tooltip,
};
