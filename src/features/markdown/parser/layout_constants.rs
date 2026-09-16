//! Layout constants for block height estimation.
//!
//! These values mirror the visual style constants defined in
//! `features::markdown::view::components::style` but live here so the parser
//! layer can use them without importing anything from the UI layer.
//!
//! Keep in sync with `MarkdownStyle` / `STYLE` whenever visual constants change.

// General
pub const CONTENT_PADDING: f32 = 24.0; // spacing::XL
pub const DIVIDER_HEIGHT: f32 = 1.0; // border::THIN
pub const SECTION_SPACING: f32 = 6.0;

// Paragraph
pub const PARAGRAPH_PADDING_V: u16 = 3; // [3, 0][0]

// Code block
pub const CODE_LABEL_BUTTON_FONT_SIZE: f32 = 11.0;
pub const CODE_LINE_FONT_SIZE: f32 = 13.0;
pub const CODE_BUTTON_PADDING_V: u16 = 2; // [2, 8][0]
pub const CODE_TOP_BAR_PADDING_V: u16 = 2; // [2, 8][0]
pub const CODE_PADDING: u16 = 10;

// Quote
pub const QUOTE_CONTENT_PADDING_V: u16 = 8; // [8, 12][0]

// Image
pub const IMAGE_MAX_WIDTH: f32 = 600.0;
pub const IMAGE_PADDING_V: u16 = 4; // [4, 0][0]

// HR
pub const HR_PADDING_V: u16 = 8; // [8, 0][0]

// Math
pub const MATH_FONT_SCALE: f32 = 1.0;
pub const MATH_PADDING: u16 = 8;

// List
pub const LIST_ITEM_PADDING: f32 = 3.0;
pub const LIST_SUB_BLOCK_LEFT_PADDING: f32 = 24.0; // spacing::XL

// HTML
pub const HTML_FONT_SIZE: f32 = 12.0;

// Block margins
pub const MARGIN_HEADING_H1: f32 = 24.0; // spacing::XL
pub const MARGIN_HEADING_H2: f32 = 18.0;
pub const MARGIN_HEADING_DEFAULT: f32 = 16.0; // spacing::L
pub const MARGIN_HORIZONTAL_RULE: f32 = 24.0; // spacing::XL
pub const MARGIN_CODE: f32 = 18.0;
pub const MARGIN_TABLE: f32 = 18.0;
pub const MARGIN_QUOTE: f32 = 18.0;
pub const MARGIN_ALERT: f32 = 18.0;
pub const MARGIN_FOOTNOTE: f32 = 12.0;
pub const MARGIN_FRONTMATTER: f32 = 24.0;
pub const MARGIN_IMAGE: f32 = 18.0;
pub const MARGIN_MERMAID: f32 = 18.0;
pub const MARGIN_LIST: f32 = 14.0;
pub const MARGIN_PARAGRAPH: f32 = 12.0;
pub const MARGIN_HTML: f32 = 10.0;
pub const MARGIN_MATH: f32 = 18.0;

// Heading layout (base_size, padding_top, padding_bottom) per level
pub const HEADING_H1: (f32, f32, f32) = (28.0, 28.0, 14.0);
pub const HEADING_H2: (f32, f32, f32) = (22.0, 22.0, 10.0);
pub const HEADING_H3: (f32, f32, f32) = (18.0, 16.0, 8.0);
pub const HEADING_DEFAULT: (f32, f32, f32) = (15.0, 12.0, 6.0);

pub const BASE_FONT_SIZE: f32 = 14.0;
pub const MIN_SCALED_SIZE: f32 = 8.0;

#[inline]
pub fn scale_size(design_size: f32, user_font_size: f32) -> f32 {
    (design_size * user_font_size / BASE_FONT_SIZE)
        .round()
        .max(MIN_SCALED_SIZE)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadingLayout {
    pub font_size: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
}

pub fn heading_layout(level: u8, user_font_size: f32) -> HeadingLayout {
    let (base_size, base_pt, base_pb) = match level {
        1 => HEADING_H1,
        2 => HEADING_H2,
        3 => HEADING_H3,
        _ => HEADING_DEFAULT,
    };
    HeadingLayout {
        font_size: scale_size(base_size, user_font_size),
        padding_top: scale_size(base_pt, user_font_size),
        padding_bottom: scale_size(base_pb, user_font_size),
    }
}
