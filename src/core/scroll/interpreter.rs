use std::time::{Duration, Instant};

/// High-level device classification derived from input patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDeviceKind {
    #[default]
    Unknown,
    DiscreteWheel,
    Touchpad,
}

/// Abstracted intent produced by the interpreter for the controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollIntent {
    /// Discrete step (e.g. mouse wheel notch, keyboard line step)
    Impulse { delta_lines: f32 },
    /// Continuous 1:1 motion (touchpad finger dragging)
    ContinuousMotion {
        delta_x: f32,
        delta_y: f32,
        time: Instant,
    },
    /// Gesture release / lift-off
    GestureEnd { time: Instant },
    /// Cancellation / reset
    Cancel,
    /// Absolute navigation jump
    NavigateTo { target_y: f32 },
}

/// Heuristic stream interpreter classifying raw scroll deltas into intents.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollInterpreter {
    device_kind: ScrollDeviceKind,
    last_event_time: Option<Instant>,
    consecutive_pixel_events: usize,
    in_continuous_gesture: bool,
}

impl Default for ScrollInterpreter {
    fn default() -> Self {
        Self {
            device_kind: ScrollDeviceKind::Unknown,
            last_event_time: None,
            consecutive_pixel_events: 0,
            in_continuous_gesture: false,
        }
    }
}

impl ScrollInterpreter {
    pub const CONTINUOUS_INTERVAL_THRESHOLD: Duration = Duration::from_millis(80);

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn device_kind(&self) -> ScrollDeviceKind {
        self.device_kind
    }

    #[inline]
    #[must_use]
    pub fn is_in_continuous_gesture(&self) -> bool {
        self.in_continuous_gesture
    }

    pub fn reset(&mut self) {
        self.device_kind = ScrollDeviceKind::Unknown;
        self.last_event_time = None;
        self.consecutive_pixel_events = 0;
        self.in_continuous_gesture = false;
    }

    /// Interpret discrete lines input (mouse wheel / keyboard).
    pub fn interpret_lines(&mut self, y: f32) -> ScrollIntent {
        self.device_kind = ScrollDeviceKind::DiscreteWheel;
        self.in_continuous_gesture = false;
        self.consecutive_pixel_events = 0;
        ScrollIntent::Impulse { delta_lines: y }
    }

    /// Interpret high-resolution pixels input (touchpad / smooth wheel).
    pub fn interpret_pixels(&mut self, dx: f32, dy: f32, time: Instant) -> ScrollIntent {
        if let Some(last) = self.last_event_time {
            let dt = time.saturating_duration_since(last);
            if dt > Self::CONTINUOUS_INTERVAL_THRESHOLD {
                self.consecutive_pixel_events = 0;
            }
        }
        self.consecutive_pixel_events += 1;
        self.last_event_time = Some(time);

        // Classify as touchpad if high-frequency stream or fractional deltas
        if self.consecutive_pixel_events >= 2 || dy.fract().abs() > f32::EPSILON {
            self.device_kind = ScrollDeviceKind::Touchpad;
        }

        if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
            self.in_continuous_gesture = false;
            ScrollIntent::GestureEnd { time }
        } else {
            self.in_continuous_gesture = true;
            ScrollIntent::ContinuousMotion {
                delta_x: dx,
                delta_y: dy,
                time,
            }
        }
    }

    /// Check if an active continuous gesture timed out without an explicit end event.
    #[must_use]
    pub fn check_gesture_timeout(&mut self, now: Instant) -> Option<ScrollIntent> {
        if self.in_continuous_gesture
            && let Some(last) = self.last_event_time
            && now.saturating_duration_since(last) >= Duration::from_millis(40)
        {
            self.in_continuous_gesture = false;
            Some(ScrollIntent::GestureEnd { time: now })
        } else {
            None
        }
    }
}
