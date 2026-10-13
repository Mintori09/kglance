use super::*;
use crate::app::test_util::test_app;
use crate::core::SettingTab;
use crate::core::config::{EpubReadingMode, RubyDisplayMode};
use iced::Size;

#[test]
fn test_tab_changed() {
    let mut app = test_app(None);
    assert_eq!(app.state.settings_tab, SettingTab::Appearance);

    let _ = handle_settings_message(&mut app, SettingsMsg::TabChanged(SettingTab::Window));
    assert_eq!(app.state.settings_tab, SettingTab::Window);

    let _ = handle_settings_message(&mut app, SettingsMsg::TabChanged(SettingTab::Previews));
    assert_eq!(app.state.settings_tab, SettingTab::Previews);

    let _ = handle_settings_message(&mut app, SettingsMsg::TabChanged(SettingTab::CacheScroll));
    assert_eq!(app.state.settings_tab, SettingTab::CacheScroll);
}

#[test]
fn test_use_current_window_size() {
    let mut app = test_app(None);
    app.state.current_window_size = Size::new(1280.0, 800.0);
    app.state.window_min_size = Size::new(800.0, 600.0);

    let _ = handle_settings_message(&mut app, SettingsMsg::UseCurrentWindowSize);
    assert_eq!(app.state.window_default_size, Size::new(1280.0, 800.0));
}

#[test]
fn test_set_window_preset() {
    let mut app = test_app(None);
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::SetWindowPreset {
            width: 1440,
            height: 900,
        },
    );
    assert_eq!(app.state.window_default_size, Size::new(1440.0, 900.0));
}

#[test]
fn test_epub_reading_mode_changed() {
    let mut app = test_app(None);
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::EpubReadingModeChanged(EpubReadingMode::Continuous),
    );
    assert_eq!(app.state.epub_reading_mode, EpubReadingMode::Continuous);
    assert_eq!(app.state.epub.reading_mode, EpubReadingMode::Continuous);
}

#[test]
fn test_epub_reading_mode_changed_with_active_content() {
    let chapters = vec![
        crate::core::types::EpubChapterInfo {
            title: "Chapter 1".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch1.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Content 1".to_string()),
            ])],
        },
        crate::core::types::EpubChapterInfo {
            title: "Chapter 2".to_string(),
            level: 1,
            anchor: None,
            file_href: "ch2.xhtml".to_string(),
            blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
                crate::parsers::markdown::Inline::Text("Content 2".to_string()),
            ])],
        },
    ];

    let mut app = test_app(Some(crate::core::PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    }));
    app.state.epub_reading_mode = EpubReadingMode::SingleChapter;
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Book",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    // Initial SingleChapter mode: active_blocks returns chapter 0 blocks
    assert_eq!(app.state.epub.reading_mode, EpubReadingMode::SingleChapter);
    assert_eq!(app.state.epub.active_blocks().len(), 1);
    assert_eq!(app.state.epub.markdown_state.block_y_offsets.len(), 1);
    assert!(app.state.epub.continuous_blocks.is_empty());

    // Switch to Continuous mode via settings message
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::EpubReadingModeChanged(EpubReadingMode::Continuous),
    );

    // Content should now be continuous with blocks generated
    assert_eq!(app.state.epub.reading_mode, EpubReadingMode::Continuous);
    assert!(!app.state.epub.continuous_blocks.is_empty());
    assert_eq!(
        app.state.epub.active_blocks().len(),
        app.state.epub.continuous_blocks.len()
    );
    assert_eq!(
        app.state.epub.markdown_state.block_y_offsets.len(),
        app.state.epub.continuous_blocks.len()
    );

    // Switch back to SingleChapter mode
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::EpubReadingModeChanged(EpubReadingMode::SingleChapter),
    );

    assert_eq!(app.state.epub.reading_mode, EpubReadingMode::SingleChapter);
    assert_eq!(app.state.epub.active_blocks().len(), 1);
    assert_eq!(app.state.epub.markdown_state.block_y_offsets.len(), 1);
}

#[test]
fn test_ruby_display_mode_changed() {
    let mut app = test_app(None);
    assert_eq!(app.state.ruby_display_mode, RubyDisplayMode::Stacked);

    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::RubyDisplayModeChanged(RubyDisplayMode::Tooltip),
    );
    assert_eq!(app.state.ruby_display_mode, RubyDisplayMode::Tooltip);

    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::RubyDisplayModeChanged(RubyDisplayMode::Parentheses),
    );
    assert_eq!(app.state.ruby_display_mode, RubyDisplayMode::Parentheses);
}

#[test]
fn test_prefer_mermaid_cli_changed() {
    let mut app = test_app(None);
    let _ = handle_settings_message(&mut app, SettingsMsg::PreferMermaidCliChanged(true));
    assert!(app.state.prefer_mermaid_cli);

    let _ = handle_settings_message(&mut app, SettingsMsg::PreferMermaidCliChanged(false));
    assert!(!app.state.prefer_mermaid_cli);
}

#[test]
fn test_cache_limits_changed() {
    let mut app = test_app(None);
    let _ = handle_settings_message(&mut app, SettingsMsg::MaxMemoryMbChanged(1024));
    assert_eq!(app.state.cache_config.max_memory_mb, 1024);

    let _ = handle_settings_message(&mut app, SettingsMsg::MaxDiskCacheMbChanged(2048));
    assert_eq!(app.state.cache_config.max_disk_cache_mb, 2048);
}

#[test]
fn test_smooth_scroll_physics_changed() {
    let mut app = test_app(None);
    let _ = handle_settings_message(&mut app, SettingsMsg::SmoothScrollChanged(false));
    assert!(!app.state.scroll_config.smooth_scroll_enabled);

    let _ = handle_settings_message(&mut app, SettingsMsg::ScrollFrictionChanged(3.2));
    assert!((app.state.scroll_config.friction - 3.2).abs() < f32::EPSILON);

    let _ = handle_settings_message(&mut app, SettingsMsg::ScrollSpringStiffnessChanged(250.0));
    assert!((app.state.scroll_config.spring_stiffness - 250.0).abs() < f32::EPSILON);
}

#[test]
fn test_reset_to_defaults() {
    let mut app = test_app(None);
    // Alter settings
    app.state.font_size = 28.0;
    app.state.prefer_mermaid_cli = true;
    app.state.cache_config.max_memory_mb = 1024;
    app.state.scroll_config.smooth_scroll_enabled = false;

    let _ = handle_settings_message(&mut app, SettingsMsg::ResetToDefaults);

    let defaults = crate::core::config::AppConfig::default();
    assert_eq!(app.state.font_size, defaults.ui.font_size);
    assert_eq!(app.state.prefer_mermaid_cli, defaults.ui.prefer_mermaid_cli);
    assert_eq!(app.state.epub.reading_mode, defaults.ui.epub_reading_mode);
    assert_eq!(app.state.json.tree_mode, defaults.ui.json_tree_view);
    assert_eq!(app.state.cache_config, defaults.cache);
    assert_eq!(app.state.scroll_config, defaults.scroll);
    assert_eq!(app.state.selection_menu_config, defaults.selection_menu);
}

#[test]
fn test_font_family_selected_updates_epub_layout_when_no_custom_epub_font() {
    let mut app = test_app(None);
    let chapters = vec![crate::core::types::EpubChapterInfo {
        title: "Chapter 1".to_string(),
        level: 1,
        anchor: None,
        file_href: "ch1.xhtml".to_string(),
        blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
            crate::parsers::markdown::Inline::Text("Sample text".to_string()),
        ])],
    }];
    app.current_content = Some(crate::core::PreviewData::Epub {
        title: "Test Book".to_string(),
        author: "Author".to_string(),
        chapters: chapters.clone(),
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    });
    crate::features::epub::state::populate_state(
        &mut app.state,
        "Test Book",
        "Author",
        &chapters,
        0,
        &std::collections::HashMap::new(),
    );

    app.state.epub_font_family = None;
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::FontFamilySelected("Noto Sans".to_string()),
    );
    assert_eq!(app.state.font_family, Some("Noto Sans".to_string()));
    assert!(!app.state.epub.markdown_state.block_y_offsets.is_empty());
}

#[test]
fn test_reset_to_defaults_updates_existing_read_positions() {
    let mut app = test_app(None);
    let path_str = "/fake/book.epub";
    app.state.file_name = path_str.to_string();
    app.state.epub_reading_mode = EpubReadingMode::Continuous;
    app.state.epub.reading_mode = EpubReadingMode::Continuous;
    app.state.read_positions.insert(
        path_str.to_string(),
        crate::core::ReadPosition {
            scroll_y: 50.0,
            chapter: 1,
            reading_mode: Some(EpubReadingMode::Continuous),
        },
    );

    let _ = handle_settings_message(&mut app, SettingsMsg::ResetToDefaults);
    let defaults = crate::core::config::AppConfig::default();
    assert_eq!(app.state.epub_reading_mode, defaults.ui.epub_reading_mode);
    assert_eq!(
        app.state
            .read_positions
            .get(path_str)
            .and_then(|p| p.reading_mode),
        Some(defaults.ui.epub_reading_mode)
    );
}

#[test]
fn test_selection_menu_toggled() {
    let mut app = test_app(None);
    assert!(app.state.selection_menu_config.enabled);

    let _ = handle_settings_message(&mut app, SettingsMsg::SelectionMenuToggled(false));
    assert!(!app.state.selection_menu_config.enabled);

    let _ = handle_settings_message(&mut app, SettingsMsg::SelectionMenuToggled(true));
    assert!(app.state.selection_menu_config.enabled);
}

#[test]
fn test_custom_action_editor_workflow() {
    let mut app = test_app(None);
    app.state.selection_menu_config.custom_actions.clear();

    // 1. Open Add
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::OpenAdd),
    );
    assert!(app.state.action_editor.is_some());
    let editor = app.state.action_editor.as_ref().unwrap();
    assert_eq!(editor.editing_index, None);

    // 2. Change fields
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::NameChanged("Translate".to_string())),
    );
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::CommandChanged(
            "trans -b :vi".to_string(),
        )),
    );
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::PopupToggled(true)),
    );
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::StreamToggled(true)),
    );

    // 3. Save new action
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::Save),
    );
    assert!(app.state.action_editor.is_none());
    assert_eq!(app.state.selection_menu_config.custom_actions.len(), 1);
    let action = &app.state.selection_menu_config.custom_actions[0];
    assert_eq!(action.name, "Translate");
    assert_eq!(action.command, "trans -b :vi");
    assert!(action.enabled);
    assert!(action.popup);
    assert!(action.stream);

    // 4. Toggle action enabled
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::ToggleEnabled {
            index: 0,
            enabled: false,
        }),
    );
    assert!(!app.state.selection_menu_config.custom_actions[0].enabled);

    // 5. Open Edit
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::OpenEdit(0)),
    );
    assert_eq!(
        app.state.action_editor.as_ref().unwrap().editing_index,
        Some(0)
    );

    // 6. Modify name and save
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::NameChanged(
            "Translate VI".to_string(),
        )),
    );
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::Save),
    );
    assert_eq!(
        app.state.selection_menu_config.custom_actions[0].name,
        "Translate VI"
    );

    // 7. Delete action
    let _ = handle_settings_message(
        &mut app,
        SettingsMsg::CustomAction(CustomActionEditorMsg::Delete(0)),
    );
    assert!(app.state.selection_menu_config.custom_actions.is_empty());
}

#[test]
fn test_custom_actions_persisted_when_changing_other_settings() {
    let mut app = test_app(None);
    app.state.selection_menu_config.custom_actions =
        vec![crate::core::selection_action::CustomAction {
            name: "Translate".to_string(),
            command: "trans -b :vi".to_string(),
            enabled: true,
            popup: true,
            toast: false,
            stream: false,
        }];

    // Change font size
    let _ = handle_settings_message(&mut app, SettingsMsg::FontSizeChanged(18.0));
    assert_eq!(app.state.selection_menu_config.custom_actions.len(), 1);
    assert_eq!(app.current_config().selection_menu.custom_actions.len(), 1);

    // Change word wrap
    let _ = handle_settings_message(&mut app, SettingsMsg::WordWrapChanged(true));
    assert_eq!(app.state.selection_menu_config.custom_actions.len(), 1);
    assert_eq!(app.current_config().selection_menu.custom_actions.len(), 1);

    // Change theme
    let _ = handle_settings_message(&mut app, SettingsMsg::ThemeChanged("light".to_string()));
    assert_eq!(app.state.selection_menu_config.custom_actions.len(), 1);
    assert_eq!(app.current_config().selection_menu.custom_actions.len(), 1);
}
