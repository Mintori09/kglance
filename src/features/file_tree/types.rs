use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
    pub children: Option<Vec<FileTreeNode>>,
}

#[derive(Debug, Clone)]
pub struct FileTreeState {
    pub visible: bool,
    pub root_dir: Option<PathBuf>,
    pub root_name: String,
    pub nodes: Vec<FileTreeNode>,
    pub selected_path: Option<PathBuf>,
    pub sidebar_width: f32,
    pub sidebar_resizing: bool,
    pub sidebar_drag_start_x: Option<f32>,
    pub sidebar_drag_start_width: f32,
}

impl Default for FileTreeState {
    fn default() -> Self {
        Self {
            visible: false,
            root_dir: None,
            root_name: String::new(),
            nodes: Vec::new(),
            selected_path: None,
            sidebar_width: 240.0,
            sidebar_resizing: false,
            sidebar_drag_start_x: None,
            sidebar_drag_start_width: 240.0,
        }
    }
}

impl FileTreeState {
    pub fn set_root(&mut self, root: PathBuf) {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.to_string_lossy().into_owned());
        self.root_name = if name.is_empty() {
            "root".to_string()
        } else {
            name
        };
        self.nodes = super::scanner::scan_directory(&root, 0);
        self.root_dir = Some(root);
    }

    pub fn update_external_path(&mut self, target_path: &Path) {
        let new_root = if target_path.is_dir() {
            target_path.to_path_buf()
        } else {
            target_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };

        let is_within_current_root = self
            .root_dir
            .as_ref()
            .is_some_and(|root| target_path.starts_with(root));

        if !is_within_current_root {
            self.set_root(new_root);
        }

        self.select_and_reveal_path(target_path);
    }

    pub fn select_and_reveal_path(&mut self, target_path: &Path) {
        self.selected_path = Some(target_path.to_path_buf());
        expand_ancestors_of_path(&mut self.nodes, target_path);
    }

    pub fn toggle_folder(&mut self, path: &Path) {
        toggle_folder_in_nodes(&mut self.nodes, path);
    }

    pub fn collapse_all(&mut self) {
        collapse_all_nodes(&mut self.nodes);
    }
}

fn expand_ancestors_of_path(nodes: &mut [FileTreeNode], target_path: &Path) {
    for node in nodes.iter_mut() {
        if node.is_dir && target_path.starts_with(&node.path) {
            node.is_expanded = true;
            if node.children.is_none() {
                // TODO: scan_directory runs on the UI thread. For large directories or
                // slow file systems (NFS/FUSE) this may block the event loop. Consider
                // delivering a FileTreeMsg::FolderLoaded(path, children) via Task instead.
                node.children = Some(super::scanner::scan_directory(&node.path, node.depth + 1));
            }
            if let Some(ref mut children) = node.children {
                expand_ancestors_of_path(children, target_path);
            }
        }
    }
}

fn toggle_folder_in_nodes(nodes: &mut [FileTreeNode], target: &Path) -> bool {
    for node in nodes.iter_mut() {
        if node.is_dir && node.path == target {
            node.is_expanded = !node.is_expanded;
            if node.is_expanded && node.children.is_none() {
                // TODO: same UI-thread blocking concern as expand_ancestors_of_path.
                node.children = Some(super::scanner::scan_directory(&node.path, node.depth + 1));
            }
            return true;
        }
        if let Some(ref mut children) = node.children
            && toggle_folder_in_nodes(children, target)
        {
            return true;
        }
    }
    false
}

fn collapse_all_nodes(nodes: &mut [FileTreeNode]) {
    for node in nodes.iter_mut() {
        if node.is_dir {
            node.is_expanded = false;
            if let Some(ref mut children) = node.children {
                collapse_all_nodes(children);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_tree_state_default() {
        let state = FileTreeState::default();
        assert!(!state.visible);
        assert!(state.nodes.is_empty());
        assert_eq!(state.sidebar_width, 240.0);
    }

    #[test]
    fn test_collapse_all_nodes() {
        let mut nodes = vec![FileTreeNode {
            path: PathBuf::from("/test"),
            name: "test".to_string(),
            is_dir: true,
            is_expanded: true,
            depth: 0,
            children: Some(vec![FileTreeNode {
                path: PathBuf::from("/test/sub"),
                name: "sub".to_string(),
                is_dir: true,
                is_expanded: true,
                depth: 1,
                children: None,
            }]),
        }];

        collapse_all_nodes(&mut nodes);
        assert!(!nodes[0].is_expanded);
        let children = nodes[0]
            .children
            .as_ref()
            .expect("children should be Some after collapse");
        assert!(!children[0].is_expanded);
    }

    #[test]
    fn test_update_external_path_persists_when_within_root() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let dir_path = temp_dir.path();
        let sub_dir = dir_path.join("src");
        let _ = std::fs::create_dir(&sub_dir);
        let file_a = sub_dir.join("main.rs");
        let _ = std::fs::write(&file_a, "fn main() {}");

        let mut state = FileTreeState::default();
        state.update_external_path(dir_path);
        assert_eq!(state.root_dir.as_deref(), Some(dir_path));

        // Subfile within root retains the original root!
        state.update_external_path(&file_a);
        assert_eq!(state.root_dir.as_deref(), Some(dir_path));
        assert_eq!(state.selected_path.as_deref(), Some(file_a.as_path()));
        // src folder was automatically expanded
        assert!(
            state
                .nodes
                .iter()
                .any(|n| n.path == sub_dir && n.is_expanded)
        );

        // Completely different directory outside root updates root!
        let other_dir = tempfile::tempdir().expect("create other temp dir");
        let other_file = other_dir.path().join("other.txt");
        let _ = std::fs::write(&other_file, "hello");
        state.update_external_path(&other_file);
        assert_eq!(state.root_dir.as_deref(), Some(other_dir.path()));
        assert_eq!(state.selected_path.as_deref(), Some(other_file.as_path()));
    }
}
