use std::time::{Duration, Instant};

pub mod gesture;
pub mod physics;
pub mod velocity;

#[cfg(test)]
mod tests;

pub use gesture::{GestureState, GestureStateMachine, ScrollInput};
pub use physics::{BoundaryBehavior, ScrollPhysics, ViewportExtent};
pub use velocity::VelocityEstimator;

pub const INTERACTIVE_HALF_LIFE: f32 = 0.08; // 80 ms half-life
pub const SMOOTH_SCROLL_MAX_DT: f32 = 0.05; // 50 ms cap to avoid quantum leaps on lag
pub const STOP_THRESHOLD: f32 = 0.5; // 0.5px threshold for stop
pub const WHEEL_SCROLL_VIEWPORT_FRACTION: f32 = 0.08;
pub const MIN_INTERACTIVE_SPEED: f32 = 120.0; // 120 px/s minimum speed near boundaries

pub const TOUCHPAD_SCROLL_MULTIPLIER: f32 = 1.75;
pub const KINETIC_FRICTION_COEFFICIENT: f32 = 1.8; // Low friction rate for long, fluid inertia glide
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

#[derive(Debug, Clone, PartialEq)]
pub struct SmoothScroller {
    pub target_y: f32,
    pub is_animating: bool,

    mode: SmoothScrollMode,

    start_y: f32,
    start_time: Option<Instant>,
    last_tick: Option<Instant>,
    duration: Duration,
    pub last_applied_y: f32,
    pub velocity: f32,
}

impl Default for SmoothScroller {
    fn default() -> Self {
        Self {
            target_y: 0.0,
            is_animating: false,
            mode: SmoothScrollMode::Interactive,
            start_y: 0.0,
            start_time: None,
            last_tick: None,
            duration: Duration::from_millis(200),
            last_applied_y: 0.0,
            velocity: 0.0,
        }
    }
}

impl SmoothScroller {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn mode(&self) -> SmoothScrollMode {
        self.mode
    }

    #[inline]
    pub fn target_y(&self) -> f32 {
        self.target_y
    }

    pub fn start_interactive(&mut self, current_y: f32, delta: f32, max_y: f32) {
        let base = match self.mode {
            SmoothScrollMode::Interactive if self.is_animating => self.target_y,
            _ => current_y,
        };

        let target = clamp_target(base + delta, max_y);

        self.mode = SmoothScrollMode::Interactive;
        self.target_y = target;
        self.is_animating = (target - current_y).abs() > STOP_THRESHOLD;
        self.last_tick = None;
        self.last_applied_y = current_y;
    }

    pub fn start_navigation(&mut self, current_y: f32, target_y: f32, max_y: f32) {
        let clamped_target = clamp_target(target_y, max_y);
        let distance = (clamped_target - current_y).abs();
        if distance <= STOP_THRESHOLD {
            self.target_y = clamped_target;
            self.is_animating = false;
            self.mode = SmoothScrollMode::Interactive;
            self.last_applied_y = clamped_target;
            self.velocity = 0.0;
            return;
        }

        let duration_secs = navigation_duration(distance);
        self.mode = SmoothScrollMode::Navigation;
        self.start_y = current_y;
        self.target_y = clamped_target;
        self.start_time = None;
        self.last_tick = None;
        self.duration = Duration::from_secs_f32(duration_secs);
        self.is_animating = true;
        self.last_applied_y = current_y;
        self.velocity = 0.0;
    }

    pub fn start_kinetic(&mut self, current_y: f32, initial_velocity: f32, max_y: f32) {
        let clamped_v = initial_velocity.clamp(-KINETIC_MAX_VELOCITY, KINETIC_MAX_VELOCITY);
        if clamped_v.abs() < KINETIC_VELOCITY_CUTOFF {
            self.is_animating = false;
            self.velocity = 0.0;
            self.target_y = current_y;
            self.last_applied_y = current_y;
            return;
        }

        // Avoid starting kinetic if already against the corresponding boundary
        if (current_y <= 0.0 && clamped_v < 0.0) || (current_y >= max_y && clamped_v > 0.0) {
            self.is_animating = false;
            self.velocity = 0.0;
            self.target_y = current_y;
            self.last_applied_y = current_y;
            return;
        }

        self.mode = SmoothScrollMode::Kinetic;
        self.velocity = clamped_v;
        self.is_animating = true;
        self.start_time = None;
        self.last_tick = None;
        self.last_applied_y = current_y;
        self.target_y = current_y;
    }

    pub fn stop(&mut self, current_y: f32) {
        self.is_animating = false;
        self.velocity = 0.0;
        self.target_y = current_y;
        self.last_applied_y = current_y;
    }

    pub fn tick(&mut self, current_y: f32, now: Instant, max_y: f32) -> Option<f32> {
        if !self.is_animating {
            self.velocity = 0.0;
            return None;
        }

        // Reclamp target dynamically every tick in case layout changed
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
                    self.is_animating = false;
                    self.last_applied_y = self.target_y;
                    self.velocity = 0.0;
                    Some(self.target_y)
                } else {
                    let clamped_dt = dt.clamp(0.0, SMOOTH_SCROLL_MAX_DT);
                    let alpha = 1.0 - 2.0_f32.powf(-clamped_dt / INTERACTIVE_HALF_LIFE);
                    let mut step = (self.target_y - current_y) * alpha;
                    let min_step = MIN_INTERACTIVE_SPEED * clamped_dt;
                    if step.abs() < min_step {
                        step = step.signum() * min_step;
                    }
                    // Do not overshoot target
                    let next_y = if (self.target_y - current_y).abs() <= step.abs() {
                        self.target_y
                    } else {
                        clamp_target(current_y + step, max_y)
                    };
                    if dt > 0.0 {
                        self.velocity = (next_y - current_y) / dt;
                    }
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
                    self.is_animating = false;
                    self.mode = SmoothScrollMode::Interactive;
                    self.last_applied_y = self.target_y;
                    self.velocity = 0.0;
                    Some(self.target_y)
                } else {
                    let eased = ease_out_cubic(t);
                    let next_y =
                        clamp_target(self.start_y + (self.target_y - self.start_y) * eased, max_y);
                    if dt > 0.0 {
                        self.velocity = (next_y - current_y) / dt;
                    }
                    self.last_applied_y = next_y;
                    Some(next_y)
                }
            }
            SmoothScrollMode::Kinetic => {
                let clamped_dt = dt.clamp(0.0, SMOOTH_SCROLL_MAX_DT);
                let step = self.velocity * clamped_dt;
                let next_y = clamp_target(current_y + step, max_y);

                // Continuous friction decay: v(t) = v0 * e^(-friction * dt)
                self.velocity *= (-KINETIC_FRICTION_COEFFICIENT * clamped_dt).exp();

                // Stop if hitting top or bottom boundary or velocity drops below cutoff
                let hit_boundary = (next_y <= 0.0 && self.velocity <= 0.0)
                    || (next_y >= max_y && self.velocity >= 0.0);
                if hit_boundary || self.velocity.abs() < KINETIC_VELOCITY_CUTOFF {
                    self.is_animating = false;
                    self.velocity = 0.0;
                    self.mode = SmoothScrollMode::Interactive;
                    self.target_y = next_y;
                    self.last_applied_y = next_y;
                    Some(next_y)
                } else {
                    self.target_y = next_y;
                    self.last_applied_y = next_y;
                    Some(next_y)
                }
            }
        }
    }

    pub fn check_interruption(&mut self, actual_y: f32) -> bool {
        if !self.is_animating {
            return false;
        }
        let threshold = match self.mode {
            SmoothScrollMode::Kinetic => 40.0,
            _ => 3.0,
        };
        let diff = (actual_y - self.last_applied_y).abs();
        if diff > threshold {
            self.is_animating = false;
            self.velocity = 0.0;
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Legacy TouchpadGestureTracker maintained for backwards compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct TouchpadGestureTracker {
    last_samples: [(Instant, f32); Self::SAMPLE_CAPACITY],
    sample_count: usize,
    last_event_time: Option<Instant>,
}

impl Default for TouchpadGestureTracker {
    fn default() -> Self {
        let dummy = Instant::now();

        Self {
            last_samples: [(dummy, 0.0); Self::SAMPLE_CAPACITY],
            sample_count: 0,
            last_event_time: None,
        }
    }
}

impl TouchpadGestureTracker {
    pub const GESTURE_TIMEOUT_MS: u64 = 60;
    const MIN_SAMPLE_DT_SECS: f32 = 0.001;
    const MAX_SAMPLE_DT_SECS: f32 = 0.120;
    const SAMPLE_CAPACITY: usize = 16;
    const VELOCITY_WINDOW_SECS: f32 = 0.120;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.sample_count = 0;
        self.last_event_time = None;
    }

    #[inline]
    pub fn has_gesture_ended(&self, now: Instant) -> bool {
        self.last_event_time.is_some_and(|last| {
            now.saturating_duration_since(last) >= Duration::from_millis(Self::GESTURE_TIMEOUT_MS)
        })
    }

    pub fn push_sample(&mut self, now: Instant, delta_y: f32) {
        if self.has_gesture_ended(now) {
            self.reset();
        }

        let index = self.sample_count % Self::SAMPLE_CAPACITY;

        self.last_samples[index] = (now, delta_y);
        self.sample_count += 1;
        self.last_event_time = Some(now);
    }

    pub fn compute_velocity(&self) -> f32 {
        if self.sample_count < 2 {
            return 0.0;
        }

        let latest_idx = (self.sample_count - 1) % Self::SAMPLE_CAPACITY;
        let latest_time = self.last_samples[latest_idx].0;

        let count = self.sample_count.min(Self::SAMPLE_CAPACITY);
        let num_intervals = count - 1;

        let mut weighted_velocity_sum = 0.0;
        let mut total_weight = 0.0;

        for i in 0..num_intervals {
            let prev_idx = (self.sample_count - count + i) % Self::SAMPLE_CAPACITY;
            let curr_idx = (self.sample_count - count + i + 1) % Self::SAMPLE_CAPACITY;

            let prev_time = self.last_samples[prev_idx].0;
            let (curr_time, delta_y) = self.last_samples[curr_idx];

            let dt = curr_time.saturating_duration_since(prev_time).as_secs_f32();
            let age_from_latest = latest_time
                .saturating_duration_since(curr_time)
                .as_secs_f32();

            if age_from_latest > Self::VELOCITY_WINDOW_SECS {
                continue;
            }

            if (Self::MIN_SAMPLE_DT_SECS..=Self::MAX_SAMPLE_DT_SECS).contains(&dt) {
                let recency = 1.0 - (age_from_latest / Self::VELOCITY_WINDOW_SECS).clamp(0.0, 1.0);
                let weight = 0.3 + 0.7 * recency;

                let interval_velocity = delta_y / dt;
                weighted_velocity_sum += interval_velocity * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            weighted_velocity_sum / total_weight
        } else {
            0.0
        }
    }

    pub fn ended_velocity(&self, now: Instant) -> Option<f32> {
        if !self.has_gesture_ended(now) {
            return None;
        }

        Some(self.compute_velocity())
    }

    #[inline]
    pub fn has_velocity_samples(&self) -> bool {
        self.sample_count >= 2
    }

    #[inline]
    pub fn sample_count(&self) -> usize {
        self.sample_count.min(Self::SAMPLE_CAPACITY)
    }

    #[inline]
    pub fn last_event_time(&self) -> Option<Instant> {
        self.last_event_time
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScrollController {
    pub gesture: GestureStateMachine,
    pub velocity_estimator: VelocityEstimator,
    pub physics: ScrollPhysics,
    pub last_tick: Option<Instant>,
    pub position_y: f32,
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
    }

    #[inline]
    #[must_use]
    pub fn velocity(&self) -> f32 {
        self.physics.velocity
    }

    #[inline]
    #[must_use]
    pub fn state(&self) -> GestureState {
        self.gesture.state()
    }

    #[inline]
    #[must_use]
    pub fn is_animating(&self) -> bool {
        matches!(
            self.gesture.state(),
            GestureState::Flinging | GestureState::SpringBack
        )
    }

    pub fn stop(&mut self) {
        self.gesture.set_state(GestureState::Idle);
        self.physics.stop();
    }

    pub fn set_boundary_behavior(&mut self, behavior: BoundaryBehavior) {
        self.physics.boundary_behavior = behavior;
    }

    /// Primary input entrypoint. Accepts pure Rust `ScrollInput`.
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

                // Direct dragging displacement
                self.position_y = (self.position_y + delta_y).clamp(0.0, max_y);
            }
            ScrollInput::End { time } => {
                if self.gesture.state() == GestureState::Dragging {
                    let v0 = self.velocity_estimator.compute_velocity(time);
                    if v0.abs() >= MIN_FLING_VELOCITY {
                        self.gesture.set_state(GestureState::Flinging);
                        self.physics.start_fling(v0);
                    } else {
                        self.gesture.set_state(GestureState::Idle);
                        self.physics.stop();
                    }
                }
            }
            ScrollInput::Cancel => {
                self.gesture.handle_input(input);
                self.velocity_estimator.reset();
                self.physics.stop();
            }
        }
    }

    /// Update frame tick with current dynamic viewport extent.
    /// Returns `Some(new_position)` if actively animating, `None` if idle.
    pub fn update(&mut self, now: Instant, extent: ViewportExtent) -> Option<f32> {
        // Fallback check: If Dragging has stalled without End signal
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
                if !is_active {
                    self.gesture.set_state(GestureState::Idle);
                }
                Some(next_y)
            }
            GestureState::Idle | GestureState::Dragging => None,
        }
    }
}
