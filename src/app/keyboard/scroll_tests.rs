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

    let g_key: iced::keyboard::Key = iced::keyboard::Key::Character("g".into());
    let t_key: iced::keyboard::Key = iced::keyboard::Key::Character("t".into());
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
fn test_pdf_gt_shortcut_toggles_outline_sidebar() {
    let mut app = test_app(Some(crate::core::PreviewData::Pdf {
        page_count: 5,
        current_page: 0,
        data: Vec::new(),
        width: 100,
        height: 100,
        outline: vec![crate::features::pdf::parser::PdfTocEntry {
            title: "Chapter 1".to_string(),
            page: 0,
            level: 1,
        }],
        page_dimensions: Vec::new(),
        first_page_text: None,
    }));
    app.state.pdf.page_count = 5;
    app.state.pdf.outline = vec![crate::features::pdf::parser::PdfTocEntry {
        title: "Chapter 1".to_string(),
        page: 0,
        level: 1,
    }];
    assert!(!app.state.pdf.sidebar_visible);

    let t_key: iced::keyboard::Key = iced::keyboard::Key::Character("t".into());
    let modifiers = iced::keyboard::Modifiers::default();

    // 1. Manually set pending_g = true (as if 'g' was pressed or sequence initiated)
    app.pending_g = true;

    // 2. Press 't' with pending_g -> toggle_sidebar returns Task
    let task = app.handle_scroll_shortcuts(&t_key, modifiers);
    assert!(task.is_some());
    assert!(!app.pending_g);

    // 3. Process the resulting SidebarToggled message
    let _ = app.update(crate::app::messages::PdfMsg::SidebarToggled.into());
    assert!(app.state.pdf.sidebar_visible);
    assert_eq!(app.state.pdf.sidebar_mode, crate::core::PdfSidebarMode::Toc);

    // 4. Toggle again -> closes sidebar
    app.pending_g = true;
    let task = app.handle_scroll_shortcuts(&t_key, modifiers);
    assert!(task.is_some());
    let _ = app.update(crate::app::messages::PdfMsg::SidebarToggled.into());
    assert!(!app.state.pdf.sidebar_visible);
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

#[test]
fn test_pdf_ctrl_g_and_gg_behavior() {
    let mut app = test_app(Some(crate::core::PreviewData::Pdf {
        page_count: 10,
        current_page: 2,
        data: Vec::new(),
        width: 100,
        height: 100,
        outline: Vec::new(),
        page_dimensions: Vec::new(),
        first_page_text: None,
    }));
    app.state.pdf.page_count = 10;
    app.state.pdf.current_page = 2;
    app.state
        .pdf
        .visible_page
        .store(2, std::sync::atomic::Ordering::Relaxed);
    assert!(!app.state.pdf.goto_page.is_visible);

    let g_key = iced::keyboard::Key::Character("g".into());
    let default_modifiers = iced::keyboard::Modifiers::default();
    let ctrl_modifiers = iced::keyboard::Modifiers::CTRL;

    // 1. Press 'g' once in PDF -> sets pending_g, does NOT open goto page
    let _ = app.handle_key_pressed(g_key.clone(), default_modifiers);
    assert!(app.pending_g);
    assert!(!app.state.pdf.goto_page.is_visible);

    // 2. Press 'g' again in PDF -> gg snaps to top and clears pending_g
    let _ = app.handle_key_pressed(g_key.clone(), default_modifiers);
    assert!(!app.pending_g);
    assert!(!app.state.pdf.goto_page.is_visible);

    // 3. Press Ctrl+G in PDF -> opens goto page dialog
    let _ = app.handle_key_pressed(g_key, ctrl_modifiers);
    assert!(app.state.pdf.goto_page.is_visible);
    assert_eq!(app.state.pdf.goto_page.input_value, "3");

    // 4. Press Escape closes dialog
    let esc_key = iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape);
    let _ = app.handle_key_pressed(esc_key, default_modifiers);
    assert!(!app.state.pdf.goto_page.is_visible);
}

#[test]
fn test_notebook_page_down_and_up_triggers_smooth_navigation() {
    let notebook_data = crate::features::notebook::types::NotebookData {
        cells: vec![
            crate::features::notebook::types::NotebookCell::Code {
                source: "print('hello world')\n".repeat(30),
                execution_count: Some(1),
                outputs: vec![],
                language: Some("python".into()),
            },
            crate::features::notebook::types::NotebookCell::Markdown {
                source: "# Section 1\nSome description text.\n".repeat(20),
                blocks: vec![],
            },
        ],
        kernel_language: "python".to_string(),
        total_cells: 2,
        fallback_notice: None,
    };
    let preview = crate::core::PreviewData::Notebook(notebook_data);
    let mut app = test_app(Some(preview.clone()));
    preview.populate_state(&mut app.state);
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 3000.0;

    let default_modifiers = iced::keyboard::Modifiers::default();
    let pgdn = iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown);
    let pgup = iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp);

    // 1. PageDown triggers smooth navigation
    let _ = app.handle_key_pressed(pgdn, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);
    assert!(app.is_active_smooth_scrolling());
    assert_eq!(app.state.markdown.smooth_scroll.target_y(), 800.0 * 0.85);

    // 2. SmoothScrollTick advances position
    let t0 = std::time::Instant::now();
    let _ = app.update(crate::app::messages::Message::SmoothScrollTick(t0));
    let t1 = t0 + std::time::Duration::from_millis(50);
    let _ = app.update(crate::app::messages::Message::SmoothScrollTick(t1));
    assert!(app.state.markdown.scroll_y > 0.0);

    // 3. PageUp triggers smooth navigation back towards top
    let _ = app.handle_key_pressed(pgup, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);
    assert!(app.is_active_smooth_scrolling());
}

#[test]
fn test_notebook_vim_motions_and_home_end() {
    let notebook_data = crate::features::notebook::types::NotebookData {
        cells: vec![crate::features::notebook::types::NotebookCell::Code {
            source: "x = 1\n".repeat(50),
            execution_count: Some(1),
            outputs: vec![],
            language: Some("python".into()),
        }],
        kernel_language: "python".to_string(),
        total_cells: 1,
        fallback_notice: None,
    };
    let preview = crate::core::PreviewData::Notebook(notebook_data);
    let mut app = test_app(Some(preview.clone()));
    preview.populate_state(&mut app.state);
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 4000.0;

    let default_modifiers = iced::keyboard::Modifiers::default();
    let j_key = iced::keyboard::Key::Character("j".into());
    let k_key = iced::keyboard::Key::Character("k".into());
    let d_key = iced::keyboard::Key::Character("d".into());
    let u_key = iced::keyboard::Key::Character("u".into());
    let end_key = iced::keyboard::Key::Named(iced::keyboard::key::Named::End);
    let home_key = iced::keyboard::Key::Named(iced::keyboard::key::Named::Home);

    // j: scroll down 80px
    let _ = app.handle_key_pressed(j_key, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);
    assert!(app.is_active_smooth_scrolling());

    // d: scroll half page down (400px)
    app.state.markdown.smooth_scroll.stop(100.0);
    app.state.markdown.scroll_y = 100.0;
    let _ = app.handle_key_pressed(d_key, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);
    assert_eq!(app.state.markdown.smooth_scroll.target_y(), 500.0);

    // u: scroll half page up (-400px clamped)
    let _ = app.handle_key_pressed(u_key, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);

    // k: scroll up
    let _ = app.handle_key_pressed(k_key, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);

    // End: snap to bottom
    let _ = app.handle_key_pressed(end_key, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);

    // Home: snap to top
    let _ = app.handle_key_pressed(home_key, default_modifiers);
    assert!(app.state.markdown.smooth_scroll.is_animating);
}

#[test]
fn test_spreadsheet_page_up_down_navigates_grid_instead_of_smooth_scroll() {
    let sheet = crate::features::sheet::types::SheetInfo {
        name: "Sheet1".to_string(),
        headers: vec!["A".to_string(), "B".to_string()],
        columns: vec![
            crate::features::sheet::types::ColumnMeta {
                name: "A".to_string(),
                col_type: crate::features::sheet::types::ColumnType::Text,
                width: 100.0,
            },
            crate::features::sheet::types::ColumnMeta {
                name: "B".to_string(),
                col_type: crate::features::sheet::types::ColumnType::Text,
                width: 100.0,
            },
        ],
        rows: (0..50)
            .map(|i| vec![format!("A{i}"), format!("B{i}")])
            .collect(),
    };
    let mut app = test_app(Some(crate::core::PreviewData::Spreadsheet {
        sheets: vec![sheet],
        active_sheet: 0,
    }));
    app.state.spreadsheet.display_indices = (0..50).collect();
    app.state.spreadsheet.prefix_widths = vec![0.0, 100.0, 200.0];
    app.state.spreadsheet.prefix_heights = (0..=50).map(|i| i as f32 * 26.0).collect();
    app.state.spreadsheet.selection = Some(crate::features::sheet::types::CellRange::single(
        crate::features::sheet::types::CellCoord { row: 0, col: 0 },
    ));

    let default_modifiers = iced::keyboard::Modifiers::default();
    let pgdn = iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown);
    let pgup = iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp);

    // PageDown should navigate grid cell row + 15, not trigger smooth scroll
    let _ = app.handle_key_pressed(pgdn.clone(), default_modifiers);
    assert!(!app.state.spreadsheet.smooth_scroll.is_animating);
    assert_eq!(
        app.state.spreadsheet.selection.unwrap().start,
        crate::features::sheet::types::CellCoord { row: 15, col: 0 }
    );

    // PageUp should navigate grid cell row - 15, back to 0
    let _ = app.handle_key_pressed(pgup, default_modifiers);
    assert!(!app.state.spreadsheet.smooth_scroll.is_animating);
    assert_eq!(
        app.state.spreadsheet.selection.unwrap().start,
        crate::features::sheet::types::CellCoord { row: 0, col: 0 }
    );

    // Shift + PageDown should extend selection to row 15
    let shift_modifiers = iced::keyboard::Modifiers::SHIFT;
    let _ = app.handle_key_pressed(pgdn, shift_modifiers);
    assert!(!app.state.spreadsheet.smooth_scroll.is_animating);
    assert_eq!(
        app.state.spreadsheet.selection.unwrap().start,
        crate::features::sheet::types::CellCoord { row: 0, col: 0 }
    );
    assert_eq!(
        app.state.spreadsheet.selection.unwrap().end,
        crate::features::sheet::types::CellCoord { row: 15, col: 0 }
    );
}

#[test]
fn test_notebook_populate_state_clears_markdown_offsets() {
    let mut app = test_app(None);
    // Simulate dirty Markdown state from previously viewed markdown file
    app.state.markdown.block_y_offsets = vec![100.0; 250];
    app.state.markdown.scroll_y = 500.0;
    app.state.markdown.toc_visible = true;

    let notebook_data = crate::features::notebook::types::NotebookData {
        cells: vec![crate::features::notebook::types::NotebookCell::Code {
            source: "a = 1".into(),
            execution_count: Some(1),
            outputs: vec![],
            language: Some("python".into()),
        }],
        kernel_language: "python".to_string(),
        total_cells: 1,
        fallback_notice: None,
    };
    let preview = crate::core::PreviewData::Notebook(notebook_data);
    preview.populate_state(&mut app.state);

    assert_eq!(app.state.markdown.scroll_y, 0.0);
    assert!(app.state.markdown.block_y_offsets.is_empty());
    assert!(app.state.markdown.block_layouts.is_empty());
    assert!(!app.state.markdown.toc_visible);
    assert!(app.state.markdown.total_content_height >= 800.0);
}
