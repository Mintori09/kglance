use crate::app::test_util::{epub_content, test_app};

#[test]
fn ctrl_a_selects_all_for_epub() {
    let mut app = test_app(Some(epub_content(&["a", "b"])));
    let task = app.handle_ctrl_a();
    assert!(task.is_some());
    assert!(app.state.epub.markdown_state.selection_range.is_some());
}

#[test]
fn ctrl_c_copies_epub_selection() {
    let mut app = test_app(Some(epub_content(&["hello"])));
    app.state.epub.markdown_state.selected_text = Some("hello".to_string());
    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _guard = runtime.enter();
    assert!(app.handle_ctrl_copy().is_some());
}

#[test]
fn ctrl_c_copies_markdown_selection_with_inline_math() {
    use crate::app::test_util::markdown_content;
    use crate::core::{SelectionPoint, SelectionRange};

    let src = "- $\\Rightarrow$ **cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn.**";
    let mut app = test_app(Some(markdown_content(src)));

    // Simulating selection on the markdown state
    app.state.markdown.selection_range = Some(SelectionRange {
        start: SelectionPoint {
            block: 1,
            offset: 0,
        },
        end: SelectionPoint {
            block: 1,
            offset: 500,
        },
    });

    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _guard = runtime.enter();

    let task = app.handle_ctrl_copy();
    assert!(
        task.is_some(),
        "handle_ctrl_copy should return Task with clipboard write"
    );
    assert!(
        app.state
            .markdown
            .selected_text
            .as_ref()
            .unwrap()
            .contains("cấu trúc thuật toán ANN")
    );
}

#[test]
fn ctrl_c_preserves_existing_selected_text_fallback() {
    use crate::app::test_util::markdown_content;

    let mut app = test_app(Some(markdown_content("- item")));
    app.state.markdown.selected_text = Some("thuật toán ANN".to_string());

    let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _guard = runtime.enter();

    let task = app.handle_ctrl_copy();
    assert!(task.is_some());
    assert_eq!(
        app.state.markdown.selected_text.as_deref(),
        Some("thuật toán ANN")
    );
}

#[test]
fn ctrl_c_returns_none_without_selection() {
    let mut app = test_app(Some(epub_content(&["hello"])));
    assert!(app.handle_ctrl_copy().is_none());
}

#[test]
fn ctrl_w_toggles_word_wrap() {
    let mut app = test_app(Some(crate::core::PreviewData::Text {
        content: "hello".to_string(),
        line_numbers: "1".to_string(),
        language: "plaintext".to_string(),
    }));
    let initial_wrap = app.state.word_wrap;
    let key = iced::keyboard::Key::Character("w".into());
    let modifiers = iced::keyboard::Modifiers::CTRL;

    let task = app.handle_ctrl_shortcuts(&key, modifiers);
    assert!(task.is_some());
    assert_eq!(app.state.word_wrap, !initial_wrap);
}

#[test]
fn ctrl_w_toggles_word_wrap_json_raw_preserves_highlight() {
    let json_str = "{\n  \"hello\": \"world\",\n  \"count\": 42\n}".to_string();
    let mut app = test_app(Some(crate::core::PreviewData::Json {
        nodes: vec![],
        content: json_str.clone(),
        pretty: json_str,
        has_parse_error: false,
    }));
    app.state.json.tree_mode = false;
    assert_eq!(app.state.json.raw_text.cached_tokens.len(), 4);
    assert!(!app.state.json.raw_text.cached_tokens[1].is_empty());

    let key = iced::keyboard::Key::Character("w".into());
    let modifiers = iced::keyboard::Modifiers::CTRL;
    let _ = app.handle_ctrl_shortcuts(&key, modifiers);

    // Verify tokens still present and wrap updated
    assert_eq!(app.state.json.raw_text.cached_tokens.len(), 4);
    assert!(!app.state.json.raw_text.cached_tokens[1].is_empty());
    assert!(app.state.json.raw_text.wrap);

    // Toggle back to non-wrap
    let _ = app.handle_ctrl_shortcuts(&key, modifiers);
    assert_eq!(app.state.json.raw_text.cached_tokens.len(), 4);
    assert!(!app.state.json.raw_text.cached_tokens[1].is_empty());
    assert!(!app.state.json.raw_text.wrap);
}

#[test]
fn ctrl_plus_minus_resizes_json_font_size() {
    let mut app = test_app(Some(crate::core::PreviewData::Json {
        nodes: vec![],
        content: "{}".to_string(),
        pretty: "{}".to_string(),
        has_parse_error: false,
    }));
    let initial_size = app.state.font_size;
    let plus_key = iced::keyboard::Key::Character("+".into());
    let minus_key = iced::keyboard::Key::Character("-".into());
    let modifiers = iced::keyboard::Modifiers::CTRL;

    let task = app.handle_ctrl_shortcuts(&plus_key, modifiers);
    assert!(task.is_some());
    assert_eq!(app.state.font_size, initial_size + 1.0);

    let task = app.handle_ctrl_shortcuts(&minus_key, modifiers);
    assert!(task.is_some());
    assert_eq!(app.state.font_size, initial_size);
}

#[test]
fn ctrl_plus_preserves_scroll_position_for_text_and_json() {
    let mut app = test_app(Some(crate::core::PreviewData::Text {
        content: "line 1\nline 2\nline 3".to_string(),
        line_numbers: "1\n2\n3".to_string(),
        language: "plaintext".to_string(),
    }));
    app.state.font_size = 14.0;
    app.state.text.scroll_y = 140.0;

    let plus_key = iced::keyboard::Key::Character("+".into());
    let modifiers = iced::keyboard::Modifiers::CTRL;

    let task = app.handle_ctrl_shortcuts(&plus_key, modifiers);
    assert!(task.is_some());
    assert_eq!(app.state.font_size, 15.0);
    assert_eq!(app.state.text.scroll_y, 140.0 * (15.0 / 14.0));

    let mut json_app = test_app(Some(crate::core::PreviewData::Json {
        nodes: vec![],
        content: "{}".to_string(),
        pretty: "{}".to_string(),
        has_parse_error: false,
    }));
    json_app.state.font_size = 14.0;
    json_app.state.json.scroll_y = 280.0;
    json_app.state.json.raw_text.scroll_y = 280.0;

    let task = json_app.handle_ctrl_shortcuts(&plus_key, modifiers);
    assert!(task.is_some());
    assert_eq!(json_app.state.font_size, 15.0);
    assert_eq!(json_app.state.json.scroll_y, 280.0 * (15.0 / 14.0));

    // Test tree mode node height preservation
    let mut json_tree_app = test_app(Some(crate::core::PreviewData::Json {
        nodes: vec![],
        content: "{}".to_string(),
        pretty: "{}".to_string(),
        has_parse_error: false,
    }));
    json_tree_app.state.font_size = 14.0;
    json_tree_app.state.json.tree_mode = true;
    json_tree_app.state.json.scroll_y = 220.0;

    let task = json_tree_app.handle_ctrl_shortcuts(&plus_key, modifiers);
    assert!(task.is_some());
    assert_eq!(json_tree_app.state.font_size, 15.0);
    let old_row_h = crate::features::json::view::tree::json_row_height(14.0);
    let new_row_h = crate::features::json::view::tree::json_row_height(15.0);
    assert_eq!(
        json_tree_app.state.json.scroll_y,
        (220.0 / old_row_h) * new_row_h
    );
}

#[test]
fn ctrl_g_toggles_goto_line_and_submits() {
    let code = (1..=100)
        .map(|i| format!("Line {i} content"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = test_app(Some(crate::app::test_util::text_content(&code, "rs")));
    app.state.text.viewport_height = 800.0;
    app.state.text.total_content_height = 100.0 * (app.state.font_size * 1.35);

    let g_key = iced::keyboard::Key::Character("g".into());
    let modifiers = iced::keyboard::Modifiers::CTRL;

    // 1. Press Ctrl+G -> opens Goto Line bar
    let task = app.handle_ctrl_shortcuts(&g_key, modifiers);
    assert!(task.is_some());
    assert!(app.state.text.goto_line_visible);

    // 2. Input line number 42
    let _ =
        crate::features::text::update::handle_goto_line_query_changed(&mut app, "42".to_string());
    assert_eq!(app.state.text.goto_line_query, "42");

    // 3. Submit
    let _ = crate::features::text::update::handle_goto_line_submitted(&mut app);
    assert!(!app.state.text.goto_line_visible);
    assert!(app.state.text.smooth_scroll.is_animating);
    let expected_y = app.state.text.display_map.get_line_y(41);
    assert_eq!(app.state.text.smooth_scroll.target_y(), expected_y);

    // 4. Test mutual exclusivity with Ctrl+F
    let f_key = iced::keyboard::Key::Character("f".into());
    let _ = app.handle_ctrl_shortcuts(&g_key, modifiers);
    assert!(app.state.text.goto_line_visible);
    let _ = app.handle_ctrl_shortcuts(&f_key, modifiers);
    assert!(app.state.text.search_visible);
    assert!(!app.state.text.goto_line_visible);
}

#[test]
fn wrap_toggle_recalculates_toc_line_numbers_and_symbol_positions() {
    // Line 1 is very long and will wrap across multiple rows when wrap is enabled
    let long_line = "let very_long_variable_definition = ".repeat(10);
    let trailing = (1..=50)
        .map(|i| format!("fn f_{i}() {{}}"))
        .collect::<Vec<_>>()
        .join("\n");
    let code = format!("{long_line}\nfn target_function() {{}}\n{trailing}");
    let mut app = test_app(Some(crate::app::test_util::text_content(&code, "rs")));
    app.state.text.display_map.viewport_width = 300.0;
    app.state.text.viewport_height = 200.0;

    // 1. Initial state (word_wrap enabled)
    app.state.word_wrap = true;
    app.state.text.wrap = true;
    app.state.text.display_map.update_geometry(
        &app.state.text.document,
        300.0,
        app.state.font_size,
        crate::features::text::WrapMode::Word,
    );
    app.state.text.total_content_height = app.state.text.display_map.total_content_height();

    let wrapped_line_2_y = app.state.text.display_map.get_line_y(1);
    let line_height = app.state.font_size * 1.35;
    // With wrap on, line 2 Y must be > 1 * line_height because line 1 wrapped into multiple rows
    assert!(wrapped_line_2_y > line_height);

    // Clicking symbol at line 2 navigates to wrapped Y position
    let _ = crate::features::text::update::handle_symbol_clicked(&mut app, 2);
    assert_eq!(app.state.text.smooth_scroll.target_y(), wrapped_line_2_y);

    // 2. Toggle word wrap off
    let w_key = iced::keyboard::Key::Character("w".into());
    let modifiers = iced::keyboard::Modifiers::CTRL;
    let _ = app.handle_ctrl_shortcuts(&w_key, modifiers);
    assert!(!app.state.word_wrap);
    assert!(!app.state.text.wrap);

    let unwrapped_line_2_y = app.state.text.display_map.get_line_y(1);
    // With wrap off, line 2 Y is exactly 1 * line_height
    assert_eq!(unwrapped_line_2_y, line_height);

    // Clicking symbol at line 2 now navigates to the new unwrapped Y position
    let _ = crate::features::text::update::handle_symbol_clicked(&mut app, 2);
    assert_eq!(app.state.text.smooth_scroll.target_y(), unwrapped_line_2_y);
}

#[test]
fn test_spreadsheet_search_requires_ctrl_f_not_arbitrary_typing() {
    let sheet = crate::features::sheet::types::SheetInfo {
        name: "Sheet1".to_string(),
        headers: vec![],
        columns: vec![],
        rows: vec![],
    };
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Spreadsheet {
        sheets: vec![sheet],
        active_sheet: 0,
    });

    // 1. Typing arbitrary characters (e.g. 'a', 'j') should NOT open search
    let a_key = iced::keyboard::Key::Character("a".into());
    let j_key = iced::keyboard::Key::Character("j".into());
    let empty_mod = iced::keyboard::Modifiers::default();

    let task_a = app.handle_type_to_search(&a_key, empty_mod);
    assert!(task_a.is_none());
    assert!(!app.state.spreadsheet.search_visible);
    assert!(app.state.spreadsheet.search_query.is_empty());

    let task_j = app.handle_type_to_search(&j_key, empty_mod);
    assert!(task_j.is_none());
    assert!(!app.state.spreadsheet.search_visible);

    // 2. Pressing Ctrl+F toggles search visibility and focuses input
    let f_key = iced::keyboard::Key::Character("f".into());
    let ctrl_mod = iced::keyboard::Modifiers::CTRL;
    let task_ctrl_f = app.handle_ctrl_shortcuts(&f_key, ctrl_mod);
    assert!(task_ctrl_f.is_some());
    assert!(app.state.spreadsheet.search_visible);

    // 3. Pressing Ctrl+F again closes search and clears query
    app.state.spreadsheet.search_query = "find_me".to_string();
    let task_ctrl_f_close = app.handle_ctrl_shortcuts(&f_key, ctrl_mod);
    assert!(task_ctrl_f_close.is_some());
    assert!(!app.state.spreadsheet.search_visible);
    assert!(app.state.spreadsheet.search_query.is_empty());
}

#[test]
fn test_ctrl_held_blocks_keyboard_scroll_shortcuts() {
    use crate::app::test_util::markdown_content;
    let mut app = test_app(Some(markdown_content("# Test\n\nSome text")));
    app.state.markdown.viewport_height = 800.0;
    app.state.markdown.total_content_height = 3000.0;
    app.state.markdown.scroll_y = 100.0;

    let ctrl_mod = iced::keyboard::Modifiers::CTRL;
    let j_key = iced::keyboard::Key::Character("j".into());
    let down_key = iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown);
    let d_key = iced::keyboard::Key::Character("d".into());

    // 1. handle_scroll_shortcuts with CTRL modifier returns None
    assert!(app.handle_scroll_shortcuts(&j_key, ctrl_mod).is_none());
    assert!(app.handle_scroll_shortcuts(&down_key, ctrl_mod).is_none());
    assert!(app.handle_scroll_shortcuts(&d_key, ctrl_mod).is_none());

    // 2. handle_key_pressed with CTRL modifier does not trigger scrolling
    let _ = app.handle_key_pressed(j_key, ctrl_mod);
    assert!(!app.state.markdown.smooth_scroll.is_animating);
    assert_eq!(app.state.markdown.scroll_y, 100.0);

    // 3. handle_scroll_shortcuts with ctrl_held = true returns None
    app.ctrl_held = true;
    let empty_mod = iced::keyboard::Modifiers::default();
    let k_key = iced::keyboard::Key::Character("k".into());
    assert!(app.handle_scroll_shortcuts(&k_key, empty_mod).is_none());
}

#[test]
fn test_ctrl_held_blocks_folder_and_grid_navigation() {
    let mut app = test_app(None);
    app.ctrl_held = true;

    // Folder navigation is blocked
    let down_key = iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown);
    assert!(app.handle_folder_navigation(&down_key).is_none());

    // Grid navigation is blocked
    assert!(app.handle_grid_navigation(&down_key).is_none());
}

#[test]
fn test_ctrl_held_scroll_delta_does_not_scroll_non_zoomable() {
    let sheet = crate::features::sheet::types::SheetInfo {
        name: "Sheet1".to_string(),
        headers: vec![],
        columns: vec![],
        rows: vec![],
    };
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Spreadsheet {
        sheets: vec![sheet],
        active_sheet: 0,
    });
    app.ctrl_held = true;

    // Scrolling mouse wheel while Ctrl is held on spreadsheet does nothing (returns Task::none)
    let _ = app.handle_scroll_delta(0.0, 80.0);
    assert_eq!(app.state.spreadsheet.scroll_x, 0.0);
    assert_eq!(app.state.spreadsheet.scroll_y, 0.0);
}

#[test]
fn test_file_navigation_next_file_shortcuts() {
    use iced::keyboard::key::Named;

    let mut app = test_app(None);

    // 1. Ctrl + Shift + ArrowRight -> NextFile
    app.ctrl_held = true;
    app.shift_held = true;
    let arrow_right = iced::keyboard::Key::Named(Named::ArrowRight);
    let task = app.handle_file_navigation(&arrow_right);
    assert!(task.is_some());

    // 2. Ctrl + PageDown (without Shift) -> NextFile
    app.ctrl_held = true;
    app.shift_held = false;
    let page_down = iced::keyboard::Key::Named(Named::PageDown);
    let task = app.handle_file_navigation(&page_down);
    assert!(task.is_some());

    // 3. Without Ctrl -> None
    app.ctrl_held = false;
    app.shift_held = true;
    assert!(app.handle_file_navigation(&arrow_right).is_none());
}

#[test]
fn test_file_navigation_prev_file_shortcuts() {
    use iced::keyboard::key::Named;

    let mut app = test_app(None);

    // 1. Ctrl + Shift + ArrowLeft -> PrevFile
    app.ctrl_held = true;
    app.shift_held = true;
    let arrow_left = iced::keyboard::Key::Named(Named::ArrowLeft);
    let task = app.handle_file_navigation(&arrow_left);
    assert!(task.is_some());

    // 2. Ctrl + PageUp (without Shift) -> PrevFile
    app.ctrl_held = true;
    app.shift_held = false;
    let page_up = iced::keyboard::Key::Named(Named::PageUp);
    let task = app.handle_file_navigation(&page_up);
    assert!(task.is_some());

    // 3. Without Ctrl -> None
    app.ctrl_held = false;
    app.shift_held = false;
    assert!(app.handle_file_navigation(&page_up).is_none());
}

#[test]
fn test_bare_arrow_keys_do_not_switch_file_in_detail_view() {
    use iced::keyboard::key::Named;

    let mut app = test_app(Some(crate::app::test_util::markdown_content("Hello")));
    app.state.view_mode = crate::core::ViewMode::Detail;

    let arrow_right = iced::keyboard::Key::Named(Named::ArrowRight);
    let arrow_left = iced::keyboard::Key::Named(Named::ArrowLeft);

    // handle_view_mode_navigation returns None for bare arrows on markdown/text
    assert!(app.handle_view_mode_navigation(&arrow_right).is_none());
    assert!(app.handle_view_mode_navigation(&arrow_left).is_none());
}

#[test]
fn test_ctrl_t_toggles_theme_and_syncs_setting() {
    let mut app = test_app(Some(crate::app::test_util::markdown_content("Hello")));
    assert_eq!(app.state.app_theme, crate::ui::theme::AppTheme::Dark);

    // handle_toggle_theme emits SystemMsg::ThemeToggled
    let task = app.handle_toggle_theme();
    assert!(task.is_some());

    // Calling handle_theme_toggled directly updates app_theme and theme_setting
    let _ = crate::app::update::misc::handle_theme_toggled(&mut app);
    assert_eq!(app.state.app_theme, crate::ui::theme::AppTheme::Light);
    assert_eq!(app.state.theme_setting, "Light");
}
