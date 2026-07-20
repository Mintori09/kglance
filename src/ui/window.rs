use std::sync::Arc;
use std::sync::Mutex;
use iced::{Element, Task, Subscription};
use iced::widget::text;
use tokio::sync::mpsc;
use crate::dbus::DaemonCommand;
use crate::ui::types::{KglanceState, Message};
use crate::parser::ParserRegistry;


pub struct KglanceApp {
    pub state: KglanceState,
    pub registry: Arc<ParserRegistry>,
    pub daemon_rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>,
    pub is_daemon: bool,
    pub window_id: Option<iced::window::Id>,
}

impl KglanceApp {
    pub fn new(
        registry: Arc<ParserRegistry>,
        daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
        initial_path: Option<&str>,
        is_daemon: bool,
    ) -> (Self, Task<Message>) {
        let app = Self {
            state: KglanceState::default(),
            registry,
            daemon_rx: Arc::new(Mutex::new(daemon_rx)),
            is_daemon,
            window_id: None,
        };

        let task = if let Some(path) = initial_path {
            let path_str = path.to_string();
            let reg = app.registry.clone();
            Task::perform(
                async move {
                    let content = reg.parse(std::path::Path::new(&path_str)).ok()?;
                    Some(Message::FileLoaded {
                        path: path_str,
                        content: Arc::new(content),
                    })
                },
                |msg| msg.unwrap_or(Message::CloseRequested),
            )
        } else {
            Task::none()
        };

        (app, task)
    }

    pub fn title(&self) -> String {
        if self.state.file_name.is_empty() {
            "Kglance Preview".to_string()
        } else {
            format!("Kglance - {}", self.state.file_name)
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseRequested => {
                if self.is_daemon {
                    if let Some(id) = self.window_id {
                        iced::window::change_mode(id, iced::window::Mode::Hidden)
                    } else {
                        Task::none()
                    }
                } else {
                    iced::exit()
                }
            }
            Message::FileLoaded { path, content: _ } => {
                self.state.file_name = path;
                self.state.content_ready = true;
                
                // Show window and focus
                if let Some(id) = self.window_id {
                    let t1 = iced::window::change_mode(id, iced::window::Mode::Windowed);
                    let t2 = iced::window::gain_focus(id);
                    Task::batch(vec![t1, t2])
                } else {
                    Task::none()
                }
            }
            Message::WindowEvent(id, event) => {
                if let iced::window::Event::Opened { .. } = event {
                    self.window_id = Some(id);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        text("Kglance Preview Window").into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let rx_opt = self.daemon_rx.clone();
        let dbus_sub = Subscription::run_with_id(
            "dbus_subscription",
            iced::stream::channel(100, move |mut output| async move {
                let rx = {
                    let mut guard = rx_opt.lock().unwrap();
                    guard.take()
                };
                if let Some(mut rx) = rx {
                    use iced::futures::SinkExt;
                    while let Some(cmd) = rx.recv().await {
                        match cmd {
                            DaemonCommand::ShowPreview { path, content } => {
                                let _ = output.send(Message::FileLoaded {
                                    path,
                                    content: Arc::new(content),
                                }).await;
                            }
                            DaemonCommand::HidePreview => {
                                let _ = output.send(Message::CloseRequested).await;
                            }
                        }
                    }
                }
            })
        );

        let event_sub = iced::window::events().map(|(id, event)| Message::WindowEvent(id, event));

        Subscription::batch(vec![dbus_sub, event_sub])
    }

    pub fn theme(&self) -> iced::Theme {
        if self.state.theme_dark {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        }
    }
}
