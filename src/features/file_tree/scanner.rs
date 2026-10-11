use std::path::Path;

use super::types::FileTreeNode;

pub fn is_ignored_entry(name: &str) -> bool {
    name.starts_with('.')
        || name.eq_ignore_ascii_case("target")
        || name.eq_ignore_ascii_case("node_modules")
        || name.eq_ignore_ascii_case("__pycache__")
        || name.eq_ignore_ascii_case("venv")
}

pub fn scan_directory(dir: &Path, depth: usize) -> Vec<FileTreeNode> {
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return entries;
    };

    for entry in read_dir.flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if is_ignored_entry(&file_name) {
            continue;
        }

        let path = entry.path();
        let is_dir = path.is_dir();
        entries.push(FileTreeNode {
            path,
            name: file_name,
            is_dir,
            is_expanded: false,
            depth,
            children: None,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored_entry() {
        assert!(is_ignored_entry(".git"));
        assert!(is_ignored_entry(".hidden"));
        assert!(is_ignored_entry("target"));
        assert!(is_ignored_entry("node_modules"));
        assert!(is_ignored_entry("TARGET"));
        assert!(!is_ignored_entry("src"));
        assert!(!is_ignored_entry("main.rs"));
        assert!(!is_ignored_entry("Cargo.toml"));
    }

    #[test]
    fn test_scan_directory_sorting() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let dir_path = temp_dir.path();

        let _ = std::fs::write(dir_path.join("b_file.txt"), "b");
        let _ = std::fs::write(dir_path.join("a_file.txt"), "a");
        let _ = std::fs::create_dir(dir_path.join("z_folder"));
        let _ = std::fs::create_dir(dir_path.join(".ignored_folder"));

        let nodes = scan_directory(dir_path, 0);

        // Folders first, then files
        assert_eq!(nodes.len(), 3);
        assert!(nodes[0].is_dir);
        assert_eq!(nodes[0].name, "z_folder");
        assert!(!nodes[1].is_dir);
        assert_eq!(nodes[1].name, "a_file.txt");
        assert!(!nodes[2].is_dir);
        assert_eq!(nodes[2].name, "b_file.txt");
    }
}
