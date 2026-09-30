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
        if !self.is_folder_navigation_available() {
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
                    self.state.folder.selected_index = Some(0);
                    Some(self.snap_to_top())
                } else {
                    self.pending_g = true;
                    None
                }
            }
            iced::keyboard::Key::Character(c) if c == "G" => {
                self.pending_g = false;
                let last = self.folder_row_count().saturating_sub(1);
                self.state.folder.selected_index = Some(last);
                Some(self.snap_to_bottom())
            }
            iced::keyboard::Key::Named(Named::ArrowLeft) => self.navigate_to_parent_folder(),
            iced::keyboard::Key::Named(Named::ArrowRight)
            | iced::keyboard::Key::Named(Named::Enter) => self.open_selected_row(),
            iced::keyboard::Key::Named(Named::Home) => {
                self.pending_home = false;
                self.state.folder.selected_index = Some(0);
                Some(self.snap_to_top())
            }
            iced::keyboard::Key::Named(Named::End) => {
                self.pending_home = false;
                let last = self.folder_row_count().saturating_sub(1);
                self.state.folder.selected_index = Some(last);
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

    fn folder_row_count(&self) -> usize {
        self.state.folder.rows.len()
    }

    fn move_selection_down(&mut self) {
        let last_index = self.folder_row_count().saturating_sub(1);
        let next_index = self
            .state
            .folder
            .selected_index
            .map_or(0, |index| (index + 1).min(last_index));

        self.state.folder.selected_index = Some(next_index);
    }

    fn move_selection_up(&mut self) {
        let previous_index = self
            .state
            .folder
            .selected_index
            .map_or(0, |index| index.saturating_sub(1));

        self.state.folder.selected_index = Some(previous_index);
    }

    fn move_selection_by(&mut self, offset: isize) {
        let last_index = self.folder_row_count().saturating_sub(1);
        let current_index = self.state.folder.selected_index.unwrap_or(0);

        let new_index = if offset < 0 {
            current_index.saturating_sub(offset.unsigned_abs())
        } else {
            current_index
                .saturating_add(offset as usize)
                .min(last_index)
        };

        self.state.folder.selected_index = Some(new_index);
    }

    fn scroll_folder_to_selected(&self) -> Task<Message> {
        if let Some(index) = self.state.folder.selected_index {
            let row_height = crate::ui::theme::tokens::tables::ROW_HEIGHT
                + crate::ui::theme::tokens::spacing::XXS;
            let row_top = index as f32 * row_height;
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
        let selected_path = Path::new(&self.state.file_name).join(&row.path);
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
                    kind: "File".to_string(),
                    size: "1 KB".to_string(),
                    raw_size: 1024,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "file1.txt".to_string(),
                    is_dir: false,
                    icon: "text-x-generic",
                },
                FolderRowState {
                    name: "file2.txt".to_string(),
                    kind: "File".to_string(),
                    size: "2 KB".to_string(),
                    raw_size: 2048,
                    modified: "2026-09-30".to_string(),
                    raw_modified: 0,
                    path: "file2.txt".to_string(),
                    is_dir: false,
                    icon: "text-x-generic",
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
}
