use iced::widget::svg;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static ICON_CACHE: Mutex<Option<HashMap<String, svg::Handle>>> = Mutex::new(None);

const CATEGORIES: &[&str] = &[
    "mimetypes",
    "apps",
    "places",
    "status",
    "devices",
    "categories",
    "emblems",
];
const SIZES: &[&str] = &["64", "48", "32", "24", "22", "16"];

fn detect_current_theme() -> String {
    let config_dir = match dirs::config_dir() {
        Some(d) => d,
        None => return "breeze".to_string(),
    };
    let kdeglobals = config_dir.join("kdeglobals");
    if let Ok(content) = std::fs::read_to_string(&kdeglobals) {
        let mut in_icons = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_icons = line.eq_ignore_ascii_case("[icons]");
                continue;
            }
            if in_icons && let Some(val) = line.strip_prefix("Theme=") {
                let theme = val.trim().to_string();
                if !theme.is_empty() {
                    return theme;
                }
            }
        }
    }
    "breeze".to_string()
}

fn find_icon_in_dir(theme_dir: &Path, icon_name: &str) -> Option<PathBuf> {
    // Breeze-style: <theme>/<category>/<size>/<icon>.svg
    for category in CATEGORIES {
        for size in SIZES {
            let path = theme_dir
                .join(category)
                .join(size)
                .join(format!("{icon_name}.svg"));
            if path.exists() {
                return Some(path);
            }
        }
    }

    // hicolor-style: <theme>/scalable/<category>/<icon>.svg
    for category in CATEGORIES {
        let path = theme_dir
            .join("scalable")
            .join(category)
            .join(format!("{icon_name}.svg"));
        if path.exists() {
            return Some(path);
        }
    }

    // hicolor-style: <theme>/<size>x<size>/<category>/<icon>.png
    for category in CATEGORIES {
        for size in SIZES {
            let path = theme_dir
                .join(format!("{size}x{size}"))
                .join(category)
                .join(format!("{icon_name}.png"));
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

fn icon_base_dirs() -> [PathBuf; 4] {
    [
        dirs::home_dir()
            .map(|h| h.join(".icons"))
            .unwrap_or_default(),
        dirs::data_dir()
            .map(|d| d.join("icons"))
            .unwrap_or_default(),
        PathBuf::from("/usr/share/icons"),
        PathBuf::from("/usr/local/share/icons"),
    ]
}

fn available_themes() -> Vec<String> {
    let mut themes = Vec::new();

    let detected = detect_current_theme();
    themes.push(detected.clone());

    let base_dirs = icon_base_dirs();

    let mut candidates = Vec::new();

    for base in &base_dirs {
        if !base.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                    && entry.path().join("index.theme").exists()
                {
                    candidates.push(name);
                }
            }
        }
    }

    candidates.sort();
    candidates.dedup();

    // Prioritize: detected → breeze → Adwaita → Qogir → rest (including hicolor)
    let fallback_order = ["breeze", "Adwaita", "Qogir", "hicolor"];
    for fallback in &fallback_order {
        if *fallback != detected && candidates.contains(&fallback.to_string()) {
            themes.push(fallback.to_string());
        }
    }
    for t in &candidates {
        if !themes.contains(t) {
            themes.push(t.clone());
        }
    }

    themes
}

fn resolve_icon_path(icon_name: &str) -> Option<PathBuf> {
    let themes = available_themes();

    let base_dirs = icon_base_dirs();

    for theme in &themes {
        for base in &base_dirs {
            let theme_dir = base.join(theme);
            if theme_dir.exists()
                && let Some(p) = find_icon_in_dir(&theme_dir, icon_name)
            {
                return Some(p);
            }
        }
    }

    None
}

pub fn get_icon_handle(icon_name: &str) -> Option<svg::Handle> {
    {
        let cache = ICON_CACHE.lock().unwrap();
        if let Some(ref map) = *cache
            && let Some(handle) = map.get(icon_name)
        {
            return Some(handle.clone());
        }
    }

    let path = resolve_icon_path(icon_name)?;
    let handle = svg::Handle::from_path(path);
    let mut cache = ICON_CACHE.lock().unwrap();
    cache
        .get_or_insert_with(HashMap::new)
        .insert(icon_name.to_string(), handle.clone());
    Some(handle)
}

pub fn icon_for_entry(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return "inode-directory";
    }

    // Match special filenames first
    let lower_name = name.to_ascii_lowercase();

    match lower_name.as_str() {
        "cargo.toml" => return "text-x-toml",
        "cargo.lock" => return "text-x-generic",
        "cmakelists.txt" => return "text-x-cmake",
        "dockerfile" => return "text-x-dockerfile",
        "containerfile" => return "text-x-dockerfile",
        "makefile" => return "text-x-makefile",
        "license" | "copying" => return "text-x-copying",
        "readme" | "readme.md" => return "text-x-readme",
        ".gitignore" => return "text-x-generic",
        ".gitattributes" => return "text-x-generic",
        ".editorconfig" => return "text-x-generic",
        ".env" => return "text-x-generic",
        _ => {}
    }

    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        // =========================
        // Rust
        // =========================
        "rs" => "text-x-rust-source",

        // =========================
        // C / C++
        // =========================
        "c" | "h" => "text-x-c-source",
        "cpp" | "cxx" | "cc" | "hpp" | "hh" | "hxx" => "text-x-c++source",

        // =========================
        // Programming Languages
        // =========================
        "py" => "text-x-python",
        "pyi" => "text-x-python",
        "pyw" => "text-x-python",

        "go" => "text-x-go",

        "java" => "text-x-java-source",
        "class" => "application-java",

        "kt" | "kts" => "text-x-kotlin",

        "swift" => "text-x-swift",

        "cs" => "text-x-csharp",

        "php" => "application-x-php",

        "rb" => "application-x-ruby",

        "lua" => "text-x-lua",

        "zig" => "text-x-zig",

        "dart" => "text-x-dart",

        "scala" => "text-x-scala",

        "hs" => "text-x-haskell",

        "ml" | "mli" => "text-x-ocaml",

        "fs" | "fsi" | "fsx" => "text-x-fsharp",

        "r" => "text-x-r-source",

        "jl" => "text-x-julia",

        "nim" => "text-x-nim",

        "clj" | "cljs" | "cljc" => "text-x-clojure",

        "erl" => "text-x-erlang",

        "ex" | "exs" => "text-x-elixir",

        "sql" => "text-x-sql",

        // =========================
        // Web
        // =========================
        "html" | "htm" => "text-html",
        "css" => "text-css",
        "scss" | "sass" => "text-x-scss",
        "less" => "text-x-less",

        "js" | "mjs" | "cjs" => "text-x-javascript",
        "jsx" => "text-jsx",

        "ts" => "text-x-typescript",
        "tsx" => "text-tsx",

        "vue" => "text-x-vue",
        "svelte" => "text-x-svelte",
        "astro" => "text-x-astro",

        // =========================
        // Config
        // =========================
        "json" => "application-json",
        "jsonc" => "application-json",
        "toml" => "text-x-toml",
        "yaml" | "yml" => "text-x-yaml",
        "xml" => "application-xml",
        "ini" | "cfg" | "conf" => "text-x-config",
        "properties" => "text-x-java-properties",

        // =========================
        // Shell
        // =========================
        "sh" => "text-x-shellscript",
        "bash" => "text-x-shellscript",
        "zsh" => "text-x-shellscript",
        "fish" => "text-x-shellscript",
        "ps1" | "psm1" => "text-x-powershell",
        "bat" | "cmd" => "application-x-ms-dos-executable",

        // =========================
        // Documents
        // =========================
        "txt" => "text-plain",
        "md" | "markdown" => "text-x-markdown",
        "typ" => "text-x-typst",
        "pdf" => "application-pdf",
        "csv" => "text-csv",
        "rtf" => "application-rtf",

        "doc" | "docx" => "application-msword",
        "xls" | "xlsx" => "application-vnd-ms-excel",
        "ppt" | "pptx" => "application-vnd-ms-powerpoint",

        "odt" => "application-vnd.oasis.opendocument.text",
        "ods" => "application-vnd.oasis.opendocument.spreadsheet",
        "odp" => "application-vnd.oasis.opendocument.presentation",

        "epub" => "application-epub+zip",
        "mobi" => "application-x-mobipocket-ebook",
        "azw" | "azw3" => "application-x-mobipocket-ebook",
        "fb2" => "application-x-fictionbook+xml",
        "djvu" | "djv" => "image-vnd.djvu",

        // =========================
        // Images
        // =========================
        "png" => "image-png",
        "jpg" | "jpeg" => "image-jpeg",
        "gif" => "image-gif",
        "bmp" => "image-bmp",
        "webp" => "image-webp",
        "svg" => "image-svg+xml",
        "ico" => "image-x-icon",
        "tif" | "tiff" => "image-tiff",
        "avif" => "image-avif",
        "heic" | "heif" => "image-heif",
        "psd" => "image-vnd.adobe.photoshop",
        "xcf" => "image-x-xcf",
        "kra" => "application-x-krita",

        // =========================
        // Video
        // =========================
        "mp4" => "video-mp4",
        "mkv" => "video-x-matroska",
        "webm" => "video-webm",
        "avi" => "video-x-msvideo",
        "mov" => "video-quicktime",
        "wmv" => "video-x-ms-wmv",
        "mpeg" | "mpg" => "video-mpeg",
        "m4v" => "video-mp4",
        "flv" => "video-x-flv",
        "3gp" => "video-3gpp",
        "ogv" => "video-ogg",

        // =========================
        // Audio
        // =========================
        "mp3" => "audio-mpeg",
        "wav" => "audio-wav",
        "flac" => "audio-flac",
        "ogg" | "oga" => "audio-vorbis",
        "opus" => "audio-opus",
        "aac" => "audio-aac",
        "m4a" => "audio-mp4",
        "mid" | "midi" => "audio-midi",
        "aiff" | "aif" => "audio-x-aiff",
        "ape" => "audio-x-ape",
        "amr" => "audio-amr",
        "wma" => "audio-x-ms-wma",

        // =========================
        // Archives
        // =========================
        "zip" => "application-zip",
        "tar" => "application-x-tar",
        "gz" | "tgz" => "application-gzip",
        "bz2" => "application-x-bzip",
        "xz" => "application-x-xz",
        "zst" => "application-zstd",
        "7z" => "application-x-7z-compressed",
        "rar" => "application-vnd.rar",
        "deb" => "application-vnd.debian.binary-package",
        "rpm" => "application-x-rpm",
        "iso" => "application-x-cd-image",

        // =========================
        // Fonts
        // =========================
        "ttf" => "font-ttf",
        "ttc" => "font-ttf",
        "otf" => "font-otf",
        "woff" | "woff2" => "font-woff",

        // =========================
        // Database
        // =========================
        "db" | "sqlite" | "sqlite3" => "application-x-sqlite3",

        // =========================
        // Certificates
        // =========================
        "pem" | "crt" | "cer" | "key" | "csr" | "p12" | "pfx" => "application-x-x509-ca-cert",

        // =========================
        // Logs
        // =========================
        "log" => "text-x-log",

        // =========================
        // Executables
        // =========================
        "exe" => "application-x-ms-dos-executable",
        "dll" => "application-x-ms-dos-executable",
        "msi" => "application-x-ms-dos-executable",
        "so" => "application-x-sharedlib",
        "appimage" => "application-x-executable",

        // =========================
        // Default
        // =========================
        _ => "text-x-generic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_theme_returns_string() {
        let theme = detect_current_theme();
        assert!(!theme.is_empty(), "theme should not be empty");
    }

    // #[test]
    // fn test_standard_icons_resolvable() {
    //     let theme = detect_current_theme();
    //     let required = &[
    //         "inode-directory",
    //         "video-x-matroska",
    //         "text-plain",
    //         "application-pdf",
    //         "application-zip",
    //         "video-mp4",
    //         "image-png",
    //         "image-jpeg",
    //     ];
    //     for name in required {
    //         let found = resolve_icon_path(name).is_some();
    //         assert!(
    //             found,
    //             "required icon '{name}' should be resolvable (theme='{theme}')"
    //         );
    //     }
    // }

    #[test]
    fn test_common_icons_found() {
        let names = &[
            "inode-directory",
            "video-x-matroska",
            "video-mp4",
            "image-png",
            "image-jpeg",
            "application-pdf",
            "application-zip",
            "text-x-markdown",
            "audio-mpeg",
        ];
        let found: Vec<&str> = names
            .iter()
            .filter(|n| resolve_icon_path(n).is_some())
            .copied()
            .collect();
        assert!(
            !found.is_empty(),
            "at least one system icon should be resolvable. names checked: {:?}",
            names
        );
    }

    #[test]
    fn test_icons_cached_after_first_load() {
        let name = "inode-directory";
        let handle1 = get_icon_handle(name);
        if handle1.is_some() {
            let handle2 = get_icon_handle(name);
            assert!(handle2.is_some(), "cached icon should still be available");
            let cache = ICON_CACHE.lock().unwrap();
            let map = cache.as_ref().unwrap();
            assert!(map.contains_key(name), "icon should be in cache");
        }
    }

    #[test]
    fn test_nonexistent_icon_returns_none() {
        let result = resolve_icon_path("this-icon-does-not-exist-xyz123");
        assert!(result.is_none(), "nonexistent icon should return None");
    }

    #[test]
    fn test_get_nonexistent_handle_returns_none() {
        let result = get_icon_handle("this-icon-does-not-exist-xyz123");
        assert!(
            result.is_none(),
            "nonexistent icon handle should return None"
        );
    }
}
