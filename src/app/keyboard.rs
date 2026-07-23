use std::path::Path;

use iced::Task;
use iced::keyboard::key::Named;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::core::{FilePreviewer, PreviewData};

impl super::KglanceApp {
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

        if let Some(task) = self.handle_scroll_shortcuts(&key, modifiers) {
            return task;
        }

        // Enter -> open file externally (folder navigation handled above)
        if matches!(key, iced::keyboard::Key::Named(Named::Enter)) {
            return self.handle_open_clicked();
        }

        if let Some(task) = self.handle_close_shortcuts(&key) {
            return task;
        }

        Task::none()
    }

    fn handle_folder_navigation(&mut self, key: &iced::keyboard::Key) -> Option<Task<Message>> {
        let rows_len = self.state.table.rows.len();

        if rows_len == 0 {
            return None;
        }

        if !matches!(self.current_content, Some(PreviewData::Folder { .. })) {
            return None;
        }

        match key {
            iced::keyboard::Key::Named(Named::ArrowDown) => {
                let new_idx = match self.state.table.selected_index {
                    Some(idx) => (idx + 1).min(rows_len - 1),
                    None => 0,
                };
                self.state.table.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::ArrowUp) => {
                let new_idx = match self.state.table.selected_index {
                    Some(idx) => idx.saturating_sub(1),
                    None => 0,
                };
                self.state.table.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::Home) => {
                self.pending_home = false;
                self.state.table.selected_index = Some(0);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::End) => {
                self.pending_home = false;
                self.state.table.selected_index = Some(rows_len - 1);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::PageUp) => {
                let new_idx = match self.state.table.selected_index {
                    Some(idx) => idx.saturating_sub(10),
                    None => 0,
                };
                self.state.table.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::PageDown) => {
                let new_idx = match self.state.table.selected_index {
                    Some(idx) => (idx + 10).min(rows_len - 1),
                    None => 0,
                };
                self.state.table.selected_index = Some(new_idx);
                Some(Task::none())
            }
            iced::keyboard::Key::Named(Named::Enter) => {
                let idx = self.state.table.selected_index?;
                let row = self.state.table.rows.get(idx)?;
                let full_path = Path::new(&self.state.file_name).join(&row.path);
                let path_str = full_path.to_string_lossy().to_string();
                let registry = self.registry.clone();
                Some(Task::perform(
                    async move {
                        let content =
                            FilePreviewer::parse(&*registry, Path::new(&path_str)).ok()?;
                        Some(Message::FileLoaded {
                            path: path_str,
                            content,
                        })
                    },
                    |msg| msg.unwrap_or(Message::CloseRequested),
                ))
            }
            _ => None,
        }
    }

    fn handle_ctrl_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        if !modifiers.control() {
            return None;
        }
        let c = match key {
            iced::keyboard::Key::Character(c) => c.as_str(),
            _ => return None,
        };

        match c {
            "c" | "C" => Some(self.handle_copy_path()),
            "t" => {
                self.state.theme_dark = !self.state.theme_dark;
                Some(Task::none())
            }
            "+" | "=" => {
                if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                    self.state.image.camera.zoom =
                        (self.state.image.camera.zoom + 0.2).clamp(0.1, 10.0);
                    Some(Task::none())
                } else if matches!(
                    self.current_content,
                    Some(PreviewData::Markdown { .. }) | Some(PreviewData::Text { .. })
                ) {
                    self.state.font_size = (self.state.font_size + 1.0).clamp(8.0, 48.0);
                    Some(Task::none())
                } else {
                    None
                }
            }
            "-" => {
                if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                    self.state.image.camera.zoom =
                        (self.state.image.camera.zoom - 0.2).clamp(0.1, 10.0);
                    Some(Task::none())
                } else if matches!(
                    self.current_content,
                    Some(PreviewData::Markdown { .. }) | Some(PreviewData::Text { .. })
                ) {
                    self.state.font_size = (self.state.font_size - 1.0).clamp(8.0, 48.0);
                    Some(Task::none())
                } else {
                    None
                }
            }
            "0" if matches!(self.current_content, Some(PreviewData::Image { .. })) => {
                self.state.image.camera.zoom = 1.0;
                self.state.image.camera.offset_x = 0.0;
                self.state.image.camera.offset_y = 0.0;
                Some(Task::none())
            }
            _ => None,
        }
    }

    fn handle_font_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        match key {
            iced::keyboard::Key::Character(c) if (c == "+" || c == "=") && modifiers.shift() => {
                self.state.font_size = 14.0;
                Some(Task::none())
            }
            _ => None,
        }
    }

    fn handle_scroll_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        let scroll_amount = 80.0;
        use iced::keyboard::Key;
        let half_page_scroll_amount = 600.0;

        match key {
            Key::Named(Named::ArrowDown) => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: scroll_amount,
                    },
                ))
            }

            Key::Character(c) if c == "j" => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: scroll_amount,
                    },
                ))
            }

            Key::Named(Named::ArrowUp) => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -scroll_amount,
                    },
                ))
            }

            Key::Character(c) if c == "k" => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -scroll_amount,
                    },
                ))
            }

            Key::Named(Named::PageDown) => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: half_page_scroll_amount,
                    },
                ))
            }

            Key::Named(Named::PageUp) => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -half_page_scroll_amount,
                    },
                ))
            }

            Key::Character(c) if c == "d" => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: half_page_scroll_amount,
                    },
                ))
            }

            Key::Character(c) if c == "u" => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -half_page_scroll_amount,
                    },
                ))
            }

            // gg -> top
            Key::Character(c) if c == "g" && !modifiers.shift() => {
                self.pending_home = false;
                if self.pending_g {
                    self.pending_g = false;

                    Some(operation::snap_to(
                        "content_scroll",
                        RelativeOffset { x: 0.0, y: 0.0 },
                    ))
                } else {
                    self.pending_g = true;
                    None
                }
            }

            // G -> bottom
            Key::Character(c) if c == "G" || (c == "g" && modifiers.shift()) => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::snap_to(
                    "content_scroll",
                    RelativeOffset { x: 0.0, y: 1.0 },
                ))
            }

            // Home (double-tap like gg) -> top
            Key::Named(Named::Home) => {
                self.pending_g = false;
                if self.pending_home {
                    self.pending_home = false;

                    Some(operation::snap_to(
                        "content_scroll",
                        RelativeOffset { x: 0.0, y: 0.0 },
                    ))
                } else {
                    self.pending_home = true;
                    None
                }
            }

            Key::Named(Named::End) => {
                self.pending_g = false;
                self.pending_home = false;

                Some(operation::snap_to(
                    "content_scroll",
                    RelativeOffset { x: 0.0, y: 1.0 },
                ))
            }

            _ => {
                self.pending_g = false;
                self.pending_home = false;
                None
            }
        }
    }

    fn handle_close_shortcuts(&mut self, key: &iced::keyboard::Key) -> Option<Task<Message>> {
        let should_close = match key {
            iced::keyboard::Key::Named(Named::Escape | Named::Backspace | Named::Space) => true,
            iced::keyboard::Key::Character(c) if c == " " || c == "\u{8}" || c == "\u{7f}" => true,
            _ => false,
        };

        if should_close {
            Some(self.handle_close())
        } else {
            None
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
