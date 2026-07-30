use std::sync::Mutex;

use kglance::core::config::{AppConfig, ConfigManager, UiConfig};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_config_dir<F>(f: F)
where
    F: FnOnce(&std::path::Path),
{
    let _guard = ENV_LOCK.lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("KGLANCE_CONFIG_DIR", tmp.path()) };
    f(tmp.path());
    unsafe { std::env::remove_var("KGLANCE_CONFIG_DIR") };
}

#[test]
fn test_ui_config_default() {
    let config = UiConfig::default();
    assert_eq!(config.theme, None);
    assert_eq!(config.font_size, 14.0);
    assert_eq!(config.font_family, None);
    assert_eq!(config.font_family_mono, None);
    assert_eq!(config.epub_font_family, None);
    assert!(config.max_text_width.is_some());
    assert!((config.max_text_width.unwrap() - 820.0).abs() < f32::EPSILON);
    assert_eq!(config.default_width, 900);
    assert_eq!(config.default_height, 600);
}

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.ui, UiConfig::default());
}

#[test]
fn test_config_serialization_round_trip() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Light".into()),
            font_size: 16.0,
            font_family: Some("Noto Sans".into()),
            font_family_mono: Some("JetBrains Mono".into()),
            epub_font_family: Some("Noto Serif".into()),
            max_text_width: Some(720.0),
            default_width: 1024,
            default_height: 768,
        },
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_get_theme_specific() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Light".into()),
            ..Default::default()
        },
    };
    assert_eq!(ConfigManager::get_theme(&config), "Light");
}

#[test]
fn test_get_theme_dark() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Dark".into()),
            ..Default::default()
        },
    };
    assert_eq!(ConfigManager::get_theme(&config), "Dark");
}

#[test]
fn test_get_theme_auto() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("auto".into()),
            ..Default::default()
        },
    };
    let theme = ConfigManager::get_theme(&config);
    assert!(theme == "Light" || theme == "Dark");
}

#[test]
fn test_get_theme_none_falls_back_to_system() {
    let config = AppConfig::default();
    assert!(config.ui.theme.is_none());
    let theme = ConfigManager::get_theme(&config);
    assert!(theme == "Light" || theme == "Dark");
}

#[test]
fn test_get_theme_custom_value_passthrough() {
    for theme in &["Blue", "invalid_theme", "custom-dark-theme"] {
        let config = AppConfig {
            ui: UiConfig {
                theme: Some((*theme).into()),
                ..Default::default()
            },
        };
        assert_eq!(ConfigManager::get_theme(&config), *theme);
    }
}

#[test]
fn test_get_theme_case_sensitive_auto() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Auto".into()),
            ..Default::default()
        },
    };
    let theme = ConfigManager::get_theme(&config);
    assert!(theme == "Light" || theme == "Dark");
}

#[test]
fn test_get_theme_unrecognized_returns_raw() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("AUTO".into()),
            ..Default::default()
        },
    };
    assert_eq!(ConfigManager::get_theme(&config), "AUTO");
}

#[test]
fn test_config_path_format() {
    let path = ConfigManager::get_config_path();
    assert!(path.ends_with("kglance/config.json"));
}

#[test]
fn test_config_dir_format() {
    let dir = ConfigManager::get_config_dir();
    assert!(dir.ends_with("kglance"));
}

#[test]
fn test_save_load_and_create_default() {
    with_temp_config_dir(|_tmp| {
        let original = AppConfig {
            ui: UiConfig {
                theme: Some("Dark".into()),
                font_size: 18.0,
                font_family: Some("Fira Sans".into()),
                ..Default::default()
            },
        };

        ConfigManager::save(&original).unwrap();
        let loaded = ConfigManager::load_or_create();
        assert_eq!(loaded.ui.theme, original.ui.theme);
        assert_eq!(loaded.ui.font_size, original.ui.font_size);
        assert_eq!(loaded.ui.font_family, original.ui.font_family);
    });

    with_temp_config_dir(|tmp| {
        let config = ConfigManager::load_or_create();
        assert!((config.ui.max_text_width.unwrap() - 820.0).abs() < f32::EPSILON);
        assert_eq!(config.ui.default_width, 900);
        assert!(tmp.join("config.json").exists());
    });
}

#[test]
fn test_load_or_create_handles_corrupted_json() {
    with_temp_config_dir(|tmp| {
        std::fs::write(tmp.join("config.json"), b"not valid json {").unwrap();

        let config = ConfigManager::load_or_create();
        assert_eq!(config.ui.default_width, 900);
        assert!((config.ui.max_text_width.unwrap() - 820.0).abs() < f32::EPSILON);

        let content = std::fs::read_to_string(tmp.join("config.json")).unwrap();
        let reparsed: AppConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(reparsed, AppConfig::default());
    });
}

#[test]
fn test_load_or_create_handles_empty_file() {
    with_temp_config_dir(|tmp| {
        std::fs::write(tmp.join("config.json"), b"").unwrap();

        let config = ConfigManager::load_or_create();
        assert_eq!(config.ui.default_width, 900);
    });
}
