use std::path::Path;

use iced::Task;
use iced::keyboard::key::Named;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::core::{FilePreviewer, PreviewData};
use crate::parsers::markdown::extract_toc;

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

        if matches!(key, iced::keyboard::Key::Named(Named::Tab)) {
            return self.update(Message::ToggleViewMode);
        }

        match &self.state.view_mode {
            crate::core::ViewMode::Detail => match &key {
                iced::keyboard::Key::Named(Named::ArrowRight) => {
                    let is_epub = self.state.file_name.to_lowercase().ends_with(".epub")
                        || self.state.file_type_text.contains("EPUB");
                    if self.state.media.has_video {
                        return self.handle_seek_relative(5.0);
                    } else if is_epub {
                        return Task::none();
                    } else {
                        return self.update(Message::NextFileClicked);
                    }
                }
                iced::keyboard::Key::Named(Named::ArrowLeft) => {
                    let is_epub = self.state.file_name.to_lowercase().ends_with(".epub")
                        || self.state.file_type_text.contains("EPUB");
                    if self.state.media.has_video {
                        return self.handle_seek_relative(-5.0);
                    } else if is_epub {
                        return Task::none();
                    } else {
                        return self.update(Message::PrevFileClicked);
                    }
                }
                _ => {}
            },
            crate::core::ViewMode::Grid(_) => {
                let total = self.state.playlist.len();
                if total > 0 {
                    let cols = self.state.grid_cols.max(1);
                    let cur = self.state.current_index;
                    let row_h = crate::core::types::GRID_ROW_HEIGHT * self.state.grid_scale;
                    let scroll_y = |row_idx: f32| -> f32 {
                        if row_idx == 0.0 { 0.0 } else { row_idx * row_h }
                    };
                    match &key {
                        iced::keyboard::Key::Named(Named::ArrowRight) => {
                            if cur + 1 < total {
                                self.state.current_index = cur + 1;
                            }
                            let row_idx = (self.state.current_index / cols) as f32;
                            return iced::widget::operation::scroll_to(
                                "grid_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: scroll_y(row_idx),
                                },
                            );
                        }
                        iced::keyboard::Key::Named(Named::ArrowLeft) => {
                            if cur > 0 {
                                self.state.current_index = cur - 1;
                            }
                            let row_idx = (self.state.current_index / cols) as f32;
                            return iced::widget::operation::scroll_to(
                                "grid_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: scroll_y(row_idx),
                                },
                            );
                        }
                        iced::keyboard::Key::Named(Named::ArrowDown) => {
                            if cur + cols < total {
                                self.state.current_index = cur + cols;
                            } else if cur < total - 1 {
                                self.state.current_index = total - 1;
                            }
                            let row_idx = (self.state.current_index / cols) as f32;
                            return iced::widget::operation::scroll_to(
                                "grid_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: scroll_y(row_idx),
                                },
                            );
                        }
                        iced::keyboard::Key::Named(Named::ArrowUp) => {
                            if cur >= cols {
                                self.state.current_index = cur - cols;
                            } else {
                                self.state.current_index = 0;
                            }
                            let row_idx = (self.state.current_index / cols) as f32;
                            return iced::widget::operation::scroll_to(
                                "grid_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: scroll_y(row_idx),
                                },
                            );
                        }

                        iced::keyboard::Key::Named(Named::Enter) => {
                            return self.update(Message::FileClickedInGrid(cur));
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(task) = self.handle_scroll_shortcuts(&key, modifiers) {
            return task;
        }

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
            iced::keyboard::Key::Named(Named::ArrowLeft) => {
                let parent = Path::new(&self.state.table.folder_path).parent()?;
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
                            .map(|content| Message::FileLoaded {
                                path: parent_str,
                                content,
                            })
                    },
                    |msg| msg.unwrap_or(Message::CloseRequested),
                ))
            }
            iced::keyboard::Key::Named(Named::ArrowRight) => {
                let idx = self.state.table.selected_index?;
                let row = self.state.table.rows.get(idx)?;
                let full_path = Path::new(&self.state.file_name).join(&row.path);
                let path_str = full_path.to_string_lossy().to_string();
                let registry = self.registry.clone();
                let path_for_err = path_str.clone();
                Some(Task::perform(
                    async move {
                        let content =
                            FilePreviewer::parse(&*registry, Path::new(&path_str)).ok()?;
                        Some(Message::FileLoaded {
                            path: path_str,
                            content,
                        })
                    },
                    move |msg| msg.unwrap_or(Message::FilePreviewError(path_for_err.clone())),
                ))
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
                let path_for_err = path_str.clone();
                Some(Task::perform(
                    async move {
                        let content =
                            FilePreviewer::parse(&*registry, Path::new(&path_str)).ok()?;
                        Some(Message::FileLoaded {
                            path: path_str,
                            content,
                        })
                    },
                    move |msg| msg.unwrap_or(Message::FilePreviewError(path_for_err.clone())),
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
            "c" | "C" => {
                if matches!(self.current_content, Some(PreviewData::Text { .. })) {
                    let selection = self.state.text.content.selection().unwrap_or_default();
                    if selection.is_empty() {
                        None
                    } else {
                        let toast = self.show_toast("Copied!");
                        Some(Task::batch(vec![iced::clipboard::write(selection), toast]))
                    }
                } else {
                    Some(self.handle_copy_path())
                }
            }
            "a" | "A" => {
                if matches!(self.current_content, Some(PreviewData::Text { .. })) {
                    use iced::widget::text_editor::Action;
                    self.state.text.content.perform(Action::SelectAll);
                    Some(Task::none())
                } else {
                    None
                }
            }
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
                    if let Some(PreviewData::Markdown { ref blocks }) = self.current_content {
                        self.state.markdown.toc = extract_toc(
                            blocks,
                            self.state.font_size,
                            &self.state.markdown.cached_image_sizes,
                        );
                    }
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
                    if let Some(PreviewData::Markdown { ref blocks }) = self.current_content {
                        self.state.markdown.toc = extract_toc(
                            blocks,
                            self.state.font_size,
                            &self.state.markdown.cached_image_sizes,
                        );
                    }
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
                if let Some(PreviewData::Markdown { ref blocks }) = self.current_content {
                    self.state.markdown.toc = extract_toc(
                        blocks,
                        self.state.font_size,
                        &self.state.markdown.cached_image_sizes,
                    );
                }
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

            // g t → toggle TOC (markdown only)
            Key::Character(c) if c == "t" && self.pending_g => {
                self.pending_g = false;
                self.pending_home = false;
                if matches!(self.current_content, Some(PreviewData::Markdown { .. })) {
                    Some(Task::done(Message::TocToggled))
                } else {
                    None
                }
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
            iced::keyboard::Key::Named(Named::Escape | Named::Space) => true,
            iced::keyboard::Key::Character(c) if c == " " => true,
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
