//! Raw colour primitives — the single source of truth for every colour value.
//!
//! Semantic layers ([`super::base`], [`super::roles`], [`super::markdown`],
//! [`super::json`], [`super::sidebar`]) reference these constants; no other
//! module should hard-code a colour literal.

use iced::Color;

// ── Neutrals ───────────────────────────────────────────────────────────────

pub(crate) const WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);
#[allow(dead_code)]
pub(crate) const BLACK: Color = Color::from_rgb(0.0, 0.0, 0.0);
pub(crate) const GRAY_500: Color = Color::from_rgb(0.5, 0.5, 0.5);

// ── Accent family ──────────────────────────────────────────────────────────

pub(crate) const ACCENT: Color = Color::from_rgb8(59, 130, 246); // #3B82F6
pub(crate) const ACCENT_HOVER: Color = Color::from_rgb8(96, 165, 250); // #60A5FA
pub(crate) const ACCENT_PRESSED: Color = Color::from_rgb8(37, 99, 235); // #2563EB

// ── Functional hues ────────────────────────────────────────────────────────

pub(crate) const DANGER: Color = Color::from_rgb8(239, 68, 68);
pub(crate) const DANGER_PRESSED: Color = Color::from_rgb8(220, 38, 38);

// ── Dark theme surfaces (Minimal clean dark) ───────────────────────────────

pub(crate) const DARK_BG: Color = Color::from_rgb8(18, 18, 18); // #121212
pub(crate) const DARK_SURFACE: Color = Color::from_rgb8(28, 28, 28); // #1C1C1C
pub(crate) const DARK_SURFACE_RAISED: Color = Color::from_rgb8(38, 38, 38); // #262626
pub(crate) const DARK_TOOLTIP: Color = Color::from_rgb8(34, 34, 34);

// ── Dark theme lines & text ────────────────────────────────────────────────

pub(crate) const DARK_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
pub(crate) const DARK_BORDER_FOCUS: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.30);
pub(crate) const DARK_RULE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub(crate) const DARK_TEXT: Color = Color::from_rgb8(245, 245, 245); // #F5F5F5
pub(crate) const DARK_TEXT_DIM: Color = Color::from_rgb8(160, 160, 160); // #A0A0A0
pub(crate) const DARK_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.50);

// ── Light theme surfaces (Minimal clean light) ──────────────────────────────

pub(crate) const LIGHT_BG: Color = Color::from_rgb8(255, 255, 255); // #FFFFFF
pub(crate) const LIGHT_SURFACE: Color = Color::from_rgb8(248, 249, 250); // #F8F9FA
pub(crate) const LIGHT_SURFACE_RAISED: Color = Color::from_rgb8(241, 243, 245); // #F1F3F5
pub(crate) const LIGHT_TOOLTIP: Color = Color::from_rgb8(255, 255, 255);

// ── Light theme lines & text ───────────────────────────────────────────────

pub(crate) const LIGHT_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.10);
pub(crate) const LIGHT_BORDER_FOCUS: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.30);
pub(crate) const LIGHT_RULE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub(crate) const LIGHT_TEXT: Color = Color::from_rgb8(17, 24, 39); // #111827
pub(crate) const LIGHT_TEXT_DIM: Color = Color::from_rgb8(100, 116, 139); // #64748B
pub(crate) const LIGHT_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);

// ── Overlay alphas (white-on-dark, black-on-light) ─────────────────────────

pub(crate) const WHITE_002: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.02);
pub(crate) const WHITE_005: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);
pub(crate) const WHITE_006: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.06);
pub(crate) const WHITE_008: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub(crate) const WHITE_010: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub(crate) const WHITE_012: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
pub(crate) const WHITE_015: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);
pub(crate) const WHITE_020: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.20);

pub(crate) const BLACK_002: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.02);
pub(crate) const BLACK_005: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.05);
pub(crate) const BLACK_006: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.06);
pub(crate) const BLACK_008: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub(crate) const BLACK_015: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.15);

// ── Markdown: inline syntax ────────────────────────────────────────────────

pub(crate) const MD_DARK_INLINE_CODE: Color = Color::from_rgb8(245, 245, 245);
pub(crate) const MD_DARK_INLINE_CODE_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub(crate) const MD_LIGHT_INLINE_CODE: Color = Color::from_rgb8(15, 23, 42);
pub(crate) const MD_LIGHT_INLINE_CODE_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.06);

// ── Markdown: link & task ──────────────────────────────────────────────────

pub(crate) const MD_DARK_LINK: Color = Color::from_rgb8(96, 165, 250);
pub(crate) const MD_LIGHT_LINK: Color = Color::from_rgb8(29, 78, 216);
pub(crate) const MD_DARK_TASK_CHECKED: Color = Color::from_rgb8(74, 222, 128);
pub(crate) const MD_LIGHT_TASK_CHECKED: Color = Color::from_rgb8(22, 163, 74);

// ── Markdown: search highlight ─────────────────────────────────────────────

pub(crate) const MD_DARK_SEARCH_ACTIVE: Color = Color::from_rgba8(234, 179, 8, 0.6);
pub(crate) const MD_DARK_SEARCH_INACTIVE: Color = Color::from_rgba8(234, 179, 8, 0.25);
pub(crate) const MD_LIGHT_SEARCH_ACTIVE: Color = Color::from_rgba8(250, 204, 21, 0.7);
pub(crate) const MD_LIGHT_SEARCH_INACTIVE: Color = Color::from_rgba8(250, 204, 21, 0.3);

// ── Markdown: tables ───────────────────────────────────────────────────────

pub(crate) const MD_DARK_TABLE_HEADER_BG: Color = Color::from_rgb8(38, 38, 38);
pub(crate) const MD_DARK_TABLE_HEADER_TEXT: Color = Color::from_rgb8(255, 255, 255);
pub(crate) const MD_DARK_TABLE_SEPARATOR: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub(crate) const MD_DARK_TABLE_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
pub(crate) const MD_LIGHT_TABLE_HEADER_BG: Color = Color::from_rgb8(241, 243, 245);
pub(crate) const MD_LIGHT_TABLE_HEADER_TEXT: Color = Color::from_rgb8(17, 24, 39);
pub(crate) const MD_LIGHT_TABLE_SEPARATOR: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub(crate) const MD_LIGHT_TABLE_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.10);

// ── Markdown: quotes & html ────────────────────────────────────────────────

pub(crate) const MD_DARK_QUOTE_ACCENT: Color = Color::from_rgb8(160, 160, 160);
pub(crate) const MD_DARK_QUOTE_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.04);
pub(crate) const MD_LIGHT_QUOTE_ACCENT: Color = Color::from_rgb8(148, 163, 184);
pub(crate) const MD_LIGHT_QUOTE_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.03);

// ── Markdown: code block fallback foreground ────────────────────────────────

pub(crate) const MD_DARK_CODE_FG: Color = Color::from_rgb8(240, 240, 240);
pub(crate) const MD_LIGHT_CODE_FG: Color = Color::from_rgb8(17, 24, 39);

// ── JSON syntax ────────────────────────────────────────────────────────────

pub(crate) const JSON_DARK_STRING: Color = Color::from_rgb8(134, 239, 172);
pub(crate) const JSON_DARK_NUMBER: Color = Color::from_rgb8(253, 186, 116);
pub(crate) const JSON_DARK_BOOL: Color = Color::from_rgb8(147, 197, 253);
pub(crate) const JSON_DARK_NULL: Color = Color::from_rgb8(160, 160, 160);
pub(crate) const JSON_DARK_OBJECT: Color = Color::from_rgb8(216, 180, 254);
pub(crate) const JSON_DARK_TEXT: Color = Color::from_rgb8(245, 245, 245);
pub(crate) const JSON_DARK_DIM: Color = Color::from_rgb8(160, 160, 160);

pub(crate) const JSON_LIGHT_STRING: Color = Color::from_rgb8(22, 101, 52);
pub(crate) const JSON_LIGHT_NUMBER: Color = Color::from_rgb8(154, 52, 18);
pub(crate) const JSON_LIGHT_BOOL: Color = Color::from_rgb8(29, 78, 216);
pub(crate) const JSON_LIGHT_NULL: Color = Color::from_rgb8(100, 116, 139);
pub(crate) const JSON_LIGHT_OBJECT: Color = Color::from_rgb8(107, 33, 168);
pub(crate) const JSON_LIGHT_TEXT: Color = Color::from_rgb8(17, 24, 39);
pub(crate) const JSON_LIGHT_DIM: Color = Color::from_rgb8(100, 116, 139);

pub(crate) const JSON_DARK_LINK: Color = Color::from_rgb8(96, 165, 250);
pub(crate) const JSON_LIGHT_LINK: Color = Color::from_rgb8(29, 78, 216);

pub(crate) const JSON_DARK_ERROR: Color = Color::from_rgb8(248, 113, 113);
pub(crate) const JSON_LIGHT_ERROR: Color = Color::from_rgb8(220, 38, 38);

pub(crate) const JSON_DARK_SELECTION: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub(crate) const JSON_LIGHT_SELECTION: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);

// ── Sidebar ────────────────────────────────────────────────────────────────

pub(crate) const SIDEBAR_DARK_ACTIVE_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
pub(crate) const SIDEBAR_DARK_ACTIVE_TEXT: Color = Color::from_rgb8(255, 255, 255);
pub(crate) const SIDEBAR_DARK_INACTIVE_TEXT: Color = Color::from_rgb8(160, 160, 160);
pub(crate) const SIDEBAR_DARK_RESIZING: Color = Color::from_rgb8(96, 165, 250);
pub(crate) const SIDEBAR_DARK_ARROW_TEXT: Color = Color::from_rgb8(140, 140, 140);

pub(crate) const SIDEBAR_LIGHT_ACTIVE_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub(crate) const SIDEBAR_LIGHT_ACTIVE_TEXT: Color = Color::from_rgb8(17, 24, 39);
pub(crate) const SIDEBAR_LIGHT_INACTIVE_TEXT: Color = Color::from_rgb8(100, 116, 139);
pub(crate) const SIDEBAR_LIGHT_RESIZING: Color = Color::from_rgb8(29, 78, 216);
pub(crate) const SIDEBAR_LIGHT_ARROW_TEXT: Color = Color::from_rgb8(120, 120, 120);

// ── EPUB chapter entries ──────────────────────────────────────────────────────

pub(crate) const EPUB_CHAPTER_TEXT_L1_DARK: Color = Color::from_rgb(0.9, 0.92, 0.95);
pub(crate) const EPUB_CHAPTER_TEXT_L2_DARK: Color = Color::from_rgb(0.75, 0.78, 0.82);
pub(crate) const EPUB_CHAPTER_TEXT_L1_LIGHT: Color = Color::from_rgb(0.2, 0.22, 0.25);
pub(crate) const EPUB_CHAPTER_TEXT_L2_LIGHT: Color = Color::from_rgb(0.4, 0.42, 0.45);

// ── Overlay shadows ─────────────────────────────────────────────────────────

pub(crate) const OVERLAY_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.25);

// ── Nord Theme Palette ───────────────────────────────────────────────────────

pub(crate) const NORD0: Color = Color::from_rgb8(46, 52, 64); // #2E3440 Darkest Polar Night (Background)
pub(crate) const NORD1: Color = Color::from_rgb8(59, 66, 82); // #3B4252 Surface
pub(crate) const NORD2: Color = Color::from_rgb8(67, 76, 94); // #434C5E Raised Surface
pub(crate) const NORD3: Color = Color::from_rgb8(76, 86, 106); // #4C566A Border
pub(crate) const NORD4: Color = Color::from_rgb8(216, 222, 233); // #D8DEE9 Text
pub(crate) const NORD_TEXT_DIM: Color = Color::from_rgb8(160, 170, 188); // High contrast secondary text
pub(crate) const NORD7: Color = Color::from_rgb8(143, 188, 187); // #8FBCBB Teal
pub(crate) const NORD8: Color = Color::from_rgb8(136, 192, 208); // #88C0D0 Ice Blue (Primary Accent)
pub(crate) const NORD9: Color = Color::from_rgb8(129, 161, 193); // #81A1C1 Secondary Accent
pub(crate) const NORD11: Color = Color::from_rgb8(191, 97, 106); // #BF616A Red / Danger
pub(crate) const NORD13: Color = Color::from_rgb8(235, 203, 139); // #EBCB8B Yellow
pub(crate) const NORD14: Color = Color::from_rgb8(163, 190, 140); // #A3BE8C Green / Success
pub(crate) const NORD15: Color = Color::from_rgb8(180, 142, 173); // #B48EAD Purple / Constant

// ── Markdown / Toast Alerts ────────────────────────────────────────────────

pub(crate) const ALERT_NOTE_FG: Color = Color::from_rgb(0.2, 0.5, 0.9);
pub(crate) const ALERT_NOTE_BG: Color = Color::from_rgba(0.2, 0.5, 0.9, 0.08);

pub(crate) const ALERT_TIP_FG: Color = Color::from_rgb(0.18, 0.68, 0.38);
pub(crate) const ALERT_TIP_BG: Color = Color::from_rgba(0.18, 0.68, 0.38, 0.08);

pub(crate) const ALERT_INFO_FG: Color = Color::from_rgb(0.58, 0.34, 0.88);
pub(crate) const ALERT_INFO_BG: Color = Color::from_rgba(0.58, 0.34, 0.88, 0.08);

pub(crate) const ALERT_WARNING_FG: Color = Color::from_rgb(0.92, 0.58, 0.12);
pub(crate) const ALERT_WARNING_BG: Color = Color::from_rgba(0.92, 0.58, 0.12, 0.08);

pub(crate) const ALERT_CAUTION_FG: Color = Color::from_rgb(0.92, 0.28, 0.28);
pub(crate) const ALERT_CAUTION_BG: Color = Color::from_rgba(0.92, 0.28, 0.28, 0.08);

// ── Code outline symbol kinds ───────────────────────────────────────────────

pub(crate) const SYMBOL_FUNCTION: Color = Color::from_rgb(0.35, 0.65, 0.95);
pub(crate) const SYMBOL_STRUCT: Color = Color::from_rgb(0.35, 0.85, 0.65);
pub(crate) const SYMBOL_CLASS: Color = Color::from_rgb(0.95, 0.65, 0.35);
pub(crate) const SYMBOL_ENUM: Color = Color::from_rgb(0.85, 0.45, 0.85);
pub(crate) const SYMBOL_TRAIT: Color = Color::from_rgb(0.95, 0.85, 0.35);
pub(crate) const SYMBOL_MODULE: Color = Color::from_rgb(0.65, 0.75, 0.85);
pub(crate) const SYMBOL_TYPE: Color = Color::from_rgb(0.45, 0.75, 0.85);

// ── Audio Vinyl player ─────────────────────────────────────────────────────

pub(crate) const VINYL_DARK_BG: Color = Color::from_rgb8(24, 25, 28);
pub(crate) const VINYL_DARK_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub(crate) const VINYL_DARK_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.35);

pub(crate) const VINYL_LIGHT_BG: Color = Color::from_rgb8(45, 48, 53);
pub(crate) const VINYL_LIGHT_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.12);
pub(crate) const VINYL_LIGHT_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.20);

// ── Floating Overlays, Controls & Selection ────────────────────────────────

pub(crate) const OVERLAY_FLOATING_DARK_BG: Color = Color::from_rgba(0.08, 0.08, 0.10, 0.75);
pub(crate) const OVERLAY_FLOATING_DARK_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
pub(crate) const OVERLAY_FLOATING_DARK_FG: Color = Color::from_rgba(0.94, 0.94, 0.96, 1.0);
pub(crate) const OVERLAY_FLOATING_DARK_TEXT_DIM: Color = Color::from_rgba(0.82, 0.82, 0.86, 1.0);
pub(crate) const OVERLAY_FLOATING_DARK_TEXT_MUTED: Color = Color::from_rgba(0.55, 0.55, 0.60, 1.0);

pub(crate) const OVERLAY_FLOATING_LIGHT_BG: Color = Color::from_rgba(0.98, 0.98, 1.0, 0.85);
pub(crate) const OVERLAY_FLOATING_LIGHT_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.10);
pub(crate) const OVERLAY_FLOATING_LIGHT_FG: Color = Color::from_rgba(0.12, 0.13, 0.16, 1.0);
pub(crate) const OVERLAY_FLOATING_LIGHT_TEXT_DIM: Color = Color::from_rgba(0.35, 0.38, 0.42, 1.0);
pub(crate) const OVERLAY_FLOATING_LIGHT_TEXT_MUTED: Color = Color::from_rgba(0.50, 0.52, 0.56, 1.0);

pub(crate) const OVERLAY_BACKDROP: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.50);

// ── Catppuccin Mocha Palette ──────────────────────────────────────────────
pub(crate) const MOCHA_BASE: Color = Color::from_rgb(0.118, 0.118, 0.180); // #1E1E2E
pub(crate) const MOCHA_MANTLE: Color = Color::from_rgb(0.094, 0.094, 0.145); // #181825
pub(crate) const MOCHA_CRUST: Color = Color::from_rgb(0.067, 0.067, 0.106); // #11111B
pub(crate) const MOCHA_SURFACE0: Color = Color::from_rgb(0.192, 0.196, 0.267); // #313244
pub(crate) const MOCHA_SURFACE1: Color = Color::from_rgb(0.271, 0.278, 0.353); // #45475A
pub(crate) const MOCHA_OVERLAY0: Color = Color::from_rgb(0.424, 0.439, 0.525); // #6C7086
pub(crate) const MOCHA_TEXT: Color = Color::from_rgb(0.804, 0.839, 0.957); // #CDD6F4
pub(crate) const MOCHA_SUBTEXT0: Color = Color::from_rgb(0.651, 0.678, 0.784); // #A6ADC8
pub(crate) const MOCHA_BLUE: Color = Color::from_rgb(0.537, 0.706, 0.980); // #89B4FA
pub(crate) const MOCHA_SKY: Color = Color::from_rgb(0.537, 0.863, 0.922); // #89DCEB
pub(crate) const MOCHA_SAPPHIRE: Color = Color::from_rgb(0.455, 0.780, 0.925); // #74C7EC
pub(crate) const MOCHA_LAVENDER: Color = Color::from_rgb(0.706, 0.745, 0.996); // #B4BEFE
pub(crate) const MOCHA_GREEN: Color = Color::from_rgb(0.651, 0.890, 0.631); // #A6E3A1
pub(crate) const MOCHA_YELLOW: Color = Color::from_rgb(0.976, 0.886, 0.686); // #F9E2AF
pub(crate) const MOCHA_RED: Color = Color::from_rgb(0.953, 0.545, 0.659); // #F38BA8
pub(crate) const MOCHA_PEACH: Color = Color::from_rgb(0.980, 0.702, 0.529); // #FAB387
pub(crate) const MOCHA_MAUVE: Color = Color::from_rgb(0.796, 0.651, 0.969); // #CBA6F7
pub(crate) const MOCHA_TEAL: Color = Color::from_rgb(0.580, 0.886, 0.835); // #94E2D5

// ── Catppuccin Latte Palette ──────────────────────────────────────────────
pub(crate) const LATTE_BASE: Color = Color::from_rgb(0.937, 0.945, 0.961); // #EFF1F5
pub(crate) const LATTE_MANTLE: Color = Color::from_rgb(0.902, 0.914, 0.937); // #E6E9EF
pub(crate) const LATTE_CRUST: Color = Color::from_rgb(0.863, 0.878, 0.914); // #DCE0E8
pub(crate) const LATTE_SURFACE0: Color = Color::from_rgb(0.800, 0.816, 0.855); // #CCD0DA
pub(crate) const LATTE_SURFACE1: Color = Color::from_rgb(0.737, 0.753, 0.800); // #BCC0CC
pub(crate) const LATTE_TEXT: Color = Color::from_rgb(0.298, 0.310, 0.412); // #4C4F69
pub(crate) const LATTE_SUBTEXT0: Color = Color::from_rgb(0.424, 0.435, 0.522); // #6C6F85
pub(crate) const LATTE_BLUE: Color = Color::from_rgb(0.118, 0.400, 0.961); // #1E66F5
pub(crate) const LATTE_SKY: Color = Color::from_rgb(0.016, 0.647, 0.898); // #04A5E5
pub(crate) const LATTE_SAPPHIRE: Color = Color::from_rgb(0.125, 0.624, 0.710); // #209FB5
pub(crate) const LATTE_LAVENDER: Color = Color::from_rgb(0.447, 0.529, 0.992); // #7287FD
pub(crate) const LATTE_GREEN: Color = Color::from_rgb(0.251, 0.627, 0.169); // #40A02B
pub(crate) const LATTE_YELLOW: Color = Color::from_rgb(0.875, 0.557, 0.114); // #DF8E1D
pub(crate) const LATTE_RED: Color = Color::from_rgb(0.824, 0.059, 0.224); // #D20F39

// ── Tokyo Night Palette ───────────────────────────────────────────────────
pub(crate) const TOKYO_BG: Color = Color::from_rgb8(26, 27, 38); // #1A1B26
pub(crate) const TOKYO_SURFACE: Color = Color::from_rgb8(36, 40, 59); // #24283B
pub(crate) const TOKYO_SURFACE_RAISED: Color = Color::from_rgb8(41, 46, 66); // #292E42
pub(crate) const TOKYO_BORDER: Color = Color::from_rgb8(65, 72, 104); // #414868
pub(crate) const TOKYO_TEXT: Color = Color::from_rgb8(192, 202, 245); // #C0CAF5
pub(crate) const TOKYO_TEXT_DIM: Color = Color::from_rgb8(154, 165, 206); // #9AA5CE
pub(crate) const TOKYO_BLUE: Color = Color::from_rgb8(122, 162, 247); // #7AA2F7
pub(crate) const TOKYO_CYAN: Color = Color::from_rgb8(125, 207, 255); // #7DCFFF
pub(crate) const TOKYO_BLUE_DARK: Color = Color::from_rgb8(61, 89, 161); // #3D59A1
pub(crate) const TOKYO_GREEN: Color = Color::from_rgb8(158, 206, 106); // #9ECE6A
pub(crate) const TOKYO_YELLOW: Color = Color::from_rgb8(224, 175, 104); // #E0AF68
pub(crate) const TOKYO_RED: Color = Color::from_rgb8(247, 118, 142); // #F7768E
pub(crate) const TOKYO_PURPLE: Color = Color::from_rgb8(187, 154, 247); // #BB9AF7
pub(crate) const TOKYO_ORANGE: Color = Color::from_rgb8(255, 158, 100); // #FF9E64

// ── Gruvbox Dark Palette ──────────────────────────────────────────────────
pub(crate) const GRUVBOX_BG: Color = Color::from_rgb8(40, 40, 40); // #282828
pub(crate) const GRUVBOX_SURFACE: Color = Color::from_rgb8(60, 56, 54); // #3C3836
pub(crate) const GRUVBOX_SURFACE_RAISED: Color = Color::from_rgb8(80, 73, 69); // #504945
pub(crate) const GRUVBOX_BORDER: Color = Color::from_rgb8(102, 92, 84); // #665C54
pub(crate) const GRUVBOX_TEXT: Color = Color::from_rgb8(251, 241, 199); // #FBF1C7 (light0 crisp contrast)
pub(crate) const GRUVBOX_TEXT_DIM: Color = Color::from_rgb8(189, 174, 147); // #BDAE93 (light3 readable dim)
pub(crate) const GRUVBOX_ORANGE: Color = Color::from_rgb8(254, 128, 25); // #FE8019
pub(crate) const GRUVBOX_ORANGE_DARK: Color = Color::from_rgb8(214, 93, 14); // #D65D0E
pub(crate) const GRUVBOX_YELLOW: Color = Color::from_rgb8(250, 189, 47); // #FABD2F
pub(crate) const GRUVBOX_GREEN: Color = Color::from_rgb8(184, 187, 38); // #B8BB26
pub(crate) const GRUVBOX_RED: Color = Color::from_rgb8(251, 73, 52); // #FB4934
pub(crate) const GRUVBOX_AQUA: Color = Color::from_rgb8(142, 192, 124); // #8EC07C
pub(crate) const GRUVBOX_PURPLE: Color = Color::from_rgb8(211, 134, 155); // #D3869B

// ── Dracula Palette ───────────────────────────────────────────────────────
pub(crate) const DRACULA_BG: Color = Color::from_rgb8(40, 42, 54); // #282A36
pub(crate) const DRACULA_SURFACE: Color = Color::from_rgb8(52, 55, 70); // #343746
pub(crate) const DRACULA_SURFACE_RAISED: Color = Color::from_rgb8(68, 71, 90); // #44475A
pub(crate) const DRACULA_BORDER: Color = Color::from_rgb8(98, 114, 164); // #6272A4
pub(crate) const DRACULA_TEXT: Color = Color::from_rgb8(248, 248, 242); // #F8F8F2
pub(crate) const DRACULA_TEXT_DIM: Color = Color::from_rgb8(171, 178, 191); // High contrast secondary text
pub(crate) const DRACULA_PURPLE: Color = Color::from_rgb8(189, 147, 249); // #BD93F9
pub(crate) const DRACULA_PINK: Color = Color::from_rgb8(255, 121, 198); // #FF79C6
pub(crate) const DRACULA_CYAN: Color = Color::from_rgb8(139, 233, 253); // #8BE9FD
pub(crate) const DRACULA_GREEN: Color = Color::from_rgb8(80, 250, 123); // #50FA7B
pub(crate) const DRACULA_YELLOW: Color = Color::from_rgb8(241, 250, 140); // #F1FA8C
pub(crate) const DRACULA_ORANGE: Color = Color::from_rgb8(255, 184, 108); // #FFB86C
pub(crate) const DRACULA_RED: Color = Color::from_rgb8(255, 85, 85); // #FF5555

pub(crate) const SELECTION_DARK_BG: Color = Color::from_rgba(0.2, 0.45, 0.85, 0.35);
pub(crate) const SELECTION_LIGHT_BG: Color = Color::from_rgba(0.0, 0.45, 0.9, 0.25);
pub(crate) const SELECTION_NORD_BG: Color = Color::from_rgba(0.53, 0.75, 0.82, 0.30);
pub(crate) const SELECTION_MOCHA_BG: Color = Color::from_rgba(0.537, 0.706, 0.980, 0.30);
pub(crate) const SELECTION_LATTE_BG: Color = Color::from_rgba(0.118, 0.400, 0.961, 0.20);
pub(crate) const SELECTION_TOKYO_BG: Color = Color::from_rgba(0.478, 0.635, 0.969, 0.30);
pub(crate) const SELECTION_GRUVBOX_BG: Color = Color::from_rgba(0.996, 0.502, 0.098, 0.30);
pub(crate) const SELECTION_DRACULA_BG: Color = Color::from_rgba(0.741, 0.576, 0.976, 0.30);

#[inline]
pub const fn syntect_to_iced_color(c: syntect::highlighting::Color) -> Color {
    Color::from_rgba(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}
