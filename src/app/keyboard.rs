use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset};

use super::Message;
use crate::core::PreviewData;

impl super::KglanceApp {
    pub fn handle_key_pressed(
        &mut self,
        key: iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Task<Message> {
        use iced::keyboard::key::Named;
        let ctrl = modifiers.control();

        if matches!(self.current_content, Some(PreviewData::Folder { .. })) {
            let rows_len = self.state.table.rows.len();
            if rows_len > 0 {
                match &key {
                    iced::keyboard::Key::Named(Named::ArrowDown) => {
                        let new_idx = match self.state.table.selected_index {
                            Some(idx) => (idx + 1).min(rows_len - 1),
                            None => 0,
                        };
                        self.state.table.selected_index = Some(new_idx);
                        return Task::none();
                    }
                    iced::keyboard::Key::Named(Named::ArrowUp) => {
                        let new_idx = match self.state.table.selected_index {
                            Some(idx) => idx.saturating_sub(1),
                            None => 0,
                        };
                        self.state.table.selected_index = Some(new_idx);
                        return Task::none();
                    }
                    iced::keyboard::Key::Named(Named::Home) => {
                        self.state.table.selected_index = Some(0);
                        return Task::none();
                    }
                    iced::keyboard::Key::Named(Named::End) => {
                        self.state.table.selected_index = Some(rows_len - 1);
                        return Task::none();
                    }
                    iced::keyboard::Key::Named(Named::PageUp) => {
                        let new_idx = match self.state.table.selected_index {
                            Some(idx) => idx.saturating_sub(10),
                            None => 0,
                        };
                        self.state.table.selected_index = Some(new_idx);
                        return Task::none();
                    }
                    iced::keyboard::Key::Named(Named::PageDown) => {
                        let new_idx = match self.state.table.selected_index {
                            Some(idx) => (idx + 10).min(rows_len - 1),
                            None => 0,
                        };
                        self.state.table.selected_index = Some(new_idx);
                        return Task::none();
                    }
                    _ => {}
                }
            }
        }

        if ctrl {
            if let iced::keyboard::Key::Character(ref c) = key {
                match c.as_str() {
                    "c" => return iced::clipboard::write(self.state.file_name.clone()),
                    "+" | "=" => {
                        if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                            self.state.image.zoom = (self.state.image.zoom + 0.2).clamp(0.1, 10.0);
                            return Task::none();
                        }
                    }
                    "-" => {
                        if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                            self.state.image.zoom = (self.state.image.zoom - 0.2).clamp(0.1, 10.0);
                            return Task::none();
                        }
                    }
                    _ => {}
                }
            }
            if let iced::keyboard::Key::Character(ref c) = key
                && c == "0"
                && matches!(self.current_content, Some(PreviewData::Image { .. }))
            {
                self.state.image.zoom = 1.0;
                return Task::none();
            }
        }

        let scroll_amount = 80.0;
        let half_page_scroll_amount = 600.0;
        let scroll_task: Option<Task<Message>> = match &key {
            iced::keyboard::Key::Named(Named::ArrowDown) => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: scroll_amount,
                },
            )),
            iced::keyboard::Key::Character(c) if c == "j" => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: scroll_amount,
                },
            )),
            iced::keyboard::Key::Named(Named::ArrowUp) => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: -scroll_amount,
                },
            )),
            iced::keyboard::Key::Character(c) if c == "k" => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: -scroll_amount,
                },
            )),
            iced::keyboard::Key::Named(Named::PageUp) => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: -half_page_scroll_amount,
                },
            )),
            iced::keyboard::Key::Named(Named::PageDown) => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: half_page_scroll_amount,
                },
            )),
            iced::keyboard::Key::Character(c) if c == "d" => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: half_page_scroll_amount,
                },
            )),
            iced::keyboard::Key::Character(c) if c == "u" => Some(operation::scroll_by(
                "content_scroll",
                AbsoluteOffset {
                    x: 0.0,
                    y: -half_page_scroll_amount,
                },
            )),
            _ => None,
        };
        if let Some(task) = scroll_task {
            return task;
        }

        let should_close = match &key {
            iced::keyboard::Key::Named(Named::Escape | Named::Backspace | Named::Space) => true,
            iced::keyboard::Key::Character(c) if c == " " || c == "\u{8}" || c == "\u{7f}" => true,
            _ => false,
        };

        if should_close {
            self.handle_close()
        } else {
            Task::none()
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
