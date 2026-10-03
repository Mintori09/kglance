use crate::app::test_util::test_app;
use crate::core::scroll::GestureState;
use crate::features::json::update::{handle_smooth_scroll_tick, handle_wheel_scrolled};
use iced::mouse::ScrollDelta;

fn create_json_test_app() -> crate::app::KglanceApp {
    let mut app = test_app(Some(crate::core::PreviewData::Json {
        nodes: vec![],
        content: "{}".to_string(),
        pretty: "{}".to_string(),
        has_parse_error: false,
    }));
    app.state.json.tree_mode = true;
    app.state.json.viewport_height = 800.0;
    app.state.json.total_content_height = 4000.0;
    app.state.json.scroll_y = 100.0;
    app
}

#[test]
fn test_json_tree_handle_wheel_scrolled_lines_and_pixels() {
    let mut app = create_json_test_app();

    // 1. Lines: Mouse wheel discrete step (-1 notch down)
    let delta_lines = ScrollDelta::Lines { x: 0.0, y: -1.0 };
    let _ = handle_wheel_scrolled(&mut app, delta_lines);
    assert_eq!(app.state.json.smooth_scroll.target_y, 220.0);
    assert!(app.state.json.smooth_scroll.is_animating);

    // 2. Pixels: Touchpad gesture produces direct displacement
    let delta_pixels = ScrollDelta::Pixels { x: 0.0, y: -20.0 };
    let _ = handle_wheel_scrolled(&mut app, delta_pixels);
    assert_eq!(app.state.json.scroll_y, 150.0);
    assert_eq!(
        app.state.json.scroll_controller.state(),
        GestureState::Dragging
    );

    // 3. Fast swipe in rapid succession
    std::thread::sleep(std::time::Duration::from_millis(15));
    let delta_pixels_fast = ScrollDelta::Pixels { x: 0.0, y: -30.0 };
    let _ = handle_wheel_scrolled(&mut app, delta_pixels_fast);
    assert_eq!(app.state.json.scroll_y, 225.0);
    assert_eq!(
        app.state.json.scroll_controller.state(),
        GestureState::Dragging
    );

    // 4. Instant fling on gesture release (zero delta)
    let delta_pixels_zero = ScrollDelta::Pixels { x: 0.0, y: 0.0 };
    let _ = handle_wheel_scrolled(&mut app, delta_pixels_zero);
    assert!(app.state.json.scroll_controller.is_animating());
    assert_eq!(
        app.state.json.scroll_controller.state(),
        GestureState::Flinging
    );

    // 5. Ctrl held down suppresses wheel scrolling
    app.ctrl_held = true;
    let old_target = app.state.json.smooth_scroll.target_y;
    let _ = handle_wheel_scrolled(&mut app, delta_lines);
    assert_eq!(app.state.json.smooth_scroll.target_y, old_target);
}

#[test]
fn test_json_tree_handle_smooth_scroll_tick() {
    let mut app = create_json_test_app();
    app.state
        .json
        .smooth_scroll
        .start_navigation(100.0, 500.0, 3000.0);

    let now1 = std::time::Instant::now();
    let _ = handle_smooth_scroll_tick(&mut app, now1);

    let now2 = now1 + std::time::Duration::from_millis(100);
    let _ = handle_smooth_scroll_tick(&mut app, now2);
    assert!(app.state.json.scroll_y > 100.0);
}
