use super::*;
use crate::app::messages::{GlobalMenuMsg, SettingsMsg};
use crate::app::test_util::test_app;
use crate::app::update::settings::handle_settings_message;
use crate::core::config::ScrollConfigOptions;
use crate::core::scroll::{ScrollController, ScrollState};

#[test]
fn test_global_menu_toggle_smooth_scroll_applied_to_controllers() {
    let mut app = test_app(None);

    // Step 1: Initialize app with smooth scroll explicitly disabled
    app.state.scroll_config.smooth_scroll_enabled = false;
    app.state.apply_scroll_controllers();

    // Verify initial state: all controllers have smooth_enabled == false
    assert!(!app.state.markdown.scroll_controller.smooth_enabled);
    assert!(!app.state.text.scroll_controller.smooth_enabled);
    assert!(!app.state.pdf.scroll_controller.smooth_enabled);
    assert!(!app.state.json.scroll_controller.smooth_enabled);
    assert!(!app.state.spreadsheet.scroll_controller.smooth_enabled);

    // Step 2: Trigger Global Menu action "toggle.smooth_scroll"
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("toggle.smooth_scroll".to_string()),
    );

    // Verify config was mutated to true
    assert!(
        app.state.scroll_config.smooth_scroll_enabled,
        "scroll_config.smooth_scroll_enabled should be true after toggle"
    );

    // Step 3: Verified fix:
    // handle_action_triggered invokes app.state.apply_scroll_controllers(),
    // so all controllers immediately reflect smooth_enabled == true
    assert!(
        app.state.markdown.scroll_controller.smooth_enabled,
        "markdown controller must be updated to true"
    );
    assert!(
        app.state.text.scroll_controller.smooth_enabled,
        "text controller must be updated to true"
    );
    assert!(
        app.state.pdf.scroll_controller.smooth_enabled,
        "pdf controller must be updated to true"
    );
    assert!(
        app.state.json.scroll_controller.smooth_enabled,
        "json controller must be updated to true"
    );
    assert!(
        app.state.spreadsheet.scroll_controller.smooth_enabled,
        "spreadsheet controller must be updated to true"
    );
}

#[test]
fn test_start_kinetic_respects_configured_friction() {
    let mut controller = ScrollController::new();

    // Step 1: Configure a custom friction of 4.0 (default is 1.8)
    let config = ScrollConfigOptions {
        smooth_scroll_enabled: true,
        friction: 4.0,
        spring_stiffness: 180.0,
    };
    controller.apply_config(&config);
    assert_eq!(controller.physics.friction_coefficient, 4.0);

    // Step 2: Trigger kinetic scroll with velocity = 1000.0, max_y = 5000.0
    controller.start_kinetic(0.0, 1000.0, 5000.0);

    // Configured friction 4.0 is strictly respected:
    // target_y = 0.0 + (1000.0 / 4.0) = 250.0
    let expected_target = 1000.0 / 4.0;
    assert!(
        (controller.target_y() - expected_target).abs() < 0.001,
        "target_y should be computed using configured friction 4.0 (250.0), got {}",
        controller.target_y()
    );

    if let ScrollState::Fling { friction, .. } = controller.scroll_state {
        assert_eq!(
            friction, 4.0,
            "ScrollState::Fling must store configured friction 4.0"
        );
    } else {
        panic!("Scroll state is not Fling");
    }
}

#[test]
fn test_settings_toggle_smooth_scroll_and_wrap_safe_without_menu_handle() {
    let mut app = test_app(None);
    assert!(app.menu_handle.is_none());

    // Settings message dispatch with menu_handle == None must succeed gracefully
    let _ = handle_settings_message(&mut app, SettingsMsg::SmoothScrollChanged(false));
    assert!(!app.state.scroll_config.smooth_scroll_enabled);
    assert!(!app.state.markdown.scroll_controller.smooth_enabled);

    let _ = handle_settings_message(&mut app, SettingsMsg::SmoothScrollChanged(true));
    assert!(app.state.scroll_config.smooth_scroll_enabled);
    assert!(app.state.markdown.scroll_controller.smooth_enabled);

    let _ = handle_settings_message(&mut app, SettingsMsg::WordWrapChanged(true));
    assert!(app.state.word_wrap);
    assert!(app.state.text.wrap);

    let _ = handle_settings_message(&mut app, SettingsMsg::WordWrapChanged(false));
    assert!(!app.state.word_wrap);
    assert!(!app.state.text.wrap);
}

#[test]
fn test_global_menu_edit_actions() {
    let mut app = test_app(None);
    app.state.file_name = "/tmp/test_file.txt".to_string();

    // 1. edit.copy with no selection falls back to copy_path
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("edit.copy".to_string()),
    );
    assert_eq!(app.state.toasts.last().unwrap().message, "Copied!");

    // 2. edit.copy with selection menu text
    app.state.selection_menu = crate::core::SelectionMenuState::Result {
        action_name: "Summarize".to_string(),
        pos: iced::Point::ORIGIN,
        content: "Full result output".to_string(),
        blocks: Vec::new(),
        is_streaming: false,
        selected_text: Some("result output".to_string()),
        selection_range: None,
        is_dragging: false,
        auto_scroll_delta: None,
    };
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("edit.copy".to_string()),
    );
    assert_eq!(app.state.toasts.last().unwrap().message, "Copied!");

    // 3. edit.select_all in text preview
    let preview = crate::core::PreviewData::Text {
        content: "line 1\nline 2".to_string(),
        line_numbers: "1\n2".to_string(),
        language: "text".to_string(),
    };
    preview.populate_state(&mut app.state);
    app.current_content = Some(preview);
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("edit.select_all".to_string()),
    );
    assert!(app.state.text.selection.is_some());

    // 4. edit.find toggles search
    assert!(!app.state.text.search_visible);
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("edit.find".to_string()),
    );
    assert!(app.state.text.search_visible);
}

#[test]
fn test_ctrl_shift_c_copies_file_path() {
    let mut app = test_app(None);
    app.state.file_name = "/path/to/my_document.pdf".to_string();
    let key = iced::keyboard::Key::Character("c".into());
    let mut mods = iced::keyboard::Modifiers::CTRL;
    mods.insert(iced::keyboard::Modifiers::SHIFT);

    let task = app.handle_ctrl_shortcuts(&key, mods);
    assert!(task.is_some());
    assert_eq!(app.state.toasts.last().unwrap().message, "Copied!");
}

#[test]
fn test_json_ctrl_m_shortcut_toggles_mode() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Json {
        nodes: Vec::new(),
        content: "{}".to_string(),
        pretty: "{}".to_string(),
        has_parse_error: false,
    });
    let key = iced::keyboard::Key::Character("m".into());
    let task = app.handle_ctrl_shortcuts(&key, iced::keyboard::Modifiers::CTRL);
    assert!(task.is_some());
}

#[test]
fn test_alt_z_toggles_word_wrap() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Text {
        content: "hello world".to_string(),
        line_numbers: "1".to_string(),
        language: "text".to_string(),
    });
    assert!(!app.state.word_wrap);
    let key = iced::keyboard::Key::Character("z".into());
    let _ = app.handle_key_pressed(key.clone(), iced::keyboard::Modifiers::ALT);
    assert!(app.state.word_wrap);
    let _ = app.handle_key_pressed(key, iced::keyboard::Modifiers::ALT);
    assert!(!app.state.word_wrap);
}

#[test]
fn test_image_f_and_1_shortcuts() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Image {
        width: 100,
        height: 100,
        data: vec![],
        exif_content: None,
        format_info: String::new(),
    });
    let key_f = iced::keyboard::Key::Character("f".into());
    let _ = app.handle_key_pressed(key_f, iced::keyboard::Modifiers::empty());
    let key_1 = iced::keyboard::Key::Character("1".into());
    let _ = app.handle_key_pressed(key_1, iced::keyboard::Modifiers::empty());
}

#[test]
fn test_file_menu_no_duplicate_copy_path() {
    let config = crate::core::config::AppConfig::default();
    let theme = crate::ui::theme::AppTheme::Dark;
    let root = crate::dbus::global_menu::build_root_menu(&config, &theme);

    // Root should only contain File, Edit, View, Actions (no empty Help menu)
    let top_labels: Vec<&str> = root.children.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(top_labels, vec!["File", "Edit", "View", "Actions"]);

    let file_menu = &root.children[0];
    assert_eq!(file_menu.label, "File");
    let file_labels: Vec<&str> = file_menu
        .children
        .iter()
        .map(|c| c.label.as_str())
        .collect();
    assert_eq!(
        file_labels,
        vec![
            "Open in Default App",
            "Open Containing Folder",
            "Close Preview"
        ]
    );

    let edit_menu = &root.children[1];
    assert_eq!(edit_menu.label, "Edit");
    let copy_item = &edit_menu.children[0];
    assert_eq!(copy_item.id, crate::dbus::global_menu::ID_EDIT_COPY);
    assert_eq!(copy_item.label, "Copy Path");
    assert_eq!(
        copy_item.shortcut.as_ref(),
        Some(&vec![vec!["Control".to_string(), "C".to_string()]])
    );

    let view_menu = &root.children[2];
    assert_eq!(view_menu.label, "View");
    assert_eq!(view_menu.children[0].label, "Theme");
    assert!(
        !view_menu
            .children
            .iter()
            .any(|item| item.label == "Toggle Dark / Light Theme")
    );
}

#[test]
fn test_dynamic_copy_selection_and_ctrl_c() {
    let mut app = test_app(None);
    app.state.file_name = "/tmp/sample.rs".to_string();

    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let config = crate::core::config::AppConfig::default();
    let (_service, handle) =
        crate::dbus::global_menu::init_global_menu(tx, &config, &app.state.app_theme).unwrap();
    app.menu_handle = Some(handle);

    // 1. No selection: has_selection is false, label is "Copy Path"
    assert!(!app.has_selection());
    app.sync_global_menu_copy_label();
    let root = app.menu_handle.as_ref().unwrap().get_root();
    let copy_item = &root.children[1].children[0];
    assert_eq!(copy_item.label, "Copy Path");

    // Ctrl+C with no selection copies file path
    let key = iced::keyboard::Key::Character("c".into());
    let task = app.handle_ctrl_shortcuts(&key, iced::keyboard::Modifiers::CTRL);
    assert!(task.is_some());
    assert_eq!(app.state.toasts.last().unwrap().message, "Copied!");

    // 2. Add text selection: has_selection becomes true, label updates to "Copy Selection"
    let preview = crate::core::PreviewData::Text {
        content: "fn main() {}".to_string(),
        line_numbers: "1".to_string(),
        language: "rust".to_string(),
    };
    preview.populate_state(&mut app.state);
    app.current_content = Some(preview);
    app.state.text.selection = Some(crate::ui::components::code_viewer::SelectionRange {
        start: crate::ui::components::code_viewer::TextPosition::new(0, 0),
        end: crate::ui::components::code_viewer::TextPosition::new(0, 7),
    });

    assert!(app.has_selection());
    app.sync_global_menu_copy_label();
    let root = app.menu_handle.as_ref().unwrap().get_root();
    let copy_item = &root.children[1].children[0];
    assert_eq!(copy_item.label, "Copy Selection");

    // Ctrl+C with selection copies selected text
    let task = app.handle_ctrl_shortcuts(&key, iced::keyboard::Modifiers::CTRL);
    assert!(task.is_some());
    assert_eq!(app.state.toasts.last().unwrap().message, "Copied selected!");

    // 3. Clear selection: label switches back to "Copy Path"
    app.state.text.clear_selection();
    assert!(!app.has_selection());
    app.sync_global_menu_copy_label();
    let root = app.menu_handle.as_ref().unwrap().get_root();
    let copy_item = &root.children[1].children[0];
    assert_eq!(copy_item.label, "Copy Path");
}

#[test]
fn test_theme_toggle_flips_dark_and_light() {
    let mut app = test_app(None);

    // Initial theme is Dark
    app.state.app_theme = crate::ui::theme::AppTheme::Dark;
    app.state.theme_setting = "dark".to_string();
    assert!(app.state.app_theme.is_dark());

    // Toggle -> Light
    let _ = crate::app::update::misc::handle_theme_toggled(&mut app);
    assert_eq!(app.state.app_theme, crate::ui::theme::AppTheme::Light);
    assert!(!app.state.app_theme.is_dark());
    assert_eq!(app.state.theme_setting, "light");

    // Toggle -> Dark
    let _ = crate::app::update::misc::handle_theme_toggled(&mut app);
    assert_eq!(app.state.app_theme, crate::ui::theme::AppTheme::Dark);
    assert!(app.state.app_theme.is_dark());
    assert_eq!(app.state.theme_setting, "dark");

    // From another dark theme (e.g. ProperDark) -> toggles to Light
    app.state.app_theme = crate::ui::theme::AppTheme::ProperDark;
    app.state.theme_setting = "proper-dark".to_string();
    assert!(app.state.app_theme.is_dark());

    let _ = crate::app::update::misc::handle_theme_toggled(&mut app);
    assert_eq!(app.state.app_theme, crate::ui::theme::AppTheme::Light);
    assert!(!app.state.app_theme.is_dark());
    assert_eq!(app.state.theme_setting, "light");
}

#[test]
fn test_global_menu_view_toggles_selection_menu_and_remember_size() {
    let mut app = test_app(None);

    let initial_sel = app.state.selection_menu_config.enabled;
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("toggle.selection_menu".to_string()),
    );
    assert_eq!(app.state.selection_menu_config.enabled, !initial_sel);

    assert!(!app.state.remember_window_size);
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("toggle.remember_size".to_string()),
    );
    assert!(app.state.remember_window_size);
}

#[test]
fn test_global_menu_epub_actions() {
    let mut app = test_app(None);

    // 1. Toggle override font
    assert!(!app.state.epub_override_book_font);
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("epub.toggle_override_font".to_string()),
    );
    assert!(app.state.epub_override_book_font);

    // 2. Reading mode
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("epub.reading_mode.continuous".to_string()),
    );
    assert_eq!(
        app.state.epub_reading_mode,
        crate::core::config::EpubReadingMode::Continuous
    );

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("epub.reading_mode.single".to_string()),
    );
    assert_eq!(
        app.state.epub_reading_mode,
        crate::core::config::EpubReadingMode::SingleChapter
    );

    // 3. Ruby display mode
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("epub.ruby_mode.tooltip".to_string()),
    );
    assert_eq!(
        app.state.ruby_display_mode,
        crate::core::config::RubyDisplayMode::Tooltip
    );

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("epub.ruby_mode.hidden".to_string()),
    );
    assert_eq!(
        app.state.ruby_display_mode,
        crate::core::config::RubyDisplayMode::Hidden
    );
}

#[test]
fn test_global_menu_json_actions() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Json {
        nodes: vec![],
        content: "{\"key\": 1}".to_string(),
        pretty: "{\n  \"key\": 1\n}".to_string(),
        has_parse_error: false,
    });

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("json.toggle_format".to_string()),
    );
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("json.expand_all".to_string()),
    );
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("json.collapse_all".to_string()),
    );
}

#[test]
fn test_global_menu_markdown_actions() {
    let mut app = test_app(None);

    assert!(!app.state.prefer_mermaid_cli);
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("markdown.toggle_mermaid_cli".to_string()),
    );
    assert!(app.state.prefer_mermaid_cli);
}

#[test]
fn test_global_menu_spreadsheet_and_image_actions() {
    let mut app = test_app(None);

    // Image flips and toggle info
    app.current_content = Some(crate::core::PreviewData::Image {
        data: vec![],
        width: 100,
        height: 100,
        format_info: "PNG".to_string(),
        exif_content: Some("Exif data".to_string()),
    });
    app.state.image.exif_content = "Exif data".to_string();

    assert!(!app.state.image.show_info);
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("image.toggle_info".to_string()),
    );
    assert!(app.state.image.show_info);

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("image.flip_h".to_string()),
    );
    assert!(app.state.image.camera.flip.0);

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("image.flip_v".to_string()),
    );
    assert!(app.state.image.camera.flip.1);
}

#[test]
fn test_global_menu_zoom_actions() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Text {
        content: "hello world".to_string(),
        line_numbers: "1".to_string(),
        language: "text".to_string(),
    });
    let initial_font_size = app.state.font_size;

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("zoom.in".to_string()),
    );
    assert_eq!(app.state.font_size, initial_font_size + 1.0);

    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("zoom.out".to_string()),
    );
    assert_eq!(app.state.font_size, initial_font_size);

    app.state.font_size = initial_font_size + 5.0;
    let _ = handle_global_menu_msg(
        &mut app,
        GlobalMenuMsg::ActionTriggered("zoom.reset".to_string()),
    );
    assert_eq!(app.state.font_size, app.state.default_font_size);
}
