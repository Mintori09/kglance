use std::path::Path;

pub const SUPPORTED_EXTS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "txt", "md", "typ", "rs", "py", "js", "ts",
    "json", "toml", "yaml", "pdf", "mp4", "mkv", "avi", "webm", "mp3", "wav", "flac", "csv", "tsv",
    "xlsx",
];

pub fn is_supported_extension(path_str: &str) -> bool {
    let path = Path::new(path_str);
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        SUPPORTED_EXTS
            .iter()
            .any(|&supported| ext.eq_ignore_ascii_case(supported))
    } else {
        false
    }
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
            if p.is_file() {
                let p_str = p.to_string_lossy().to_string();
                if is_supported_extension(&p_str) {
                    files.push(p_str);
                }
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
