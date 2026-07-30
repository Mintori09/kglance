use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use iced::Font;

pub fn get_code_font(font_family_mono: Option<&str>) -> Font {
    match font_family_mono {
        Some(name) => Font::with_name(Box::leak(resolve_font_name(name).into_boxed_str())),
        None => Font::MONOSPACE,
    }
}

pub fn get_main_font(font_family: Option<&str>) -> Font {
    match font_family {
        Some(name) => Font::with_name(Box::leak(resolve_font_name(name).into_boxed_str())),
        None => Font::DEFAULT,
    }
}

pub(crate) fn resolve_font_name(name: &str) -> String {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = name.to_lowercase();

    {
        let guard = cache.lock().unwrap();
        if let Some(resolved) = guard.get(&key) {
            return resolved.clone();
        }
    }

    let resolved = std::process::Command::new("fc-match")
        .args([name, "--format=%{family[0]}"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !s.is_empty() { Some(s) } else { None }
            } else {
                None
            }
        })
        .unwrap_or_else(|| name.to_string());

    let mut guard = cache.lock().unwrap();
    guard.insert(key, resolved.clone());
    resolved
}
