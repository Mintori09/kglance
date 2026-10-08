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
        self.ctrl_held = modifiers.control()
            || modifiers.command()
            || matches!(key, iced::keyboard::Key::Named(Named::Control));
        self.shift_held =
            modifiers.shift() || matches!(key, iced::keyboard::Key::Named(Named::Shift));

        if matches!(key, iced::keyboard::Key::Named(Named::Copy))
            && let Some(task) = self.handle_ctrl_copy()
        {
            return task;
        }

        if self.ctrl_held || modifiers.control() || modifiers.command() {
            if let Some(task) = self.handle_file_navigation(&key) {
                return task;
            }
            if let Some(task) = self.handle_ctrl_shortcuts(&key, modifiers) {
                return task;
            }
            return Task::none();
        }

        if let Some(task) = self.handle_folder_navigation(&key) {
            return task;
        }

        if let Some(task) = self.handle_font_shortcuts(&key, modifiers) {
            return task;
        }

        if matches!(key, iced::keyboard::Key::Named(Named::Tab)) {
            self.stop_active_scroll_animations();
            return self.update(crate::app::messages::NavigationMsg::ToggleViewMode.into());
        }

        if let Some(task) = self.handle_type_to_search(&key, modifiers) {
            return task;
        }

        if let Some(task) = self.handle_view_mode_navigation(&key) {
            return task;
        }

        if let Some(task) = self.handle_json_tree_navigation(&key, modifiers) {
            return task;
        }

        if let Some(task) = self.handle_scroll_shortcuts(&key, modifiers) {
            return task;
        }

        if let Some(task) = self.handle_typst_shortcut(&key) {
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

        if self.state.image.show_info && matches!(key, iced::keyboard::Key::Named(Named::Escape)) {
            return self.update(crate::app::messages::ImageMsg::CloseInfo.into());
        }

        if let Some(task) = self.handle_close_shortcuts(&key) {
            return task;
        }

        if let Some(task) = self.handle_vim_search_open(&key) {
            return task;
        }

        if matches!(self.current_content, Some(PreviewData::Image { .. })) {
            match &key {
                iced::keyboard::Key::Character(c) if c == "=" => {
                    return self.update(crate::app::messages::ImageMsg::FitToWindow.into());
                }
                iced::keyboard::Key::Character(c) if c.eq_ignore_ascii_case("i") => {
                    return self.update(crate::app::messages::ImageMsg::ToggleInfo.into());
                }
                _ => {}
            }
        }

        if matches!(self.current_content, Some(PreviewData::Audio { .. })) {
            match &key {
                iced::keyboard::Key::Character(c)
                    if c.eq_ignore_ascii_case("p") || c.eq_ignore_ascii_case("k") =>
                {
                    return self.update(crate::app::messages::AudioMsg::PlayPauseClicked.into());
                }
                iced::keyboard::Key::Character(c) if c.eq_ignore_ascii_case("m") => {
                    return self.update(crate::app::messages::AudioMsg::ToggleMuteClicked.into());
                }
                iced::keyboard::Key::Named(Named::ArrowUp) => {
                    let next_vol = (self.state.audio.volume + 0.05).min(1.0);
                    return self
                        .update(crate::app::messages::AudioMsg::VolumeChanged(next_vol).into());
                }
                iced::keyboard::Key::Named(Named::ArrowDown) => {
                    let next_vol = (self.state.audio.volume - 0.05).max(0.0);
                    return self
                        .update(crate::app::messages::AudioMsg::VolumeChanged(next_vol).into());
                }
                iced::keyboard::Key::Character(c)
                    if c.len() == 1 && c.chars().all(|ch| ch.is_ascii_digit()) =>
                {
                    if let Some(digit) = c.chars().next().and_then(|ch| ch.to_digit(10)) {
                        let ratio = digit as f32 / 10.0;
                        return self
                            .update(crate::app::messages::AudioMsg::SeekClicked(ratio).into());
                    }
                }
                _ => {}
            }
        }

        if self.state.media.has_video {
            match &key {
                iced::keyboard::Key::Character(c)
                    if c.eq_ignore_ascii_case("p") || c.eq_ignore_ascii_case("k") =>
                {
                    return self.update(crate::app::messages::MediaMsg::PlayPauseClicked.into());
                }
                iced::keyboard::Key::Character(c)
                    if c.eq_ignore_ascii_case("c") || c.eq_ignore_ascii_case("v") =>
                {
                    return self.update(crate::app::messages::MediaMsg::CycleSubtitleTrack.into());
                }
                iced::keyboard::Key::Character(c)
                    if c.len() == 1 && c.chars().all(|ch| ch.is_ascii_digit()) =>
                {
                    if let Some(digit) = c.chars().next().and_then(|ch| ch.to_digit(10)) {
                        let ratio = digit as f32 / 10.0;
                        return self
                            .update(crate::app::messages::MediaMsg::SeekClicked(ratio).into());
                    }
                }
                _ => {}
            }
        }

        Task::none()
    }

    fn handle_typst_shortcut(&mut self, key: &iced::keyboard::Key) -> Option<Task<Message>> {
        if self.ctrl_held || !matches!(self.current_content, Some(PreviewData::Typst { .. })) {
            return None;
        }
        match key {
            iced::keyboard::Key::Character(c) if c.eq_ignore_ascii_case("s") => {
                Some(self.update(crate::app::messages::TypstMsg::ToggleSource.into()))
            }
            _ => None,
        }
    }

    pub(super) fn handle_file_navigation(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        if !self.ctrl_held {
            return None;
        }

        use iced::keyboard::key::Named;
        let ctrl_shift = self.shift_held;

        let msg = match key {
            iced::keyboard::Key::Named(Named::ArrowRight) if ctrl_shift => {
                crate::app::messages::NavigationMsg::NextFileClicked
            }
            iced::keyboard::Key::Named(Named::PageDown) if !ctrl_shift => {
                crate::app::messages::NavigationMsg::NextFileClicked
            }
            iced::keyboard::Key::Named(Named::ArrowLeft) if ctrl_shift => {
                crate::app::messages::NavigationMsg::PrevFileClicked
            }
            iced::keyboard::Key::Named(Named::PageUp) if !ctrl_shift => {
                crate::app::messages::NavigationMsg::PrevFileClicked
            }
            _ => return None,
        };

        self.stop_active_scroll_animations();
        Some(self.update(msg.into()))
    }

    pub(super) fn handle_view_mode_navigation(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        if self.ctrl_held {
            return None;
        }
        match &self.state.view_mode {
            crate::core::ViewMode::Detail => match key {
                iced::keyboard::Key::Named(Named::ArrowRight) => {
                    if self.state.media.has_video {
                        self.stop_active_scroll_animations();
                        Some(crate::features::video::update::handle_seek_relative(
                            self, 5.0,
                        ))
                    } else if matches!(self.current_content, Some(PreviewData::Audio { .. })) {
                        self.stop_active_scroll_animations();
                        Some(crate::features::audio::update::handle_seek_relative(
                            self, 5.0,
                        ))
                    } else if self.is_epub_content() {
                        self.stop_active_scroll_animations();
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
                        None
                    }
                }
                iced::keyboard::Key::Named(Named::ArrowLeft) => {
                    if self.state.media.has_video {
                        self.stop_active_scroll_animations();
                        Some(crate::features::video::update::handle_seek_relative(
                            self, -5.0,
                        ))
                    } else if matches!(self.current_content, Some(PreviewData::Audio { .. })) {
                        self.stop_active_scroll_animations();
                        Some(crate::features::audio::update::handle_seek_relative(
                            self, -5.0,
                        ))
                    } else if self.is_epub_content() {
                        self.stop_active_scroll_animations();
                        if !self.state.epub.chapters.is_empty() {
                            let prev_ch = self.state.epub.active_chapter.saturating_sub(1);
                            Some(self.update(
                                crate::app::messages::EpubMsg::ChapterClicked(prev_ch).into(),
                            ))
                        } else {
                            Some(Task::none())
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
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
