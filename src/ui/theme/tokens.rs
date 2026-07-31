pub mod spacing {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const S: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}

pub mod typography {
    pub const SIZE_CAPTION: f32 = 10.0;
    pub const SIZE_SMALL: f32 = 12.0;
    pub const SIZE_BODY: f32 = 14.0;
    pub const SIZE_TITLE: f32 = 18.0;
    pub const SIZE_HEADER: f32 = 24.0;

    pub const LINE_HEIGHT_DENSE: f32 = 1.2;
    pub const LINE_HEIGHT_NORMAL: f32 = 1.4;
    pub const LINE_HEIGHT_RELAXED: f32 = 1.6;
}

pub mod border {
    pub const THIN: f32 = 1.0;
    pub const MEDIUM: f32 = 2.0;
    pub const THICK: f32 = 3.0;
}

pub mod radius {
    pub const NONE: f32 = 0.0;
    pub const SMALL: f32 = 4.0;
    pub const MEDIUM: f32 = 8.0;
    pub const LARGE: f32 = 12.0;
    pub const FULL: f32 = 9999.0;
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

    pub const PREVIEW_TITLE_SIZE: f32 = typography::SIZE_HEADER;
    pub const PREVIEW_BODY_SIZE: f32 = typography::SIZE_BODY;
    pub const SAMPLE_TEXT_SIZE: f32 = 36.0;

    pub const CARD_PADDING: f32 = spacing::L;
    pub const ELEMENT_SPACING: f32 = spacing::M;
}

pub mod tables {
    use super::{border, radius, spacing, typography};

    pub const ROW_HEIGHT: f32 = 36.0;
    pub const HEADER_HEIGHT: f32 = 40.0;

    pub const PADDING_HORIZONTAL: f32 = spacing::M;
    pub const PADDING_VERTICAL: f32 = spacing::S;

    pub const FONT_SIZE_HEADER: f32 = typography::SIZE_BODY;
    pub const FONT_SIZE_BODY: f32 = typography::SIZE_SMALL;

    pub const BORDER_WIDTH: f32 = border::THIN;
    pub const CORNER_RADIUS: f32 = radius::SMALL;
}
