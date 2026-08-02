use std::path::Path;

use iced::Task;
use iced::keyboard::key::Named;

use super::Message;
use crate::core::{FilePreviewer, PreviewData};

const FOLDER_PAGE_STEP: usize = 10;

use crate::app::KglanceApp;

impl KglanceApp {
    pub(super) fn handle_folder_navigation(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        let rows_len = self.state.folder.rows.len();

        if rows_len == 0 || !matches!(self.current_content, Some(PreviewData::Folder { .. })) {
            return None;
        }

        match key {
            iced::keyboard::Key::Named(Named::ArrowDown) => {
                let new_idx = match self.state.folder.selected_index {
                    Some(idx) => (idx + 1).min(rows_len - 1),
                    None => 0,
                };
                self.state.folder.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::ArrowUp) => {
                let new_idx = match self.state.folder.selected_index {
                    Some(idx) => idx.saturating_sub(1),
                    None => 0,
                };
                self.state.folder.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::ArrowLeft) => {
                let parent = Path::new(&self.state.folder.folder_path).parent()?;
                let parent_str = parent.to_string_lossy().to_string();
                let registry = self.registry.clone();
                Some(Task::perform(
                    async move {
                        let parent_path = Path::new(&parent_str);
                        if !parent_path.exists() {
                            return None;
                        }
                        FilePreviewer::parse(&*registry, parent_path)
                            .ok()
                            .map(|content| {
                                crate::app::messages::SystemMsg::FileLoaded {
                                    path: parent_str,
                                    content,
                                }
                                .into()
                            })
                    },
                    |msg| msg.unwrap_or(crate::app::messages::ActionMsg::CloseRequested.into()),
                ))
            }
            iced::keyboard::Key::Named(Named::ArrowRight)
            | iced::keyboard::Key::Named(Named::Enter) => {
                let idx = self.state.folder.selected_index?;
                let row = self.state.folder.rows.get(idx)?;
                let full_path = Path::new(&self.state.file_name).join(&row.path);
                let path_str = full_path.to_string_lossy().to_string();
                Some(crate::app::update::navigation::load_file_task(
                    self,
                    path_str,
                    |path| crate::app::messages::SystemMsg::FilePreviewError(path).into(),
                ))
            }
            iced::keyboard::Key::Named(Named::Home) => {
                self.pending_home = false;
                self.state.folder.selected_index = Some(0);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::End) => {
                self.pending_home = false;
                self.state.folder.selected_index = Some(rows_len - 1);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::PageUp) => {
                let new_idx = match self.state.folder.selected_index {
                    Some(idx) => idx.saturating_sub(FOLDER_PAGE_STEP),
                    None => 0,
                };
                self.state.folder.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::PageDown) => {
                let new_idx = match self.state.folder.selected_index {
                    Some(idx) => (idx + FOLDER_PAGE_STEP).min(rows_len - 1),
                    None => 0,
                };
                self.state.folder.selected_index = Some(new_idx);
                Some(Task::none())
            }
            _ => None,
        }
    }
}
