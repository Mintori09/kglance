use iced::Font;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub fn intern_font_name(name: &str) -> &'static str {
    static INTERNED: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let set = INTERNED.get_or_init(|| Mutex::new(HashSet::new()));
    let mut guard = set
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(&interned) = guard.get(name) {
        interned
    } else {
        let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
        guard.insert(leaked);
        leaked
    }
}

pub fn get_code_font(font_family_mono: Option<&str>) -> Font {
    match font_family_mono {
        Some(name) => Font::with_name(intern_font_name(&resolve_font_name(name))),
        None => Font::MONOSPACE,
    }
}

pub fn get_main_font(font_family: Option<&str>) -> Font {
    match font_family {
        Some(name) => Font::with_name(intern_font_name(&resolve_font_name(name))),
        None => Font::DEFAULT,
    }
}

pub fn get_main_font_bold(font_family: Option<&str>) -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..get_main_font(font_family)
    }
}

pub fn get_main_font_medium(font_family: Option<&str>) -> Font {
    Font {
        weight: iced::font::Weight::Medium,
        ..get_main_font(font_family)
    }
}

pub fn get_epub_font(epub_font_family: Option<&str>, font_family: Option<&str>) -> Font {
    get_main_font(epub_font_family.or(font_family))
}

pub(crate) fn resolve_font_name(name: &str) -> String {
    crate::parsers::font::resolve_font_name(name)
}
