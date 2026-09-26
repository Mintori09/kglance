use crate::core::scroll::constants::{
    DEFAULT_MAX_OVERSHOOT_RATIO, KINETIC_DECAY_RATE, KINETIC_MAX_SPEED,
    KINETIC_SPRING_VELOCITY_CUTOFF, KINETIC_STOP_SPEED, MAX_PHYSICS_DT,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportExtent {
    pub content_height: f32,
    pub viewport_height: f32,
}
impl ViewportExtent {
    #[inline]
    #[must_use]
    pub fn max_scroll_y(&self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum BoundaryBehavior {
    #[default]
    Clamp,

    Soft,

    Spring {
        stiffness: f32,
        damping: f32,
        max_overshoot_ratio: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollPhysics {
    pub velocity: f32,
    pub boundary_behavior: BoundaryBehavior,
    pub friction_coefficient: f32,
}

impl Default for ScrollPhysics {
    fn default() -> Self {
        Self {
            velocity: 0.0,
            boundary_behavior: BoundaryBehavior::default(),
            friction_coefficient: KINETIC_DECAY_RATE,
        }
    }
}

impl ScrollPhysics {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_fling(&mut self, initial_velocity: f32) {
        self.velocity = if initial_velocity.is_finite() {
            initial_velocity.clamp(-KINETIC_MAX_SPEED, KINETIC_MAX_SPEED)
        } else {
            0.0
        };
    }

    pub fn stop(&mut self) {
        self.velocity = 0.0;
    }

    pub fn step(&mut self, current_y: f32, dt: f32, extent: ViewportExtent) -> (f32, bool) {
        let max_y = extent.max_scroll_y();

        if !current_y.is_finite() {
            self.velocity = 0.0;
            return (0.0, false);
        }

        if !dt.is_finite() || dt <= 0.0 {
            return (current_y.clamp(0.0, max_y), false);
        }

        let dt = dt.min(MAX_PHYSICS_DT);

        match self.boundary_behavior {
            BoundaryBehavior::Clamp => self.step_clamp(current_y, dt, max_y),
            BoundaryBehavior::Soft => self.step_soft(current_y, dt, max_y),
            BoundaryBehavior::Spring {
                stiffness,
                damping,
                max_overshoot_ratio,
            } => self.step_spring(
                current_y,
                dt,
                extent,
                max_y,
                stiffness,
                damping,
                max_overshoot_ratio,
            ),
        }
    }

    fn step_clamp(&mut self, current_y: f32, dt: f32, max_y: f32) -> (f32, bool) {
        let next_y = current_y + self.velocity * dt;
        self.decay_velocity(dt);

        let hit_boundary =
            (next_y <= 0.0 && self.velocity <= 0.0) || (next_y >= max_y && self.velocity >= 0.0);

        if hit_boundary || self.velocity.abs() < KINETIC_STOP_SPEED {
            self.velocity = 0.0;
            return (next_y.clamp(0.0, max_y), false);
        }

        (next_y.clamp(0.0, max_y), true)
    }

    #[inline]
    pub fn decay_velocity(&mut self, dt: f32) {
        self.decay_velocity_with_rate(dt, self.friction_coefficient);
    }

    #[inline]
    pub fn decay_velocity_with_rate(&mut self, dt: f32, decay_rate: f32) {
        if !decay_rate.is_finite() || decay_rate <= 0.0 {
            return;
        }

        self.velocity *= (-decay_rate * dt).exp();

        if !self.velocity.is_finite() {
            self.velocity = 0.0;
        }
    }

    fn step_soft(&mut self, current_y: f32, dt: f32, max_y: f32) -> (f32, bool) {
        let next_y = current_y + self.velocity * dt;
        let outside = next_y < 0.0 || next_y > max_y;

        let decay_rate = if outside {
            self.friction_coefficient * 8.0
        } else {
            self.friction_coefficient
        };

        self.decay_velocity_with_rate(dt, decay_rate);

        if self.velocity.abs() < KINETIC_STOP_SPEED {
            self.velocity = 0.0;
            return (next_y.clamp(0.0, max_y), false);
        }

        (next_y.clamp(0.0, max_y), true)
    }

    #[allow(clippy::too_many_arguments)]
    fn step_spring(
        &mut self,
        current_y: f32,
        dt: f32,
        extent: ViewportExtent,
        max_y: f32,
        stiffness: f32,
        damping: f32,
        max_overshoot_ratio: f32,
    ) -> (f32, bool) {
        let stiffness = if stiffness.is_finite() {
            stiffness.clamp(0.0, 1000.0)
        } else {
            100.0
        };

        let damping = if damping.is_finite() {
            damping.clamp(0.0, 100.0)
        } else {
            15.0
        };

        let max_overshoot_ratio = if max_overshoot_ratio.is_finite() {
            max_overshoot_ratio.max(0.0)
        } else {
            DEFAULT_MAX_OVERSHOOT_RATIO
        };

        let max_overshoot = (extent.viewport_height * max_overshoot_ratio).clamp(30.0, 200.0);

        let is_already_outside = current_y < 0.0 || current_y > max_y;
        let predicted_step = self.velocity * dt;
        let predicted_y = current_y + predicted_step;
        let will_cross_boundary = predicted_y < 0.0 || predicted_y > max_y;

        if is_already_outside || will_cross_boundary {
            return self.step_spring_region(
                current_y,
                dt,
                max_y,
                stiffness,
                damping,
                max_overshoot,
            );
        }

        self.decay_velocity(dt);
        let next_y = predicted_y.clamp(0.0, max_y);

        if self.velocity.abs() < KINETIC_STOP_SPEED {
            self.velocity = 0.0;
            (next_y, false)
        } else {
            (next_y, true)
        }
    }

    fn step_spring_region(
        &mut self,
        current_y: f32,
        dt: f32,
        max_y: f32,
        stiffness: f32,
        damping: f32,
        max_overshoot: f32,
    ) -> (f32, bool) {
        let displacement = if current_y < 0.0 {
            current_y
        } else if current_y > max_y {
            current_y - max_y
        } else {
            let predicted_y = current_y + self.velocity * dt;
            let target_boundary = if predicted_y < 0.0 { 0.0 } else { max_y };
            predicted_y.clamp(-max_overshoot, max_y + max_overshoot) - target_boundary
        };

        let spring_accel = -stiffness * displacement - damping * self.velocity;
        self.velocity += spring_accel * dt;
        self.velocity = self.velocity.clamp(-KINETIC_MAX_SPEED, KINETIC_MAX_SPEED);

        let next_y = (current_y + self.velocity * dt).clamp(-max_overshoot, max_y + max_overshoot);

        let next_displacement = if next_y < 0.0 {
            next_y
        } else if next_y > max_y {
            next_y - max_y
        } else {
            0.0
        };

        if next_displacement.abs() <= 0.5 && self.velocity.abs() < KINETIC_SPRING_VELOCITY_CUTOFF {
            self.velocity = 0.0;
            let settled_y = if current_y < max_y * 0.5 { 0.0 } else { max_y };
            return (settled_y, false);
        }

        (next_y, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXTENT: ViewportExtent = ViewportExtent {
        content_height: 2000.0,
        viewport_height: 500.0,
    };

    #[test]
    fn test_viewport_extent_max_scroll() {
        assert_eq!(EXTENT.max_scroll_y(), 1500.0);

        let small_extent = ViewportExtent {
            content_height: 300.0,
            viewport_height: 500.0,
        };
        assert_eq!(small_extent.max_scroll_y(), 0.0);

        let equal_extent = ViewportExtent {
            content_height: 500.0,
            viewport_height: 500.0,
        };
        assert_eq!(equal_extent.max_scroll_y(), 0.0);
    }

    #[test]
    fn test_fling_velocity_clamping_and_non_finite() {
        let mut physics = ScrollPhysics::new();

        physics.start_fling(20_000.0);
        assert_eq!(physics.velocity, KINETIC_MAX_SPEED);

        physics.start_fling(-30_000.0);
        assert_eq!(physics.velocity, -KINETIC_MAX_SPEED);

        physics.start_fling(f32::NAN);
        assert_eq!(physics.velocity, 0.0);

        physics.start_fling(f32::INFINITY);
        assert_eq!(physics.velocity, 0.0);
    }

    #[test]
    fn test_stop_fling() {
        let mut physics = ScrollPhysics::new();
        physics.start_fling(500.0);
        assert_eq!(physics.velocity, 500.0);

        physics.stop();
        assert_eq!(physics.velocity, 0.0);
    }

    #[test]
    fn test_step_invalid_inputs_safety() {
        let mut physics = ScrollPhysics::new();
        physics.velocity = 500.0;

        let (y, running) = physics.step(100.0, 0.0, EXTENT);
        assert!(!running);
        assert_eq!(y, 100.0);

        let (y, running) = physics.step(100.0, -0.016, EXTENT);
        assert!(!running);
        assert_eq!(y, 100.0);

        let (y, running) = physics.step(100.0, f32::NAN, EXTENT);
        assert!(!running);
        assert_eq!(y, 100.0);

        let (y, running) = physics.step(f32::NAN, 0.016, EXTENT);
        assert!(!running);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_clamp_normal_decay() {
        let mut physics = ScrollPhysics::new();
        physics.boundary_behavior = BoundaryBehavior::Clamp;
        physics.start_fling(1000.0);

        let mut current_y = 100.0;
        let mut steps = 0;

        while steps < 1000 {
            let (next_y, running) = physics.step(current_y, 0.016, EXTENT);
            current_y = next_y;
            steps += 1;
            if !running {
                break;
            }
        }

        assert!(steps < 1000);
        assert_eq!(physics.velocity, 0.0);
        assert!(current_y > 100.0 && current_y <= EXTENT.max_scroll_y());
    }

    #[test]
    fn test_clamp_hit_boundaries() {
        let mut physics = ScrollPhysics::new();
        physics.boundary_behavior = BoundaryBehavior::Clamp;

        // Hit top boundary
        physics.start_fling(-1000.0);
        let (next_y, running) = physics.step(5.0, 0.016, EXTENT);
        assert!(!running);
        assert_eq!(next_y, 0.0);
        assert_eq!(physics.velocity, 0.0);

        // Hit bottom boundary
        physics.start_fling(1000.0);
        let (next_y, running) = physics.step(EXTENT.max_scroll_y() - 5.0, 0.016, EXTENT);
        assert!(!running);
        assert_eq!(next_y, EXTENT.max_scroll_y());
        assert_eq!(physics.velocity, 0.0);
    }

    #[test]
    fn test_soft_friction_acceleration_outside() {
        let mut physics_inside = ScrollPhysics::new();
        physics_inside.boundary_behavior = BoundaryBehavior::Soft;
        physics_inside.velocity = 1000.0;
        let (_, _) = physics_inside.step(500.0, 0.016, EXTENT);

        let mut physics_outside = ScrollPhysics::new();
        physics_outside.boundary_behavior = BoundaryBehavior::Soft;
        physics_outside.velocity = 1000.0;
        let (_, _) = physics_outside.step(EXTENT.max_scroll_y() + 50.0, 0.016, EXTENT);

        assert!(physics_outside.velocity < physics_inside.velocity);
    }

    #[test]
    fn test_spring_overshoot_clamping() {
        let mut physics = ScrollPhysics::new();
        physics.boundary_behavior = BoundaryBehavior::Spring {
            stiffness: 100.0,
            damping: 10.0,
            max_overshoot_ratio: 0.1,
        };

        // Extreme speed towards top
        physics.velocity = -20_000.0;
        let (next_y, running) = physics.step(0.0, 0.016, EXTENT);
        assert!(running);
        let max_overshoot = (EXTENT.viewport_height * 0.1).clamp(30.0, 200.0);
        assert_eq!(next_y, -max_overshoot);
    }

    #[test]
    fn test_spring_rebound_force_direction() {
        let mut physics = ScrollPhysics::new();
        physics.boundary_behavior = BoundaryBehavior::Spring {
            stiffness: 200.0,
            damping: 15.0,
            max_overshoot_ratio: 0.2,
        };

        // When out at top (negative y), rebound accelerates positively
        physics.velocity = 0.0;
        let (next_y, running) = physics.step(-30.0, 0.016, EXTENT);
        assert!(running);
        assert!(physics.velocity > 0.0);
        assert!(next_y > -30.0);

        // When out at bottom (past max_scroll_y), rebound accelerates negatively
        physics.velocity = 0.0;
        let (next_y, running) = physics.step(EXTENT.max_scroll_y() + 30.0, 0.016, EXTENT);
        assert!(running);
        assert!(physics.velocity < 0.0);
        assert!(next_y < EXTENT.max_scroll_y() + 30.0);
    }

    #[test]
    fn test_spring_settle_to_rest() {
        let mut physics = ScrollPhysics::new();
        physics.boundary_behavior = BoundaryBehavior::Spring {
            stiffness: 100.0,
            damping: 15.0,
            max_overshoot_ratio: 0.1,
        };

        // Settle near top
        physics.velocity = 1.0;
        let (settled_y, running) = physics.step(-0.3, 0.016, EXTENT);
        assert!(!running);
        assert_eq!(settled_y, 0.0);
        assert_eq!(physics.velocity, 0.0);

        // Settle near bottom
        physics.velocity = -1.0;
        let (settled_y, running) = physics.step(EXTENT.max_scroll_y() + 0.3, 0.016, EXTENT);
        assert!(!running);
        assert_eq!(settled_y, EXTENT.max_scroll_y());
        assert_eq!(physics.velocity, 0.0);
    }
}
