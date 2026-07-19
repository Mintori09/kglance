use std::path::PathBuf;

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        })
}

pub fn detect_dark_mode() -> bool {
    let kdeglobals = config_dir().join("kdeglobals");
    let content = match std::fs::read_to_string(&kdeglobals) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut in_general = false;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_general = t.eq_ignore_ascii_case("[General]");
            continue;
        }
        if in_general && t.starts_with("ColorScheme=") {
            return t["ColorScheme=".len()..].contains("Dark");
        }
    }
    false
}

pub fn slint_color(hex: u32) -> slint::Color {
    let r = ((hex >> 16) & 0xff) as f32 / 255.0;
    let g = ((hex >> 8) & 0xff) as f32 / 255.0;
    let b = (hex & 0xff) as f32 / 255.0;
    slint::Color::from_rgb_f32(r, g, b)
}

pub struct Palette {
    pub bg: u32,
    pub text: u32,
    pub text_muted: u32,
    pub text_faint: u32,
    pub border: u32,
    pub row_even: u32,
    pub row_odd: u32,
    pub row_hover: u32,
    pub status_bg: u32,
    pub text_mono: u32,
}

pub const LIGHT: Palette = Palette {
    bg: 0xf8f9fa,
    text: 0x212529,
    text_muted: 0x495057,
    text_faint: 0x6c757d,
    border: 0xdee2e6,
    row_even: 0xffffff,
    row_odd: 0xf1f3f5,
    row_hover: 0xe9ecef,
    status_bg: 0xe9ecef,
    text_mono: 0x333333,
};

pub const DARK: Palette = Palette {
    bg: 0x1e1e1e,
    text: 0xd4d4d4,
    text_muted: 0x969696,
    text_faint: 0x808080,
    border: 0x404040,
    row_even: 0x252526,
    row_odd: 0x2d2d2d,
    row_hover: 0x37373d,
    status_bg: 0x333333,
    text_mono: 0xcccccc,
};
