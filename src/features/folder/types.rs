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

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct FolderRowState {
    pub name: String,
    pub display_name: String,
    pub kind: String,
    pub size: String,
    pub raw_size: u64,
    pub modified: String,
    pub raw_modified: i64,
    pub path: String,
    pub is_dir: bool,
    pub icon: &'static str,
    pub depth: usize,
    pub parent_path: Option<String>,
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
    pub expanded_paths: HashSet<String>,
}

pub fn sort_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
    if !sort.active {
        return;
    }

    let has_hierarchy = rows.iter().any(|r| r.depth > 0 || r.parent_path.is_some());
    if has_hierarchy {
        sort_hierarchical_folder_rows(rows, sort);
    } else {
        sort_flat_folder_rows(rows, sort);
    }
}

fn sort_flat_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
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

fn sort_hierarchical_folder_rows(rows: &mut [FolderRowState], sort: &SortState) {
    use std::collections::HashMap;

    // Build children mapping keyed by parent_path: Option<String>
    let mut children_map: HashMap<Option<String>, Vec<FolderRowState>> = HashMap::new();
    let old_rows = rows.to_vec();

    for row in old_rows {
        children_map
            .entry(row.parent_path.clone())
            .or_default()
            .push(row);
    }

    // Helper to sort a sibling group and recursively collect in DFS pre-order
    fn collect_sorted(
        parent_key: Option<String>,
        children_map: &mut HashMap<Option<String>, Vec<FolderRowState>>,
        sort: &SortState,
        out: &mut Vec<FolderRowState>,
    ) {
        if let Some(mut siblings) = children_map.remove(&parent_key) {
            sort_flat_folder_rows(&mut siblings, sort);
            for item in siblings {
                let item_path = item.path.clone();
                let is_dir = item.is_dir;
                out.push(item);
                if is_dir {
                    collect_sorted(Some(item_path), children_map, sort, out);
                }
            }
        }
    }

    let mut sorted_rows = Vec::with_capacity(rows.len());

    // Root items can have parent_path == None or parent_path not present in children_map as a child of someone else.
    // Determine the root parent keys: keys that are not equal to any row's path.
    let all_paths: HashSet<String> = children_map
        .values()
        .flat_map(|v| v.iter().map(|r| r.path.clone()))
        .collect();

    let mut root_keys: Vec<Option<String>> = children_map
        .keys()
        .filter(|k| match k {
            None => true,
            Some(p) => !all_paths.contains(p),
        })
        .cloned()
        .collect();

    root_keys.sort_by(|a, b| match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(sa), Some(sb)) => sa.cmp(sb),
    });

    for root_key in root_keys {
        collect_sorted(root_key, &mut children_map, sort, &mut sorted_rows);
    }

    // If any orphaned entries remain, sort and push them
    let mut remaining_keys: Vec<Option<String>> = children_map.keys().cloned().collect();
    remaining_keys.sort_by(|a, b| match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(sa), Some(sb)) => sa.cmp(sb),
    });
    for rem_key in remaining_keys {
        collect_sorted(rem_key, &mut children_map, sort, &mut sorted_rows);
    }

    rows.clone_from_slice(&sorted_rows);
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
                display_name: "beta".to_string(),
                kind: "File".to_string(),
                size: "10 B".to_string(),
                raw_size: 10,
                modified: "today".to_string(),
                raw_modified: 100,
                path: "/path/beta".to_string(),
                is_dir: false,
                icon: "file",
                depth: 0,
                parent_path: None,
            },
            FolderRowState {
                name: "Alpha".to_string(),
                display_name: "Alpha".to_string(),
                kind: "File".to_string(),
                size: "20 B".to_string(),
                raw_size: 20,
                modified: "today".to_string(),
                raw_modified: 200,
                path: "/path/Alpha".to_string(),
                is_dir: false,
                icon: "file",
                depth: 0,
                parent_path: None,
            },
            FolderRowState {
                name: "gamma_dir".to_string(),
                display_name: "gamma_dir".to_string(),
                kind: "Folder".to_string(),
                size: "—".to_string(),
                raw_size: 0,
                modified: "today".to_string(),
                raw_modified: 50,
                path: "/path/gamma_dir".to_string(),
                is_dir: true,
                icon: "folder",
                depth: 0,
                parent_path: None,
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
    fn test_sort_hierarchical_preserves_tree_structure() {
        let mut rows = vec![
            FolderRowState {
                name: "src".to_string(),
                display_name: "src".to_string(),
                kind: "Folder".to_string(),
                size: "—".to_string(),
                raw_size: 0,
                modified: "today".to_string(),
                raw_modified: 10,
                path: "src".to_string(),
                is_dir: true,
                icon: "folder",
                depth: 0,
                parent_path: None,
            },
            FolderRowState {
                name: "main.rs".to_string(),
                display_name: "main.rs".to_string(),
                kind: "Rust Source".to_string(),
                size: "100 B".to_string(),
                raw_size: 100,
                modified: "today".to_string(),
                raw_modified: 20,
                path: "src/main.rs".to_string(),
                is_dir: false,
                icon: "file",
                depth: 1,
                parent_path: Some("src".to_string()),
            },
            FolderRowState {
                name: "app".to_string(),
                display_name: "app".to_string(),
                kind: "Folder".to_string(),
                size: "—".to_string(),
                raw_size: 0,
                modified: "today".to_string(),
                raw_modified: 15,
                path: "src/app".to_string(),
                is_dir: true,
                icon: "folder",
                depth: 1,
                parent_path: Some("src".to_string()),
            },
            FolderRowState {
                name: "view.rs".to_string(),
                display_name: "view.rs".to_string(),
                kind: "Rust Source".to_string(),
                size: "50 B".to_string(),
                raw_size: 50,
                modified: "today".to_string(),
                raw_modified: 25,
                path: "src/app/view.rs".to_string(),
                is_dir: false,
                icon: "file",
                depth: 2,
                parent_path: Some("src/app".to_string()),
            },
            FolderRowState {
                name: "Cargo.toml".to_string(),
                display_name: "Cargo.toml".to_string(),
                kind: "TOML Config".to_string(),
                size: "30 B".to_string(),
                raw_size: 30,
                modified: "today".to_string(),
                raw_modified: 5,
                path: "Cargo.toml".to_string(),
                is_dir: false,
                icon: "file",
                depth: 0,
                parent_path: None,
            },
        ];

        let sort = SortState {
            field: SortField::Name,
            ascending: true,
            active: true,
        };

        sort_folder_rows(&mut rows, &sort);

        // Hierarchical structure:
        // Root: "src" (dir), "Cargo.toml" (file)
        // Under "src": "app" (dir), "main.rs" (file)
        // Under "src/app": "view.rs" (file)
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["src", "app", "view.rs", "main.rs", "Cargo.toml"]
        );
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
