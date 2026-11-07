use std::path::Path;

pub const SUPPORTED_EXTS: &[&str] = &[
    "7z",
    "aac",
    "avi",
    "bash",
    "bat",
    "bib",
    "bmp",
    "c",
    "cfg",
    "cmake",
    "conf",
    "cpp",
    "css",
    "csv",
    "db",
    "db3",
    "diff",
    "doc",
    "dockerfile",
    "docx",
    "epub",
    "fish",
    "flac",
    "flv",
    "gif",
    "go",
    "gradle",
    "graphql",
    "gz",
    "h",
    "hpp",
    "html",
    "ico",
    "ini",
    "ipynb",
    "java",
    "jpeg",
    "jpg",
    "js",
    "json",
    "jsonc",
    "jsonl",
    "jsx",
    "kra",
    "kt",
    "log",
    "lua",
    "m4a",
    "m4v",
    "makefile",
    "markdown",
    "md",
    "mdown",
    "mdwn",
    "mkd",
    "mkdn",
    "mkv",
    "mov",
    "mp3",
    "mp4",
    "ndjson",
    "obj",
    "odp",
    "ods",
    "odt",
    "ogg",
    "opus",
    "ora",
    "otf",
    "patch",
    "pdf",
    "php",
    "pl",
    "pm",
    "png",
    "ppt",
    "pptx",
    "proto",
    "ps1",
    "psd",
    "py",
    "r",
    "rb",
    "rs",
    "s3db",
    "scss",
    "sh",
    "sl3",
    "sql",
    "sqlite",
    "sqlite3",
    "stl",
    "svg",
    "swift",
    "tab",
    "tar",
    "tex",
    "tgz",
    "toml",
    "ts",
    "tsv",
    "tsx",
    "ttf",
    "txt",
    "txz",
    "typ",
    "vim",
    "wav",
    "webm",
    "webp",
    "wmv",
    "woff",
    "woff2",
    "xls",
    "xlsx",
    "xml",
    "xz",
    "yaml",
    "yml",
    "zip",
    "zsh",
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
        assert!(is_supported_path(Path::new("model.stl")));
        assert!(is_supported_path(Path::new("model.OBJ")));
        assert!(!is_supported_path(Path::new("test.exe")));
        assert!(!is_supported_path(Path::new("no_ext")));
    }

    #[test]
    fn test_supported_exts_matches_registry() {
        let registry = crate::engine::build_registry();
        for parser in &registry.parsers {
            for ext in parser.supported_extensions() {
                let lower_ext = ext.to_ascii_lowercase();
                assert!(
                    SUPPORTED_EXTS.contains(&lower_ext.as_str()),
                    "Extension '{ext}' from parser is missing in SUPPORTED_EXTS"
                );
            }
        }
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
