use kglance::core::config::{AppConfig, ConfigManager, UiConfig};
use kglance::ui::theme::AppTheme;
use tempfile::tempdir;

#[test]
fn test_ui_config_default() {
    let config = UiConfig::default();
    assert_eq!(config.theme, Some("dark".into()));
    assert_eq!(config.font_size, 14.0);
    assert_eq!(config.font_family, None);
    assert_eq!(config.font_family_mono, None);
    assert_eq!(config.epub_font_family, None);
    assert!(config.max_text_width.is_some());
    assert!((config.max_text_width.unwrap() - 820.0).abs() < f32::EPSILON);
    assert_eq!(config.default_width, 1024);
    assert_eq!(config.default_height, 768);
    assert_eq!(config.min_width, 800);
    assert_eq!(config.min_height, 600);
    assert!(!config.prefer_mermaid_cli);
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
            epub_reading_mode: kglance::core::config::EpubReadingMode::Continuous,
            max_text_width: Some(720.0),
            default_width: 1024,
            default_height: 768,
            min_width: 800,
            min_height: 600,
            prefer_mermaid_cli: false,
            word_wrap: false,
            json_tree_view: false,
        },
        ..Default::default()
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_get_theme_light() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Light".into()),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(ConfigManager::get_theme_setting(&config), "Light");
    assert_eq!(ConfigManager::resolve_theme("Light"), AppTheme::Light);
}

#[test]
fn test_get_theme_dark() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Dark".into()),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(ConfigManager::get_theme_setting(&config), "Dark");
    assert_eq!(ConfigManager::resolve_theme("Dark"), AppTheme::Dark);
}

#[test]
fn test_get_theme_nord() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("Nord".into()),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(ConfigManager::get_theme_setting(&config), "Nord");
    assert_eq!(ConfigManager::resolve_theme("Nord"), AppTheme::Nord);
}

#[test]
fn test_get_theme_auto() {
    let config = AppConfig {
        ui: UiConfig {
            theme: Some("auto".into()),
            ..Default::default()
        },
        ..Default::default()
    };
    let theme_setting = ConfigManager::get_theme_setting(&config);
    assert_eq!(theme_setting, "dark");
    let resolved = ConfigManager::resolve_theme(&theme_setting);
    assert_eq!(resolved, AppTheme::Dark);
}

#[test]
fn test_get_theme_none_falls_back_to_system() {
    let mut config = AppConfig::default();
    config.ui.theme = None;
    assert!(config.ui.theme.is_none());
    let theme_setting = ConfigManager::get_theme_setting(&config);
    let resolved = ConfigManager::resolve_theme(&theme_setting);
    assert_eq!(resolved, AppTheme::Dark);
}

#[test]
fn test_config_path_format() {
    let path = ConfigManager::get_config_path();
    let path_str = path.to_string_lossy();
    assert!(path_str.ends_with("kglance/config.json") || path.ends_with("config.json"));
}

#[test]
fn test_config_dir_format() {
    let dir = ConfigManager::get_config_dir();
    let dir_str = dir.to_string_lossy();
    assert!(
        dir_str.ends_with("kglance")
            || dir.ends_with("kglance")
            || std::env::var("KGLANCE_CONFIG_DIR").is_ok()
    );
}

#[test]
fn test_save_load_and_create_default() {
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("config.json");

    let original = AppConfig {
        ui: UiConfig {
            theme: Some("Dark".into()),
            font_size: 18.0,
            font_family: Some("Fira Sans".into()),
            ..Default::default()
        },
        ..Default::default()
    };

    ConfigManager::save_to_path(&config_path, &original).unwrap();
    let loaded = ConfigManager::load_from_path(&config_path);
    assert_eq!(loaded.ui.theme, original.ui.theme);
    assert_eq!(loaded.ui.font_size, original.ui.font_size);
    assert_eq!(loaded.ui.font_family, original.ui.font_family);

    let tmp2 = tempdir().unwrap();
    let config_path2 = tmp2.path().join("config.json");
    let config = ConfigManager::load_from_path(&config_path2);
    assert!((config.ui.max_text_width.unwrap() - 820.0).abs() < f32::EPSILON);
    assert_eq!(config.ui.default_width, 1024);
    assert_eq!(config.ui.min_width, 800);
    assert!(config_path2.exists());
}

#[test]
fn test_load_or_create_handles_corrupted_json() {
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("config.json");
    std::fs::write(&config_path, b"not valid json {").unwrap();

    let config = ConfigManager::load_from_path(&config_path);
    assert_eq!(config.ui.default_width, 1024);
    assert_eq!(config.ui.min_height, 600);
    assert!((config.ui.max_text_width.unwrap() - 820.0).abs() < f32::EPSILON);

    let content = std::fs::read_to_string(&config_path).unwrap();
    let reparsed: AppConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(reparsed, AppConfig::default());
}

#[test]
fn test_load_or_create_handles_empty_file() {
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("config.json");
    std::fs::write(&config_path, b"").unwrap();

    let config = ConfigManager::load_from_path(&config_path);
    assert_eq!(config.ui.default_width, 1024);
    assert_eq!(config.ui.min_width, 800);
}

#[test]
fn test_load_or_create_legacy_config_without_min_fields() {
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("config.json");
    let legacy = r#"{
        "ui": {
            "theme": "Dark",
            "font_size": 15.0,
            "default_width": 900,
            "default_height": 600
        }
    }"#;
    std::fs::write(&config_path, legacy).unwrap();

    let config = ConfigManager::load_from_path(&config_path);
    assert_eq!(config.ui.theme.as_deref(), Some("Dark"));
    assert_eq!(config.ui.font_size, 15.0);
    assert_eq!(config.ui.default_width, 900);
    assert_eq!(config.ui.default_height, 600);
    assert_eq!(config.ui.min_width, 800);
    assert_eq!(config.ui.min_height, 600);
}

#[test]
fn test_cache_config_default() {
    let config = AppConfig::default();
    assert_eq!(
        config.cache.max_memory_mb,
        kglance::core::config::DEFAULT_CACHE_MAX_MEMORY_MB
    );
    assert_eq!(config.cache.max_memory_mb, 512);
}

#[test]
fn test_cache_config_deserialization() {
    let mut config = AppConfig::default();
    config.cache.max_memory_mb = 256;
    let json_custom = serde_json::to_string(&config).unwrap();
    let parsed: AppConfig = serde_json::from_str(&json_custom).expect("failed to parse config");
    assert_eq!(parsed.cache.max_memory_mb, 256);

    // Test legacy JSON without "cache" field
    let legacy_json = r#"{
        "ui": {
            "theme": "Dark",
            "font_size": 14.0,
            "default_width": 1024,
            "default_height": 768
        }
    }"#;
    let parsed_legacy: AppConfig =
        serde_json::from_str(legacy_json).expect("failed to parse legacy config");
    assert_eq!(parsed_legacy.cache.max_memory_mb, 512);
}
