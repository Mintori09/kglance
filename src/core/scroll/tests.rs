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

    // Fast flick ends immediately after the last sample
    controller.handle_input(
        ScrollInput::End {
            time: last_time + Duration::from_millis(5),
        },
        extent,
    );

    assert_eq!(controller.state(), GestureState::Flinging);
    assert!(
        controller.velocity().abs() > 500.0,
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

    // Fingers were held stationary in pause_then_lift trace (dt=45ms with 0.0 delta)
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
fn physics_frame_rate_independence() {
    let mut c60 = ScrollController::new();
    let mut c120 = ScrollController::new();

    c60.set_boundary_behavior(BoundaryBehavior::Clamp);
    c120.set_boundary_behavior(BoundaryBehavior::Clamp);

    c60.physics.start_fling(2000.0);
    c120.physics.start_fling(2000.0);
    c60.gesture.set_state(GestureState::Flinging);
    c120.gesture.set_state(GestureState::Flinging);

    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    let start = Instant::now();
    // 60 Hz (16ms * 30 = 480ms)
    for i in 1..=30 {
        let now = start + Duration::from_millis(i * 16);
        c60.update(now, extent);
    }

    // 120 Hz (8ms * 60 = 480ms)
    for i in 1..=60 {
        let now = start + Duration::from_millis(i * 8);
        c120.update(now, extent);
    }

    assert!(
        (c60.position_y() - c120.position_y()).abs() < 15.0,
        "60Hz pos: {}, 120Hz pos: {}",
        c60.position_y(),
        c120.position_y()
    );
}

#[test]
fn spring_boundary_bounces_back_to_boundary() {
    let mut controller = ScrollController::new();
    controller.set_boundary_behavior(BoundaryBehavior::default());

    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    // Position past top boundary (overshoot)
    controller.set_position_y(-50.0);
    controller.gesture.set_state(GestureState::SpringBack);

    let start = Instant::now();
    // Simulate spring oscillations settling over 50 frames (~800ms)
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

#[test]
fn robust_fallback_triggers_when_gesture_stalls() {
    let mut controller = ScrollController::new();
    let extent = ViewportExtent {
        content_height: 5000.0,
        viewport_height: 800.0,
    };

    let t0 = Instant::now();
    controller.handle_input(
        ScrollInput::Motion {
            delta_x: 0.0,
            delta_y: -40.0,
            time: t0,
        },
        extent,
    );
    assert_eq!(controller.state(), GestureState::Dragging);

    // Stalled without End event for 45ms (exceeds DRAG_FALLBACK_TIMEOUT_SECS = 40ms)
    let t1 = t0 + Duration::from_millis(45);
    controller.update(t1, extent);

    // Should have automatically transitioned to End -> Flinging (since velocity > MIN_FLING_VELOCITY)
    // Wait, with 1 sample velocity is 0, so it transitions to Idle
    assert_eq!(controller.state(), GestureState::Idle);
}
