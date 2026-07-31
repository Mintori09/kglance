use iced::Color;

use crate::core::types::EpubState;
use crate::ui::components::sidebar::INDENT_PER_LEVEL;
use crate::ui::theme::color::primitive;
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
            Some(primitive::EPUB_CHAPTER_TEXT_L1_LIGHT)
        } else {
            Some(primitive::EPUB_CHAPTER_TEXT_L2_LIGHT)
        }
    } else if level == 1 {
        Some(primitive::EPUB_CHAPTER_TEXT_L1_DARK)
    } else {
        Some(primitive::EPUB_CHAPTER_TEXT_L2_DARK)
    }
}
