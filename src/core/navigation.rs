use std::path::Path;

pub const SUPPORTED_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "txt", "md", "typ", "rs", "py", "js", "ts",
    "json", "toml", "yaml", "pdf", "mp4", "mkv", "avi", "webm", "mp3", "wav", "flac", "csv", "tsv",
    "xlsx", "epub",
];

pub fn is_supported_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTS.iter().any(|&s| ext.eq_ignore_ascii_case(s)))
        .unwrap_or(false)
}

pub fn scan_sibling_files(file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    let parent = match path.parent() {
        Some(p) => p,
        None => return vec![file_path.to_string()],
    };

    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            let p = entry.path();

            if p.is_file() && is_supported_path(&p) {
                files.push(p.to_string_lossy().into_owned());
            }
        }
    }

    files.sort_by_key(|a| a.to_lowercase());
    if files.is_empty() {
        vec![file_path.to_string()]
    } else {
        files
    }
}
