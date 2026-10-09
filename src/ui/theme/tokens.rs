pub mod spacing {
    pub const NONE: f32 = 0.0;
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

pub mod border {
    pub const NONE: f32 = 0.0;
    pub const THIN: f32 = 1.0;
    pub const MEDIUM: f32 = 2.0;
    pub const THICK: f32 = 3.0;
}

pub mod radius {
    pub const NONE: f32 = 0.0;
    pub const XS: f32 = 2.0;
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 12.0;
    pub const FLOATING_PILL: f32 = 12.0;
    pub const FULL: f32 = 9999.0;

    // Backwards compatibility aliases
    pub const SMALL: f32 = SM;
    pub const MEDIUM: f32 = LG;
    pub const LARGE: f32 = XL;
}

pub mod typography {
    pub const BADGE: f32 = 10.0;
    pub const CAPTION: f32 = 11.0;
    pub const BODY: f32 = 12.0;
    pub const BODY_MD: f32 = 13.0;
    pub const BODY_LG: f32 = 14.0;
    pub const TITLE_SM: f32 = 14.0;
    pub const TITLE: f32 = 16.0;
    pub const DISPLAY: f32 = 18.0;
    pub const HERO: f32 = 24.0;
    pub const SAMPLE_TEXT: f32 = 36.0;

    pub const ICON_SM: f32 = 11.0;
    pub const ICON_MD: f32 = 12.0;
    pub const ICON_LG: f32 = 16.0;
}

pub mod elevation {
    use iced::{Color, Shadow, Vector};

    #[inline]
    pub fn none() -> Shadow {
        Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::ZERO,
            blur_radius: 0.0,
        }
    }

    #[inline]
    pub fn low(color: Color) -> Shadow {
        Shadow {
            color,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        }
    }

    #[inline]
    pub fn medium(color: Color) -> Shadow {
        Shadow {
            color,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        }
    }

    #[inline]
    pub fn high(color: Color) -> Shadow {
        Shadow {
            color,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        }
    }

    #[inline]
    pub fn floating_pill(color: Color) -> Shadow {
        Shadow {
            color,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        }
    }
}

pub mod sidebar {
    use super::{radius, spacing};

    pub const DEFAULT_WIDTH: f32 = 240.0;
    pub const MIN_WIDTH: f32 = 160.0;
    pub const MAX_WIDTH: f32 = 400.0;

    pub const PADDING_HORIZONTAL: f32 = spacing::M;
    pub const PADDING_VERTICAL: f32 = spacing::S;
    pub const ITEM_SPACING: f32 = spacing::XS;

    pub const CORNER_RADIUS: f32 = radius::MEDIUM;
}

pub mod header {
    use super::{radius, spacing};

    pub const HEIGHT: f32 = 48.0;
    pub const PADDING_HORIZONTAL: f32 = spacing::L;
    pub const PADDING_VERTICAL: f32 = spacing::S;
    pub const GAP: f32 = spacing::M;

    pub const CORNER_RADIUS: f32 = radius::NONE;
}

pub mod grid {
    use super::spacing;

    pub const DEFAULT_COLUMNS: usize = 4;
    pub const CARD_MIN_WIDTH: f32 = 180.0;
    pub const CARD_MAX_WIDTH: f32 = 320.0;

    pub const CELL_PADDING: f32 = spacing::S;
    pub const GRID_GAP: f32 = spacing::M;
}

pub mod font_view {
    use super::{spacing, typography};

    pub const PREVIEW_TITLE_SIZE: f32 = typography::HERO;
    pub const PREVIEW_BODY_SIZE: f32 = typography::BODY_LG;
    pub const SAMPLE_TEXT_SIZE: f32 = typography::SAMPLE_TEXT;

    pub const CARD_PADDING: f32 = spacing::L;
    pub const ELEMENT_SPACING: f32 = spacing::M;
}

pub mod tables {
    use super::{border, radius, spacing, typography};

    pub const ROW_HEIGHT: f32 = 36.0;
    pub const HEADER_HEIGHT: f32 = 40.0;

    pub const PADDING_HORIZONTAL: f32 = spacing::M;
    pub const PADDING_VERTICAL: f32 = spacing::S;

    pub const FONT_SIZE_HEADER: f32 = typography::BODY_LG;
    pub const FONT_SIZE_BODY: f32 = typography::BODY;

    pub const BORDER_WIDTH: f32 = border::THIN;
    pub const CORNER_RADIUS: f32 = radius::SMALL;
}
