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
}

pub fn detect_system_theme() -> String {
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
            // theme: detect_system_theme(),
            theme: None,
            font_size: 14.0,
            font_family: None,
            font_family_mono: None,
            epub_font_family: None,
            max_text_width: Some(820.0),
            default_width: 900,
            default_height: 600,
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
        if let Ok(dir) = std::env::var("KGLANCE_CONFIG_DIR") {
            return PathBuf::from(dir);
        }
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("kglance")
    }

    pub fn get_config_path() -> PathBuf {
        Self::get_config_dir().join("config.json")
    }

    pub fn load_or_create() -> AppConfig {
        let path = Self::get_config_path();
        let loaded = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<AppConfig>(&content).ok());

        if let Some(config) = loaded {
            return config;
        }

        let config = AppConfig::default();
        let _ = Self::save(&config);
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

    pub fn get_theme(config: &AppConfig) -> String {
        if let Some(theme) = &config.ui.theme {
            if theme == "Auto" || theme == "auto" {
                detect_system_theme()
            } else {
                theme.to_string()
            }
        } else {
            detect_system_theme()
        }
    }
}
