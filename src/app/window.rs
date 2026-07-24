use iced::Task;

use super::Message;
use crate::core::types::{GRID_GAP, GRID_ITEM_WIDTH};

impl super::KglanceApp {
    fn update_grid_cols(&mut self, width: f32) {
        self.state.window_width = width;
        let scale = self.state.grid_scale;
        self.state.grid_cols = Self::calc_grid_cols(width, scale);
    }

    pub(crate) fn recalc_grid_cols(&mut self) {
        let w = self.state.window_width;
        let scale = self.state.grid_scale;
        if w > 0.0 {
            self.state.grid_cols = Self::calc_grid_cols(w, scale);
        }
    }

    fn calc_grid_cols(width: f32, scale: f32) -> usize {
        let item_w = GRID_ITEM_WIDTH * scale;
        let gap = GRID_GAP * scale;
        ((width - gap) / (item_w + gap)).floor().max(1.0) as usize
    }
    fn close_current(&mut self) -> Task<Message> {
        if let Some(tx) = &self.video_tx {
            let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Stop);
        }
        if self.is_daemon {
            self.current_content = None;
            if let Some(id) = self.window_id.take() {
                iced::window::close(id)
            } else {
                Task::none()
            }
        } else {
            iced::exit()
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
        let toast = self.show_toast("Copied!");
        Task::batch(vec![
            iced::clipboard::write(self.state.file_name.clone()),
            toast,
        ])
    }

    pub fn show_toast(&mut self, message: impl Into<String>) -> Task<Message> {
        let id = self.state.next_toast_id;
        self.state.next_toast_id += 1;
        self.state.toasts.push(crate::core::ToastInfo {
            id,
            message: message.into(),
        });
        Task::perform(
            tokio::time::sleep(std::time::Duration::from_secs(2)),
            move |_| Message::ToastDismissed(id),
        )
    }

    pub fn handle_daemon_open_window(&mut self, path: String) -> Task<Message> {
        self.state.file_name = path;
        self.state.content_ready = false;
        self.current_content = None;

        if let Some(id) = self.window_id {
            Task::batch(vec![
                iced::window::set_mode(id, iced::window::Mode::Windowed),
                iced::window::gain_focus(id),
            ])
        } else {
            let settings = iced::window::Settings {
                size: iced::Size::new(1024.0, 768.0),
                min_size: Some(iced::Size::new(800.0, 600.0)),
                exit_on_close_request: false,
                decorations: true,
                ..Default::default()
            };
            let (id, open_task) = iced::window::open(settings);
            let _ = id;
            open_task.map(|wid| {
                Message::WindowEvent(
                    wid,
                    iced::window::Event::Opened {
                        position: None,
                        size: iced::Size::ZERO,
                    },
                )
            })
        }
    }

    pub fn handle_window_event(
        &mut self,
        id: iced::window::Id,
        event: iced::window::Event,
    ) -> Task<Message> {
        match event {
            iced::window::Event::Opened { size, .. } => {
                self.probe.mark_window_opened(); // P2
                self.window_id = Some(id);
                self.update_grid_cols(size.width);
                match &self.current_content {
                    Some(crate::core::PreviewData::Image { width, height, .. }) => {
                        let size =
                            crate::ui::handlers::image::calculate_window_size(*width, *height);
                        return iced::window::resize(id, size);
                    }
                    Some(crate::core::PreviewData::Media { .. }) => {
                        return iced::window::resize(id, iced::Size::new(850.0, 550.0));
                    }
                    _ => {}
                }
            }
            iced::window::Event::CloseRequested => {
                if self.is_daemon {
                    self.current_content = None;
                    self.window_id = None;
                    return iced::window::close(id);
                } else {
                    return iced::exit();
                }
            }
            iced::window::Event::Resized(size) => {
                self.update_grid_cols(size.width);
            }
            _ => {}
        }
        Task::none()
    }
}
