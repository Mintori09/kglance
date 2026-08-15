//! Semantic colour tokens for the Markdown view.

use iced::{Color, Theme};

use super::primitive;

#[derive(Clone, Copy)]
pub struct MarkdownColors {
    pub search_active_bg: Color,
    pub search_inactive_bg: Color,
    pub table_header_bg: Color,
    pub table_header_text: Color,
    pub table_separator: Color,
    pub table_border: Color,
    pub quote_accent: Color,
    pub quote_bg: Color,
    pub html_fg: Color,
    pub math: Color,
}

impl MarkdownColors {
    pub const DARK: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::MD_DARK_SEARCH_ACTIVE,
        search_inactive_bg: primitive::MD_DARK_SEARCH_INACTIVE,
        table_header_bg: primitive::MD_DARK_TABLE_HEADER_BG,
        table_header_text: primitive::MD_DARK_TABLE_HEADER_TEXT,
        table_separator: primitive::MD_DARK_TABLE_SEPARATOR,
        table_border: primitive::MD_DARK_TABLE_BORDER,
        quote_accent: primitive::MD_DARK_QUOTE_ACCENT,
        quote_bg: primitive::MD_DARK_QUOTE_BG,
        html_fg: primitive::GRAY_500,
        math: primitive::DARK_TEXT,
    };

    pub const LIGHT: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::MD_LIGHT_SEARCH_ACTIVE,
        search_inactive_bg: primitive::MD_LIGHT_SEARCH_INACTIVE,
        table_header_bg: primitive::MD_LIGHT_TABLE_HEADER_BG,
        table_header_text: primitive::MD_LIGHT_TABLE_HEADER_TEXT,
        table_separator: primitive::MD_LIGHT_TABLE_SEPARATOR,
        table_border: primitive::MD_LIGHT_TABLE_BORDER,
        quote_accent: primitive::MD_LIGHT_QUOTE_ACCENT,
        quote_bg: primitive::MD_LIGHT_QUOTE_BG,
        html_fg: primitive::GRAY_500,
        math: primitive::LIGHT_TEXT,
    };

    pub const NORD: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::NORD13,
        search_inactive_bg: primitive::NORD2,
        table_header_bg: primitive::NORD2,
        table_header_text: primitive::NORD4,
        table_separator: primitive::NORD3,
        table_border: primitive::NORD3,
        quote_accent: primitive::NORD9,
        quote_bg: primitive::NORD1,
        html_fg: primitive::NORD3,
        math: primitive::NORD4,
    };

    pub fn palette(theme: &Theme) -> &'static MarkdownColors {
        match theme {
            Theme::Dark => &Self::DARK,
            _ => &Self::LIGHT,
        }
    }
}
