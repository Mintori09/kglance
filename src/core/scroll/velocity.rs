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

/// Strategy for estimating release velocity from gesture history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EstimatorStrategy {
    Linear,
    #[default]
    Polynomial2,
    WeightedRecent,
}

/// Trait for regression algorithms estimating velocity from a series of samples.
pub trait VelocityEstimatorAlgorithm: Send + Sync {
    fn estimate(&self, samples: &[Sample], now: Instant) -> f32;
}

/// Linear Regression (LSQ1): fits $y(t) = a t + b$. Velocity is slope $a$.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LinearRegression;

impl VelocityEstimatorAlgorithm for LinearRegression {
    fn estimate(&self, samples: &[Sample], _now: Instant) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }

        let t_latest = samples.last().unwrap().time;
        let mut t_pts = Vec::with_capacity(samples.len());
        let mut y_pts = Vec::with_capacity(samples.len());

        let mut cum_y = 0.0;
        for s in samples {
            // Relative time to latest: t' in [-T, 0]
            let t = -t_latest.saturating_duration_since(s.time).as_secs_f32();
            cum_y += s.delta_y;
            t_pts.push(t);
            y_pts.push(cum_y);
        }

        let n = t_pts.len() as f32;
        let mean_t = t_pts.iter().sum::<f32>() / n;
        let mean_y = y_pts.iter().sum::<f32>() / n;

        let mut num = 0.0;
        let mut den = 0.0;
        for i in 0..t_pts.len() {
            let dt = t_pts[i] - mean_t;
            let dy = y_pts[i] - mean_y;
            num += dt * dy;
            den += dt * dt;
        }

        if den.abs() > 1e-7 { num / den } else { 0.0 }
    }
}

/// Degree 2 Polynomial Regression (LSQ2 - AOSP Touch Default):
/// fits $y(t') = a (t')^2 + b t' + c$ where $t' = t - t_{\text{latest}} \le 0$.
/// Instantaneous velocity at latest sample ($t'=0$) is directly $y'(0) = b$.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PolynomialRegression2;

impl VelocityEstimatorAlgorithm for PolynomialRegression2 {
    fn estimate(&self, samples: &[Sample], now: Instant) -> f32 {
        if samples.len() < 3 {
            return LinearRegression.estimate(samples, now);
        }

        let t_latest = samples.last().unwrap().time;
        let mut t_pts = Vec::with_capacity(samples.len());
        let mut y_pts = Vec::with_capacity(samples.len());

        let mut cum_y = 0.0;
        for s in samples {
            let t = -t_latest.saturating_duration_since(s.time).as_secs_f32();
            cum_y += s.delta_y;
            t_pts.push(t);
            y_pts.push(cum_y);
        }

        let mut s0 = 0.0_f32; // sum(1) = n
        let mut s1 = 0.0_f32; // sum(t')
        let mut s2 = 0.0_f32; // sum(t'^2)
        let mut s3 = 0.0_f32; // sum(t'^3)
        let mut s4 = 0.0_f32; // sum(t'^4)
        let mut sy0 = 0.0_f32; // sum(y)
        let mut sy1 = 0.0_f32; // sum(t'*y)
        let mut sy2 = 0.0_f32; // sum(t'^2*y)

        for i in 0..t_pts.len() {
            let t = t_pts[i];
            let y = y_pts[i];
            let t2 = t * t;
            let t3 = t2 * t;
            let t4 = t3 * t;

            s0 += 1.0;
            s1 += t;
            s2 += t2;
            s3 += t3;
            s4 += t4;
            sy0 += y;
            sy1 += t * y;
            sy2 += t2 * y;
        }

        // Solve 3x3 normal equations for [c, b, a]^T:
        // [s0 s1 s2] [c]   [sy0]
        // [s1 s2 s3] [b] = [sy1]
        // [s2 s3 s4] [a]   [sy2]
        let det = s0 * (s2 * s4 - s3 * s3) - s1 * (s1 * s4 - s3 * s2) + s2 * (s1 * s3 - s2 * s2);

        if det.abs() < 1e-12 {
            return LinearRegression.estimate(samples, now);
        }

        // Cramer's rule for b (column 1):
        // [s0 sy0 s2]
        // [s1 sy1 s3]
        // [s2 sy2 s4]
        let det_b =
            s0 * (sy1 * s4 - s3 * sy2) - sy0 * (s1 * s4 - s3 * s2) + s2 * (s1 * sy2 - sy1 * s2);

        det_b / det
    }
}

/// Weighted Least Squares / Recency Weighted Average (WLSQ2 / WLSQ):
/// Exponential recency weight: $w_i = e^{-\lambda \cdot \Delta t}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedRecentRegression {
    pub lambda: f32,
}

impl Default for WeightedRecentRegression {
    fn default() -> Self {
        Self { lambda: 12.0 }
    }
}

impl VelocityEstimatorAlgorithm for WeightedRecentRegression {
    fn estimate(&self, samples: &[Sample], _now: Instant) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }

        let latest = samples.last().unwrap();
        let mut weighted_velocity_sum = 0.0;
        let mut total_weight = 0.0;

        for i in 0..(samples.len() - 1) {
            let prev = &samples[i];
            let curr = &samples[i + 1];

            let dt = curr.time.saturating_duration_since(prev.time).as_secs_f32();
            let age_from_latest = latest
                .time
                .saturating_duration_since(curr.time)
                .as_secs_f32();

            if age_from_latest > VELOCITY_WINDOW_SECS {
                continue;
            }

            if (MIN_SAMPLE_DT_SECS..=MAX_SAMPLE_DT_SECS).contains(&dt) {
                let weight = (-self.lambda * age_from_latest).exp();
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

/// Dynamic velocity estimator maintaining sample ring buffer and staleness guards.
#[derive(Debug, Clone, PartialEq)]
pub struct VelocityEstimator {
    samples: [Option<Sample>; SAMPLE_CAPACITY],
    count: usize,
    last_event_time: Option<Instant>,
    last_meaningful_time: Option<Instant>,
    strategy: EstimatorStrategy,
}

impl Default for VelocityEstimator {
    fn default() -> Self {
        Self {
            samples: [None; SAMPLE_CAPACITY],
            count: 0,
            last_event_time: None,
            last_meaningful_time: None,
            strategy: EstimatorStrategy::Polynomial2,
        }
    }
}

impl VelocityEstimator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_strategy(strategy: EstimatorStrategy) -> Self {
        Self {
            strategy,
            ..Self::default()
        }
    }

    pub fn set_strategy(&mut self, strategy: EstimatorStrategy) {
        self.strategy = strategy;
    }

    #[inline]
    #[must_use]
    pub fn strategy(&self) -> EstimatorStrategy {
        self.strategy
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

    /// Extract ordered recent samples inside the sliding window.
    fn collect_window_samples(&self) -> Vec<Sample> {
        let valid_count = self.sample_count();
        if valid_count < 2 {
            return Vec::new();
        }

        let latest_idx = (self.count - 1) % SAMPLE_CAPACITY;
        let Some(latest) = self.samples[latest_idx] else {
            return Vec::new();
        };

        let mut out = Vec::with_capacity(valid_count);
        for i in 0..valid_count {
            let idx = (self.count - valid_count + i) % SAMPLE_CAPACITY;
            if let Some(sample) = self.samples[idx] {
                let age = latest
                    .time
                    .saturating_duration_since(sample.time)
                    .as_secs_f32();
                if age <= VELOCITY_WINDOW_SECS {
                    out.push(sample);
                }
            }
        }
        out
    }

    /// Estimate release velocity using the selected strategy over recent sample history.
    #[must_use]
    pub fn compute_velocity(&self, now: Instant) -> f32 {
        if self.count < 2 || self.is_stale(now) {
            return 0.0;
        }

        let window_samples = self.collect_window_samples();
        if window_samples.len() < 2 {
            return 0.0;
        }

        match self.strategy {
            EstimatorStrategy::Linear => LinearRegression.estimate(&window_samples, now),
            EstimatorStrategy::Polynomial2 => PolynomialRegression2.estimate(&window_samples, now),
            EstimatorStrategy::WeightedRecent => {
                WeightedRecentRegression::default().estimate(&window_samples, now)
            }
        }
    }
}
