use crate::app::test_util::{markdown_content, test_app};

#[test]
fn test_scroll_page_triggers_smooth_navigation() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 3000.0;
    state.scroll_y = 100.0;

    let _ = app.scroll_page(0.85);

    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert!(state.smooth_scroll.is_animating);
    assert_eq!(
        state.smooth_scroll.mode(),
        crate::core::SmoothScrollMode::Navigation
    );
    // 100.0 + 800.0 * 0.85 = 780.0
    assert_eq!(state.smooth_scroll.target_y(), 780.0);
}

#[test]
fn test_scroll_half_page_triggers_smooth_navigation() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 3000.0;
    state.scroll_y = 100.0;

    let _ = app.scroll_page(0.5);

    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert!(state.smooth_scroll.is_animating);
    // 100.0 + 800.0 * 0.5 = 500.0
    assert_eq!(state.smooth_scroll.target_y(), 500.0);
}

#[test]
fn test_scroll_page_up_clamps_at_top() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 3000.0;
    state.scroll_y = 200.0;

    let _ = app.scroll_page(-0.85);

    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert!(state.smooth_scroll.is_animating);
    assert_eq!(state.smooth_scroll.target_y(), 0.0);
}

#[test]
fn test_scroll_page_accumulates_rapid_keystrokes() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 1000.0;
    state.total_content_height = 5000.0;
    state.scroll_y = 0.0;

    // First PageDown: target = 0 + 850 = 850
    let _ = app.scroll_page(0.85);
    assert_eq!(
        crate::features::markdown::update::active_markdown_state(&app)
            .smooth_scroll
            .target_y(),
        850.0
    );

    // While in flight (simulated current scroll_y = 200), second PageDown arrives
    crate::features::markdown::update::active_markdown_state_mut(&mut app).scroll_y = 200.0;
    let _ = app.scroll_page(0.85);

    // Target accumulates to 850 + 850 = 1700
    assert_eq!(
        crate::features::markdown::update::active_markdown_state(&app)
            .smooth_scroll
            .target_y(),
        1700.0
    );
}

#[test]
fn test_snap_to_top_and_bottom_triggers_smooth_navigation() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 3000.0;
    state.scroll_y = 500.0;

    // Snap to top (gg or Home)
    let _ = app.snap_to_top();
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.smooth_scroll.target_y(), 0.0);
    assert_eq!(
        state.smooth_scroll.mode(),
        crate::core::SmoothScrollMode::Navigation
    );
    assert!(state.smooth_scroll.is_animating);

    // Snap to bottom (G or End)
    let _ = app.snap_to_bottom();
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.smooth_scroll.target_y(), 2200.0); // 3000.0 - 800.0
    assert_eq!(
        state.smooth_scroll.mode(),
        crate::core::SmoothScrollMode::Navigation
    );
    assert!(state.smooth_scroll.is_animating);
}

#[test]
fn test_scroll_down_to_end_with_j() {
    let md = std::fs::read_to_string("testing-file/markdown.md").unwrap();
    let mut app = test_app(Some(markdown_content(&md)));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    let total_h = state.total_content_height;
    let max_scroll = total_h - 800.0;

    let mut now = std::time::Instant::now();
    for _ in 0..200 {
        let _ = app.scroll_by(super::SCROLL_LINE_AMOUNT);
        for _ in 0..10 {
            now += std::time::Duration::from_millis(16);
            let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
        }
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert!(
        (state.scroll_y - max_scroll).abs() < 1.0,
        "Expected scroll_y to reach max_scroll {max_scroll}, got {}",
        state.scroll_y
    );
}

#[test]
fn test_snap_to_top_and_bottom_reaches_boundaries() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 3000.0;
    state.scroll_y = 1500.0;

    // Snap to bottom (G)
    let _ = app.snap_to_bottom();
    let mut now = std::time::Instant::now();
    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, 2200.0);
    assert!(!state.smooth_scroll.is_animating);

    // Snap to top (gg)
    let _ = app.snap_to_top();
    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, 0.0);
    assert!(!state.smooth_scroll.is_animating);
}

#[test]
fn test_large_file_snap_to_top_and_bottom_optical_glide() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 200_000.0;
    state.scroll_y = 150_000.0;

    // Snap to top (gg) from 150,000px
    let _ = app.snap_to_top();
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, 1600.0);
    assert_eq!(state.smooth_scroll.target_y(), 0.0);
    assert!(state.smooth_scroll.is_animating);

    let mut now = std::time::Instant::now();
    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, 0.0);
    assert!(!state.smooth_scroll.is_animating);

    // Snap to bottom (G) from 0.0
    let _ = app.snap_to_bottom();
    let max_scroll = 200_000.0 - 800.0;
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, max_scroll - 1600.0);
    assert_eq!(state.smooth_scroll.target_y(), max_scroll);
    assert!(state.smooth_scroll.is_animating);

    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, max_scroll);
    assert!(!state.smooth_scroll.is_animating);
}

#[test]
fn test_gg_and_g_key_press_with_scrolled_events() {
    let mut app = test_app(Some(markdown_content("# Heading\n\nContent paragraph")));
    let state = crate::features::markdown::update::active_markdown_state_mut(&mut app);
    state.viewport_height = 800.0;
    state.total_content_height = 3000.0;
    state.scroll_y = 0.0;

    let g_key = iced::keyboard::Key::Character("g".into());
    let cap_g_key = iced::keyboard::Key::Character("G".into());
    let empty_mod = iced::keyboard::Modifiers::default();

    // 1. Press 'G' to go to bottom
    let _ = app.handle_key_pressed(cap_g_key, empty_mod);
    let mut now = std::time::Instant::now();
    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
        let current_y = crate::features::markdown::update::active_markdown_state(&app).scroll_y;
        // Simulate Iced emitting Scrolled event on each frame
        let _ = crate::features::markdown::update::handle_markdown_scrolled(
            &mut app, current_y, 800.0, 3000.0,
        );
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, 2200.0);
    assert!(!state.smooth_scroll.is_animating);

    // 2. Press 'g' once -> sets pending_g
    let _ = app.handle_key_pressed(g_key.clone(), empty_mod);
    assert!(app.pending_g);

    // 3. Press 'g' twice -> triggers snap_to_top
    let _ = app.handle_key_pressed(g_key, empty_mod);
    assert!(!app.pending_g);

    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::markdown::update::handle_smooth_scroll_tick(&mut app, now);
        let current_y = crate::features::markdown::update::active_markdown_state(&app).scroll_y;
        let _ = crate::features::markdown::update::handle_markdown_scrolled(
            &mut app, current_y, 800.0, 3000.0,
        );
    }
    let state = crate::features::markdown::update::active_markdown_state(&app);
    assert_eq!(state.scroll_y, 0.0);
    assert!(!state.smooth_scroll.is_animating);
}

#[test]
fn test_text_scroll_page_and_vim_navigation() {
    use crate::app::test_util::text_content;

    let code = (0..200)
        .map(|i| format!("fn line_{i}() {{ println!(\"Hello {i}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = test_app(Some(text_content(&code, "rs")));
    app.state.text.viewport_height = 800.0;
    app.state.text.total_content_height = 4000.0;
    app.state.text.scroll_y = 0.0;

    // 1. Scroll page down (PageDown or 'd')
    let _ = app.scroll_page(0.5);
    assert!(app.state.text.smooth_scroll.is_animating);
    assert_eq!(app.state.text.smooth_scroll.target_y(), 400.0);

    // 2. Step smooth scroll animation
    let mut now = std::time::Instant::now();
    for _ in 0..30 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
    }
    assert_eq!(app.state.text.scroll_y, 400.0);
    assert!(!app.state.text.smooth_scroll.is_animating);

    // 3. Snap to bottom (G)
    let _ = app.snap_to_bottom();
    assert_eq!(app.state.text.smooth_scroll.target_y(), 3200.0); // 4000.0 - 800.0
    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
    }
    assert_eq!(app.state.text.scroll_y, 3200.0);

    // 4. Snap to top (gg)
    let _ = app.snap_to_top();
    assert_eq!(app.state.text.smooth_scroll.target_y(), 0.0);
    for _ in 0..50 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
    }
    assert_eq!(app.state.text.scroll_y, 0.0);
}

#[test]
fn test_text_touchpad_velocity_kinetic_scroll() {
    use crate::app::test_util::text_content;

    let code = (0..500)
        .map(|i| format!("fn line_{i}() {{ println!(\"Code Line {i}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = test_app(Some(text_content(&code, "rs")));
    app.state.text.viewport_height = 800.0;
    app.state.text.total_content_height = 10000.0;
    app.state.text.scroll_y = 0.0;

    // 1. First swipe
    let _ = crate::features::text::update::handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -20.0 },
    );
    assert_eq!(app.state.text.scroll_y, 50.0);
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // Simulate intermediate UI scrollable viewport event
    let _ = crate::features::text::update::handle_text_scrolled_values(
        &mut app, 50.0, 800.0, 800.0, 10000.0,
    );
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // 2. Second rapid swipe
    std::thread::sleep(std::time::Duration::from_millis(15));
    let _ = crate::features::text::update::handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -30.0 },
    );
    assert_eq!(app.state.text.scroll_y, 125.0);
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // Simulate intermediate UI scrollable viewport event
    let _ = crate::features::text::update::handle_text_scrolled_values(
        &mut app, 125.0, 800.0, 800.0, 10000.0,
    );
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // 3. Opportunistic zero delta on finger release
    let _ = crate::features::text::update::handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 },
    );
    assert!(app.state.text.scroll_controller.is_animating());
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Flinging
    );

    // 4. Step physics frames
    let mut now = std::time::Instant::now();
    for _ in 0..60 {
        now += std::time::Duration::from_millis(16);
        let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
    }

    // Must glide significantly further than swipe displacement
    assert!(app.state.text.scroll_y > 125.0 + 100.0);
}

#[test]
fn test_text_touchpad_hold_finger_does_not_fling() {
    use crate::app::test_util::text_content;

    let code = (0..500)
        .map(|i| format!("fn line_{i}() {{ println!(\"Code Line {i}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = test_app(Some(text_content(&code, "rs")));
    app.state.text.viewport_height = 800.0;
    app.state.text.total_content_height = 10000.0;
    app.state.text.scroll_y = 0.0;

    // User swipes with finger
    let _ = crate::features::text::update::handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -20.0 },
    );
    assert_eq!(app.state.text.scroll_y, 50.0);
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // User keeps finger held on touchpad (45ms pass without new events)
    let now = std::time::Instant::now() + std::time::Duration::from_millis(45);
    let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);

    // Must settle to Idle without flinging
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Idle
    );
    assert!(!app.state.text.scroll_controller.is_animating());
    assert_eq!(app.state.text.scroll_y, 50.0);
}

#[test]
fn test_text_touchpad_touch_interrupts_ongoing_fling() {
    use crate::app::test_util::text_content;

    let code = (0..500)
        .map(|i| format!("fn line_{i}() {{ println!(\"Code Line {i}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = test_app(Some(text_content(&code, "rs")));
    app.state.text.viewport_height = 800.0;
    app.state.text.total_content_height = 10000.0;
    app.state.text.scroll_y = 100.0;

    // Start a kinetic fling
    app.state
        .text
        .scroll_controller
        .start_kinetic(100.0, 2000.0, 9200.0);
    assert!(app.state.text.scroll_controller.is_animating());
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Flinging
    );

    // Advance 16ms of fling
    let now1 = std::time::Instant::now() + std::time::Duration::from_millis(16);
    let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now1);
    let y_during_fling = app.state.text.scroll_y;
    assert!(y_during_fling > 100.0);

    // User touches touchpad to stop fling
    let _ = crate::features::text::update::handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Pixels { x: 0.0, y: -0.5 },
    );
    assert!(!app.state.text.scroll_controller.is_animating());
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );

    // User keeps finger held on touchpad for 45ms
    let now2 = std::time::Instant::now() + std::time::Duration::from_millis(50);
    let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now2);
    assert_eq!(
        app.state.text.scroll_controller.state(),
        crate::core::scroll::GestureState::Idle
    );
    assert!(!app.state.text.scroll_controller.is_animating());
}

#[test]
fn test_text_gt_shortcut_toggles_outline() {
    use crate::app::test_util::text_content;

    let code = "fn main() {\n    println!(\"Hello\");\n}\n";
    let mut app = test_app(Some(text_content(code, "rs")));
    assert!(!app.state.text.outline_visible);

    let g_key = iced::keyboard::Key::Character("g".into());
    let t_key = iced::keyboard::Key::Character("t".into());
    let modifiers = iced::keyboard::Modifiers::default();

    let _ = app.handle_scroll_shortcuts(&g_key, modifiers);
    assert!(app.pending_g);

    let task = app.handle_scroll_shortcuts(&t_key, modifiers);
    assert!(task.is_some());
    assert!(!app.pending_g);

    let _ = app.update(crate::app::messages::TextMsg::ToggleOutline.into());
    assert!(app.state.text.outline_visible);
}

#[test]
fn test_ge_shortcut_toggles_file_tree() {
    use crate::app::test_util::text_content;

    let code = "fn main() {}\n";
    let mut app = test_app(Some(text_content(code, "rs")));
    assert!(!app.state.file_tree.visible);

    let g_key = iced::keyboard::Key::Character("g".into());
    let e_key = iced::keyboard::Key::Character("e".into());
    let modifiers = iced::keyboard::Modifiers::default();

    let _ = app.handle_scroll_shortcuts(&g_key, modifiers);
    assert!(app.pending_g);

    let task = app.handle_scroll_shortcuts(&e_key, modifiers);
    assert!(task.is_some());
    assert!(!app.pending_g);

    app.state.text.scroll_y = 120.0;
    let _ = app.update(crate::app::messages::FileTreeMsg::Toggle.into());
    assert!(app.state.file_tree.visible);
    assert_eq!(app.state.text.scroll_y, 120.0);

    // Toggle again closes it and still retains scroll_y
    let _ = app.update(crate::app::messages::FileTreeMsg::Toggle.into());
    assert!(!app.state.file_tree.visible);
    assert_eq!(app.state.text.scroll_y, 120.0);
}

#[test]
fn test_text_wheel_scroll_interrupts_active_kinetic_scroll() {
    use crate::app::test_util::text_content;

    let code = (0..500)
        .map(|i| format!("fn line_{i}() {{ println!(\"Code Line {i}\"); }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = test_app(Some(text_content(&code, "rs")));
    app.state.text.viewport_height = 800.0;
    app.state.text.total_content_height = 10000.0;
    app.state.text.scroll_y = 100.0;

    // 1. Start a kinetic fling
    app.state
        .text
        .scroll_controller
        .start_kinetic(100.0, 1800.0, 9200.0);
    assert!(app.state.text.scroll_controller.is_animating());

    // 2. User turns the mouse wheel (Lines)
    let delta = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 };
    let _ = crate::features::text::update::handle_wheel_scrolled(&mut app, delta);

    // Active kinetic fling must be cancelled
    assert!(!app.state.text.scroll_controller.is_animating());
    assert!(app.state.text.smooth_scroll.is_animating);

    // 3. Frame tick must advance smooth scroll towards wheel target, NOT kinetic position
    let now = std::time::Instant::now();
    let _ = crate::features::text::update::handle_smooth_scroll_tick(&mut app, now);
    assert!(!app.state.text.scroll_controller.is_animating());
    assert!(app.state.text.scroll_y > 100.0);
}

#[test]
fn test_pdf_wheel_scroll_interrupts_active_kinetic_scroll() {
    let mut app = test_app(None);
    app.state.pdf.viewport_height = 800.0;
    app.state.pdf.total_content_height = 8000.0;
    app.state.pdf.scroll_y = 150.0;

    // 1. Start kinetic fling
    app.state
        .pdf
        .scroll_controller
        .start_kinetic(150.0, 2000.0, 7200.0);
    assert!(app.state.pdf.scroll_controller.is_animating());

    // 2. Mouse wheel turns
    let delta = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -2.0 };
    let _ = crate::features::pdf::update::handle_wheel_scrolled(&mut app, delta);

    // Active kinetic fling must be cancelled
    assert!(!app.state.pdf.scroll_controller.is_animating());
    assert!(app.state.pdf.smooth_scroll.is_animating);

    // 3. Frame tick must advance smooth scroll, NOT kinetic fling
    let now = std::time::Instant::now();
    let _ = crate::features::pdf::update::handle_smooth_scroll_tick(&mut app, now);
    assert!(!app.state.pdf.scroll_controller.is_animating());
    assert!(app.state.pdf.scroll_y > 150.0);
}

#[test]
fn test_pdf_wheel_scrolled_when_smooth_scroll_disabled() {
    let mut app = test_app(None);
    app.state.pdf.viewport_height = 800.0;
    app.state.pdf.total_content_height = 8000.0;
    app.state.pdf.scroll_y = 150.0;
    app.state.scroll_config.smooth_scroll_enabled = false;
    app.state.apply_scroll_controllers();

    // Scroll down 1 notch (step = -( -1.0 ) * 120.0 = 120.0) -> scroll_y: 150.0 -> 270.0
    let delta = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 };
    let _ = crate::features::pdf::update::handle_wheel_scrolled(&mut app, delta);

    assert_eq!(app.state.pdf.scroll_y, 270.0);
    assert!(!app.state.pdf.smooth_scroll.is_animating());
    assert_eq!(app.state.pdf.smooth_scroll.position_y(), 270.0);

    // Clamping: Scroll up past the top boundary clamps to 0.0
    let delta_up_excess = iced::mouse::ScrollDelta::Lines { x: 0.0, y: 10.0 };
    let _ = crate::features::pdf::update::handle_wheel_scrolled(&mut app, delta_up_excess);
    assert_eq!(app.state.pdf.scroll_y, 0.0);
    assert!(!app.state.pdf.smooth_scroll.is_animating());
    assert_eq!(app.state.pdf.smooth_scroll.position_y(), 0.0);
}
