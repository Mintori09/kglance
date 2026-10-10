#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ImageLoadState {
    #[default]
    Loading,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Angle(pub f32);

impl Angle {
    pub const ZERO: Self = Self(0.0);

    pub fn from_radians(rad: f32) -> Self {
        Self(rad.rem_euclid(2.0 * std::f32::consts::PI))
    }

    pub fn from_degrees(deg: f32) -> Self {
        Self(deg.rem_euclid(360.0).to_radians())
    }

    pub fn to_radians(self) -> f32 {
        self.0
    }

    pub fn to_degrees(self) -> f32 {
        self.0.to_degrees()
    }

    pub fn rotate_clockwise_90(&mut self) {
        let current_deg = (self.0.to_degrees() / 90.0).round() as i32;
        let next_deg = (current_deg + 1).rem_euclid(4) * 90;
        self.0 = (next_deg as f32).to_radians();
    }

    pub fn rotate_counter_clockwise_90(&mut self) {
        let current_deg = (self.0.to_degrees() / 90.0).round() as i32;
        let next_deg = (current_deg - 1).rem_euclid(4) * 90;
        self.0 = (next_deg as f32).to_radians();
    }

    pub fn is_perpendicular(self) -> bool {
        let deg = ((self.0.to_degrees() / 90.0).round() as i32).rem_euclid(4);
        deg == 1 || deg == 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_rotations() {
        let mut angle = Angle::ZERO;
        assert_eq!(angle.to_degrees(), 0.0);
        assert!(!angle.is_perpendicular());

        angle.rotate_clockwise_90();
        assert_eq!(angle.to_degrees().round(), 90.0);
        assert!(angle.is_perpendicular());

        angle.rotate_clockwise_90();
        assert_eq!(angle.to_degrees().round(), 180.0);
        assert!(!angle.is_perpendicular());

        angle.rotate_clockwise_90();
        assert_eq!(angle.to_degrees().round(), 270.0);
        assert!(angle.is_perpendicular());

        angle.rotate_clockwise_90();
        assert_eq!(angle.to_degrees().round(), 0.0);
        assert!(!angle.is_perpendicular());

        angle.rotate_counter_clockwise_90();
        assert_eq!(angle.to_degrees().round(), 270.0);
        assert!(angle.is_perpendicular());

        angle.rotate_counter_clockwise_90();
        assert_eq!(angle.to_degrees().round(), 180.0);
        assert!(!angle.is_perpendicular());
    }
}
