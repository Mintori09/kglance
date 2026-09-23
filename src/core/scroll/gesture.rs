use std::time::Instant;

/// Maximum duration (e.g. 40ms) without samples in Dragging before considering the gesture ended.
pub const DRAG_FALLBACK_TIMEOUT_SECS: f32 = 0.040;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GestureState {
    #[default]
    Idle,
    Dragging,
    Flinging,
    SpringBack,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollInput {
    Motion {
        delta_x: f32,
        delta_y: f32,
        time: Instant,
    },
    End {
        time: Instant,
    },
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GestureStateMachine {
    state: GestureState,
    last_motion_time: Option<Instant>,
}

impl GestureStateMachine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn state(&self) -> GestureState {
        self.state
    }

    #[inline]
    #[must_use]
    pub fn last_motion_time(&self) -> Option<Instant> {
        self.last_motion_time
    }

    pub fn set_state(&mut self, state: GestureState) {
        self.state = state;
    }

    /// Process input and return whether state transitioned.
    pub fn handle_input(&mut self, input: ScrollInput) -> Option<GestureState> {
        match input {
            ScrollInput::Motion {
                delta_x,
                delta_y,
                time,
            } => {
                // If zero delta arrived, treat as instantaneous End (opportunistic)
                if delta_x.abs() <= f32::EPSILON && delta_y.abs() <= f32::EPSILON {
                    return self.handle_input(ScrollInput::End { time });
                }

                self.last_motion_time = Some(time);
                match self.state {
                    GestureState::Idle | GestureState::Flinging | GestureState::SpringBack => {
                        self.state = GestureState::Dragging;
                        Some(GestureState::Dragging)
                    }
                    GestureState::Dragging => None,
                }
            }
            ScrollInput::End { .. } => {
                match self.state {
                    GestureState::Dragging => {
                        // Will transition to Flinging or Idle depending on velocity checked by controller
                        None
                    }
                    _ => None,
                }
            }
            ScrollInput::Cancel => {
                self.state = GestureState::Idle;
                self.last_motion_time = None;
                Some(GestureState::Idle)
            }
        }
    }

    /// Tick-based robust fallback for when hardware/compositor did not send zero-delta end event.
    #[must_use]
    pub fn check_timeout(&mut self, now: Instant) -> bool {
        if self.state == GestureState::Dragging
            && let Some(last) = self.last_motion_time
            && now.saturating_duration_since(last).as_secs_f32() >= DRAG_FALLBACK_TIMEOUT_SECS
        {
            return true;
        }
        false
    }
}
