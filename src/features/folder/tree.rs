use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::core::types::human_file_kind;
use crate::core::utils::human_size;
use crate::features::archive::types::ArchiveEntry;
use crate::features::folder::types::{DirEntry, FolderRowState};
use crate::ui::theme::icon_theme::icon_for_entry;

pub const CHEVRON_RIGHT_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>"#;
pub const CHEVRON_DOWN_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>"#;

struct ArchiveTreeNode {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: String,
    children: BTreeMap<String, ArchiveTreeNode>,
}

pub fn build_archive_tree_rows(entries: Vec<ArchiveEntry>) -> Vec<FolderRowState> {
    let mut root_children: BTreeMap<String, ArchiveTreeNode> = BTreeMap::new();

    for entry in entries {
        let clean_path = entry.path.trim_matches('/').replace('\\', "/");
        if clean_path.is_empty() {
            continue;
        }

        let parts: Vec<&str> = clean_path.split('/').filter(|p| !p.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }

        let mut current_map = &mut root_children;
        let mut accumulated_path = String::new();

        for (i, part) in parts.iter().enumerate() {
            if !accumulated_path.is_empty() {
                accumulated_path.push('/');
            }
            accumulated_path.push_str(part);

            let is_last = i == parts.len() - 1;
            let node_is_dir = if is_last { entry.is_dir } else { true };
            let (node_size, node_modified) = if is_last {
                (entry.size, entry.modified.clone())
            } else {
                (0, String::new())
            };

            let node = current_map
                .entry((*part).to_string())
                .or_insert_with(|| ArchiveTreeNode {
                    name: (*part).to_string(),
                    path: accumulated_path.clone(),
                    is_dir: node_is_dir,
                    size: node_size,
                    modified: node_modified.clone(),
                    children: BTreeMap::new(),
                });

            if is_last {
                node.is_dir = entry.is_dir;
                node.size = entry.size;
                if !entry.modified.is_empty() {
                    node.modified = entry.modified.clone();
                }
            }

            current_map = &mut node.children;
        }
    }

    let mut rows = Vec::new();
    flatten_archive_nodes(&root_children, 0, None, &mut rows);
    rows
}

fn flatten_archive_nodes(
    nodes: &BTreeMap<String, ArchiveTreeNode>,
    depth: usize,
    parent_path: Option<&str>,
    rows: &mut Vec<FolderRowState>,
) {
    let mut sorted_nodes: Vec<&ArchiveTreeNode> = nodes.values().collect();
    sorted_nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    for node in sorted_nodes {
        let icon = icon_for_entry(&node.name, node.is_dir);
        let kind = human_file_kind(&node.name, node.is_dir).to_string();
        let size = if node.is_dir {
            "—".to_string()
        } else {
            human_size(node.size)
        };
        let raw_size = if node.is_dir { 0 } else { node.size };

        rows.push(FolderRowState {
            name: node.name.clone(),
            display_name: node.name.clone(),
            kind,
            size,
            raw_size,
            modified: node.modified.clone(),
            raw_modified: 0,
            path: node.path.clone(),
            is_dir: node.is_dir,
            icon,
            depth,
            parent_path: parent_path.map(str::to_string),
        });

        if node.is_dir && !node.children.is_empty() {
            flatten_archive_nodes(&node.children, depth + 1, Some(&node.path), rows);
        }
    }
}

pub fn build_folder_tree_rows(entries: Vec<DirEntry>) -> Vec<FolderRowState> {
    entries
        .into_iter()
        .map(|entry| {
            let icon = icon_for_entry(&entry.name, entry.is_dir);
            let kind = human_file_kind(&entry.name, entry.is_dir).to_string();
            let size = if entry.is_dir {
                "—".to_string()
            } else {
                human_size(entry.size)
            };
            let raw_size = if entry.is_dir { 0 } else { entry.size };
            let parent_path = Path::new(&entry.path)
                .parent()
                .map(|p| p.to_string_lossy().into_owned());

            FolderRowState {
                name: entry.name.clone(),
                display_name: entry.name,
                kind,
                size,
                raw_size,
                modified: entry.modified,
                raw_modified: entry.raw_modified,
                path: entry.path,
                is_dir: entry.is_dir,
                icon,
                depth: 0,
                parent_path,
            }
        })
        .collect()
}

pub fn filter_visible_folder_rows<'a>(
    rows: &'a [FolderRowState],
    expanded_paths: &HashSet<String>,
) -> Vec<&'a FolderRowState> {
    let mut visible = Vec::new();
    let mut visible_parent_paths = HashSet::new();

    for row in rows {
        let is_visible = if row.depth == 0 {
            true
        } else if let Some(ref parent) = row.parent_path {
            visible_parent_paths.contains(parent) && expanded_paths.contains(parent)
        } else {
            false
        };

        if is_visible {
            visible.push(row);
            if row.is_dir && expanded_paths.contains(&row.path) {
                visible_parent_paths.insert(row.path.clone());
            }
        }
    }

    visible
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_archive_tree_hierarchical() {
        let entries = vec![
            ArchiveEntry {
                path: "src/main.rs".to_string(),
                size: 100,
                is_dir: false,
                modified: "2026-01-01".to_string(),
                archive_path: "test.zip".to_string(),
            },
            ArchiveEntry {
                path: "src/app/view.rs".to_string(),
                size: 200,
                is_dir: false,
                modified: "2026-01-01".to_string(),
                archive_path: "test.zip".to_string(),
            },
            ArchiveEntry {
                path: "README.md".to_string(),
                size: 50,
                is_dir: false,
                modified: "2026-01-01".to_string(),
                archive_path: "test.zip".to_string(),
            },
        ];

        let rows = build_archive_tree_rows(entries);

        // Folders come first: "src" (depth 0), inside src: "app" (depth 1), view.rs (depth 2), main.rs (depth 1)
        let src_row = rows.iter().find(|r| r.name == "src").unwrap();
        assert_eq!(src_row.depth, 0);
        assert!(src_row.is_dir);
        assert_eq!(src_row.display_name, "src");

        let app_row = rows.iter().find(|r| r.name == "app").unwrap();
        assert_eq!(app_row.depth, 1);
        assert!(app_row.is_dir);
        assert_eq!(app_row.parent_path.as_deref(), Some("src"));

        let view_row = rows.iter().find(|r| r.name == "view.rs").unwrap();
        assert_eq!(view_row.depth, 2);
        assert!(!view_row.is_dir);
        assert_eq!(view_row.parent_path.as_deref(), Some("src/app"));

        let readme_row = rows.iter().find(|r| r.name == "README.md").unwrap();
        assert_eq!(readme_row.depth, 0);
        assert_eq!(readme_row.parent_path, None);
    }

    #[test]
    fn test_filter_visible_folder_rows() {
        let entries = vec![
            ArchiveEntry {
                path: "src/main.rs".to_string(),
                size: 100,
                is_dir: false,
                modified: "2026-01-01".to_string(),
                archive_path: "test.zip".to_string(),
            },
            ArchiveEntry {
                path: "src/app/view.rs".to_string(),
                size: 200,
                is_dir: false,
                modified: "2026-01-01".to_string(),
                archive_path: "test.zip".to_string(),
            },
            ArchiveEntry {
                path: "README.md".to_string(),
                size: 50,
                is_dir: false,
                modified: "2026-01-01".to_string(),
                archive_path: "test.zip".to_string(),
            },
        ];

        let rows = build_archive_tree_rows(entries);
        let mut expanded = HashSet::new();

        // Initially nothing expanded -> only depth 0 visible
        let visible = filter_visible_folder_rows(&rows, &expanded);
        let visible_names: Vec<&str> = visible.iter().map(|r| r.display_name.as_str()).collect();
        assert_eq!(visible_names, vec!["src", "README.md"]);

        // Expand "src" -> depth 0 and depth 1 visible ("app" and "main.rs")
        expanded.insert("src".to_string());
        let visible = filter_visible_folder_rows(&rows, &expanded);
        let visible_names: Vec<&str> = visible.iter().map(|r| r.display_name.as_str()).collect();
        assert_eq!(visible_names, vec!["src", "app", "main.rs", "README.md"]);

        // Expand "src/app" -> depth 2 "view.rs" also visible
        expanded.insert("src/app".to_string());
        let visible = filter_visible_folder_rows(&rows, &expanded);
        let visible_names: Vec<&str> = visible.iter().map(|r| r.display_name.as_str()).collect();
        assert_eq!(
            visible_names,
            vec!["src", "app", "view.rs", "main.rs", "README.md"]
        );
    }
}
