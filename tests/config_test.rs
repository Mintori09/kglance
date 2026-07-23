use kglance::core::config::AppConfig;

#[test]
fn test_default_config_creation() {
    let config = AppConfig::default();
    assert!(config.ui.theme == "Dark" || config.ui.theme == "Light");
    assert_eq!(config.preview.max_file_size_mb, 50);
    assert!(config.plugins.enable_plugins);
}

#[test]
fn test_config_json_serialization() {
    let config = AppConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}
