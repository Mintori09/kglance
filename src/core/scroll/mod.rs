use std::time::{Duration, Instant};

pub mod gesture;
pub mod interpreter;
pub mod physics;
pub mod telemetry;
pub mod velocity;

#[cfg(test)]
mod tests;

pub use gesture::{GestureState, GestureStateMachine, ScrollInput};
pub use interpreter::{ScrollDeviceKind, ScrollIntent, ScrollInterpreter};
pub use physics::{BoundaryBehavior, ScrollPhysics, ViewportExtent};
pub use telemetry::{FrameTiming, ScrollTelemetry, TelemetryStats};
pub use velocity::{
    EstimatorStrategy, LinearRegression, PolynomialRegression2, VelocityEstimator,
    VelocityEstimator as TouchpadGestureTracker, VelocityEstimatorAlgorithm,
    WeightedRecentRegression,
};

pub const INTERACTIVE_HALF_LIFE: f32 = 0.08; // 80 ms half-life
pub const SMOOTH_SCROLL_MAX_DT: f32 = 0.05; // 50 ms cap to avoid quantum leaps on lag
pub const STOP_THRESHOLD: f32 = 0.5; // 0.5px threshold for stop
pub const WHEEL_SCROLL_VIEWPORT_FRACTION: f32 = 0.15;
pub const MIN_INTERACTIVE_SPEED: f32 = 120.0; // 120 px/s minimum speed near boundaries

pub const TOUCHPAD_SCROLL_MULTIPLIER: f32 = 2.5;
pub const KINETIC_FRICTION_COEFFICIENT: f32 = 1.8; // Low friction rate for fluid inertia glide
pub const KINETIC_VELOCITY_CUTOFF: f32 = 15.0; // Stop when velocity falls below 15 px/s
pub const KINETIC_MAX_VELOCITY: f32 = 12000.0; // Cap kinetic velocity to prevent chaotic jumps
pub const MIN_FLING_VELOCITY: f32 = 100.0;

#[inline]
pub fn max_scroll_y(content_height: f32, viewport_height: f32) -> f32 {
    (content_height - viewport_height).max(0.0)
}

#[inline]
pub fn clamp_target(target: f32, max_y: f32) -> f32 {
    target.clamp(0.0, max_y)
}

pub fn navigation_duration(distance: f32) -> f32 {
    let ms = (120.0 + distance.abs() * 0.08).clamp(140.0, 350.0);
    ms / 1000.0
}

/// Ease-out cubic curve for navigation stops
#[inline]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = (t.clamp(0.0, 1.0) - 1.0).clamp(-1.0, 0.0);
    t * t * t + 1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmoothScrollMode {
    #[default]
    Interactive,
    Navigation,
    Kinetic,
}

/// Authoritative closed-form trajectory state for zero-divergence animation.
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollState {
    Idle {
        position: f32,
    },
    Dragging {
        position: f32,
    },
    Fling {
        start_time: Instant,
        start_pos: f32,
        v0: f32,
        friction: f32,
        cutoff_time: Duration,
    },
    Navigation {
        start_time: Instant,
        start_pos: f32,
        target_pos: f32,
        duration: Duration,
    },
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::Idle { position: 0.0 }
    }
}

impl ScrollState {
    /// Sample position and animation active status at exact absolute timestamp `now`.
    #[must_use]
    pub fn sample(&self, now: Instant) -> (f32, bool) {
        match self {
            Self::Idle { position } | Self::Dragging { position } => (*position, false),
            Self::Fling {
                start_time,
                start_pos,
                v0,
                friction,
                cutoff_time,
            } => {
                let elapsed = now.saturating_duration_since(*start_time);
                if elapsed >= *cutoff_time || *friction <= 0.0 {
                    let total_dist = *v0 / friction.max(0.001);
                    (*start_pos + total_dist, false)
                } else {
                    let t = elapsed.as_secs_f32();
                    let pos = *start_pos + (*v0 / *friction) * (1.0 - (-*friction * t).exp());
                    (pos, true)
                }
            }
            Self::Navigation {
                start_time,
                start_pos,
                target_pos,
                duration,
            } => {
                let elapsed = now.saturating_duration_since(*start_time);
                let total_secs = duration.as_secs_f32().max(0.001);
                let progress = (elapsed.as_secs_f32() / total_secs).min(1.0);
                let ease_out = 1.0 - (1.0 - progress).powi(3);
                let pos = *start_pos + (*target_pos - *start_pos) * ease_out;
                (pos, progress < 1.0)
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn is_animating(&self) -> bool {
        matches!(self, Self::Fling { .. } | Self::Navigation { .. })
    }
}

/// Unified pure Rust scroll controller combining discrete, kinetic, and navigation models.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollController {
    pub scroll_state: ScrollState,
    pub gesture: GestureStateMachine,
    pub velocity_estimator: VelocityEstimator,
    pub interpreter: ScrollInterpreter,
    pub physics: ScrollPhysics,

    pub position_y: f32,
    pub target_y: f32,
    pub is_animating: bool,
    pub velocity: f32,
    pub last_applied_y: f32,

    mode: SmoothScrollMode,
    start_y: f32,
    start_time: Option<Instant>,
    last_tick: Option<Instant>,
    duration: Duration,
}

impl Default for ScrollController {
    fn default() -> Self {
        Self {
            scroll_state: ScrollState::Idle { position: 0.0 },
            gesture: GestureStateMachine::default(),
            velocity_estimator: VelocityEstimator::default(),
            interpreter: ScrollInterpreter::default(),
            physics: ScrollPhysics::default(),
            position_y: 0.0,
            target_y: 0.0,
            is_animating: false,
            velocity: 0.0,
            last_applied_y: 0.0,
            mode: SmoothScrollMode::Interactive,
            start_y: 0.0,
            start_time: None,
            last_tick: None,
            duration: Duration::from_millis(200),
        }
    }
}

impl ScrollController {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn position_y(&self) -> f32 {
        self.position_y
    }

    pub fn set_position_y(&mut self, y: f32) {
        self.position_y = y;
        self.target_y = y;
        self.last_applied_y = y;
        self.scroll_state = ScrollState::Idle { position: y };
    }

    #[inline]
    #[must_use]
    pub fn target_y(&self) -> f32 {
        self.target_y
    }

    #[inline]
    #[must_use]
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    #[inline]
    #[must_use]
    pub fn is_animating(&self) -> bool {
        self.is_animating || self.scroll_state.is_animating()
    }

    #[inline]
    #[must_use]
    pub fn mode(&self) -> SmoothScrollMode {
        self.mode
    }

    #[inline]
    #[must_use]
    pub fn state(&self) -> GestureState {
        self.gesture.state()
    }

    pub fn set_boundary_behavior(&mut self, behavior: BoundaryBehavior) {
        self.physics.boundary_behavior = behavior;
    }

    pub fn start_interactive(&mut self, current_y: f32, delta: f32, max_y: f32) {
        let base = match self.mode {
            SmoothScrollMode::Interactive if self.is_animating => self.target_y,
            _ => current_y,
        };

        let target = clamp_target(base + delta, max_y);
        self.mode = SmoothScrollMode::Interactive;
        self.position_y = current_y;
        self.target_y = target;
        self.is_animating = (target - current_y).abs() > STOP_THRESHOLD;
        self.last_tick = None;
        self.last_applied_y = current_y;

        if self.is_animating {
            let dur = Duration::from_millis(160);
            self.scroll_state = ScrollState::Navigation {
                start_time: Instant::now(),
                start_pos: current_y,
                target_pos: target,
                duration: dur,
            };
        } else {
            self.scroll_state = ScrollState::Idle { position: target };
        }
    }

    pub fn start_navigation(&mut self, current_y: f32, target_y: f32, max_y: f32) {
        let clamped_target = clamp_target(target_y, max_y);
        let distance = (clamped_target - current_y).abs();
        if distance <= STOP_THRESHOLD {
            self.stop(clamped_target);
            return;
        }

        let duration_secs = navigation_duration(distance);
        let dur = Duration::from_secs_f32(duration_secs);
        self.mode = SmoothScrollMode::Navigation;
        self.position_y = current_y;
        self.start_y = current_y;
        self.target_y = clamped_target;
        self.start_time = None;
        self.last_tick = None;
        self.duration = dur;
        self.is_animating = true;
        self.last_applied_y = current_y;
        self.velocity = 0.0;

        self.scroll_state = ScrollState::Navigation {
            start_time: Instant::now(),
            start_pos: current_y,
            target_pos: clamped_target,
            duration: dur,
        };
    }

    pub fn start_kinetic(&mut self, current_y: f32, initial_velocity: f32, max_y: f32) {
        let clamped_v = initial_velocity.clamp(-KINETIC_MAX_VELOCITY, KINETIC_MAX_VELOCITY);
        if clamped_v.abs() < KINETIC_VELOCITY_CUTOFF {
            self.stop(current_y);
            return;
        }

        let friction = KINETIC_FRICTION_COEFFICIENT;
        let cutoff_secs = (clamped_v.abs() / KINETIC_VELOCITY_CUTOFF).max(1.0).ln() / friction;
        let cutoff_time = Duration::from_secs_f32(cutoff_secs);

        self.mode = SmoothScrollMode::Kinetic;
        self.position_y = current_y;
        self.velocity = clamped_v;
        self.is_animating = true;
        self.start_time = None;
        self.last_tick = None;
        self.last_applied_y = current_y;
        self.target_y = (current_y + clamped_v / friction).clamp(0.0, max_y);

        self.scroll_state = ScrollState::Fling {
            start_time: Instant::now(),
            start_pos: current_y,
            v0: clamped_v,
            friction,
            cutoff_time,
        };
        self.gesture.set_state(GestureState::Flinging);
        self.physics.start_fling(clamped_v);
    }

    pub fn stop(&mut self, current_y: f32) {
        self.position_y = current_y;
        self.target_y = current_y;
        self.last_applied_y = current_y;
        self.is_animating = false;
        self.velocity = 0.0;
        self.mode = SmoothScrollMode::Interactive;
        self.scroll_state = ScrollState::Idle {
            position: current_y,
        };
        self.gesture.set_state(GestureState::Idle);
        self.physics.stop();
    }

    pub fn stop_in_place(&mut self) {
        self.stop(self.position_y);
    }

    /// Sample position directly via closed-form trajectory.
    #[inline]
    #[must_use]
    pub fn sample(&self, now: Instant) -> (f32, bool) {
        self.scroll_state.sample(now)
    }

    /// Primary input entrypoint handling hardware motion and gestural liftoff.
    pub fn handle_input(&mut self, input: ScrollInput, extent: ViewportExtent) {
        let max_y = extent.max_scroll_y();

        match input {
            ScrollInput::Motion {
                delta_x,
                delta_y,
                time,
            } => {
                if delta_x.abs() <= f32::EPSILON && delta_y.abs() <= f32::EPSILON {
                    self.handle_input(ScrollInput::End { time }, extent);
                    return;
                }

                if self.gesture.handle_input(input).is_some() {
                    self.physics.stop();
                }
                self.velocity_estimator.push_sample(time, delta_y);

                self.position_y = (self.position_y + delta_y).clamp(0.0, max_y);
                self.target_y = self.position_y;
                self.last_applied_y = self.position_y;
                self.scroll_state = ScrollState::Dragging {
                    position: self.position_y,
                };
            }
            ScrollInput::End { time } => {
                if self.gesture.state() == GestureState::Dragging {
                    let v0 = self.velocity_estimator.compute_velocity(time);
                    if v0.abs() >= MIN_FLING_VELOCITY {
                        self.gesture.set_state(GestureState::Flinging);
                        self.start_kinetic(self.position_y, v0, max_y);
                    } else {
                        self.stop(self.position_y);
                    }
                }
            }
            ScrollInput::Cancel => {
                self.gesture.handle_input(input);
                self.velocity_estimator.reset();
                self.stop(self.position_y);
            }
        }
    }

    /// Advance time tick and return updated position if active.
    pub fn tick(&mut self, current_y: f32, now: Instant, max_y: f32) -> Option<f32> {
        if !self.is_animating && !self.scroll_state.is_animating() {
            self.velocity = 0.0;
            return None;
        }

        self.target_y = clamp_target(self.target_y, max_y);

        let dt = match (self.last_tick.replace(now), self.start_time) {
            (Some(last), _) => now.saturating_duration_since(last).as_secs_f32(),
            (None, Some(start)) => now.saturating_duration_since(start).as_secs_f32(),
            (None, None) => 1.0 / 125.0,
        };

        match self.mode {
            SmoothScrollMode::Interactive => {
                let diff = self.target_y - current_y;
                if diff.abs() <= STOP_THRESHOLD {
                    self.stop(self.target_y);
                    Some(self.target_y)
                } else {
                    let clamped_dt = dt.clamp(0.0, SMOOTH_SCROLL_MAX_DT);
                    let alpha = 1.0 - 2.0_f32.powf(-clamped_dt / INTERACTIVE_HALF_LIFE);
                    let mut step = (self.target_y - current_y) * alpha;
                    let min_step = MIN_INTERACTIVE_SPEED * clamped_dt;
                    if step.abs() < min_step {
                        step = step.signum() * min_step;
                    }
                    let next_y = if (self.target_y - current_y).abs() <= step.abs() {
                        self.target_y
                    } else {
                        clamp_target(current_y + step, max_y)
                    };
                    if dt > 0.0 {
                        self.velocity = (next_y - current_y) / dt;
                    }
                    self.position_y = next_y;
                    self.last_applied_y = next_y;
                    Some(next_y)
                }
            }
            SmoothScrollMode::Navigation => {
                let start_time = *self.start_time.get_or_insert(now);
                let elapsed = now.saturating_duration_since(start_time).as_secs_f32();
                let total_dur = self.duration.as_secs_f32().max(0.001);
                let t = (elapsed / total_dur).clamp(0.0, 1.0);

                if t >= 1.0 {
                    self.stop(self.target_y);
                    Some(self.target_y)
                } else {
                    let eased = ease_out_cubic(t);
                    let next_y =
                        clamp_target(self.start_y + (self.target_y - self.start_y) * eased, max_y);
                    if dt > 0.0 {
                        self.velocity = (next_y - current_y) / dt;
                    }
                    self.position_y = next_y;
                    self.last_applied_y = next_y;
                    Some(next_y)
                }
            }
            SmoothScrollMode::Kinetic => {
                let (sampled_y, is_active) = self.sample(now);
                let clamped_y = clamp_target(sampled_y, max_y);
                if dt > 0.0 {
                    self.velocity = (clamped_y - current_y) / dt;
                }

                let hit_boundary = (clamped_y <= 0.0 && self.velocity <= 0.0)
                    || (clamped_y >= max_y && self.velocity >= 0.0);

                if !is_active || hit_boundary || self.velocity.abs() < KINETIC_VELOCITY_CUTOFF {
                    self.stop(clamped_y);
                    Some(clamped_y)
                } else {
                    self.position_y = clamped_y;
                    self.target_y = clamped_y;
                    self.last_applied_y = clamped_y;
                    Some(clamped_y)
                }
            }
        }
    }

    pub fn update(&mut self, now: Instant, extent: ViewportExtent) -> Option<f32> {
        if self.gesture.check_timeout(now) {
            self.handle_input(ScrollInput::End { time: now }, extent);
        }

        let dt = match self.last_tick.replace(now) {
            Some(last) => now.saturating_duration_since(last).as_secs_f32().min(0.05),
            None => 1.0 / 120.0,
        };

        match self.gesture.state() {
            GestureState::Flinging | GestureState::SpringBack => {
                let (next_y, is_active) = self.physics.step(self.position_y, dt, extent);
                self.position_y = next_y;
                self.velocity = self.physics.velocity;
                if !is_active {
                    self.gesture.set_state(GestureState::Idle);
                    self.is_animating = false;
                }
                Some(next_y)
            }
            GestureState::Idle | GestureState::Dragging => None,
        }
    }

    pub fn check_interruption(&mut self, actual_y: f32) -> bool {
        if !self.is_animating() {
            return false;
        }
        let threshold = match self.mode {
            SmoothScrollMode::Kinetic => 40.0,
            _ => 3.0,
        };
        let diff = (actual_y - self.last_applied_y).abs();
        if diff > threshold {
            self.stop(actual_y);
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub type SmoothScroller = ScrollController;
pub type SmoothScrollState = ScrollController;
