use std::time::Duration;

use iced::window::{self, Event as WindowEvent, Mode, Settings as WindowSettings};
use iced::{Size, Task, clipboard};

use super::KglanceApp;
use super::Message;
use crate::core::types::{GRID_GAP, GRID_ITEM_WIDTH};
use crate::core::{PreviewData, ToastInfo};

const TOAST_DURATION_SECS: u64 = 2;
const MIN_GRID_COLUMNS: usize = 1;

impl KglanceApp {
    pub fn title(&self) -> String {
        if self.state.file_name.is_empty() {
            "Kglance Preview".to_string()
        } else {
            let name = std::path::Path::new(&self.state.file_name)
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or(std::borrow::Cow::Borrowed(&self.state.file_name));
            format!("Kglance - {name}")
        }
    }

    pub fn title_daemon(&self, _window_id: iced::window::Id) -> String {
        self.title()
    }

    fn update_grid_cols(&mut self, window_width: f32, window_height: f32) {
        self.state.window_width = window_width;
        self.state.window_height = window_height;
        self.state.grid_cols = Self::calculate_grid_cols(window_width, self.state.grid_scale);
    }

    pub(crate) fn recalc_grid_cols(&mut self) {
        if self.state.window_width > 0.0 {
            self.update_grid_cols(self.state.window_width, self.state.window_height);
        }
    }

    fn calculate_grid_cols(available_width: f32, scale: f32) -> usize {
        let scaled_item_width = GRID_ITEM_WIDTH * scale;
        let scaled_gap = GRID_GAP * scale;

        let available_content_width = available_width - scaled_gap;
        let item_slot_width = scaled_item_width + scaled_gap;

        let calculated_cols = (available_content_width / item_slot_width).floor();
        calculated_cols.max(MIN_GRID_COLUMNS as f32) as usize
    }

    fn close_current(&mut self) -> Task<Message> {
        self.record_read_position();
        if self.state.read_positions_dirty {
            let _ = self.state.read_positions.save();
            self.state.read_positions_dirty = false;
        }

        self.video = None;

        if self.is_daemon {
            self.current_content = None;
            self.state.cache.clear();
            self.state.pending_preloads.clear();
            self.invalidate_render_generations();
            self.is_window_opening = false;
            self.is_gui_open
                .store(false, std::sync::atomic::Ordering::Release);
            self.window_id.take().map_or_else(Task::none, window::close)
        } else {
            let _ = std::io::Write::flush(&mut std::io::stdout());
            std::process::exit(0);
        }
    }

    pub fn handle_close(&mut self) -> Task<Message> {
        self.close_current()
    }

    pub fn handle_open_clicked(&mut self) -> Task<Message> {
        let _ = std::process::Command::new("xdg-open")
            .arg(&self.state.file_name)
            .spawn();

        self.close_current()
    }

    pub fn handle_copy_path(&mut self) -> Task<Message> {
        let copy_task = clipboard::write(self.state.file_name.clone());
        let toast_task = self.show_toast("Copied!");

        Task::batch(vec![copy_task, toast_task])
    }

    pub fn show_toast(&mut self, message: impl Into<String>) -> Task<Message> {
        let toast_id = self.state.next_toast_id;
        self.state.next_toast_id += 1;

        self.state.toasts.push(ToastInfo {
            id: toast_id,
            message: message.into(),
        });

        Task::perform(
            tokio::time::sleep(Duration::from_secs(TOAST_DURATION_SECS)),
            move |_| crate::app::messages::SystemMsg::ToastDismissed(toast_id).into(),
        )
    }

    pub fn handle_daemon_open_window(&mut self, path: String) -> Task<Message> {
        self.state.file_name = path;
        self.state.content_ready = false;

        if let Some(window_id) = self.window_id {
            Task::batch(vec![
                window::set_mode(window_id, Mode::Windowed),
                window::gain_focus(window_id),
            ])
        } else if !self.is_window_opening {
            self.is_window_opening = true;
            self.create_new_window()
        } else {
            Task::none()
        }
    }

    pub(crate) fn create_new_window(&self) -> Task<Message> {
        let default_size = self.state.window_default_size;
        let settings = WindowSettings {
            size: default_size,
            min_size: Some(self.state.window_min_size),
            icon: crate::load_app_icon(),
            exit_on_close_request: false,
            decorations: true,
            ..Default::default()
        };

        let (_, open_task) = window::open(settings);

        open_task.map(move |opened_window_id| {
            crate::app::messages::SystemMsg::WindowEvent(
                opened_window_id,
                WindowEvent::Opened {
                    position: None,
                    size: default_size,
                },
            )
            .into()
        })
    }

    pub fn handle_window_event(
        &mut self,
        window_id: window::Id,
        event: WindowEvent,
    ) -> Task<Message> {
        match event {
            WindowEvent::Opened { size, .. } => self.handle_window_opened(window_id, size),
            WindowEvent::CloseRequested => self.handle_window_close_requested(window_id),
            WindowEvent::Resized(size) => {
                if size.width > 0.0 && size.height > 0.0 {
                    self.update_grid_cols(size.width, size.height);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn handle_window_opened(&mut self, window_id: window::Id, size: Size) -> Task<Message> {
        self.window_id = Some(window_id);
        self.is_window_opening = false;
        self.is_gui_open
            .store(true, std::sync::atomic::Ordering::Release);

        let width = if size.width > 0.0 {
            size.width
        } else if self.state.window_width > 0.0 {
            self.state.window_width
        } else {
            self.state.window_default_size.width
        };

        let height = if size.height > 0.0 {
            size.height
        } else if self.state.window_height > 0.0 {
            self.state.window_height
        } else {
            self.state.window_default_size.height
        };

        self.update_grid_cols(width, height);

        if let Some(content) = self
            .current_content
            .as_ref()
            .filter(|c| c.supports_custom_initial_size())
        {
            let target_size = content.initial_window_size(&self.state);
            if target_size.width > 0.0 && target_size.height > 0.0 {
                return window::resize(window_id, target_size);
            }
        }

        Task::none()
    }

    fn handle_window_close_requested(&mut self, window_id: window::Id) -> Task<Message> {
        self.video = None;
        if self.is_daemon {
            self.current_content = None;
            self.state.cache.clear();
            self.state.pending_preloads.clear();
            self.invalidate_render_generations();
            self.window_id = None;
            self.is_window_opening = false;
            self.is_gui_open
                .store(false, std::sync::atomic::Ordering::Release);
            window::close(window_id)
        } else {
            self.close_current()
        }
    }
}

impl PreviewData {
    fn supports_custom_initial_size(&self) -> bool {
        matches!(
            self,
            PreviewData::Image { .. } | PreviewData::Font { .. } | PreviewData::Media { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::app::test_util::test_app;

    #[test]
    fn test_gui_open_state_synced_on_window_lifecycle() {
        let mut app = test_app(None);
        app.is_daemon = true;
        assert!(!app.is_gui_open.load(std::sync::atomic::Ordering::Acquire));

        let window_id = iced::window::Id::unique();

        // Window opened
        let _task = app.handle_window_opened(window_id, iced::Size::new(800.0, 600.0));
        assert!(app.is_gui_open.load(std::sync::atomic::Ordering::Acquire));

        app.state.cache.put(
            "dummy_path.txt".to_string(),
            crate::core::CachedContent::from_preview(std::sync::Arc::new(
                crate::core::PreviewData::Text {
                    content: "hello".to_string(),
                    line_numbers: "1".to_string(),
                    language: "text".to_string(),
                },
            )),
        );
        assert!(!app.state.cache.is_empty());

        // Window close requested
        let _task = app.handle_window_close_requested(window_id);
        assert!(!app.is_gui_open.load(std::sync::atomic::Ordering::Acquire));
        assert!(app.state.cache.is_empty());

        // Background preload completion while closed should not repopulate cache
        let _ = crate::app::update::file::handle_preload_completed(
            &mut app,
            "stale_preload.txt".to_string(),
            std::sync::Arc::new(crate::core::PreviewData::Text {
                content: "stale".to_string(),
                line_numbers: "1".to_string(),
                language: "text".to_string(),
            }),
            None,
        );
        assert!(app.state.cache.is_empty());
    }

    #[test]
    fn test_prevent_duplicate_window_creation_when_opening() {
        let mut app = test_app(None);
        app.is_daemon = true;
        assert!(!app.is_window_opening);

        // First call should set is_window_opening = true
        let _tasks = app.prepare_window_tasks();
        assert!(app.is_window_opening);

        // Subsequent call while still opening should NOT create another window task
        let tasks2 = app.prepare_window_tasks();
        assert!(tasks2.is_empty());

        // Once window is opened, is_window_opening should reset to false
        let window_id = iced::window::Id::unique();
        let _task = app.handle_window_opened(window_id, iced::Size::new(800.0, 600.0));
        assert!(!app.is_window_opening);
        assert_eq!(app.window_id, Some(window_id));
    }
}
