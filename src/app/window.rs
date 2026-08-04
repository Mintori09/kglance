use std::time::Duration;

use iced::window::{self, Event as WindowEvent, Mode, Settings as WindowSettings};
use iced::{Size, Task, clipboard};

use super::KglanceApp;
use super::Message;
use crate::core::types::{GRID_GAP, GRID_ITEM_WIDTH};
use crate::core::{PreviewData, ToastInfo};
use crate::features::video::handler::PlayerCommand;

const TOAST_DURATION_SECS: u64 = 2;
const MIN_GRID_COLUMNS: usize = 1;

impl KglanceApp {
    fn update_grid_cols(&mut self, window_width: f32) {
        self.state.window_width = window_width;
        self.state.grid_cols = Self::calculate_grid_cols(window_width, self.state.grid_scale);
    }

    pub(crate) fn recalc_grid_cols(&mut self) {
        if self.state.window_width > 0.0 {
            self.update_grid_cols(self.state.window_width);
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

        self.stop_video_player();

        if self.is_daemon {
            self.current_content = None;
            self.window_id.take().map_or_else(Task::none, window::close)
        } else {
            iced::exit()
        }
    }

    fn stop_video_player(&mut self) {
        if let Some(video_sender) = &self.video_tx {
            let _ = video_sender.try_send(PlayerCommand::Stop);
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
        self.current_content = None;

        if let Some(window_id) = self.window_id {
            Task::batch(vec![
                window::set_mode(window_id, Mode::Windowed),
                window::gain_focus(window_id),
            ])
        } else {
            self.create_new_window()
        }
    }

    pub(crate) fn create_new_window(&self) -> Task<Message> {
        let settings = WindowSettings {
            size: self.state.window_default_size,
            min_size: Some(self.state.window_min_size),
            icon: crate::load_app_icon(),
            exit_on_close_request: false,
            decorations: true,
            ..Default::default()
        };

        let (_, open_task) = window::open(settings);

        open_task.map(|opened_window_id| {
            crate::app::messages::SystemMsg::WindowEvent(
                opened_window_id,
                WindowEvent::Opened {
                    position: None,
                    size: Size::ZERO,
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
                self.update_grid_cols(size.width);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn handle_window_opened(&mut self, window_id: window::Id, size: Size) -> Task<Message> {
        self.window_id = Some(window_id);
        self.update_grid_cols(size.width);

        if let Some(content) = self
            .current_content
            .as_ref()
            .filter(|c| c.supports_custom_initial_size())
        {
            return window::resize(window_id, content.initial_window_size());
        }

        Task::none()
    }

    fn handle_window_close_requested(&mut self, window_id: window::Id) -> Task<Message> {
        if self.is_daemon {
            self.current_content = None;
            self.window_id = None;
            window::close(window_id)
        } else {
            iced::exit()
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
