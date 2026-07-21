//! Startup phase profiler.
//!
//! Tracks high-resolution timestamps at each phase of the preview pipeline
//! from the moment `FileLoaded` is received to the first rendered frame.
//!
//! Phases:
//!   P0 → FileLoaded received (update() entry)
//!   P1 → handle_file_loaded() done (state ready, tasks dispatched)
//!   P2 → window::open Task resolved (WindowEvent::Opened received)
//!   P3 → view() first call (widget tree build start)
//!   P4 → view() first call complete (widget tree built)
//!
//! Enable by setting env var: KGLANCE_PROBE=1

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static PROBE_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn is_enabled() -> bool {
    PROBE_ENABLED.load(Ordering::Relaxed)
}

pub fn init() {
    if std::env::var("KGLANCE_PROBE").is_ok() {
        PROBE_ENABLED.store(true, Ordering::Relaxed);
        eprintln!("[PROBE] Startup profiler enabled");
    }
}

/// Per-request startup probe. Stores timestamps for each phase.
#[derive(Debug, Default)]
pub struct StartupProbe {
    /// T0: FileLoaded received by update()
    pub t0_file_loaded: Option<Instant>,
    /// T1: handle_file_loaded() returned — state+tasks ready
    pub t1_state_ready: Option<Instant>,
    /// T2: WindowEvent::Opened received — OS window exists
    pub t2_window_opened: Option<Instant>,
    /// T3: view() first call begins — widget tree build start
    pub t3_view_start: Option<Instant>,
    /// T4: view() first call done — widget tree built, frame will be submitted
    pub t4_view_done: Option<Instant>,
    /// Whether we've already printed the report for this request
    reported: bool,
}

impl StartupProbe {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn mark_file_loaded(&mut self) {
        if is_enabled() {
            self.t0_file_loaded = Some(Instant::now());
            self.reported = false;
            eprintln!("[PROBE] P0 FileLoaded received");
        }
    }

    pub fn mark_state_ready(&mut self) {
        if is_enabled() {
            self.t1_state_ready = Some(Instant::now());
            if let Some(t0) = self.t0_file_loaded {
                eprintln!(
                    "[PROBE] P1 state+tasks ready   +{:>8.3}ms (from P0)",
                    self.t1_state_ready
                        .unwrap()
                        .duration_since(t0)
                        .as_secs_f64()
                        * 1000.0
                );
            }
        }
    }

    pub fn mark_window_opened(&mut self) {
        if is_enabled() {
            self.t2_window_opened = Some(Instant::now());
            if let Some(t0) = self.t0_file_loaded {
                eprintln!(
                    "[PROBE] P2 WindowEvent::Opened +{:>8.3}ms (from P0)",
                    self.t2_window_opened
                        .unwrap()
                        .duration_since(t0)
                        .as_secs_f64()
                        * 1000.0
                );
            }
            if let Some(t1) = self.t1_state_ready {
                eprintln!(
                    "[PROBE]    window::open cost   +{:>8.3}ms (P1→P2)",
                    self.t2_window_opened
                        .unwrap()
                        .duration_since(t1)
                        .as_secs_f64()
                        * 1000.0
                );
            }
        }
    }

    pub fn mark_view_start(&mut self) {
        if is_enabled() && self.t3_view_start.is_none() {
            self.t3_view_start = Some(Instant::now());
            if let Some(t0) = self.t0_file_loaded {
                eprintln!(
                    "[PROBE] P3 view() first call   +{:>8.3}ms (from P0)",
                    self.t3_view_start.unwrap().duration_since(t0).as_secs_f64() * 1000.0
                );
            }
            if let Some(t2) = self.t2_window_opened {
                eprintln!(
                    "[PROBE]    surface→view gap     +{:>8.3}ms (P2→P3)",
                    self.t3_view_start.unwrap().duration_since(t2).as_secs_f64() * 1000.0
                );
            }
        }
    }

    pub fn mark_view_done(&mut self) {
        if is_enabled() && self.t3_view_start.is_some() && self.t4_view_done.is_none() {
            self.t4_view_done = Some(Instant::now());
            let build_cost = self
                .t4_view_done
                .unwrap()
                .duration_since(self.t3_view_start.unwrap())
                .as_secs_f64()
                * 1000.0;
            eprintln!("[PROBE] P4 widget tree built   +{build_cost:>8.3}ms (P3→P4, tree cost)");
            self.print_report();
        }
    }

    fn print_report(&mut self) {
        if self.reported {
            return;
        }
        self.reported = true;
        let Some(t0) = self.t0_file_loaded else {
            return;
        };
        let total = self
            .t4_view_done
            .unwrap_or_else(Instant::now)
            .duration_since(t0)
            .as_secs_f64()
            * 1000.0;

        eprintln!("[PROBE] ─────────────────────────────────────────");
        eprintln!("[PROBE] STARTUP LATENCY BREAKDOWN");
        eprintln!("[PROBE] ─────────────────────────────────────────");

        let p0_to_p1 = self
            .t1_state_ready
            .map(|t| t.duration_since(t0).as_secs_f64() * 1000.0);
        let p1_to_p2 = self
            .t1_state_ready
            .zip(self.t2_window_opened)
            .map(|(t1, t2)| t2.duration_since(t1).as_secs_f64() * 1000.0);
        let p2_to_p3 = self
            .t2_window_opened
            .zip(self.t3_view_start)
            .map(|(t2, t3)| t3.duration_since(t2).as_secs_f64() * 1000.0);
        let p3_to_p4 = self
            .t3_view_start
            .zip(self.t4_view_done)
            .map(|(t3, t4)| t4.duration_since(t3).as_secs_f64() * 1000.0);

        let fmt = |v: Option<f64>| -> String {
            v.map(|x| format!("{x:>8.3}ms"))
                .unwrap_or_else(|| "  (none) ".to_string())
        };

        eprintln!("[PROBE]  P0→P1  handle_file_loaded   {}", fmt(p0_to_p1));
        eprintln!("[PROBE]  P1→P2  window::open (Winit)  {}", fmt(p1_to_p2));
        eprintln!("[PROBE]  P2→P3  surface commit→view   {}", fmt(p2_to_p3));
        eprintln!("[PROBE]  P3→P4  widget tree build      {}", fmt(p3_to_p4));
        eprintln!("[PROBE] ─────────────────────────────────────────");
        eprintln!("[PROBE]  TOTAL  P0→first frame         {:>8.3}ms", total);
        eprintln!("[PROBE] ─────────────────────────────────────────");

        // Identify the dominant bottleneck
        let phases = [
            ("handle_file_loaded", p0_to_p1),
            ("window::open (Winit/compositor)", p1_to_p2),
            ("surface commit → first view()", p2_to_p3),
            ("widget tree build", p3_to_p4),
        ];
        if let Some((name, cost)) = phases
            .iter()
            .filter_map(|(n, v)| v.map(|x| (*n, x)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        {
            eprintln!("[PROBE] ▶ BOTTLENECK: {name} ({cost:.1}ms)");
        }
        eprintln!("[PROBE] ─────────────────────────────────────────");
    }
}
