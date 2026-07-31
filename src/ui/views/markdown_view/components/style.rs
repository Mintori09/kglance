use crate::ui::theme::glass;
use crate::ui::theme::tokens::{border, radius, spacing, tables, typography};
use iced::widget::{button, container};
use iced::{Border, Color, Shadow, Theme};

pub(crate) const STYLE: MarkdownStyle = MarkdownStyle {
    general: GeneralStyle {
        content_padding: spacing::L,
        section_spacing: spacing::XS,
        item_spacing_small: spacing::XXS,
        divider_height: border::THIN,
        button_border_radius: radius::SMALL,
    },
    paragraph: ParagraphStyle { padding: [2, 0] },
    inline: InlineStyle {
        inline_code_color: Color::from_rgb(0.8, 0.35, 0.35),
        image_alt_color: Color::from_rgb(0.5, 0.5, 0.5),
        math_color: Color::from_rgb(0.5, 0.2, 0.7),
        link_button_border_width: 0.0,
        wrap_spacing: spacing::XS,
        wrap_line_spacing: spacing::XS,
        tooltip_gap: spacing::S,
        tooltip_font_size: typography::SIZE_SMALL,
        tooltip_padding: [4, 8],
    },
    code: CodeStyle {
        border_radius: radius::SMALL,
        padding: 10,
        line_font_size: 13.0,
        label_button_font_size: 11.0,
        top_bar_padding: [2, 8],
        button_padding: [2, 8],
    },
    table: TableStyle {
        border_radius: radius::MEDIUM,
        header_padding: [8, 12],
        cell_padding: [8, 12],
        header_font_size: tables::FONT_SIZE_HEADER,
        cell_font_size: tables::FONT_SIZE_BODY,
        min_column_weight: 10.0,
    },
    list: ListStyle {
        bullet_color: Color::from_rgb(0.5, 0.5, 0.5),
        item_spacing: spacing::S,
        item_padding: spacing::XXS,
        sub_block_left_padding: spacing::XL,
    },
    quote: QuoteStyle {
        bar_width: spacing::XS,
        content_padding: [8, 12],
    },
    image: ImageStyle {
        max_width: 600.0,
        padding: [4, 0],
    },
    mermaid: MermaidStyle {
        image_padding: 14,
        badge_font_size: 11.0,
        badge_padding: [4, 10],
    },
    html: HtmlStyle {
        font_size: typography::SIZE_SMALL,
        preview_truncate: 80,
    },
    hr: HrStyle { padding: [8, 0] },
    block: BlockMarginStyle {
        heading_h1: spacing::XL,
        heading_h2: spacing::L,
        heading_default: spacing::L,
        horizontal_rule: spacing::XL,
        code: spacing::L,
        table: spacing::L,
        quote: spacing::L,
        image: spacing::L,
        mermaid: spacing::L,
        list: spacing::M,
        paragraph: spacing::S,
        html: spacing::S,
    },
    toc: TocStyle {
        scroll_offset_margin: 50.0,
        indent_per_level: crate::ui::components::sidebar::INDENT_PER_LEVEL,
        entry_font_size: crate::ui::components::sidebar::SIDEBAR_ENTRY_FONT_SIZE,
        entry_padding: [
            crate::ui::components::sidebar::SIDEBAR_ENTRY_PADDING_V as u16,
            4,
        ],
        container_padding: spacing::S,
        item_spacing: crate::ui::components::sidebar::SIDEBAR_ITEM_SPACING,
        chevron_placeholder_width: 15.0,
        sidebar_border_width: crate::ui::components::sidebar::SIDEBAR_BORDER_WIDTH,
    },
};

#[derive(Clone, Copy)]
pub(crate) struct MarkdownPalette {
    pub link: Color,
    pub search_active_bg: Color,
    pub search_inactive_bg: Color,
    pub table_header_bg: Color,
    pub table_header_text: Color,
    pub table_separator: Color,
    pub table_border: Color,
    pub task_checked: Color,
    pub quote_accent: Color,
    pub quote_bg: Color,
    pub html_fg: Color,
}

pub(crate) const DARK_MD_PALETTE: MarkdownPalette = MarkdownPalette {
    link: Color::from_rgb(0.4, 0.6, 1.0),
    search_active_bg: Color::from_rgb(0.7, 0.6, 0.15),
    search_inactive_bg: Color::from_rgb(0.5, 0.4, 0.1),
    table_header_bg: Color::from_rgb(0.2, 0.22, 0.25),
    table_header_text: Color::from_rgb(0.95, 0.95, 0.95),
    table_separator: Color::from_rgba(0.45, 0.47, 0.5, 0.6),
    table_border: Color::from_rgb(0.45, 0.47, 0.5),
    task_checked: Color::from_rgb(0.4, 0.8, 0.4),
    quote_accent: Color::from_rgb(0.45, 0.5, 0.65),
    quote_bg: Color::from_rgba(1.0, 1.0, 1.0, 0.03),
    html_fg: Color::from_rgb(0.5, 0.5, 0.5),
};

pub(crate) const LIGHT_MD_PALETTE: MarkdownPalette = MarkdownPalette {
    link: Color::from_rgb(0.3, 0.5, 0.9),
    search_active_bg: Color::from_rgb(1.0, 0.85, 0.3),
    search_inactive_bg: Color::from_rgb(0.95, 0.8, 0.2),
    table_header_bg: Color::from_rgb(0.9, 0.91, 0.93),
    table_header_text: Color::from_rgb(0.2, 0.2, 0.2),
    table_separator: Color::from_rgba(0.6, 0.62, 0.65, 0.5),
    table_border: Color::from_rgb(0.6, 0.62, 0.65),
    task_checked: Color::from_rgb(0.1, 0.6, 0.2),
    quote_accent: Color::from_rgb(0.6, 0.5, 0.8),
    quote_bg: Color::from_rgba(0.0, 0.0, 0.0, 0.02),
    html_fg: Color::from_rgb(0.5, 0.5, 0.5),
};

pub(crate) fn md_palette(theme: &Theme) -> &'static MarkdownPalette {
    if matches!(theme, Theme::Dark) {
        &DARK_MD_PALETTE
    } else {
        &LIGHT_MD_PALETTE
    }
}

pub(crate) fn md_palette_for(is_dark: bool) -> &'static MarkdownPalette {
    if is_dark {
        &DARK_MD_PALETTE
    } else {
        &LIGHT_MD_PALETTE
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MarkdownStyle {
    pub general: GeneralStyle,
    pub paragraph: ParagraphStyle,
    pub inline: InlineStyle,
    pub code: CodeStyle,
    pub table: TableStyle,
    pub list: ListStyle,
    pub quote: QuoteStyle,
    pub image: ImageStyle,
    pub mermaid: MermaidStyle,
    pub html: HtmlStyle,
    pub hr: HrStyle,
    pub block: BlockMarginStyle,
    pub toc: TocStyle,
}

#[derive(Clone, Copy)]
pub(crate) struct GeneralStyle {
    pub content_padding: f32,
    pub section_spacing: f32,
    pub item_spacing_small: f32,
    pub divider_height: f32,
    pub button_border_radius: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct ParagraphStyle {
    pub padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct InlineStyle {
    pub inline_code_color: Color,
    pub image_alt_color: Color,
    pub math_color: Color,
    pub link_button_border_width: f32,
    pub wrap_spacing: f32,
    pub wrap_line_spacing: f32,
    pub tooltip_gap: f32,
    pub tooltip_font_size: f32,
    pub tooltip_padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct CodeStyle {
    pub border_radius: f32,
    pub padding: u16,
    pub line_font_size: f32,
    pub label_button_font_size: f32,
    pub top_bar_padding: [u16; 2],
    pub button_padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct TableStyle {
    pub border_radius: f32,
    pub header_padding: [u16; 2],
    pub cell_padding: [u16; 2],
    pub header_font_size: f32,
    pub cell_font_size: f32,
    pub min_column_weight: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct ListStyle {
    pub bullet_color: Color,
    pub item_spacing: f32,
    pub item_padding: f32,
    pub sub_block_left_padding: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct QuoteStyle {
    pub bar_width: f32,
    pub content_padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct ImageStyle {
    pub max_width: f32,
    pub padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct MermaidStyle {
    pub image_padding: u16,
    pub badge_font_size: f32,
    pub badge_padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct HtmlStyle {
    pub font_size: f32,
    pub preview_truncate: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct HrStyle {
    pub padding: [u16; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct BlockMarginStyle {
    pub heading_h1: f32,
    pub heading_h2: f32,
    pub heading_default: f32,
    pub horizontal_rule: f32,
    pub code: f32,
    pub table: f32,
    pub quote: f32,
    pub image: f32,
    pub mermaid: f32,
    pub list: f32,
    pub paragraph: f32,
    pub html: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct TocStyle {
    pub scroll_offset_margin: f32,
    pub indent_per_level: f32,
    pub entry_font_size: f32,
    pub entry_padding: [u16; 2],
    pub container_padding: f32,
    pub item_spacing: f32,
    pub chevron_placeholder_width: f32,
    pub sidebar_border_width: f32,
}

pub(super) fn heading_layout(level: u8) -> (f32, f32, f32) {
    match level {
        1 => (32.0, 24.0, 12.0),
        2 => (24.0, 20.0, 8.0),
        3 => (20.0, 12.0, 4.0),
        _ => (16.0, 8.0, 4.0),
    }
}

pub(super) fn code_block_style(theme: &Theme) -> container::Style {
    let p = glass::palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: STYLE.code.border_radius.into(),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub(super) fn copy_button_style(theme: &Theme, status: button::Status) -> button::Style {
    let p = glass::palette(theme);
    let bg = p.bg;
    button::Style {
        background: Some(match status {
            button::Status::Hovered | button::Status::Pressed => {
                iced::Background::Color(Color { a: 0.3, ..bg })
            }
            _ => iced::Background::Color(Color { a: 0.0, ..bg }),
        }),
        text_color: match status {
            button::Status::Hovered => p.text,
            _ => p.text_dim,
        },
        border: Border {
            radius: STYLE.general.button_border_radius.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

pub(super) fn language_label_style(theme: &Theme) -> container::Style {
    let p = glass::palette(theme);
    container::Style {
        background: Some(p.bg.into()),
        text_color: Some(p.text_dim),
        ..Default::default()
    }
}

pub(super) fn divider_line_style(theme: &Theme) -> container::Style {
    let p = glass::palette(theme);
    container::Style {
        background: Some(p.border.into()),
        ..Default::default()
    }
}

pub(super) fn table_header_style(theme: &Theme) -> container::Style {
    let mp = md_palette(theme);
    container::Style {
        background: Some(mp.table_header_bg.into()),
        text_color: Some(mp.table_header_text),
        ..Default::default()
    }
}

pub(super) fn table_separator_style(theme: &Theme) -> container::Style {
    let mp = md_palette(theme);
    container::Style {
        background: Some(mp.table_separator.into()),
        ..Default::default()
    }
}

pub(super) fn table_row_background_style(theme: &Theme, row_index: usize) -> container::Style {
    let p = glass::palette(theme);
    let bg = if row_index.is_multiple_of(2) {
        p.surface
    } else {
        p.bg
    };
    container::Style {
        background: Some(bg.into()),
        ..Default::default()
    }
}

pub(super) fn table_border_style(theme: &Theme) -> container::Style {
    let mp = md_palette(theme);
    container::Style {
        border: Border {
            color: mp.table_border,
            width: 1.0,
            radius: STYLE.table.border_radius.into(),
        },
        ..Default::default()
    }
}

pub(super) fn mermaid_badge_style(theme: &Theme) -> container::Style {
    let p = glass::palette(theme);
    container::Style {
        background: Some(p.surface.into()),
        text_color: Some(p.text_dim),
        border: Border {
            color: p.border,
            width: 1.0,
            radius: STYLE.code.border_radius.into(),
        },
        ..Default::default()
    }
}
