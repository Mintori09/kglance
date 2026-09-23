use std::time::Instant;

/// Capacity of the circular sample buffer (~120ms at high polling rates).
pub const SAMPLE_CAPACITY: usize = 16;
/// Sliding window duration for velocity estimation.
pub const VELOCITY_WINDOW_SECS: f32 = 0.120;
/// Inactive duration after which velocity is strictly zeroed out (staleness micro-brake).
pub const STALENESS_TIMEOUT_SECS: f32 = 0.040;
/// Minimum time interval between samples to avoid division by zero.
pub const MIN_SAMPLE_DT_SECS: f32 = 0.001;
/// Maximum time interval between successive samples before gesture discontinuity.
pub const MAX_SAMPLE_DT_SECS: f32 = 0.100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub time: Instant,
    pub delta_y: f32,
}

/// Dynamic velocity estimator maintaining sample ring buffer and staleness guards.
#[derive(Debug, Clone, PartialEq)]
pub struct VelocityEstimator {
    samples: [Option<Sample>; SAMPLE_CAPACITY],
    count: usize,
    last_event_time: Option<Instant>,
    last_meaningful_time: Option<Instant>,
}

impl Default for VelocityEstimator {
    fn default() -> Self {
        Self {
            samples: [None; SAMPLE_CAPACITY],
            count: 0,
            last_event_time: None,
            last_meaningful_time: None,
        }
    }
}

impl VelocityEstimator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.samples = [None; SAMPLE_CAPACITY];
        self.count = 0;
        self.last_event_time = None;
        self.last_meaningful_time = None;
    }

    #[inline]
    #[must_use]
    pub fn last_event_time(&self) -> Option<Instant> {
        self.last_event_time
    }

    #[inline]
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.count.min(SAMPLE_CAPACITY)
    }

    pub fn push_sample(&mut self, time: Instant, delta_y: f32) {
        if let Some(last) = self.last_event_time {
            let dt = time.saturating_duration_since(last).as_secs_f32();
            if dt > MAX_SAMPLE_DT_SECS {
                self.reset();
            }
        }

        let idx = self.count % SAMPLE_CAPACITY;
        self.samples[idx] = Some(Sample { time, delta_y });
        self.count += 1;
        self.last_event_time = Some(time);

        if delta_y.abs() > 0.5 {
            self.last_meaningful_time = Some(time);
        }
    }

    /// Checks if user paused/held fingers before releasing.
    #[must_use]
    pub fn is_stale(&self, now: Instant) -> bool {
        if let Some(meaningful_time) = self.last_meaningful_time {
            now.saturating_duration_since(meaningful_time).as_secs_f32() >= STALENESS_TIMEOUT_SECS
        } else {
            true
        }
    }

    /// Estimate release velocity using Weighted Least Squares (WLSQ) over recent sample history.
    /// Exponential recency weight: w_i = e^(-lambda * dt).
    #[must_use]
    pub fn compute_velocity(&self, now: Instant) -> f32 {
        if self.count < 2 || self.is_stale(now) {
            return 0.0;
        }

        let valid_count = self.sample_count();
        let num_intervals = valid_count - 1;
        let mut weighted_velocity_sum = 0.0;
        let mut total_weight = 0.0;

        let latest_idx = (self.count - 1) % SAMPLE_CAPACITY;
        let Some(latest) = self.samples[latest_idx] else {
            return 0.0;
        };

        for i in 0..num_intervals {
            let prev_idx = (self.count - valid_count + i) % SAMPLE_CAPACITY;
            let curr_idx = (self.count - valid_count + i + 1) % SAMPLE_CAPACITY;

            let (Some(prev), Some(curr)) = (self.samples[prev_idx], self.samples[curr_idx]) else {
                continue;
            };

            let dt = curr.time.saturating_duration_since(prev.time).as_secs_f32();
            let age_from_latest = latest
                .time
                .saturating_duration_since(curr.time)
                .as_secs_f32();

            if age_from_latest > VELOCITY_WINDOW_SECS {
                continue;
            }

            if (MIN_SAMPLE_DT_SECS..=MAX_SAMPLE_DT_SECS).contains(&dt) {
                // Exponential weight: 1.0 at latest, decaying towards window boundary
                let lambda = 12.0; // decay constant
                let weight = (-lambda * age_from_latest).exp();

                let interval_velocity = curr.delta_y / dt;
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
}
