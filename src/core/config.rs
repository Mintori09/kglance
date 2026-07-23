use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: f32,
    pub default_width: u32,
    pub default_height: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_size: 14.0,
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
pub struct CliPluginConfig {
    pub name: String,
    pub file_extensions: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
    pub output_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NativePluginConfig {
    pub name: String,
    pub file_extensions: Vec<String>,
    pub library_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginsConfig {
    pub enable_plugins: bool,
    pub plugin_dir: String,
    pub cli_plugins: Vec<CliPluginConfig>,
    pub native_plugins: Vec<NativePluginConfig>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enable_plugins: true,
            plugin_dir: "~/.config/kglance/plugins".to_string(),
            cli_plugins: vec![CliPluginConfig {
                name: "PostScript via Ghostscript".to_string(),
                file_extensions: vec!["ps".to_string(), "eps".to_string()],
                command: "gs".to_string(),
                args: vec![
                    "-q".to_string(),
                    "-dQUIET".to_string(),
                    "-dSAFER".to_string(),
                    "-dBATCH".to_string(),
                    "-dNOPAUSE".to_string(),
                    "-sDEVICE=png16m".to_string(),
                    "-r150".to_string(),
                    "-sOutputFile=%stdout".to_string(),
                    "{file}".to_string(),
                ],
                output_type: "Image".to_string(),
            }],
            native_plugins: Vec::new(),
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
    pub plugins: PluginsConfig,
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
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
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
