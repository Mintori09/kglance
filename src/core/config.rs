use crate::log_error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_CACHE_MAX_MEMORY_MB: usize = 512;
pub const DEFAULT_CACHE_MAX_DISK_MB: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpubReadingMode {
    #[default]
    SingleChapter,
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RubyDisplayMode {
    #[default]
    Stacked,
    Tooltip,
    Parentheses,
    Hidden,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UiConfig {
    pub theme: Option<String>,
    pub font_size: f32,
    pub font_family: Option<String>,
    pub font_family_mono: Option<String>,
    pub epub_font_family: Option<String>,
    #[serde(default)]
    pub epub_reading_mode: EpubReadingMode,
    #[serde(default)]
    pub ruby_display_mode: RubyDisplayMode,
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
    #[serde(default)]
    pub remember_window_size: bool,
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

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Some("dark".into()),
            font_size: 14.0,
            font_family: None,
            font_family_mono: None,
            epub_font_family: None,
            epub_reading_mode: EpubReadingMode::default(),
            ruby_display_mode: RubyDisplayMode::default(),
            max_text_width: Some(820.0),
            default_width: 1024,
            default_height: 768,
            min_width: default_min_width(),
            min_height: default_min_height(),
            prefer_mermaid_cli: false,
            word_wrap: false,
            json_tree_view: default_json_tree_view(),
            remember_window_size: false,
        }
    }
}

fn default_cache_max_memory_mb() -> usize {
    DEFAULT_CACHE_MAX_MEMORY_MB
}

fn default_cache_max_disk_mb() -> usize {
    DEFAULT_CACHE_MAX_DISK_MB
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    #[serde(default = "default_cache_max_memory_mb")]
    pub max_memory_mb: usize,
    #[serde(default = "default_cache_max_disk_mb")]
    pub max_disk_cache_mb: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: DEFAULT_CACHE_MAX_MEMORY_MB,
            max_disk_cache_mb: DEFAULT_CACHE_MAX_DISK_MB,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrollConfigOptions {
    #[serde(default = "default_smooth_scroll_enabled")]
    pub smooth_scroll_enabled: bool,
    #[serde(default = "default_scroll_friction")]
    pub friction: f32,
    #[serde(default = "default_scroll_spring_stiffness")]
    pub spring_stiffness: f32,
}

fn default_smooth_scroll_enabled() -> bool {
    true
}

fn default_scroll_friction() -> f32 {
    crate::core::scroll::KINETIC_FRICTION_COEFFICIENT
}

fn default_scroll_spring_stiffness() -> f32 {
    180.0
}

impl Default for ScrollConfigOptions {
    fn default() -> Self {
        Self {
            smooth_scroll_enabled: default_smooth_scroll_enabled(),
            friction: default_scroll_friction(),
            spring_stiffness: default_scroll_spring_stiffness(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub scroll: ScrollConfigOptions,
    #[serde(default)]
    pub selection_menu: crate::core::selection_action::SelectionMenuConfig,
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

    pub fn load_from_path(path: &std::path::Path) -> AppConfig {
        if !path.exists() {
            let config = AppConfig::default();
            let _ = Self::save_to_path(path, &config);
            return config;
        }

        let raw = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                log_error!(
                    "[kglance] Failed to read config file {}: {err}",
                    path.display()
                );
                return AppConfig::default();
            }
        };

        let sanitized = strip_json_comments(&raw);

        match serde_json::from_str::<AppConfig>(&sanitized) {
            Ok(mut config) => {
                if let Some(ref t) = config.ui.theme
                    && t.eq_ignore_ascii_case("auto")
                {
                    config.ui.theme = Some("dark".to_string());
                }
                config
            }
            Err(err) => {
                log_error!(
                    "[kglance] Failed to parse config JSON at {}: {err}",
                    path.display()
                );
                AppConfig::default()
            }
        }
    }

    pub fn load_or_create() -> AppConfig {
        Self::load_from_path(&Self::get_config_path())
    }

    pub fn save_to_path(path: &std::path::Path, config: &AppConfig) -> Result<(), String> {
        static SAVE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;

        let pid = std::process::id();
        let counter = SAVE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let file_stem = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("config.json");
        let tmp_path = parent.join(format!(".{file_stem}.tmp.{pid}.{counter}"));

        fs::write(&tmp_path, json).map_err(|e| e.to_string())?;
        fs::rename(&tmp_path, path).map_err(|e| {
            let _ = fs::remove_file(&tmp_path);
            e.to_string()
        })?;
        Ok(())
    }

    pub fn save(config: &AppConfig) -> Result<(), String> {
        Self::save_to_path(&Self::get_config_path(), config)
    }

    pub fn get_theme_setting(config: &AppConfig) -> String {
        match config.ui.theme.as_deref() {
            Some(t) if !t.eq_ignore_ascii_case("auto") => t.to_string(),
            _ => "dark".to_string(),
        }
    }

    pub fn resolve_theme(setting_str: &str) -> AppTheme {
        setting_str.parse().unwrap_or(AppTheme::Dark)
    }
}

pub fn strip_json_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            output.push(ch);
        } else if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next_ch in chars.by_ref() {
                if next_ch == '\n' {
                    output.push('\n');
                    break;
                }
            }
        } else if ch == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(next_ch) = chars.next() {
                if next_ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
                if next_ch == '\n' {
                    output.push('\n');
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
