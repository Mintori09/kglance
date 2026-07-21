#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub zoom: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transform_x(&self, x: f32, viewport_width: f32) -> f32 {
        viewport_width / 2.0 + self.offset_x + x * self.zoom
    }

    pub fn transform_y(&self, y: f32, viewport_height: f32) -> f32 {
        viewport_height / 2.0 + self.offset_y + y * self.zoom
    }

    pub fn transform_size(&self, size: f32) -> f32 {
        size * self.zoom
    }
}
