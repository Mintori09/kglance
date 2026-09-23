use super::*;
use std::time::{Duration, Instant};

#[derive(serde::Deserialize)]
struct TraceEvent {
    dt_us: u32,
    dx: f32,
    dy: f32,
}

fn run_trace(json_str: &str) -> (ScrollController, Instant) {
    let events: Vec<TraceEvent> = serde_json::from_str(json_str).expect("parse trace JSON");
    let mut controller = ScrollController::new();
    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    let mut current_time = Instant::now();
    for ev in events {
        current_time += Duration::from_micros(ev.dt_us as u64);
        controller.handle_input(
            ScrollInput::Motion {
                delta_x: ev.dx,
                delta_y: ev.dy,
                time: current_time,
            },
            extent,
        );
    }

    (controller, current_time)
}

#[test]
fn replay_fast_flick_triggers_flinging() {
    let json = include_str!("../../../tests/traces/fast_flick.json");
    let (mut controller, last_time) = run_trace(json);
    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    controller.handle_input(
        ScrollInput::End {
            time: last_time + Duration::from_millis(5),
        },
        extent,
    );

    assert_eq!(controller.state(), GestureState::Flinging);
    assert!(
        controller.velocity().abs() > 400.0,
        "Velocity: {}",
        controller.velocity()
    );
}

#[test]
fn replay_pause_then_lift_produces_zero_velocity() {
    let json = include_str!("../../../tests/traces/pause_then_lift.json");
    let (mut controller, last_time) = run_trace(json);
    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    controller.handle_input(
        ScrollInput::End {
            time: last_time + Duration::from_millis(5),
        },
        extent,
    );

    assert_eq!(controller.state(), GestureState::Idle);
    assert_eq!(controller.velocity(), 0.0);
}

#[test]
fn replay_slow_crawl_produces_no_fling() {
    let json = include_str!("../../../tests/traces/slow_crawl.json");
    let (mut controller, last_time) = run_trace(json);
    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    controller.handle_input(
        ScrollInput::End {
            time: last_time + Duration::from_millis(5),
        },
        extent,
    );

    assert_eq!(controller.state(), GestureState::Idle);
    assert_eq!(controller.velocity(), 0.0);
}

#[test]
fn replay_jittery_swipe_estimates_smooth_velocity() {
    let json = include_str!("../../../tests/traces/jittery_swipe.json");
    let (mut controller, last_time) = run_trace(json);
    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    controller.handle_input(
        ScrollInput::End {
            time: last_time + Duration::from_millis(5),
        },
        extent,
    );

    assert_eq!(controller.state(), GestureState::Flinging);
    assert!(controller.velocity().abs() > 200.0);
}

#[test]
fn test_all_velocity_estimators_with_synthetic_trajectory() {
    let now = Instant::now();
    let dt = Duration::from_millis(10);

    // Synthetic constant velocity motion: v = 500 px/s (5 px per 10ms)
    let samples: Vec<velocity::Sample> = (0..10)
        .map(|i| velocity::Sample {
            time: now + dt * i as u32,
            delta_y: 5.0,
        })
        .collect();

    let query_time = now + Duration::from_millis(95);

    let lsq1 = LinearRegression.estimate(&samples, query_time);
    let lsq2 = PolynomialRegression2.estimate(&samples, query_time);
    let wlsq = WeightedRecentRegression::default().estimate(&samples, query_time);

    assert!(
        (lsq1 - 500.0).abs() < 1.0,
        "LSQ1 expected ~500.0, got {lsq1}"
    );
    assert!(
        (lsq2 - 500.0).abs() < 5.0,
        "LSQ2 expected ~500.0, got {lsq2}"
    );
    assert!(
        (wlsq - 500.0).abs() < 1.0,
        "WLSQ expected ~500.0, got {wlsq}"
    );
}

#[test]
fn test_closed_form_fling_trajectory_math() {
    let start_time = Instant::now();
    let state = ScrollState::Fling {
        start_time,
        start_pos: 100.0,
        v0: 1800.0,
        friction: 1.8,
        cutoff_time: Duration::from_secs(2),
    };

    // At t = 0
    let (p0, active0) = state.sample(start_time);
    assert_eq!(p0, 100.0);
    assert!(active0);

    // At t = 1.0s: pos = 100 + (1800/1.8) * (1 - e^-1.8) = 100 + 1000 * 0.8347 = 934.7
    let (p1, active1) = state.sample(start_time + Duration::from_secs(1));
    let expected = 100.0 + (1800.0 / 1.8) * (1.0 - (-1.8_f32).exp());
    assert!((p1 - expected).abs() < 0.1);
    assert!(active1);

    // At t = 3.0s (after cutoff): pos = 100 + 1000 = 1100.0, active = false
    let (p_end, active_end) = state.sample(start_time + Duration::from_secs(3));
    assert_eq!(p_end, 1100.0);
    assert!(!active_end);
}

#[test]
fn test_scroll_telemetry_recording_and_stats() {
    let mut telemetry = ScrollTelemetry::new();
    let now = Instant::now();

    for i in 0..10 {
        let t_base = now + Duration::from_millis(i * 16);
        let timing = FrameTiming {
            t_input_received: Some(t_base),
            t_update_begin: t_base + Duration::from_micros(100),
            t_update_end: t_base + Duration::from_micros(600),
            t_draw_begin: t_base + Duration::from_micros(700),
            t_draw_end: t_base + Duration::from_micros(1500),
            t_redraw_requested: t_base + Duration::from_micros(1600),
        };
        telemetry.record_frame(timing);
    }

    assert_eq!(telemetry.count(), 10);
    let stats = telemetry.compute_stats(Duration::from_millis(16));
    assert_eq!(stats.sample_count, 10);
    assert!(stats.p50_app_cost_ms > 0.0);
    assert!(stats.p50_app_cost_ms < 2.0);
    assert_eq!(stats.missed_cadence_ratio, 0.0);
}

#[test]
fn test_scroll_interpreter_classification() {
    let mut interpreter = ScrollInterpreter::new();
    let now = Instant::now();

    // Discrete wheel
    let intent_lines = interpreter.interpret_lines(-1.0);
    assert_eq!(intent_lines, ScrollIntent::Impulse { delta_lines: -1.0 });
    assert_eq!(interpreter.device_kind(), ScrollDeviceKind::DiscreteWheel);

    // Continuous trackpad motion
    let intent_pixels = interpreter.interpret_pixels(0.0, -15.5, now);
    assert_eq!(
        intent_pixels,
        ScrollIntent::ContinuousMotion {
            delta_x: 0.0,
            delta_y: -15.5,
            time: now
        }
    );
    assert_eq!(interpreter.device_kind(), ScrollDeviceKind::Touchpad);
    assert!(interpreter.is_in_continuous_gesture());

    // Gesture timeout
    let timeout_intent = interpreter.check_gesture_timeout(now + Duration::from_millis(50));
    assert!(matches!(
        timeout_intent,
        Some(ScrollIntent::GestureEnd { .. })
    ));
    assert!(!interpreter.is_in_continuous_gesture());
}

#[test]
fn spring_boundary_bounces_back_to_boundary() {
    let mut controller = ScrollController::new();
    controller.set_boundary_behavior(BoundaryBehavior::default());

    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    controller.set_position_y(-50.0);
    controller.gesture.set_state(GestureState::SpringBack);

    let start = Instant::now();
    for i in 1..=80 {
        let now = start + Duration::from_millis(i * 10);
        controller.update(now, extent);
    }

    assert!(
        controller.position_y().abs() < 0.5,
        "Settled pos: {}",
        controller.position_y()
    );
    assert_eq!(controller.state(), GestureState::Idle);
    assert_eq!(controller.velocity(), 0.0);
}
