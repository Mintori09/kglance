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
    pub inline_code_fg: Color,
    pub inline_code_bg: Color,
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
        inline_code_fg: primitive::MD_DARK_INLINE_CODE,
        inline_code_bg: primitive::MD_DARK_INLINE_CODE_BG,
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
        inline_code_fg: primitive::MD_LIGHT_INLINE_CODE,
        inline_code_bg: primitive::MD_LIGHT_INLINE_CODE_BG,
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
        quote_accent: primitive::NORD8,
        quote_bg: primitive::NORD1,
        inline_code_fg: primitive::NORD8,
        inline_code_bg: primitive::WHITE_006,
        html_fg: primitive::NORD_TEXT_DIM,
        math: primitive::NORD4,
    };

    pub const CATPPUCCIN_MOCHA: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::MOCHA_YELLOW,
        search_inactive_bg: primitive::MOCHA_SURFACE1,
        table_header_bg: primitive::MOCHA_SURFACE1,
        table_header_text: primitive::MOCHA_TEXT,
        table_separator: primitive::MOCHA_SURFACE1,
        table_border: primitive::MOCHA_SURFACE1,
        quote_accent: primitive::MOCHA_BLUE,
        quote_bg: primitive::MOCHA_SURFACE0,
        inline_code_fg: primitive::MOCHA_PEACH,
        inline_code_bg: primitive::MOCHA_SURFACE0,
        html_fg: primitive::MOCHA_SUBTEXT0,
        math: primitive::MOCHA_TEXT,
    };

    pub const CATPPUCCIN_LATTE: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::LATTE_YELLOW,
        search_inactive_bg: primitive::LATTE_SURFACE1,
        table_header_bg: primitive::LATTE_SURFACE1,
        table_header_text: primitive::LATTE_TEXT,
        table_separator: primitive::LATTE_SURFACE1,
        table_border: primitive::LATTE_SURFACE1,
        quote_accent: primitive::LATTE_BLUE,
        quote_bg: primitive::LATTE_SURFACE0,
        inline_code_fg: primitive::LATTE_BLUE,
        inline_code_bg: primitive::LATTE_SURFACE0,
        html_fg: primitive::LATTE_SUBTEXT0,
        math: primitive::LATTE_TEXT,
    };

    pub const TOKYO_NIGHT: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::TOKYO_YELLOW,
        search_inactive_bg: primitive::TOKYO_SURFACE,
        table_header_bg: primitive::TOKYO_SURFACE_RAISED,
        table_header_text: primitive::TOKYO_TEXT,
        table_separator: primitive::TOKYO_BORDER,
        table_border: primitive::TOKYO_BORDER,
        quote_accent: primitive::TOKYO_BLUE,
        quote_bg: primitive::TOKYO_SURFACE,
        inline_code_fg: primitive::TOKYO_CYAN,
        inline_code_bg: primitive::TOKYO_SURFACE,
        html_fg: primitive::TOKYO_TEXT_DIM,
        math: primitive::TOKYO_TEXT,
    };

    pub const GRUVBOX_DARK: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::GRUVBOX_YELLOW,
        search_inactive_bg: primitive::GRUVBOX_SURFACE,
        table_header_bg: primitive::GRUVBOX_SURFACE_RAISED,
        table_header_text: primitive::GRUVBOX_TEXT,
        table_separator: primitive::GRUVBOX_BORDER,
        table_border: primitive::GRUVBOX_BORDER,
        quote_accent: primitive::GRUVBOX_ORANGE,
        quote_bg: primitive::GRUVBOX_SURFACE,
        inline_code_fg: primitive::GRUVBOX_YELLOW,
        inline_code_bg: primitive::GRUVBOX_SURFACE,
        html_fg: primitive::GRUVBOX_TEXT_DIM,
        math: primitive::GRUVBOX_TEXT,
    };

    pub const DRACULA: MarkdownColors = MarkdownColors {
        search_active_bg: primitive::DRACULA_YELLOW,
        search_inactive_bg: primitive::DRACULA_SURFACE,
        table_header_bg: primitive::DRACULA_SURFACE_RAISED,
        table_header_text: primitive::DRACULA_TEXT,
        table_separator: primitive::DRACULA_BORDER,
        table_border: primitive::DRACULA_BORDER,
        quote_accent: primitive::DRACULA_PURPLE,
        quote_bg: primitive::DRACULA_SURFACE,
        inline_code_fg: primitive::DRACULA_PINK,
        inline_code_bg: primitive::DRACULA_SURFACE,
        html_fg: primitive::DRACULA_TEXT_DIM,
        math: primitive::DRACULA_TEXT,
    };

    pub fn palette(theme: &Theme) -> &'static MarkdownColors {
        let app_theme = crate::ui::theme::AppTheme::from(theme);
        &app_theme.palette().markdown
    }
}
