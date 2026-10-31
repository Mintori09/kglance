use iced::Color;

use crate::core::types::EpubState;
use crate::features::epub::view::constants::{
    CHAPTER_TITLE_SIZE_LEVEL_ONE, CHAPTER_TITLE_SIZE_OTHER, MAX_INDENT,
};
use crate::ui::theme::color::primitive;

pub fn clamp_active_chapter(state: &EpubState) -> usize {
    state
        .active_chapter
        .min(state.chapters.len().saturating_sub(1))
}

pub const EPUB_INDENT_PER_LEVEL: f32 = 16.0;

pub fn calculate_indent(level: u8) -> f32 {
    ((level.saturating_sub(1)) as f32 * EPUB_INDENT_PER_LEVEL).min(MAX_INDENT)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryVisualState {
    Active,
    InactiveLight,
    InactiveDark,
}

impl EntryVisualState {
    pub fn new(is_active: bool, is_light_background: bool) -> Self {
        if is_active {
            Self::Active
        } else if is_light_background {
            Self::InactiveLight
        } else {
            Self::InactiveDark
        }
    }
}

pub fn entry_text_color(state: EntryVisualState, level: u8) -> Option<Color> {
    match state {
        EntryVisualState::Active => None,
        EntryVisualState::InactiveLight => {
            if level == 1 {
                Some(primitive::EPUB_CHAPTER_TEXT_L1_LIGHT)
            } else {
                Some(primitive::EPUB_CHAPTER_TEXT_L2_LIGHT)
            }
        }
        EntryVisualState::InactiveDark => {
            if level == 1 {
                Some(primitive::EPUB_CHAPTER_TEXT_L1_DARK)
            } else {
                Some(primitive::EPUB_CHAPTER_TEXT_L2_DARK)
            }
        }
    }
}
