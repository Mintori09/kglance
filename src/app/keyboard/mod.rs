pub(super) mod ctrl;
pub(super) mod folder;
pub(super) mod grid;
pub(super) mod scroll;
pub(super) mod search;

use iced::Task;
use iced::keyboard::key::Named;

use super::Message;
use crate::core::PreviewData;

impl super::KglanceApp {
    pub(super) fn is_epub_content(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Epub { .. }))
            || self.state.file_name.to_lowercase().ends_with(".epub")
            || self.state.file_type_text.contains("EPUB")
    }

    pub fn handle_key_pressed(
        &mut self,
        key: iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Task<Message> {
        if let Some(task) = self.handle_folder_navigation(&key) {
            return task;
        }

        if let Some(task) = self.handle_ctrl_shortcuts(&key, modifiers) {
            return task;
        }

        if let Some(task) = self.handle_font_shortcuts(&key, modifiers) {
            return task;
        }

        if matches!(key, iced::keyboard::Key::Named(Named::Tab)) {
            let is_video = self.state.media.has_video;
            if self.is_epub_content() || is_video {
                return Task::none();
            }
            return self.update(crate::app::messages::NavigationMsg::ToggleViewMode.into());
        }

        if let Some(task) = self.handle_type_to_search(&key, modifiers) {
            return task;
        }

        if let Some(task) = self.handle_view_mode_navigation(&key) {
            return task;
        }

        if let Some(task) = self.handle_scroll_shortcuts(&key, modifiers) {
            return task;
        }

        if matches!(key, iced::keyboard::Key::Named(Named::Enter)) {
            return self.handle_open_clicked();
        }

        // Search-aware Space/Escape: Space types, Escape closes search
        if self.is_search_active() {
            match &key {
                iced::keyboard::Key::Named(Named::Space) => return Task::none(),
                iced::keyboard::Key::Character(c) if c == " " => return Task::none(),
                iced::keyboard::Key::Named(Named::Escape) => {
                    return self.handle_search_close();
                }
                _ => {}
            }
        }

        if matches!(self.state.view_mode, crate::core::ViewMode::Settings) {
            if matches!(key, iced::keyboard::Key::Named(Named::Escape)) {
                return self
                    .update(crate::app::messages::NavigationMsg::ToggleSettingsClicked.into());
            }
            return Task::none();
        }

        if let Some(task) = self.handle_close_shortcuts(&key) {
            return task;
        }

        if let Some(task) = self.handle_vim_search_open(&key) {
            return task;
        }

        Task::none()
    }

    fn handle_view_mode_navigation(&mut self, key: &iced::keyboard::Key) -> Option<Task<Message>> {
        match &self.state.view_mode {
            crate::core::ViewMode::Detail => {
                match key {
                    iced::keyboard::Key::Named(Named::ArrowRight) => {
                        if self.state.media.has_video {
                            Some(self.handle_seek_relative(5.0))
                        } else if self.is_epub_content() {
                            if !self.state.epub.chapters.is_empty() {
                                let next_ch = (self.state.epub.active_chapter + 1)
                                    .min(self.state.epub.chapters.len() - 1);
                                Some(self.update(
                                    crate::app::messages::EpubMsg::ChapterClicked(next_ch).into(),
                                ))
                            } else {
                                Some(Task::none())
                            }
                        } else {
                            Some(self.update(
                                crate::app::messages::NavigationMsg::NextFileClicked.into(),
                            ))
                        }
                    }
                    iced::keyboard::Key::Named(Named::ArrowLeft) => {
                        if self.state.media.has_video {
                            Some(self.handle_seek_relative(-5.0))
                        } else if self.is_epub_content() {
                            if !self.state.epub.chapters.is_empty() {
                                let prev_ch = self.state.epub.active_chapter.saturating_sub(1);
                                Some(self.update(
                                    crate::app::messages::EpubMsg::ChapterClicked(prev_ch).into(),
                                ))
                            } else {
                                Some(Task::none())
                            }
                        } else {
                            Some(self.update(
                                crate::app::messages::NavigationMsg::PrevFileClicked.into(),
                            ))
                        }
                    }
                    _ => None,
                }
            }
            crate::core::ViewMode::Grid(_) => self.handle_grid_navigation(key),
            crate::core::ViewMode::Settings => None,
        }
    }

    pub fn handle_ctrl_changed(&mut self, held: bool) -> Task<Message> {
        self.ctrl_held = held;
        Task::none()
    }

    pub fn handle_shift_changed(&mut self, held: bool) -> Task<Message> {
        self.shift_held = held;
        Task::none()
    }

    pub fn handle_modifiers_changed(
        &mut self,
        modifiers: iced::keyboard::Modifiers,
    ) -> Task<Message> {
        self.ctrl_held = modifiers.control();
        self.shift_held = modifiers.shift();
        Task::none()
    }
}
