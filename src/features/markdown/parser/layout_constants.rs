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
pub const QUOTE_CONTENT_PADDING_V: u16 = 6; // [6, 14][0]

// Image
pub const IMAGE_MAX_WIDTH: f32 = 600.0;
pub const IMAGE_PADDING_V: u16 = 4; // [4, 0][0]
pub const IMAGE_PADDING_BOTTOM: f32 = 4.0;
pub const PARAGRAPH_PADDING_TOP: f32 = 3.0;
pub const MAX_IMAGE_CAPTION_GAP: f32 = 8.0;
pub const MAX_IMAGE_TEXT_GAP: f32 = 16.0;

/// Calculates the display dimensions `(width, height)` for an image constrained by `max_width`.
///
/// Preserves the aspect ratio when explicit or cached dimensions exceed `max_width`.
#[inline]
pub fn compute_image_display_size(
    cached_size: Option<(u32, u32)>,
    explicit_width: Option<u32>,
    explicit_height: Option<u32>,
    max_width: f32,
) -> (Option<f32>, Option<f32>) {
    // Determine the reference dimensions and aspect ratio
    let (ref_w, ref_h) = match (cached_size, explicit_width, explicit_height) {
        (Some((cw, ch)), _, _) if cw > 0 && ch > 0 => (Some(cw as f32), Some(ch as f32)),
        (_, Some(ew), Some(eh)) if ew > 0 && eh > 0 => (Some(ew as f32), Some(eh as f32)),
        _ => (
            explicit_width.map(|w| w as f32),
            explicit_height.map(|h| h as f32),
        ),
    };

    match (ref_w, ref_h) {
        (Some(w), Some(h)) if w > 0.0 => {
            let target_w = explicit_width.map(|ew| ew as f32).unwrap_or(w);
            let display_w = target_w.min(max_width);
            let display_h = (h * display_w) / w;
            (Some(display_w), Some(display_h))
        }
        (Some(w), None) => {
            let target_w = explicit_width.map(|ew| ew as f32).unwrap_or(w);
            let display_w = target_w.min(max_width);
            (Some(display_w), None)
        }
        (None, Some(h)) => (None, Some(h)),
        _ => (None, None),
    }
}

// HR
pub const HR_PADDING_V: u16 = 4; // [4, 0][0]

// Math
pub const MATH_FONT_SCALE: f32 = 1.0;
pub const MATH_PADDING: u16 = 8;

// List
pub const LIST_ITEM_PADDING: f32 = 2.0;
pub const LIST_ITEM_SPACING: f32 = 4.0;
pub const LIST_SUB_BLOCK_LEFT_PADDING: f32 = 20.0;

// HTML
pub const HTML_FONT_SIZE: f32 = 12.0;

// Block margins
pub const MARGIN_HEADING_H1: f32 = 16.0;
pub const MARGIN_HEADING_H2: f32 = 14.0;
pub const MARGIN_HEADING_DEFAULT: f32 = 10.0;
pub const MARGIN_HORIZONTAL_RULE: f32 = 14.0;
pub const MARGIN_CODE: f32 = 16.0;
pub const MARGIN_TABLE: f32 = 16.0;
pub const MARGIN_QUOTE: f32 = 14.0;
pub const MARGIN_ALERT: f32 = 16.0;
pub const MARGIN_FOOTNOTE: f32 = 12.0;
pub const MARGIN_FRONTMATTER: f32 = 20.0;
pub const MARGIN_IMAGE: f32 = 16.0;
pub const MARGIN_MERMAID: f32 = 16.0;
pub const MARGIN_LIST: f32 = 10.0;
pub const MARGIN_PARAGRAPH: f32 = 10.0;
pub const MARGIN_HTML: f32 = 10.0;
pub const MARGIN_MATH: f32 = 16.0;

// Heading layout (base_size, padding_top, padding_bottom) per level
pub const HEADING_H1: (f32, f32, f32) = (26.0, 16.0, 8.0);
pub const HEADING_H2: (f32, f32, f32) = (20.0, 12.0, 6.0);
pub const HEADING_H3: (f32, f32, f32) = (16.0, 8.0, 4.0);
pub const HEADING_DEFAULT: (f32, f32, f32) = (14.0, 6.0, 4.0);

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
