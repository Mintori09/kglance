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

pub fn scan_directory_files(dir_path: &Path) -> Vec<String> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let p = entry.path();

            if p.is_file() && is_supported_path(&p) {
                files.push(p.to_string_lossy().into_owned());
            }
        }
    }

    files.sort_by_cached_key(|path| path.to_ascii_lowercase());
    files
}

pub fn scan_sibling_files(file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    let parent = match path.parent() {
        Some(p) => p,
        None => return vec![file_path.to_string()],
    };

    let files = scan_directory_files(parent);
    if files.is_empty() {
        if path.exists() {
            vec![file_path.to_string()]
        } else {
            Vec::new()
        }
    } else {
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_path() {
        assert!(is_supported_path(Path::new("test.png")));
        assert!(is_supported_path(Path::new("test.TXT")));
        assert!(is_supported_path(Path::new("test.markdown.md")));
        assert!(!is_supported_path(Path::new("test.exe")));
        assert!(!is_supported_path(Path::new("no_ext")));
    }

    #[test]
    fn test_scan_directory_files_filters_unsupported() {
        let temp_dir =
            std::env::temp_dir().join(format!("kglance_nav_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let valid_file = temp_dir.join("valid.png");
        let invalid_file = temp_dir.join("invalid.unknown_ext");
        let sub_dir = temp_dir.join("subdir.png"); // directory named with .png

        let _ = std::fs::write(&valid_file, b"test");
        let _ = std::fs::write(&invalid_file, b"test");
        let _ = std::fs::create_dir_all(&sub_dir);

        let scanned = scan_directory_files(&temp_dir);
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0], valid_file.to_string_lossy());

        let _ = std::fs::remove_file(&valid_file);
        let _ = std::fs::remove_file(&invalid_file);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scan_sibling_files_fallback_when_empty() {
        let temp_dir =
            std::env::temp_dir().join(format!("kglance_empty_dir_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let nonexistent = temp_dir.join("non_existent_file_test_99999.png");
        let res = scan_sibling_files(&nonexistent.to_string_lossy());
        // Nonexistent file in empty directory returns empty vec
        assert!(res.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
