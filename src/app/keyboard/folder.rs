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
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::ArrowUp) => {
                self.move_selection_up();
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::ArrowLeft) => self.navigate_to_parent_folder(),
            iced::keyboard::Key::Named(Named::ArrowRight)
            | iced::keyboard::Key::Named(Named::Enter) => self.open_selected_row(),
            iced::keyboard::Key::Named(Named::Home) => {
                self.pending_home = false;
                self.state.folder.selected_index = Some(0);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::End) => {
                self.pending_home = false;
                self.state.folder.selected_index = Some(self.folder_row_count() - 1);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::PageUp) => {
                self.move_selection_by(-FOLDER_PAGE_STEP);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::PageDown) => {
                self.move_selection_by(FOLDER_PAGE_STEP);
                Some(Task::none())
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
        let last_index = self.folder_row_count() - 1;
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

    fn navigate_to_parent_folder(&self) -> Option<Task<Message>> {
        let parent_path = Path::new(&self.state.folder.folder_path).parent()?;
        let parent_path = parent_path.to_string_lossy().into_owned();
        let registry = self.registry.clone();

        Some(Task::perform(
            async move {
                let path = Path::new(&parent_path);

                if !path.exists() {
                    return None;
                }

                FilePreviewer::parse(&*registry, path).ok().map(|content| {
                    crate::app::messages::SystemMsg::FileLoaded {
                        path: parent_path,
                        content,
                    }
                    .into()
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
