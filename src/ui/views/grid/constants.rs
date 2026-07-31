use crate::core::types::{GRID_GAP, GRID_ITEM_WIDTH};
use crate::ui::theme::tokens::{grid as grid_tokens, spacing, typography};

pub const MAX_LABEL_LENGTH: usize = 20;
pub const ELLIPSIS: char = '…';
pub const DEFAULT_FILE_ICON_EMOJI: &str = "📄";

const BASE_CONTAINER_WIDTH: f32 = 136.0;
const BASE_CONTAINER_HEIGHT: f32 = 96.0;
const BASE_IMAGE_WIDTH: f32 = 130.0;
const BASE_IMAGE_HEIGHT: f32 = 90.0;
const BASE_ICON_SIZE: f32 = 48.0;
const BASE_FONT_SIZE: f32 = typography::SIZE_CAPTION;

const MIN_ICON_SIZE: f32 = 16.0;
const MAX_ICON_SIZE: f32 = 96.0;
const MIN_FONT_SIZE: f32 = typography::SIZE_CAPTION - spacing::XXS;
const MAX_FONT_SIZE: f32 = typography::SIZE_HEADER;

pub const CARD_INNER_SPACING: f32 = spacing::XS;
pub const CARD_PADDING: f32 = grid_tokens::CELL_PADDING;
pub const CARD_GRID_SPACING: f32 = grid_tokens::GRID_GAP;

pub struct ScaledDimensions {
    pub item_width: f32,
    pub gap: f32,
    pub container_width: f32,
    pub container_height: f32,
    pub image_width: f32,
    pub image_height: f32,
    pub icon_size: f32,
    pub font_size: f32,
}

impl ScaledDimensions {
    pub fn new(scale: f32) -> Self {
        Self {
            item_width: GRID_ITEM_WIDTH * scale,
            gap: GRID_GAP * scale,
            container_width: (BASE_CONTAINER_WIDTH * scale).round(),
            container_height: (BASE_CONTAINER_HEIGHT * scale).round(),
            image_width: (BASE_IMAGE_WIDTH * scale).round(),
            image_height: (BASE_IMAGE_HEIGHT * scale).round(),
            icon_size: (BASE_ICON_SIZE * scale)
                .round()
                .clamp(MIN_ICON_SIZE, MAX_ICON_SIZE),
            font_size: (BASE_FONT_SIZE * scale)
                .round()
                .clamp(MIN_FONT_SIZE, MAX_FONT_SIZE),
        }
    }
}
