#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
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
}

pub fn sort_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
    if !sort.active {
        return;
    }
    match sort.field {
        SortField::Name => {
            rows.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortField::Kind => {
            rows.sort_by(|a, b| a.kind.cmp(&b.kind));
        }
        SortField::Size => {
            rows.sort_by_key(|a| a.raw_size);
        }
        SortField::Modified => {
            rows.sort_by_key(|a| a.raw_modified);
        }
    }
    if !sort.ascending {
        rows.reverse();
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
        ".env" | ".envrc" | ".env.local" => return "Environment Config",
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
