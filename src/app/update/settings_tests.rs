use super::*;
use crate::app::test_util::test_app;
use crate::core::SettingTab;
use crate::core::config::EpubReadingMode;
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
    assert_eq!(app.state.cache_config, defaults.cache);
    assert_eq!(app.state.scroll_config, defaults.scroll);
}
