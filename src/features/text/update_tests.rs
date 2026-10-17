use super::*;
use crate::features::text::create_text_state;
use crate::ui::theme::AppTheme;

#[test]
fn test_update_tokens_for_viewport_with_word_wrap() {
    // Create 200 long lines that wrap to 5 rows each when viewport is narrow
    let long_line = "let very_long_variable_name_with_lots_of_data = \"some long string value\";\n";
    let content: String = (0..200).map(|_| long_line).collect();

    let mut state = create_text_state(
        content,
        "rs",
        14.0,
        true, // word_wrap enabled
        AppTheme::Dark,
        200.0, // narrow viewport forcing wrapping
    );

    // Scroll to mid-document (e.g. scroll_y = 5000.0)
    let scroll_y = 5000.0;
    state.viewport_height = 600.0;
    update_tokens_for_viewport(&mut state, scroll_y, AppTheme::Dark);

    // Verify that tokens were fetched for the logical lines at this scroll position
    let (start_vrow, end_vrow) = state.display_map.compute_visible_range(scroll_y, 600.0, 0);
    let (expected_start_line, _) = state.display_map.visual_row_to_line(start_vrow);
    let (expected_end_line, _) = state
        .display_map
        .visual_row_to_line(end_vrow.saturating_sub(1));

    let token_start = state.cached_tokens_start_line;
    let token_end = token_start + state.cached_tokens.len();

    assert!(
        token_start <= expected_start_line,
        "token_start ({token_start}) should be <= expected_start_line ({expected_start_line})"
    );
    assert!(
        token_end >= expected_end_line,
        "token_end ({token_end}) should be >= expected_end_line ({expected_end_line})"
    );
    assert!(
        !state.cached_tokens.is_empty(),
        "cached_tokens must not be empty"
    );
}

#[test]
fn test_text_drag_and_auto_scroll() {
    let mut app = KglanceApp::default();
    let content: String = (0..100)
        .map(|i| format!("line {i} with some content\n"))
        .collect();
    app.state.text = create_text_state(content, "txt", 14.0, false, AppTheme::Dark, 800.0);
    app.state.text.viewport_height = 400.0;

    let start_pos = crate::ui::components::code_viewer::TextPosition::new(10, 2);
    let _ = handle_selection_drag_started(&mut app, start_pos);
    assert!(app.state.text.is_dragging_selection);
    assert_eq!(app.state.text.drag_start, Some(start_pos));

    // Auto scroll downwards
    let _ = handle_auto_scroll(&mut app, Some(20.0), iced::Point::new(100.0, 450.0));
    assert_eq!(app.state.text.auto_scroll_delta, Some(20.0));

    let _ = handle_auto_scroll_tick(&mut app);
    assert!(app.state.text.scroll_y > 0.0);
    assert!(app.state.text.selection.is_some());
    let sel = app.state.text.selection.unwrap();
    assert_eq!(sel.start, start_pos);

    // Drag ended
    let _ = handle_selection_drag_ended(&mut app);
    assert!(!app.state.text.is_dragging_selection);
    assert_eq!(app.state.text.auto_scroll_delta, None);
    assert_eq!(app.state.text.drag_start, None);
}

#[test]
fn test_toggle_outline_preserves_scroll() {
    let mut app = KglanceApp::default();
    let content: String = (0..200)
        .map(|i| format!("line {i} with some content\n"))
        .collect();
    app.state.text = create_text_state(content, "txt", 14.0, false, AppTheme::Dark, 800.0);
    app.state.text.viewport_height = 400.0;
    app.state.text.scroll_y = 500.0;

    assert!(!app.state.text.outline_visible);
    let _ = handle_toggle_outline(&mut app);
    assert!(app.state.text.outline_visible);
    assert_eq!(app.state.text.scroll_y, 500.0);

    let _ = handle_toggle_outline(&mut app);
    assert!(!app.state.text.outline_visible);
    assert_eq!(app.state.text.scroll_y, 500.0);
}

#[test]
fn test_search_closed_preserves_scroll() {
    let mut app = KglanceApp::default();
    let content: String = (0..200)
        .map(|i| format!("line {i} with some content\n"))
        .collect();
    app.state.text = create_text_state(content, "txt", 14.0, false, AppTheme::Dark, 800.0);
    app.state.text.viewport_height = 400.0;
    app.state.text.scroll_y = 750.0;
    app.state.text.search_visible = true;
    app.state.text.search_query = "line 10".to_string();

    let _ = handle_search_closed(&mut app);
    assert!(!app.state.text.search_visible);
    assert!(app.state.text.search_query.is_empty());
    assert_eq!(app.state.text.scroll_y, 750.0);
}

#[test]
fn test_text_selection_drag_ended_triggers_selection_menu() {
    let mut app = KglanceApp::default();
    let content = "Hello world from text selection";
    app.state.text = create_text_state(
        content.to_string(),
        "txt",
        14.0,
        false,
        AppTheme::Dark,
        800.0,
    );
    app.state.selection_menu_config.enabled = true;
    app.state.last_cursor_position = Some(iced::Point::new(150.0, 200.0));

    assert!(matches!(
        app.state.selection_menu,
        crate::core::selection_action::SelectionMenuState::Hidden
    ));

    let start_pos = crate::ui::components::code_viewer::TextPosition::new(0, 6);
    let end_pos = crate::ui::components::code_viewer::TextPosition::new(0, 11);
    let _ = handle_selection_changed(&mut app, Some(SelectionRange::new(start_pos, end_pos)));

    let _ = handle_selection_drag_ended(&mut app);

    match &app.state.selection_menu {
        crate::core::selection_action::SelectionMenuState::Toolbar { pos, selected_text } => {
            assert_eq!(*pos, iced::Point::new(150.0, 200.0));
            assert_eq!(selected_text, "world");
        }
        other => panic!("Expected Toolbar state, got {other:?}"),
    }

    let _ = handle_selection_changed(&mut app, None);
    assert!(matches!(
        app.state.selection_menu,
        crate::core::selection_action::SelectionMenuState::Hidden
    ));
}

#[test]
fn test_text_wheel_scrolled_when_smooth_scroll_disabled() {
    let mut app = KglanceApp::default();
    let content: String = (0..200)
        .map(|i| format!("line {i} with some content\n"))
        .collect();
    app.state.text = create_text_state(content, "txt", 14.0, false, AppTheme::Dark, 800.0);
    app.state.text.viewport_height = 800.0;
    app.state.text.scroll_y = 100.0;
    app.state.scroll_config.smooth_scroll_enabled = false;
    app.state.apply_scroll_controllers();

    // Lines: 1 notch down (-1.0) with viewport_height = 800.0 (step = 120.0px) -> scroll_y: 100.0 -> 220.0
    let delta_lines = iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 };
    let _ = handle_wheel_scrolled(&mut app, delta_lines);

    assert_eq!(app.state.text.scroll_y, 220.0);
    assert!(!app.state.text.smooth_scroll.is_animating());
    assert_eq!(app.state.text.smooth_scroll.position_y(), 220.0);

    // Clamping: Scroll up past the top boundary clamps to 0.0
    let delta_up_excess = iced::mouse::ScrollDelta::Lines { x: 0.0, y: 10.0 };
    let _ = handle_wheel_scrolled(&mut app, delta_up_excess);
    assert_eq!(app.state.text.scroll_y, 0.0);
    assert!(!app.state.text.smooth_scroll.is_animating());
    assert_eq!(app.state.text.smooth_scroll.position_y(), 0.0);
}
