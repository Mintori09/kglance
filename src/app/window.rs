use iced::Task;

use super::Message;

impl super::KglanceApp {
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

    pub fn handle_copy_path(&self) -> Task<Message> {
        iced::clipboard::write(self.state.file_name.clone())
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
                decorations: false,
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
            iced::window::Event::Opened { .. } => {
                self.probe.mark_window_opened(); // P2
                self.window_id = Some(id);
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
            _ => {}
        }
        Task::none()
    }
}
