const BASE_FONT_SIZE: f32 = 14.0;
const MIN_SCALED_SIZE: f32 = 8.0;

pub(crate) fn scale_size(design_size: f32, user_font_size: f32) -> f32 {
    (design_size * user_font_size / BASE_FONT_SIZE)
        .round()
        .max(MIN_SCALED_SIZE)
}
