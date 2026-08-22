#[derive(Debug, Clone)]
pub struct PageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Page dimensions in PDF points (1 point = 1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageDimensions {
    pub width_pts: f32,
    pub height_pts: f32,
}

impl PageDimensions {
    /// aspect ratio (width / height). Returns 1.0 for zero-height pages
    pub fn aspect_ratio(&self) -> f32 {
        if self.height_pts > 0.0 {
            self.width_pts / self.height_pts
        } else {
            1.0
        }
    }

    /// pixel height when scaled to a given display width
    pub fn display_height(&self, display_width: f32) -> f32 {
        display_width / self.aspect_ratio()
    }
}
