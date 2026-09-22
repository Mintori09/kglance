use std::time::{Duration, Instant};

pub const INTERACTIVE_HALF_LIFE: f32 = 0.08; // 80 ms half-life
pub const SMOOTH_SCROLL_MAX_DT: f32 = 0.05; // 50 ms cap to avoid quantum leaps on lag
pub const STOP_THRESHOLD: f32 = 0.5; // 0.5px threshold for stop
pub const WHEEL_SCROLL_VIEWPORT_FRACTION: f32 = 0.08;
pub const MIN_INTERACTIVE_SPEED: f32 = 120.0; // 120 px/s minimum speed near boundaries

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

    pub fn tick(&mut self, current_y: f32, now: Instant, max_y: f32) -> Option<f32> {
        if !self.is_animating {
            self.velocity = 0.0;
            return None;
        }

        // Reclamp target dynamically every tick in case layout changed
        self.target_y = clamp_target(self.target_y, max_y);

        let dt = self
            .last_tick
            .replace(now)
            .map(|last| now.saturating_duration_since(last).as_secs_f32())
            .unwrap_or(1.0 / 125.0);

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
        }
    }

    pub fn check_interruption(&mut self, actual_y: f32) -> bool {
        if !self.is_animating {
            return false;
        }
        // Cancel if external interaction (e.g. user dragging the scrollbar) deviates
        // by more than the threshold in either direction.
        let diff = (actual_y - self.last_applied_y).abs();
        if diff > 3.0 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_scroll_is_frame_rate_independent() {
        let target = 1000.0;
        let max_y = 2000.0;

        let mut a = SmoothScroller::default();
        let mut b = SmoothScroller::default();

        a.start_navigation(0.0, target, max_y);
        b.start_navigation(0.0, target, max_y);

        let start = Instant::now();

        let mut y_a = 0.0;
        let mut y_b = 0.0;

        // 60 Hz
        for i in 1..=30 {
            let now = start + Duration::from_millis(i * 16);
            if let Some(y) = a.tick(y_a, now, max_y) {
                y_a = y;
            }
        }

        // 30 Hz
        for i in 1..=15 {
            let now = start + Duration::from_millis(i * 33);
            if let Some(y) = b.tick(y_b, now, max_y) {
                y_b = y;
            }
        }

        assert!((y_a - y_b).abs() < 2.0, "60Hz={y_a}, 30Hz={y_b}");
    }

    #[test]
    fn interactive_scroll_is_frame_rate_independent() {
        let delta = 800.0;
        let max_y = 2000.0;

        let mut a = SmoothScroller::default();
        let mut b = SmoothScroller::default();

        a.start_interactive(0.0, delta, max_y);
        b.start_interactive(0.0, delta, max_y);

        let start = Instant::now();

        let mut y_a = 0.0;
        let mut y_b = 0.0;

        // 60 Hz (16ms)
        for i in 1..=20 {
            let now = start + Duration::from_millis(i * 16);
            if let Some(y) = a.tick(y_a, now, max_y) {
                y_a = y;
            }
        }

        // 30 Hz (33ms)
        for i in 1..=10 {
            let now = start + Duration::from_millis(i * 33);
            if let Some(y) = b.tick(y_b, now, max_y) {
                y_b = y;
            }
        }

        // Exponential decay over ~320ms should produce closely matching positions
        assert!(
            (y_a - y_b).abs() < 25.0,
            "Interactive 60Hz={y_a}, 30Hz={y_b}, diff={}",
            (y_a - y_b).abs()
        );
    }

    #[test]
    fn interactive_scroll_accumulates_wheel_input() {
        let mut scroller = SmoothScroller::default();

        scroller.start_interactive(0.0, 100.0, 2000.0);
        scroller.start_interactive(5.0, 100.0, 2000.0);
        scroller.start_interactive(10.0, 100.0, 2000.0);

        assert_eq!(scroller.target_y(), 300.0);
    }

    #[test]
    fn interactive_scroll_clamps_target() {
        let mut scroller = SmoothScroller::default();

        scroller.start_interactive(100.0, -500.0, 400.0);
        assert_eq!(scroller.target_y(), 0.0);

        scroller.start_interactive(100.0, 1000.0, 400.0);
        assert_eq!(scroller.target_y(), 400.0);
    }

    #[test]
    fn navigation_interrupted_by_wheel() {
        let mut scroller = SmoothScroller::default();
        scroller.start_navigation(0.0, 1000.0, 2000.0);
        assert_eq!(scroller.mode(), SmoothScrollMode::Navigation);

        // While navigation is in flight at current_y = 200.0, a new wheel event occurs
        scroller.start_interactive(200.0, 50.0, 2000.0);
        assert_eq!(scroller.mode(), SmoothScrollMode::Interactive);
        assert_eq!(scroller.target_y(), 250.0);
    }

    #[test]
    fn check_interruption_behavior() {
        let mut scroller = SmoothScroller::default();
        scroller.start_navigation(0.0, 500.0, 1000.0);
        scroller.last_applied_y = 100.0;

        // 1. Programmatic scroll or minor rounding/layout diff (<= 3px): NOT cancelled
        assert!(!scroller.check_interruption(101.5));
        assert!(scroller.is_animating);

        assert!(!scroller.check_interruption(98.5));
        assert!(scroller.is_animating);

        // 2. User drags scrollbar backwards significantly (> 3px): CANCELLED
        assert!(scroller.check_interruption(90.0));
        assert!(!scroller.is_animating);

        // 3. User drags scrollbar forwards significantly (> 3px): CANCELLED
        let mut scroller_fwd = SmoothScroller::default();
        scroller_fwd.start_navigation(0.0, 500.0, 1000.0);
        scroller_fwd.last_applied_y = 100.0;
        assert!(scroller_fwd.check_interruption(120.0));
        assert!(!scroller_fwd.is_animating);
    }

    #[test]
    fn navigation_duration_bounds() {
        assert_eq!(navigation_duration(0.0), 0.14);
        assert_eq!(navigation_duration(500.0), 0.16);
        assert_eq!(navigation_duration(50_000.0), 0.35);
    }

    #[test]
    fn max_scroll_y_calculation() {
        assert_eq!(max_scroll_y(2500.0, 800.0), 1700.0);
        assert_eq!(max_scroll_y(500.0, 800.0), 0.0);
    }

    #[test]
    fn interactive_scroll_maintains_speed_near_boundaries() {
        let mut scroller = SmoothScroller::default();
        // Only 15px from top boundary (0.0)
        scroller.start_interactive(15.0, -80.0, 1000.0);
        assert_eq!(scroller.target_y(), 0.0);

        let t0 = Instant::now();
        // First tick sets last_tick
        let _ = scroller.tick(15.0, t0, 1000.0);
        // Second tick after 16ms measures actual dt = 16ms
        let t1 = t0 + Duration::from_millis(16);
        let next_y = scroller.tick(15.0, t1, 1000.0).expect("should tick");
        let moved = (15.0 - next_y).abs();
        assert!(moved >= 1.5, "Expected step >= 1.5px, got {moved}");
    }
}
