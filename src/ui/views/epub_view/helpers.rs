use iced::Color;

use crate::core::types::EpubState;
use crate::ui::components::sidebar::INDENT_PER_LEVEL;
use crate::ui::views::epub_view::constants::{
    CHAPTER_TITLE_SIZE_LEVEL_ONE, CHAPTER_TITLE_SIZE_OTHER, MAX_INDENT,
};

pub fn clamp_active_chapter(state: &EpubState) -> usize {
    state
        .active_chapter
        .min(state.chapters.len().saturating_sub(1))
}

pub fn calculate_indent(level: u8) -> f32 {
    ((level.saturating_sub(1)) as f32 * INDENT_PER_LEVEL).min(MAX_INDENT)
}

pub fn entry_font_weight(level: u8) -> iced::font::Weight {
    if level == 1 {
        iced::font::Weight::Bold
    } else {
        iced::font::Weight::Normal
    }
}

pub fn entry_font_size(level: u8) -> f32 {
    if level == 1 {
        CHAPTER_TITLE_SIZE_LEVEL_ONE
    } else {
        CHAPTER_TITLE_SIZE_OTHER
    }
}

pub fn entry_text_color(is_active: bool, is_light_background: bool, level: u8) -> Option<Color> {
    if is_active {
        return None;
    }
    if is_light_background {
        if level == 1 {
            Some(Color::from_rgb(0.2, 0.22, 0.25))
        } else {
            Some(Color::from_rgb(0.4, 0.42, 0.45))
        }
    } else if level == 1 {
        Some(Color::from_rgb(0.9, 0.92, 0.95))
    } else {
        Some(Color::from_rgb(0.75, 0.78, 0.82))
    }
}
