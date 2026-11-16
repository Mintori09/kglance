use std::path::Path;

use iced::Task;
use iced::keyboard::key::Named;

use super::Message;
use crate::app::KglanceApp;
use crate::core::{FilePreviewer, PreviewData};

const FOLDER_PAGE_STEP: isize = 10;

impl KglanceApp {
    pub(super) fn handle_folder_navigation(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        if self.ctrl_held || !self.is_folder_navigation_available() {
            return None;
        }

        match key {
            iced::keyboard::Key::Named(Named::ArrowDown) => {
                self.move_selection_down();
                Some(self.scroll_folder_to_selected())
            }
            iced::keyboard::Key::Named(Named::ArrowUp) => {
                self.move_selection_up();
                Some(self.scroll_folder_to_selected())
            }
            iced::keyboard::Key::Character(c) if c == "j" => {
                self.move_selection_down();
                Some(self.scroll_folder_to_selected())
            }
            iced::keyboard::Key::Character(c) if c == "k" => {
                self.move_selection_up();
                Some(self.scroll_folder_to_selected())
            }
            iced::keyboard::Key::Character(c) if c == "g" => {
                if self.pending_g {
                    self.pending_g = false;
                    let visible_indices =
                        crate::features::folder::visible_folder_row_indices(&self.state.folder);
                    self.state.folder.selected_index = visible_indices.first().copied();
                    Some(self.snap_to_top())
                } else {
                    self.pending_g = true;
                    None
                }
            }
            iced::keyboard::Key::Character(c) if c == "G" => {
                self.pending_g = false;
                let visible_indices =
                    crate::features::folder::visible_folder_row_indices(&self.state.folder);
                self.state.folder.selected_index = visible_indices.last().copied();
                Some(self.snap_to_bottom())
            }
            iced::keyboard::Key::Named(Named::ArrowLeft) => self.handle_folder_collapse_or_parent(),
            iced::keyboard::Key::Character(c) if c == "h" => {
                self.handle_folder_collapse_or_parent()
            }
            iced::keyboard::Key::Named(Named::ArrowRight) => self.handle_folder_expand_or_open(),
            iced::keyboard::Key::Character(c) if c == "l" => self.handle_folder_expand_or_open(),
            iced::keyboard::Key::Named(Named::Enter) => self.open_selected_row(),
            iced::keyboard::Key::Named(Named::Home) => {
                self.pending_home = false;
                let visible_indices =
                    crate::features::folder::visible_folder_row_indices(&self.state.folder);
                self.state.folder.selected_index = visible_indices.first().copied();
                Some(self.snap_to_top())
            }
            iced::keyboard::Key::Named(Named::End) => {
                self.pending_home = false;
                let visible_indices =
                    crate::features::folder::visible_folder_row_indices(&self.state.folder);
                self.state.folder.selected_index = visible_indices.last().copied();
                Some(self.snap_to_bottom())
            }
            iced::keyboard::Key::Named(Named::PageUp) => {
                self.move_selection_by(-FOLDER_PAGE_STEP);
                Some(self.scroll_folder_to_selected())
            }
            iced::keyboard::Key::Named(Named::PageDown) => {
                self.move_selection_by(FOLDER_PAGE_STEP);
                Some(self.scroll_folder_to_selected())
            }
            _ => None,
        }
    }

    fn is_folder_navigation_available(&self) -> bool {
        !self.state.folder.rows.is_empty()
            && matches!(self.current_content, Some(PreviewData::Folder { .. }))
    }

    fn move_selection_down(&mut self) {
        let visible_indices =
            crate::features::folder::visible_folder_row_indices(&self.state.folder);
        if visible_indices.is_empty() {
            return;
        }
        let current_pos = self
            .state
            .folder
            .selected_index
            .and_then(|idx| visible_indices.iter().position(|&i| i == idx));

        let next_pos = match current_pos {
            Some(pos) => (pos + 1).min(visible_indices.len().saturating_sub(1)),
            None => 0,
        };

        self.state.folder.selected_index = Some(visible_indices[next_pos]);
    }

    fn move_selection_up(&mut self) {
        let visible_indices =
            crate::features::folder::visible_folder_row_indices(&self.state.folder);
        if visible_indices.is_empty() {
            return;
        }
        let current_pos = self
            .state
            .folder
            .selected_index
            .and_then(|idx| visible_indices.iter().position(|&i| i == idx));

        let previous_pos = match current_pos {
            Some(pos) => pos.saturating_sub(1),
            None => 0,
        };

        self.state.folder.selected_index = Some(visible_indices[previous_pos]);
    }

    fn move_selection_by(&mut self, offset: isize) {
        let visible_indices =
            crate::features::folder::visible_folder_row_indices(&self.state.folder);
        if visible_indices.is_empty() {
            return;
        }
        let last_pos = visible_indices.len().saturating_sub(1);
        let current_pos = self
            .state
            .folder
            .selected_index
            .and_then(|idx| visible_indices.iter().position(|&i| i == idx))
            .unwrap_or(0);

        let new_pos = if offset < 0 {
            current_pos.saturating_sub(offset.unsigned_abs())
        } else {
            current_pos.saturating_add(offset as usize).min(last_pos)
        };

        self.state.folder.selected_index = Some(visible_indices[new_pos]);
    }

    fn scroll_folder_to_selected(&self) -> Task<Message> {
        if let Some(index) = self.state.folder.selected_index {
            let visible_indices =
                crate::features::folder::visible_folder_row_indices(&self.state.folder);
            let vis_idx = match visible_indices.iter().position(|&i| i == index) {
                Some(pos) => pos,
                None => return Task::none(),
            };
            let row_height = crate::features::folder::view::FOLDER_ROW_TOTAL_HEIGHT;
            let row_top = vis_idx as f32 * row_height;
            let row_bottom = row_top + row_height;
            let current_y = self.state.folder.scroll_y;
            let vh = if self.state.folder.viewport_height > 0.0 {
                self.state.folder.viewport_height
            } else {
                600.0
            };

            let target_y = if row_top < current_y {
                row_top
            } else if row_bottom > current_y + vh {
                (row_bottom - vh).max(0.0)
            } else {
                return Task::none();
            };

            iced::widget::operation::scroll_to(
                "content_scroll",
                iced::widget::operation::AbsoluteOffset {
                    x: 0.0,
                    y: target_y,
                },
            )
        } else {
            Task::none()
        }
    }

    fn handle_folder_collapse_or_parent(&mut self) -> Option<Task<Message>> {
        if let Some(selected_index) = self.state.folder.selected_index
            && let Some(row) = self.state.folder.rows.get(selected_index)
            && row.is_dir
            && self.state.folder.expanded_paths.contains(&row.path)
        {
            self.state.folder.expanded_paths.remove(&row.path);
            return Some(Task::none());
        }
        self.navigate_to_parent_folder()
    }

    fn handle_folder_expand_or_open(&mut self) -> Option<Task<Message>> {
        if let Some(selected_index) = self.state.folder.selected_index
            && let Some(row) = self.state.folder.rows.get(selected_index)
            && row.is_dir
        {
            if !self.state.folder.expanded_paths.contains(&row.path) {
                let path = row.path.clone();
                let depth = row.depth;
                self.state.folder.expanded_paths.insert(path.clone());

                let has_children = self
                    .state
                    .folder
                    .rows
                    .iter()
                    .any(|r| r.parent_path.as_deref() == Some(&path));

                if !has_children
                    && let Ok(mut children) =
                        crate::features::folder::tree::read_child_folder_rows(&path, depth)
                    && !children.is_empty()
                {
                    if self.state.folder.sort_state.active {
                        crate::core::sort_folder_rows(&mut children, &self.state.folder.sort_state);
                    }
                    let insert_pos = selected_index + 1;
                    self.state
                        .folder
                        .rows
                        .splice(insert_pos..insert_pos, children);
                }
                return Some(Task::none());
            }
            return None;
        }
        self.open_selected_row()
    }

    fn navigate_to_parent_folder(&self) -> Option<Task<Message>> {
        let parent_path = Path::new(&self.state.folder.folder_path).parent()?;
        let parent_path = parent_path.to_string_lossy().into_owned();
        let registry = self.registry.clone();
        let gen_id = self
            .state
            .generation_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let generation_id = self.state.generation_id.clone();

        Some(Task::perform(
            async move {
                let path = Path::new(&parent_path);

                if !path.exists() {
                    return None;
                }

                FilePreviewer::parse(&*registry, path)
                    .ok()
                    .and_then(|content| {
                        if generation_id.load(std::sync::atomic::Ordering::Relaxed) != gen_id {
                            return None;
                        }
                        Some(
                            crate::app::messages::SystemMsg::FileLoaded {
                                path: parent_path,
                                content,
                                generation_id: gen_id,
                            }
                            .into(),
                        )
                    })
            },
            |message| message.unwrap_or(crate::app::messages::ActionMsg::CloseRequested.into()),
        ))
    }

    fn open_selected_row(&self) -> Option<Task<Message>> {
        let selected_index = self.state.folder.selected_index?;
        let row = self.state.folder.rows.get(selected_index)?;
        let row_path = Path::new(&row.path);
        let selected_path = if row_path.is_absolute() {
            row_path.to_path_buf()
        } else {
            Path::new(&self.state.file_name).join(row_path)
        };
        let path = selected_path.to_string_lossy().into_owned();

        Some(crate::app::update::navigation::load_file_task(
            self,
            path,
            |path| crate::app::messages::SystemMsg::FilePreviewError(path).into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::test_app;
    use crate::core::types::FolderRowState;

    #[test]
    fn test_folder_navigation_selection_and_bounds() {
        let mut app = test_app(None);
        app.current_content = Some(PreviewData::Folder {
            rows: vec![
                FolderRowState {
                    name: "file1.txt".to_string(),
                    display_name: "file1.txt".to_string(),
                    kind: "File".to_string(),
                    size: "1 KB".to_string(),
                    raw_size: 1024,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "file1.txt".to_string(),
                    is_dir: false,
                    icon: "text-x-generic",
                    depth: 0,
                    parent_path: None,
                },
                FolderRowState {
                    name: "file2.txt".to_string(),
                    display_name: "file2.txt".to_string(),
                    kind: "File".to_string(),
                    size: "2 KB".to_string(),
                    raw_size: 2048,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "file2.txt".to_string(),
                    is_dir: false,
                    icon: "text-x-generic",
                    depth: 0,
                    parent_path: None,
                },
            ],
            total_size: 3072,
        });
        app.state.folder.rows = match app.current_content.as_ref().unwrap() {
            PreviewData::Folder { rows, .. } => rows.clone(),
            _ => Vec::new(),
        };

        assert_eq!(app.state.folder.selected_index, None);

        // Move down -> selects 0
        app.move_selection_down();
        assert_eq!(app.state.folder.selected_index, Some(0));

        // Move down -> selects 1
        app.move_selection_down();
        assert_eq!(app.state.folder.selected_index, Some(1));

        // Move down again -> clamped to last index 1
        app.move_selection_down();
        assert_eq!(app.state.folder.selected_index, Some(1));

        // Move up -> selects 0
        app.move_selection_up();
        assert_eq!(app.state.folder.selected_index, Some(0));

        // Move up again -> clamped to 0
        app.move_selection_up();
        assert_eq!(app.state.folder.selected_index, Some(0));
    }

    #[test]
    fn test_folder_navigation_skips_collapsed_children() {
        let mut app = test_app(None);
        app.current_content = Some(PreviewData::Folder {
            rows: vec![
                FolderRowState {
                    name: "folder".to_string(),
                    display_name: "folder".to_string(),
                    kind: "Folder".to_string(),
                    size: "—".to_string(),
                    raw_size: 0,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "folder".to_string(),
                    is_dir: true,
                    icon: "folder",
                    depth: 0,
                    parent_path: None,
                },
                FolderRowState {
                    name: "hidden_child.txt".to_string(),
                    display_name: "hidden_child.txt".to_string(),
                    kind: "File".to_string(),
                    size: "1 KB".to_string(),
                    raw_size: 1024,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "folder/hidden_child.txt".to_string(),
                    is_dir: false,
                    icon: "text-x-generic",
                    depth: 1,
                    parent_path: Some("folder".to_string()),
                },
                FolderRowState {
                    name: "sibling.txt".to_string(),
                    display_name: "sibling.txt".to_string(),
                    kind: "File".to_string(),
                    size: "2 KB".to_string(),
                    raw_size: 2048,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "sibling.txt".to_string(),
                    is_dir: false,
                    icon: "text-x-generic",
                    depth: 0,
                    parent_path: None,
                },
            ],
            total_size: 3072,
        });
        app.state.folder.rows = match app.current_content.as_ref().unwrap() {
            PreviewData::Folder { rows, .. } => rows.clone(),
            _ => Vec::new(),
        };

        // When "folder" is collapsed: visible are index 0 ("folder") and index 2 ("sibling.txt")
        app.move_selection_down();
        assert_eq!(app.state.folder.selected_index, Some(0));

        // Moving down skips index 1 (hidden child) and selects index 2 (sibling.txt)!
        app.move_selection_down();
        assert_eq!(app.state.folder.selected_index, Some(2));

        // Moving up selects index 0
        app.move_selection_up();
        assert_eq!(app.state.folder.selected_index, Some(0));

        // When "folder" is expanded: index 1 is also visible
        app.state.folder.expanded_paths.insert("folder".to_string());
        app.move_selection_down();
        assert_eq!(app.state.folder.selected_index, Some(1));
    }
}
