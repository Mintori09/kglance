#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub raw_modified: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Name,
    Kind,
    Modified,
    Size,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SortState {
    pub field: SortField,
    pub ascending: bool,
    pub active: bool,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            field: SortField::Name,
            ascending: true,
            active: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FolderRowState {
    pub name: String,
    pub kind: String,
    pub size: String,
    pub raw_size: u64,
    pub modified: String,
    pub raw_modified: i64,
    pub path: String,
    pub is_dir: bool,
    pub icon: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct FolderState {
    pub rows: Vec<FolderRowState>,
    pub sort_state: SortState,
    pub selected_index: Option<usize>,
    pub total_size: u64,
    pub folder_path: String,
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub search_visible: bool,
    pub search_query: String,
}

pub fn sort_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
    if !sort.active {
        return;
    }
    rows.sort_by(|a, b| {
        // Folders always come first regardless of field or ascending order
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let ord = match sort.field {
                    SortField::Name => compare_case_insensitive(&a.name, &b.name),
                    SortField::Kind => a.kind.cmp(&b.kind),
                    SortField::Size => a.raw_size.cmp(&b.raw_size),
                    SortField::Modified => a.raw_modified.cmp(&b.raw_modified),
                };
                if sort.ascending { ord } else { ord.reverse() }
            }
        }
    });
}

fn compare_case_insensitive(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().flat_map(char::to_lowercase);
    let mut b_chars = b.chars().flat_map(char::to_lowercase);

    loop {
        match (a_chars.next(), b_chars.next()) {
            (Some(c1), Some(c2)) => match c1.cmp(&c2) {
                std::cmp::Ordering::Equal => continue,
                ord => return ord,
            },
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, None) => return a.cmp(b),
        }
    }
}

pub fn human_file_kind(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return "Folder";
    }

    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "cargo.toml" => return "TOML Config",
        "cargo.lock" => return "Cargo Lockfile",
        "cmakelists.txt" => return "CMake Config",
        "dockerfile" | "containerfile" => return "Docker Config",
        "makefile" => return "Makefile Script",
        "license" | "license-mit" | "license-apache" | "copying" => return "License File",
        "readme" | "readme.md" => return "Markdown Document",
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "Git Config",
        ".editorconfig" => return "EditorConfig",
        _ if lower.starts_with(".env") => return "Environment Config",
        "justfile" => return "Justfile Script",
        _ => {}
    }

    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "rs" => "Rust Source",
        "toml" => "TOML Config",
        "json" | "json5" | "jsonc" => "JSON Document",
        "yaml" | "yml" => "YAML Document",
        "xml" => "XML Document",
        "md" | "markdown" | "mdown" => "Markdown Document",
        "html" | "htm" => "HTML Document",
        "css" | "scss" | "sass" | "less" => "CSS Stylesheet",
        "js" | "mjs" | "cjs" => "JavaScript Source",
        "ts" | "mts" | "cts" => "TypeScript Source",
        "jsx" => "React JSX",
        "tsx" => "React TSX",
        "py" | "pyi" | "pyw" => "Python Source",
        "c" | "h" => "C Source",
        "cpp" | "cxx" | "cc" | "hpp" | "hh" | "hxx" => "C++ Source",
        "go" => "Go Source",
        "java" => "Java Source",
        "kt" | "kts" => "Kotlin Source",
        "swift" => "Swift Source",
        "php" => "PHP Script",
        "sh" | "bash" | "zsh" | "fish" => "Shell Script",
        "sql" => "SQL Query",
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "ico" | "avif" | "tiff" => "Image",
        "svg" => "SVG Vector",
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" => "Video",
        "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "opus" => "Audio",
        "pdf" => "PDF Document",
        "epub" => "EPUB E-Book",
        "typ" => "Typst Document",
        "kra" => "Krita Image",
        "psd" => "Photoshop Document",
        "xlsx" | "xls" | "ods" | "csv" | "tsv" => "Spreadsheet",
        "docx" | "doc" | "odt" | "rtf" => "Office Document",
        "pptx" | "ppt" | "odp" => "Presentation",
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "tbz2" | "xz" | "txz" | "7z" | "rar" | "zst"
        | "lz4" => "Archive",
        "wasm" => "WebAssembly Binary",
        "exe" | "dll" | "so" | "dylib" => "Binary Executable",
        "ttf" | "otf" | "woff" | "woff2" => "Font",
        "lock" => "Lockfile",
        "log" => "Log File",
        "txt" => "Text Document",
        _ => "File",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_folder_rows_case_insensitive_no_alloc() {
        let mut rows = vec![
            FolderRowState {
                name: "beta".to_string(),
                kind: "File".to_string(),
                size: "10 B".to_string(),
                raw_size: 10,
                modified: "today".to_string(),
                raw_modified: 100,
                path: "/path/beta".to_string(),
                is_dir: false,
                icon: "file",
            },
            FolderRowState {
                name: "Alpha".to_string(),
                kind: "File".to_string(),
                size: "20 B".to_string(),
                raw_size: 20,
                modified: "today".to_string(),
                raw_modified: 200,
                path: "/path/Alpha".to_string(),
                is_dir: false,
                icon: "file",
            },
            FolderRowState {
                name: "gamma_dir".to_string(),
                kind: "Folder".to_string(),
                size: "—".to_string(),
                raw_size: 0,
                modified: "today".to_string(),
                raw_modified: 50,
                path: "/path/gamma_dir".to_string(),
                is_dir: true,
                icon: "folder",
            },
        ];

        let sort = SortState {
            field: SortField::Name,
            ascending: true,
            active: true,
        };

        sort_folder_rows(&mut rows, &sort);

        // Folders come first
        assert_eq!(rows[0].name, "gamma_dir");
        // Alpha < beta case-insensitively
        assert_eq!(rows[1].name, "Alpha");
        assert_eq!(rows[2].name, "beta");
    }

    #[test]
    fn test_human_file_kind() {
        assert_eq!(human_file_kind(".cargo", true), "Folder");
        assert_eq!(human_file_kind("src", true), "Folder");
        assert_eq!(human_file_kind("main.rs", false), "Rust Source");
        assert_eq!(human_file_kind("Cargo.toml", false), "TOML Config");
        assert_eq!(human_file_kind("README.md", false), "Markdown Document");
        assert_eq!(human_file_kind(".envrc", false), "Environment Config");
        assert_eq!(
            human_file_kind("heaptrack.kglance.238476.zst", false),
            "Archive"
        );
        assert_eq!(human_file_kind("unknown.xyz123", false), "File");
    }
}
