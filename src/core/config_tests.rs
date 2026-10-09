use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_default_config_values() {
    let cfg = AppConfig::default();

    // UI defaults
    assert_eq!(cfg.ui.theme, Some("dark".into()));
    assert_eq!(cfg.ui.font_size, 14.0);
    assert_eq!(cfg.ui.font_family, None);
    assert_eq!(cfg.ui.font_family_mono, None);
    assert_eq!(cfg.ui.epub_font_family, None);
    assert_eq!(cfg.ui.max_text_width, Some(820.0));
    assert_eq!(cfg.ui.default_width, 1024);
    assert_eq!(cfg.ui.default_height, 768);
    assert_eq!(cfg.ui.min_width, 800);
    assert_eq!(cfg.ui.min_height, 600);
    assert!(!cfg.ui.prefer_mermaid_cli);
    assert!(!cfg.ui.word_wrap);
    assert!(cfg.ui.json_tree_view);

    // Cache defaults
    assert_eq!(cfg.cache.max_memory_mb, DEFAULT_CACHE_MAX_MEMORY_MB);
    assert_eq!(cfg.cache.max_disk_cache_mb, DEFAULT_CACHE_MAX_DISK_MB);

    // Scroll defaults
    assert!(cfg.scroll.smooth_scroll_enabled);
    assert_eq!(
        cfg.scroll.friction,
        crate::core::scroll::KINETIC_FRICTION_COEFFICIENT
    );
    assert_eq!(cfg.scroll.spring_stiffness, 180.0);
}

#[test]
fn test_load_or_create_when_file_not_exists() {
    let dir = tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.json");
    assert!(!config_path.exists());

    let loaded = ConfigManager::load_from_path(&config_path);
    let expected_default = AppConfig::default();
    assert_eq!(loaded, expected_default);

    // Ensure the default file was created on disk
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).expect("read written config");
    let parsed: AppConfig = serde_json::from_str(&content).expect("parse written config");
    assert_eq!(parsed, expected_default);
}

#[test]
fn test_load_existing_custom_config() {
    let dir = tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.json");

    let custom_json = r#"{
  "ui": {
    "theme": "Nord",
    "font_size": 18.0,
    "font_family": "Fira Code",
    "font_family_mono": "JetBrains Mono",
    "epub_font_family": "Serif",
    "max_text_width": 900.0,
    "default_width": 1280,
    "default_height": 900,
    "min_width": 700,
    "min_height": 500,
    "prefer_mermaid_cli": true,
    "word_wrap": true,
    "json_tree_view": false
  },
  "cache": {
    "max_memory_mb": 1024
  },
  "scroll": {
    "smooth_scroll_enabled": false,
    "friction": 2.5,
    "spring_stiffness": 200.0
  }
}"#;

    fs::write(&config_path, custom_json).expect("write custom config");

    let loaded = ConfigManager::load_from_path(&config_path);

    let expected = AppConfig {
        ui: UiConfig {
            theme: Some("Nord".into()),
            font_size: 18.0,
            font_family: Some("Fira Code".into()),
            font_family_mono: Some("JetBrains Mono".into()),
            epub_font_family: Some("Serif".into()),
            epub_reading_mode: EpubReadingMode::SingleChapter,
            max_text_width: Some(900.0),
            default_width: 1280,
            default_height: 900,
            min_width: 700,
            min_height: 500,
            prefer_mermaid_cli: true,
            word_wrap: true,
            json_tree_view: false,
        },
        cache: CacheConfig {
            max_memory_mb: 1024,
            max_disk_cache_mb: DEFAULT_CACHE_MAX_DISK_MB,
        },
        scroll: ScrollConfigOptions {
            smooth_scroll_enabled: false,
            friction: 2.5,
            spring_stiffness: 200.0,
        },
    };

    assert_eq!(loaded, expected);
}

#[test]
fn test_save_and_reload_persisted_changes() {
    let dir = tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.json");

    // Load initial config (creates default config)
    let mut config = ConfigManager::load_from_path(&config_path);
    assert_eq!(config.ui.font_size, 14.0);
    assert!(!config.ui.word_wrap);

    // Modify fields as would happen in settings UI
    config.ui.font_size = 16.5;
    config.ui.word_wrap = true;
    config.ui.theme = Some("Light".into());
    config.cache.max_memory_mb = 256;

    // Save
    ConfigManager::save_to_path(&config_path, &config).expect("save config");

    // Load again in fresh call
    let reloaded = ConfigManager::load_from_path(&config_path);
    assert_eq!(reloaded, config);

    // Verify on disk file content
    let content = fs::read_to_string(&config_path).expect("read file");
    let parsed: AppConfig = serde_json::from_str(&content).expect("parse file");
    assert_eq!(parsed, config);
}

#[test]
fn test_load_fallback_when_corrupted_json() {
    let dir = tempdir().expect("create temp dir");
    let config_path = dir.path().join("config.json");
    fs::write(&config_path, "not a valid json {{{").expect("write bad json");

    // When corrupted, load_from_path should fallback to default and rewrite valid config
    let loaded = ConfigManager::load_from_path(&config_path);
    let expected_default = AppConfig::default();
    assert_eq!(loaded, expected_default);

    // Verify file on disk is now recovered with valid config
    let content = fs::read_to_string(&config_path).expect("read recovered file");
    let parsed: AppConfig = serde_json::from_str(&content).expect("parse recovered json");
    assert_eq!(parsed, expected_default);
}

#[test]
fn test_theme_resolution() {
    assert_eq!(ConfigManager::resolve_theme("Light"), AppTheme::Light);
    assert_eq!(ConfigManager::resolve_theme("light"), AppTheme::Light);
    assert_eq!(
        ConfigManager::resolve_theme("breeze-light"),
        AppTheme::Light
    );
    assert_eq!(ConfigManager::resolve_theme("Dark"), AppTheme::Dark);
    assert_eq!(ConfigManager::resolve_theme("dark"), AppTheme::Dark);
    assert_eq!(ConfigManager::resolve_theme("breeze-dark"), AppTheme::Dark);
    assert_eq!(ConfigManager::resolve_theme("Nord"), AppTheme::Nord);
    assert_eq!(ConfigManager::resolve_theme("nord"), AppTheme::Nord);
    assert_eq!(
        ConfigManager::resolve_theme("Catppuccin-Mocha"),
        AppTheme::CatppuccinMocha
    );
    assert_eq!(
        ConfigManager::resolve_theme("catppuccin_mocha"),
        AppTheme::CatppuccinMocha
    );
    assert_eq!(
        ConfigManager::resolve_theme("mocha"),
        AppTheme::CatppuccinMocha
    );
    assert_eq!(
        ConfigManager::resolve_theme("Catppuccin-Latte"),
        AppTheme::CatppuccinLatte
    );
    assert_eq!(
        ConfigManager::resolve_theme("latte"),
        AppTheme::CatppuccinLatte
    );
    assert_eq!(
        ConfigManager::resolve_theme("Tokyo-Night"),
        AppTheme::TokyoNight
    );
    assert_eq!(
        ConfigManager::resolve_theme("tokyonight"),
        AppTheme::TokyoNight
    );
    assert_eq!(
        ConfigManager::resolve_theme("Gruvbox"),
        AppTheme::GruvboxDark
    );
    assert_eq!(
        ConfigManager::resolve_theme("gruvbox-dark"),
        AppTheme::GruvboxDark
    );
    assert_eq!(ConfigManager::resolve_theme("Dracula"), AppTheme::Dracula);
    assert_eq!(ConfigManager::resolve_theme("dracula"), AppTheme::Dracula);
    assert_eq!(ConfigManager::resolve_theme("unknown"), AppTheme::Dark);
}

#[test]
fn test_epub_reading_mode_config_serde() {
    let default_cfg = AppConfig::default();
    assert_eq!(
        default_cfg.ui.epub_reading_mode,
        EpubReadingMode::SingleChapter
    );

    let parsed_continuous: EpubReadingMode =
        serde_json::from_str("\"continuous\"").expect("parse continuous mode");
    assert_eq!(parsed_continuous, EpubReadingMode::Continuous);

    let parsed_single: EpubReadingMode =
        serde_json::from_str("\"single_chapter\"").expect("parse single_chapter mode");
    assert_eq!(parsed_single, EpubReadingMode::SingleChapter);

    let serialized = serde_json::to_string(&EpubReadingMode::Continuous).unwrap();
    assert_eq!(serialized, "\"continuous\"");

    let mut cfg = AppConfig::default();
    cfg.ui.epub_reading_mode = EpubReadingMode::Continuous;
    let json = serde_json::to_string(&cfg).unwrap();
    let roundtrip: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.ui.epub_reading_mode, EpubReadingMode::Continuous);
}
