use crate::{log_debug, log_error};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    pub theme: Option<String>,
    pub font_size: f32,
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    pub epub_font_family: Option<String>,
    pub max_text_width: Option<f32>,
    pub default_width: u32,
    pub default_height: u32,
    #[serde(default = "default_min_width")]
    pub min_width: u32,
    #[serde(default = "default_min_height")]
    pub min_height: u32,
    #[serde(default)]
    pub prefer_mermaid_cli: bool,
    #[serde(default)]
    pub word_wrap: bool,
    #[serde(default = "default_json_tree_view")]
    pub json_tree_view: bool,
}

fn default_json_tree_view() -> bool {
    true
}

fn default_min_width() -> u32 {
    800
}

fn default_min_height() -> u32 {
    600
}

use crate::ui::theme::AppTheme;

pub fn detect_system_theme() -> String {
    // 1. KDE Plasma 6 check via kreadconfig6
    if let Ok(output) = std::process::Command::new("kreadconfig6")
        .args(["--group", "General", "--key", "ColorScheme"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
        if !stdout.trim().is_empty() {
            if stdout.contains("dark") || stdout.contains("breeze-dark") || stdout.contains("black")
            {
                return "Dark".to_string();
            } else if stdout.contains("light") || stdout.contains("breeze-light") {
                return "Light".to_string();
            }
        }
    }

    // 2. GNOME / Freedesktop color-scheme check via gsettings
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("prefer-light") {
            return "Light".to_string();
        } else if stdout.contains("prefer-dark") {
            return "Dark".to_string();
        }
    }

    "Dark".to_string()
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Some("Auto".into()),
            font_size: 14.0,
            font_family: None,
            font_family_mono: None,
            epub_font_family: None,
            max_text_width: Some(820.0),
            default_width: 1024,
            default_height: 768,
            min_width: default_min_width(),
            min_height: default_min_height(),
            prefer_mermaid_cli: false,
            word_wrap: false,
            json_tree_view: default_json_tree_view(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    pub ui: UiConfig,
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("KGLANCE_CONFIG_DIR")
            && !dir.trim().is_empty()
        {
            return PathBuf::from(dir);
        }

        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("kglance");
            return config_dir;
        }

        dirs::home_dir()
            .map(|h| h.join(".config").join("kglance"))
            .unwrap_or_else(|| PathBuf::from(".kglance"))
    }

    pub fn get_config_path() -> PathBuf {
        Self::get_config_dir().join("config.json")
    }

    pub fn load_or_create() -> AppConfig {
        let path = Self::get_config_path();
        let raw = fs::read_to_string(&path).ok();

        let loaded = match raw.as_deref() {
            Some(content) => match serde_json::from_str::<AppConfig>(content) {
                Ok(cfg) => Some(cfg),
                Err(err) => {
                    log_error!("[kglance] Failed to parse config JSON: {err}");
                    None
                }
            },
            None => None,
        };

        if let Some(config) = loaded {
            let reserialized = serde_json::to_string_pretty(&config).unwrap_or_default();
            if raw.as_deref() != Some(reserialized.as_str()) && Self::save(&config).is_err() {
                eprintln!("[kglance] failed to update config");
            }
            return config;
        }

        let config = AppConfig::default();
        let _ = Self::save(&config);
        log_debug!(
            "[TRACE:CONFIG_LOAD] Loaded config.ui.json_tree_view = {}",
            config.ui.json_tree_view
        );
        config
    }

    pub fn save(config: &AppConfig) -> Result<(), String> {
        let dir = Self::get_config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(Self::get_config_path(), json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_theme_setting(config: &AppConfig) -> String {
        config
            .ui
            .theme
            .clone()
            .unwrap_or_else(|| "Auto".to_string())
    }

    pub fn resolve_theme(setting_str: &str) -> AppTheme {
        match setting_str {
            "Auto" | "auto" => {
                let detected = detect_system_theme();
                if detected == "Light" {
                    AppTheme::Light
                } else {
                    AppTheme::Dark
                }
            }
            "Light" | "light" => AppTheme::Light,
            "Nord" | "nord" => AppTheme::Nord,
            _ => AppTheme::Dark,
        }
    }
}

#[test]
#[ignore]
fn test_debug_deserialize_config() {
    let path = ConfigManager::get_config_path();
    println!("\n[TEST PATH] {:?}", path);

    let raw = fs::read_to_string(&path).expect("Failed to read config file");
    println!("[TEST RAW CONTENT]\n{}", raw);

    match serde_json::from_str::<AppConfig>(&raw) {
        Ok(cfg) => {
            println!(
                "[TEST SUCCESS] ui.json_tree_view = {}",
                cfg.ui.json_tree_view
            );
        }
        Err(err) => {
            panic!("[TEST FAILED] Serde deserialize error: {:?}", err);
        }
    }
}
