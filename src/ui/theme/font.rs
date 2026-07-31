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
    crate::parsers::font::resolve_font_name(name)
}
