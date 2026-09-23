use std::time::{Duration, Instant};

pub const TELEMETRY_CAPACITY: usize = 128;

/// Non-invasive frame timing stamps for measuring app work duration and frame cadence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTiming {
    pub t_input_received: Option<Instant>,
    pub t_update_begin: Instant,
    pub t_update_end: Instant,
    pub t_draw_begin: Instant,
    pub t_draw_end: Instant,
    pub t_redraw_requested: Instant,
}

impl FrameTiming {
    /// Pure application CPU processing duration:
    /// `(t_update_end - t_update_begin) + (t_draw_end - t_draw_begin)`
    #[inline]
    #[must_use]
    pub fn app_processing_time(&self) -> Duration {
        let update_dur = self
            .t_update_end
            .saturating_duration_since(self.t_update_begin);
        let draw_dur = self.t_draw_end.saturating_duration_since(self.t_draw_begin);
        update_dur.saturating_add(draw_dur)
    }
}

/// Aggregated statistical summary for telemetry analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryStats {
    pub sample_count: usize,
    pub p50_frame_interval_ms: f32,
    pub p90_frame_interval_ms: f32,
    pub p99_frame_interval_ms: f32,
    pub p50_app_cost_ms: f32,
    pub p90_app_cost_ms: f32,
    pub p99_app_cost_ms: f32,
    pub missed_cadence_ratio: f32,
}

/// Lightweight fixed-capacity telemetry recorder.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollTelemetry {
    timings: [Option<FrameTiming>; TELEMETRY_CAPACITY],
    count: usize,
    last_frame_time: Option<Instant>,
    frame_intervals_ms: [f32; TELEMETRY_CAPACITY],
    app_costs_ms: [f32; TELEMETRY_CAPACITY],
}

impl Default for ScrollTelemetry {
    fn default() -> Self {
        Self {
            timings: [None; TELEMETRY_CAPACITY],
            count: 0,
            last_frame_time: None,
            frame_intervals_ms: [0.0; TELEMETRY_CAPACITY],
            app_costs_ms: [0.0; TELEMETRY_CAPACITY],
        }
    }
}

impl ScrollTelemetry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.timings = [None; TELEMETRY_CAPACITY];
        self.count = 0;
        self.last_frame_time = None;
        self.frame_intervals_ms = [0.0; TELEMETRY_CAPACITY];
        self.app_costs_ms = [0.0; TELEMETRY_CAPACITY];
    }

    #[inline]
    #[must_use]
    pub fn count(&self) -> usize {
        self.count.min(TELEMETRY_CAPACITY)
    }

    /// Record a completed frame's timing data.
    pub fn record_frame(&mut self, timing: FrameTiming) {
        let idx = self.count % TELEMETRY_CAPACITY;
        self.timings[idx] = Some(timing);

        let dt_ms = if let Some(last) = self.last_frame_time {
            timing
                .t_draw_end
                .saturating_duration_since(last)
                .as_secs_f32()
                * 1000.0
        } else {
            0.0
        };
        self.last_frame_time = Some(timing.t_draw_end);

        let app_cost_ms = timing.app_processing_time().as_secs_f32() * 1000.0;

        self.frame_intervals_ms[idx] = dt_ms;
        self.app_costs_ms[idx] = app_cost_ms;
        self.count += 1;
    }

    /// Calculate percentile metrics ($p_{50}, p_{90}, p_{99}$) and missed cadence ratio.
    #[must_use]
    pub fn compute_stats(&self, target_interval: Duration) -> TelemetryStats {
        let n = self.count();
        if n == 0 {
            return TelemetryStats {
                sample_count: 0,
                p50_frame_interval_ms: 0.0,
                p90_frame_interval_ms: 0.0,
                p99_frame_interval_ms: 0.0,
                p50_app_cost_ms: 0.0,
                p90_app_cost_ms: 0.0,
                p99_app_cost_ms: 0.0,
                missed_cadence_ratio: 0.0,
            };
        }

        let mut intervals: Vec<f32> = (0..n).map(|i| self.frame_intervals_ms[i]).collect();
        let mut costs: Vec<f32> = (0..n).map(|i| self.app_costs_ms[i]).collect();

        intervals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        costs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let target_ms = target_interval.as_secs_f32() * 1000.0;
        let missed_threshold = target_ms * 1.5;
        let missed_count = intervals
            .iter()
            .filter(|&&val| val > missed_threshold)
            .count();
        let missed_cadence_ratio = if n > 0 {
            missed_count as f32 / n as f32
        } else {
            0.0
        };

        TelemetryStats {
            sample_count: n,
            p50_frame_interval_ms: percentile(&intervals, 0.50),
            p90_frame_interval_ms: percentile(&intervals, 0.90),
            p99_frame_interval_ms: percentile(&intervals, 0.99),
            p50_app_cost_ms: percentile(&costs, 0.50),
            p90_app_cost_ms: percentile(&costs, 0.90),
            p99_app_cost_ms: percentile(&costs, 0.99),
            missed_cadence_ratio,
        }
    }
}

fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f32 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}
