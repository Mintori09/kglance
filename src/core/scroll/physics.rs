pub const KINETIC_FRICTION_COEFFICIENT: f32 = 1.8;
pub const KINETIC_VELOCITY_CUTOFF: f32 = 15.0;
pub const KINETIC_MAX_VELOCITY: f32 = 12000.0;
pub const DEFAULT_MAX_OVERSHOOT_RATIO: f32 = 0.12;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryBehavior {
    Clamp,
    Soft,
    Spring {
        stiffness: f32,
        damping: f32,
        max_overshoot_ratio: f32,
    },
}

impl Default for BoundaryBehavior {
    fn default() -> Self {
        Self::Spring {
            stiffness: 180.0,
            damping: 24.0,
            max_overshoot_ratio: DEFAULT_MAX_OVERSHOOT_RATIO,
        }
    }
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
            friction_coefficient: KINETIC_FRICTION_COEFFICIENT,
        }
    }
}

impl ScrollPhysics {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_fling(&mut self, initial_velocity: f32) {
        self.velocity = initial_velocity.clamp(-KINETIC_MAX_VELOCITY, KINETIC_MAX_VELOCITY);
    }

    pub fn stop(&mut self) {
        self.velocity = 0.0;
    }

    /// Advance physics by `dt` seconds given `current_y` and `extent`.
    /// Returns `(next_y, is_active)`.
    pub fn step(&mut self, current_y: f32, dt: f32, extent: ViewportExtent) -> (f32, bool) {
        let max_y = extent.max_scroll_y();

        match self.boundary_behavior {
            BoundaryBehavior::Clamp => {
                let step = self.velocity * dt;
                let next_y = (current_y + step).clamp(0.0, max_y);
                self.velocity *= (-self.friction_coefficient * dt).exp();

                let hit_boundary = (next_y <= 0.0 && self.velocity <= 0.0)
                    || (next_y >= max_y && self.velocity >= 0.0);
                if hit_boundary || self.velocity.abs() < KINETIC_VELOCITY_CUTOFF {
                    self.velocity = 0.0;
                    (next_y, false)
                } else {
                    (next_y, true)
                }
            }
            BoundaryBehavior::Soft => {
                let step = self.velocity * dt;
                let mut next_y = current_y + step;
                if next_y < 0.0 || next_y > max_y {
                    // Fast dissipation on boundary
                    self.velocity *= (-self.friction_coefficient * 8.0 * dt).exp();
                    next_y = next_y.clamp(0.0, max_y);
                } else {
                    self.velocity *= (-self.friction_coefficient * dt).exp();
                }

                if self.velocity.abs() < KINETIC_VELOCITY_CUTOFF {
                    self.velocity = 0.0;
                    (next_y.clamp(0.0, max_y), false)
                } else {
                    (next_y, true)
                }
            }
            BoundaryBehavior::Spring {
                stiffness,
                damping,
                max_overshoot_ratio,
            } => {
                let max_overshoot =
                    (extent.viewport_height * max_overshoot_ratio).clamp(30.0, 200.0);

                // Check if in overshoot region
                let displacement = if current_y < 0.0 {
                    current_y
                } else if current_y > max_y {
                    current_y - max_y
                } else {
                    0.0
                };

                if displacement.abs() > 0.1 {
                    // Spring-damper ODE: a = -k * x - c * v
                    let spring_accel = -stiffness * displacement - damping * self.velocity;
                    self.velocity += spring_accel * dt;
                    let next_y = (current_y + self.velocity * dt)
                        .clamp(-max_overshoot, max_y + max_overshoot);

                    if displacement.abs() < 0.5 && self.velocity.abs() < KINETIC_VELOCITY_CUTOFF {
                        self.velocity = 0.0;
                        let settled_y = if current_y < 0.0 { 0.0 } else { max_y };
                        (settled_y, false)
                    } else {
                        (next_y, true)
                    }
                } else {
                    // Normal friction decay
                    let step = self.velocity * dt;
                    let next_y = current_y + step;
                    self.velocity *= (-self.friction_coefficient * dt).exp();

                    if (0.0..=max_y).contains(&next_y)
                        && self.velocity.abs() < KINETIC_VELOCITY_CUTOFF
                    {
                        self.velocity = 0.0;
                        (next_y, false)
                    } else {
                        (next_y, true)
                    }
                }
            }
        }
    }
}
