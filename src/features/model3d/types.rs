use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct Model3dMesh {
    pub vertices: Arc<[f32]>,
    pub indices: Arc<[u32]>,
    pub bounds: BoundingBox,
    pub triangle_count: usize,
    pub vertex_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: [f32::INFINITY, f32::INFINITY, f32::INFINITY],
            max: [f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY],
        }
    }

    pub fn add_point(&mut self, p: [f32; 3]) {
        self.min[0] = self.min[0].min(p[0]);
        self.min[1] = self.min[1].min(p[1]);
        self.min[2] = self.min[2].min(p[2]);

        self.max[0] = self.max[0].max(p[0]);
        self.max[1] = self.max[1].max(p[1]);
        self.max[2] = self.max[2].max(p[2]);
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn size(&self) -> [f32; 3] {
        [
            (self.max[0] - self.min[0]).max(0.0),
            (self.max[1] - self.min[1]).max(0.0),
            (self.max[2] - self.min[2]).max(0.0),
        ]
    }

    pub fn max_dimension(&self) -> f32 {
        let s = self.size();
        s[0].max(s[1]).max(s[2])
    }

    pub fn is_valid(&self) -> bool {
        self.min[0].is_finite() && self.max[0].is_finite()
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadingMode {
    Flat,
    Smooth,
    Wireframe,
}

impl fmt::Display for ShadingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flat => write!(f, "Flat"),
            Self::Smooth => write!(f, "Smooth"),
            Self::Wireframe => write!(f, "Wireframe"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Model3dState {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub base_distance: f32,
    pub is_dragging: bool,
    pub last_cursor: Option<iced::Point>,
    pub shading: ShadingMode,
    pub auto_rotate: bool,
    pub show_grid: bool,
}

impl Model3dState {
    pub fn new(bounds: &BoundingBox) -> Self {
        let max_dim = bounds.max_dimension();
        let base_distance = if max_dim > 0.0 { max_dim * 2.2 } else { 5.0 };
        Self {
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: std::f32::consts::FRAC_PI_6,
            distance: base_distance,
            base_distance,
            is_dragging: false,
            last_cursor: None,
            shading: ShadingMode::Smooth,
            auto_rotate: true,
            show_grid: true,
        }
    }

    pub fn reset_camera(&mut self) {
        self.yaw = std::f32::consts::FRAC_PI_4;
        self.pitch = std::f32::consts::FRAC_PI_6;
        self.distance = self.base_distance;
    }
}

impl Default for Model3dState {
    fn default() -> Self {
        Self::new(&BoundingBox::new())
    }
}
