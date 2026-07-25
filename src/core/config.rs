use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: f32,
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
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
            theme: detect_system_theme(),
            font_size: 14.0,
            font_family: None,
            font_family_mono: None,
            max_text_width: Some(820.0),
            default_width: 900,
            default_height: 600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreviewConfig {
    pub max_file_size_mb: u64,
    pub render_timeout_ms: u64,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: 50,
            render_timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsersConfig {
    pub enable_syntax_highlighting: bool,
    pub enable_video_preview: bool,
    pub enable_office_parser: bool,
}

impl Default for ParsersConfig {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            enable_video_preview: true,
            enable_office_parser: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub preview: PreviewConfig,
    pub parsers: ParsersConfig,
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_config_dir() -> PathBuf {
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
}
