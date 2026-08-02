//! Raw colour primitives — the single source of truth for every colour value.
//!
//! Semantic layers ([`super::base`], [`super::roles`], [`super::markdown`],
//! [`super::json`], [`super::sidebar`]) reference these constants; no other
//! module should hard-code a colour literal.

use iced::Color;

// ── Neutrals ───────────────────────────────────────────────────────────────

pub(crate) const WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);
pub(crate) const BLACK: Color = Color::from_rgb(0.0, 0.0, 0.0);
pub(crate) const GRAY_500: Color = Color::from_rgb(0.5, 0.5, 0.5);

// ── Accent family ──────────────────────────────────────────────────────────

pub(crate) const ACCENT: Color = Color::from_rgb(0.0, 0.55, 1.0);
pub(crate) const ACCENT_HOVER: Color = Color::from_rgb(0.05, 0.62, 1.0);
pub(crate) const ACCENT_PRESSED: Color = Color::from_rgb(0.0, 0.42, 0.85);

// ── Functional hues ────────────────────────────────────────────────────────

pub(crate) const DANGER: Color = Color::from_rgb8(232, 17, 35);
pub(crate) const DANGER_PRESSED: Color = Color::from_rgb8(180, 0, 0);

// ── Dark theme surfaces ────────────────────────────────────────────────────

pub(crate) const DARK_BG: Color = Color::from_rgba(0.08, 0.09, 0.11, 1.0);
pub(crate) const DARK_SURFACE: Color = Color::from_rgba(0.14, 0.16, 0.20, 0.85);
pub(crate) const DARK_SURFACE_RAISED: Color = Color::from_rgba(0.18, 0.21, 0.26, 0.90);
pub(crate) const DARK_TOOLTIP: Color = Color::from_rgba(0.12, 0.14, 0.18, 0.95);

// ── Dark theme lines & text ────────────────────────────────────────────────

pub(crate) const DARK_BORDER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub(crate) const DARK_BORDER_FOCUS: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.16);
pub(crate) const DARK_RULE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.07);
pub(crate) const DARK_TEXT: Color = Color::from_rgb(0.93, 0.94, 0.96);
pub(crate) const DARK_TEXT_DIM: Color = Color::from_rgba(0.93, 0.94, 0.96, 0.50);
pub(crate) const DARK_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.40);

// ── Light theme surfaces ───────────────────────────────────────────────────

pub(crate) const LIGHT_BG: Color = Color::from_rgba(0.93, 0.94, 0.96, 1.0);
pub(crate) const LIGHT_SURFACE: Color = Color::from_rgba(0.98, 0.98, 1.0, 0.82);
pub(crate) const LIGHT_SURFACE_RAISED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.88);
pub(crate) const LIGHT_TOOLTIP: Color = Color::from_rgba(0.22, 0.24, 0.28, 0.95);

// ── Light theme lines & text ───────────────────────────────────────────────

pub(crate) const LIGHT_BORDER: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.07);
pub(crate) const LIGHT_BORDER_FOCUS: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.14);
pub(crate) const LIGHT_RULE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub(crate) const LIGHT_TEXT: Color = Color::from_rgb(0.12, 0.13, 0.16);
pub(crate) const LIGHT_TEXT_DIM: Color = Color::from_rgba(0.12, 0.13, 0.16, 0.50);
pub(crate) const LIGHT_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);

// ── Overlay alphas (white-on-dark, black-on-light) ─────────────────────────

pub(crate) const WHITE_005: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.05);
pub(crate) const WHITE_006: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.06);
pub(crate) const WHITE_010: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
pub(crate) const WHITE_012: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);
pub(crate) const WHITE_015: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);
pub(crate) const WHITE_020: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.20);

pub(crate) const BLACK_005: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.05);
pub(crate) const BLACK_006: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.06);
pub(crate) const BLACK_008: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
pub(crate) const BLACK_015: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.15);

// ── Markdown: inline syntax (theme-independent) ────────────────────────────

pub(crate) const MD_INLINE_CODE: Color = Color::from_rgb(0.8, 0.35, 0.35);
pub(crate) const MD_MATH: Color = Color::from_rgb(0.5, 0.2, 0.7);

// ── Markdown: link & task (shared via `roles`) ─────────────────────────────

pub(crate) const MD_DARK_LINK: Color = Color::from_rgb(0.4, 0.6, 1.0);
pub(crate) const MD_LIGHT_LINK: Color = Color::from_rgb(0.3, 0.5, 0.9);
pub(crate) const MD_DARK_TASK_CHECKED: Color = Color::from_rgb(0.4, 0.8, 0.4);
pub(crate) const MD_LIGHT_TASK_CHECKED: Color = Color::from_rgb(0.1, 0.6, 0.2);

// ── Markdown: search highlight ─────────────────────────────────────────────

pub(crate) const MD_DARK_SEARCH_ACTIVE: Color = Color::from_rgb(0.7, 0.6, 0.15);
pub(crate) const MD_DARK_SEARCH_INACTIVE: Color = Color::from_rgb(0.5, 0.4, 0.1);
pub(crate) const MD_LIGHT_SEARCH_ACTIVE: Color = Color::from_rgb(1.0, 0.85, 0.3);
pub(crate) const MD_LIGHT_SEARCH_INACTIVE: Color = Color::from_rgb(0.95, 0.8, 0.2);

// ── Markdown: tables ───────────────────────────────────────────────────────

pub(crate) const MD_DARK_TABLE_HEADER_BG: Color = Color::from_rgb(0.2, 0.22, 0.25);
pub(crate) const MD_DARK_TABLE_HEADER_TEXT: Color = Color::from_rgb(0.95, 0.95, 0.95);
pub(crate) const MD_DARK_TABLE_SEPARATOR: Color = Color::from_rgba(0.45, 0.47, 0.5, 0.6);
pub(crate) const MD_DARK_TABLE_BORDER: Color = Color::from_rgb(0.45, 0.47, 0.5);
pub(crate) const MD_LIGHT_TABLE_HEADER_BG: Color = Color::from_rgb(0.9, 0.91, 0.93);
pub(crate) const MD_LIGHT_TABLE_HEADER_TEXT: Color = Color::from_rgb(0.2, 0.2, 0.2);
pub(crate) const MD_LIGHT_TABLE_SEPARATOR: Color = Color::from_rgba(0.6, 0.62, 0.65, 0.5);
pub(crate) const MD_LIGHT_TABLE_BORDER: Color = Color::from_rgb(0.6, 0.62, 0.65);

// ── Markdown: quotes & html ────────────────────────────────────────────────

pub(crate) const MD_DARK_QUOTE_ACCENT: Color = Color::from_rgb(0.45, 0.5, 0.65);
pub(crate) const MD_DARK_QUOTE_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.03);
pub(crate) const MD_LIGHT_QUOTE_ACCENT: Color = Color::from_rgb(0.6, 0.5, 0.8);
pub(crate) const MD_LIGHT_QUOTE_BG: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.02);

// ── Markdown: code block fallback foreground ────────────────────────────────

pub(crate) const MD_DARK_CODE_FG: Color = Color::from_rgb(0.8, 0.8, 0.8);
pub(crate) const MD_LIGHT_CODE_FG: Color = Color::from_rgb(0.2, 0.2, 0.2);

// ── JSON syntax ────────────────────────────────────────────────────────────

pub(crate) const JSON_DARK_STRING: Color = Color::from_rgb(0.6, 0.9, 0.4);
pub(crate) const JSON_DARK_NUMBER: Color = Color::from_rgb(0.8, 0.6, 0.3);
pub(crate) const JSON_DARK_BOOL: Color = Color::from_rgb(0.4, 0.7, 1.0);
pub(crate) const JSON_DARK_NULL: Color = Color::from_rgb(0.6, 0.6, 0.6);
pub(crate) const JSON_DARK_OBJECT: Color = Color::from_rgb(0.7, 0.7, 0.9);
pub(crate) const JSON_DARK_TEXT: Color = Color::from_rgb(0.85, 0.85, 0.85);
pub(crate) const JSON_DARK_DIM: Color = Color::from_rgb(0.5, 0.5, 0.5);

pub(crate) const JSON_LIGHT_STRING: Color = Color::from_rgb(0.2, 0.6, 0.1);
pub(crate) const JSON_LIGHT_NUMBER: Color = Color::from_rgb(0.7, 0.4, 0.0);
pub(crate) const JSON_LIGHT_BOOL: Color = Color::from_rgb(0.0, 0.3, 0.8);
pub(crate) const JSON_LIGHT_NULL: Color = Color::from_rgb(0.5, 0.5, 0.5);
pub(crate) const JSON_LIGHT_OBJECT: Color = Color::from_rgb(0.3, 0.3, 0.6);
pub(crate) const JSON_LIGHT_TEXT: Color = Color::from_rgb(0.15, 0.15, 0.15);
pub(crate) const JSON_LIGHT_DIM: Color = Color::from_rgb(0.6, 0.6, 0.6);

pub(crate) const JSON_DARK_LINK: Color = Color::from_rgb(0.4, 0.7, 1.0);
pub(crate) const JSON_LIGHT_LINK: Color = Color::from_rgb(0.0, 0.3, 0.8);

pub(crate) const JSON_DARK_ERROR: Color = Color::from_rgb(1.0, 0.5, 0.3);
pub(crate) const JSON_LIGHT_ERROR: Color = Color::from_rgb(0.8, 0.2, 0.0);

pub(crate) const JSON_DARK_SELECTION: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
pub(crate) const JSON_LIGHT_SELECTION: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.05);

// ── Sidebar ────────────────────────────────────────────────────────────────

pub(crate) const SIDEBAR_DARK_ACTIVE_BG: Color = Color::from_rgba(0.25, 0.55, 0.95, 0.20);
pub(crate) const SIDEBAR_DARK_ACTIVE_TEXT: Color = Color::from_rgb(0.4, 0.75, 1.0);
pub(crate) const SIDEBAR_DARK_INACTIVE_TEXT: Color = Color::from_rgb(0.8, 0.82, 0.85);
pub(crate) const SIDEBAR_DARK_RESIZING: Color = Color::from_rgb(0.3, 0.65, 1.0);
pub(crate) const SIDEBAR_DARK_ARROW_TEXT: Color = Color::from_rgb(0.6, 0.65, 0.7);

pub(crate) const SIDEBAR_LIGHT_ACTIVE_BG: Color = Color::from_rgba(0.0, 0.45, 0.9, 0.12);
pub(crate) const SIDEBAR_LIGHT_ACTIVE_TEXT: Color = Color::from_rgb(0.0, 0.4, 0.85);
pub(crate) const SIDEBAR_LIGHT_INACTIVE_TEXT: Color = Color::from_rgb(0.3, 0.32, 0.35);
pub(crate) const SIDEBAR_LIGHT_RESIZING: Color = Color::from_rgb(0.0, 0.45, 0.85);
pub(crate) const SIDEBAR_LIGHT_ARROW_TEXT: Color = Color::from_rgb(0.5, 0.55, 0.6);

// ── EPUB chapter entries ──────────────────────────────────────────────────────

pub(crate) const EPUB_CHAPTER_TEXT_L1_DARK: Color = Color::from_rgb(0.9, 0.92, 0.95);
pub(crate) const EPUB_CHAPTER_TEXT_L2_DARK: Color = Color::from_rgb(0.75, 0.78, 0.82);
pub(crate) const EPUB_CHAPTER_TEXT_L1_LIGHT: Color = Color::from_rgb(0.2, 0.22, 0.25);
pub(crate) const EPUB_CHAPTER_TEXT_L2_LIGHT: Color = Color::from_rgb(0.4, 0.42, 0.45);

// ── Overlay shadows ─────────────────────────────────────────────────────────

pub(crate) const OVERLAY_SHADOW: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.25);

// ── Nord Theme Palette ───────────────────────────────────────────────────────

pub(crate) const NORD0: Color = Color::from_rgb(0.18, 0.20, 0.25); // #2E3440 Darkest Polar Night
pub(crate) const NORD1: Color = Color::from_rgb(0.23, 0.26, 0.32); // #3B4252 Surface
pub(crate) const NORD2: Color = Color::from_rgb(0.26, 0.30, 0.37); // #434C5E Raised Surface
pub(crate) const NORD3: Color = Color::from_rgb(0.30, 0.34, 0.42); // #4C566A Border / Selection
pub(crate) const NORD4: Color = Color::from_rgb(0.85, 0.87, 0.91); // #D8DEE9 Text
pub(crate) const NORD7: Color = Color::from_rgb(0.56, 0.74, 0.73); // #8FBCBB Teal
pub(crate) const NORD8: Color = Color::from_rgb(0.53, 0.75, 0.82); // #88C0D0 Ice Blue (Primary Accent)
pub(crate) const NORD9: Color = Color::from_rgb(0.51, 0.63, 0.76); // #81A1C1 Secondary Accent
pub(crate) const NORD11: Color = Color::from_rgb(0.75, 0.38, 0.42); // #BF616A Red / Danger
pub(crate) const NORD13: Color = Color::from_rgb(0.92, 0.80, 0.55); // #EBCB8B Yellow
pub(crate) const NORD14: Color = Color::from_rgb(0.64, 0.75, 0.55); // #A3BE8C Green / Success
pub(crate) const NORD15: Color = Color::from_rgb(0.71, 0.56, 0.68); // #B48EAD Purple / Constant
