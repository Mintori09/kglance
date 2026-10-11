use iced::Task;
use std::path::{Path, PathBuf};

use crate::app::KglanceApp;
use crate::app::messages::{FileTreeMsg, Message};

pub fn handle_file_tree_msg(app: &mut KglanceApp, msg: FileTreeMsg) -> Task<Message> {
    match msg {
        FileTreeMsg::Toggle => handle_toggle_file_tree(app),
        FileTreeMsg::ToggleFolder(path) => {
            app.state.file_tree.toggle_folder(&path);
            Task::none()
        }
        FileTreeMsg::CollapseAll => {
            app.state.file_tree.collapse_all();
            Task::none()
        }
        FileTreeMsg::SelectFile(path) => handle_select_file(app, path),
    }
}

fn handle_toggle_file_tree(app: &mut KglanceApp) -> Task<Message> {
    let prev_scroll_y = app.active_scroll_y();

    if app.state.file_tree.visible {
        app.state.file_tree.visible = false;
    } else {
        app.state.file_tree.visible = true;
        // Close document outlines if open to avoid clutter
        app.state.markdown.toc_visible = false;
        app.state.text.outline_visible = false;
        app.state.pdf.sidebar_visible = false;
        app.state.epub.sidebar_visible = false;

        if !app.state.file_name.is_empty() {
            let p = Path::new(&app.state.file_name);
            if app.state.file_tree.root_dir.is_none() {
                app.state.file_tree.update_external_path(p);
            } else {
                app.state.file_tree.select_and_reveal_path(p);
            }
        }
    }

    if let Some(y) = prev_scroll_y {
        iced::widget::operation::scroll_to(
            "content_scroll",
            iced::widget::operation::AbsoluteOffset { x: 0.0, y },
        )
    } else {
        Task::none()
    }
}

fn handle_select_file(app: &mut KglanceApp, path: PathBuf) -> Task<Message> {
    let path_str = path.to_string_lossy().into_owned();
    if path_str == app.state.file_name {
        return Task::none();
    }

    app.state.file_tree.selected_path = Some(path);
    if let Some(pos) = app.state.playlist.iter().position(|p| p == &path_str) {
        app.state.current_index = pos;
    }

    if let Some(cached) = app.state.cache.get(&path_str)
        && let Some(cached_data) = cached.as_preview().cloned()
    {
        let gen_id = app
            .state
            .generation_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        return app.update(
            crate::app::messages::SystemMsg::FileLoaded {
                path: path_str,
                content: (*cached_data).clone(),
                generation_id: gen_id,
            }
            .into(),
        );
    }

    app.current_content = None;
    app.state.reset_content_state_for_loading(path_str.clone());

    crate::app::update::navigation::load_file_task(app, path_str, |p| {
        crate::app::messages::SystemMsg::FilePreviewError(p).into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::test_app;

    #[test]
    fn test_handle_toggle_file_tree() {
        let mut app = test_app(None);
        app.state.file_name = "src/main.rs".to_string();
        app.state.markdown.toc_visible = true;

        assert!(!app.state.file_tree.visible);

        // Toggle on
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::Toggle);
        assert!(app.state.file_tree.visible);
        // Should close TOC
        assert!(!app.state.markdown.toc_visible);
        let initial_root = app.state.file_tree.root_dir.clone();
        assert!(initial_root.is_some());

        // Toggle off and on again preserves the same root!
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::Toggle);
        assert!(!app.state.file_tree.visible);
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::Toggle);
        assert!(app.state.file_tree.visible);
        assert_eq!(app.state.file_tree.root_dir, initial_root);
    }

    #[test]
    fn test_toggle_outline_closes_file_tree_and_vice_versa() {
        let mut app = test_app(None);
        app.state.file_name = "src/main.rs".to_string();

        // 1. Open file tree
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::Toggle);
        assert!(app.state.file_tree.visible);
        assert!(!app.state.text.outline_visible);

        // 2. Click outline button in footer -> closes file tree and opens outline
        let _ = crate::features::text::update::handle_toggle_outline(&mut app);
        assert!(app.state.text.outline_visible);
        assert!(!app.state.file_tree.visible);

        // 3. Open file tree again -> closes outline and opens file tree
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::Toggle);
        assert!(app.state.file_tree.visible);
        assert!(!app.state.text.outline_visible);
    }

    #[test]
    fn test_handle_toggle_folder_and_collapse_all() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let dir_path = temp_dir.path();
        let sub_dir = dir_path.join("sub_folder");
        let _ = std::fs::create_dir(&sub_dir);
        let _ = std::fs::write(sub_dir.join("inner.txt"), "hello");

        let mut app = test_app(None);
        app.state.file_tree.set_root(dir_path.to_path_buf());

        assert_eq!(app.state.file_tree.nodes.len(), 1);
        assert!(!app.state.file_tree.nodes[0].is_expanded);
        assert!(app.state.file_tree.nodes[0].children.is_none());

        // Toggle folder open -> lazy loads inner.txt
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::ToggleFolder(sub_dir.clone()));
        assert!(app.state.file_tree.nodes[0].is_expanded);
        assert_eq!(
            app.state.file_tree.nodes[0]
                .children
                .as_ref()
                .unwrap()
                .len(),
            1
        );

        // Collapse all
        let _ = handle_file_tree_msg(&mut app, FileTreeMsg::CollapseAll);
        assert!(!app.state.file_tree.nodes[0].is_expanded);
    }

    #[test]
    fn test_handle_select_same_file_is_noop() {
        let mut app = test_app(None);
        app.state.file_name = "src/main.rs".to_string();

        let _ = handle_file_tree_msg(
            &mut app,
            FileTreeMsg::SelectFile(PathBuf::from("src/main.rs")),
        );
        // file_name unchanged, still ready
        assert_eq!(app.state.file_name, "src/main.rs");
    }
}
