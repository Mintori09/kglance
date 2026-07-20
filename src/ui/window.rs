use std::sync::Arc;
use std::sync::Mutex;
use iced::{Element, Task, Subscription, Length};
use iced::widget::{column, row, text, container, button};
use tokio::sync::mpsc;
use crate::dbus::DaemonCommand;
use crate::ui::types::{KglanceState, Message};
use crate::parser::{ParserRegistry, ParsedContent};

pub struct KglanceApp {
    pub state: KglanceState,
    pub registry: Arc<ParserRegistry>,
    pub daemon_rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>,
    pub is_daemon: bool,
    pub window_id: Option<iced::window::Id>,
    pub current_content: Option<Arc<ParsedContent>>,
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
            current_content: None,
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

    fn populate_state_from_content(&mut self, content: Arc<ParsedContent>) {
        match &*content {
            ParsedContent::Text { content: text_content, language, line_count, .. } => {
                self.state.text.content = text_content.clone();
                self.state.text.line_numbers = (1..=*line_count)
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.state.file_type_text = format!("Text ({})", language);
            }
            ParsedContent::Image { format, width, height, exif, .. } => {
                self.state.image = crate::ui::types::ImageState::default();
                if let Some(exif_data) = exif {
                    self.state.image.exif_content = format!(
                        "Camera Make: {}\nCamera Model: {}\nDate Taken: {}\nGPS Lat: {}\nGPS Lon: {}\nExposure: {}\nF-Number: {}\nISO: {}\nFocal Length: {}",
                        exif_data.camera_make.as_deref().unwrap_or("N/A"),
                        exif_data.camera_model.as_deref().unwrap_or("N/A"),
                        exif_data.date_taken.as_deref().unwrap_or("N/A"),
                        exif_data.gps_lat.as_deref().unwrap_or("N/A"),
                        exif_data.gps_lon.as_deref().unwrap_or("N/A"),
                        exif_data.exposure.as_deref().unwrap_or("N/A"),
                        exif_data.f_number.as_deref().unwrap_or("N/A"),
                        exif_data.iso.as_deref().unwrap_or("N/A"),
                        exif_data.focal_length.as_deref().unwrap_or("N/A")
                    );
                }
                self.state.file_type_text = format!("Image ({:?} - {}x{})", format, width, height);
            }
            ParsedContent::Pdf { page_count, .. } => {
                self.state.pdf = crate::ui::types::PdfState::default();
                self.state.pdf.page_count = *page_count as usize;
                self.state.file_type_text = "PDF Document".to_string();
            }
            ParsedContent::Archive { entries, total_files } => {
                self.state.table.rows = entries.iter().map(|entry| {
                    crate::ui::types::TableRowState {
                        name: entry.path.clone(),
                        kind: if entry.is_dir { "Directory".to_string() } else { "File".to_string() },
                        size: crate::parser::human_size(entry.size),
                        modified: entry.modified.clone(),
                        path: entry.path.clone(),
                        is_dir: entry.is_dir,
                    }
                }).collect();
                self.state.file_type_text = format!("Archive ({} files)", total_files);
            }
            ParsedContent::Folder { entries } => {
                self.state.table.rows = entries.iter().map(|entry| {
                    crate::ui::types::TableRowState {
                        name: entry.name.clone(),
                        kind: if entry.is_dir { "Directory".to_string() } else { "File".to_string() },
                        size: crate::parser::human_size(entry.size),
                        modified: entry.modified.clone(),
                        path: entry.name.clone(),
                        is_dir: entry.is_dir,
                    }
                }).collect();
                self.state.file_type_text = "Folder".to_string();
            }
            ParsedContent::Markdown { content: md_content, .. } => {
                self.state.text.content = md_content.clone();
                self.state.file_type_text = "Markdown Document".to_string();
            }
            ParsedContent::Video { duration, .. } => {
                self.state.media = crate::ui::types::MediaState::default();
                self.state.media.metadata = format!("Video Duration: {:.2}s", duration);
                self.state.file_type_text = "Video File".to_string();
            }
            ParsedContent::Audio { metadata, .. } => {
                self.state.media = crate::ui::types::MediaState::default();
                self.state.media.metadata = metadata.clone();
                self.state.file_type_text = "Audio File".to_string();
            }
            ParsedContent::Office { content: office_content, format, page_count } => {
                self.state.text.content = office_content.clone();
                self.state.file_type_text = format!("Office ({:?}, {} pages)", format, page_count);
            }
            ParsedContent::Font { name, metadata, .. } => {
                self.state.text.content = format!("Font: {}\n\n{}", name, metadata);
                self.state.file_type_text = "Font File".to_string();
            }
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
            Message::OpenClicked => {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&self.state.file_name)
                    .spawn();
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
            Message::CopyPathClicked => {
                iced::clipboard::write(self.state.file_name.clone())
            }
            Message::FileLoaded { path, content } => {
                self.state.file_name = path;
                self.state.content_ready = true;
                
                self.populate_state_from_content(content.clone());
                self.current_content = Some(content);
                
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
            Message::ToggleExifSidebar => {
                self.state.image.show_exif = !self.state.image.show_exif;
                Task::none()
            }
            Message::ImageZoomIn => {
                self.state.image.zoom += 0.1;
                Task::none()
            }
            Message::ImageZoomOut => {
                self.state.image.zoom = (self.state.image.zoom - 0.1).max(0.1);
                Task::none()
            }
            Message::ImageRotateLeft => {
                self.state.image.rotation = (self.state.image.rotation - 90) % 360;
                Task::none()
            }
            Message::ImageRotateRight => {
                self.state.image.rotation = (self.state.image.rotation + 90) % 360;
                Task::none()
            }
            Message::ImageReset => {
                self.state.image = crate::ui::types::ImageState::default();
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = container(
            row![
                column![
                    text(&self.state.file_name).size(20),
                    text(&self.state.file_type_text).size(12).style(|theme: &iced::Theme| {
                        let palette = theme.extended_palette();
                        text::Style {
                            color: Some(palette.background.weak.text),
                        }
                    })
                ].width(Length::Fill),
                button(text("Copy Path")).on_press(Message::CopyPathClicked),
                button(text("Open Externally")).on_press(Message::OpenClicked),
                button(text("Close")).on_press(Message::CloseRequested),
            ]
            .spacing(15)
        )
        .padding(15)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                ..Default::default()
            }
        });

        let preview_body: Element<'_, Message> = if let Some(content) = &self.current_content {
            match &**content {
                ParsedContent::Text { .. } | ParsedContent::Markdown { .. } | ParsedContent::Office { .. } | ParsedContent::Font { .. } => {
                    crate::ui::components::view_text(&self.state.text)
                }
                ParsedContent::Image { data, .. } => {
                    crate::ui::components::view_image(&self.state.image, data, false)
                }
                ParsedContent::Pdf { first_page, .. } => {
                    crate::ui::components::view_pdf(&self.state.pdf, &first_page.data, first_page.width, first_page.height)
                }
                ParsedContent::Archive { .. } | ParsedContent::Folder { .. } => {
                    crate::ui::components::view_table(&self.state.table)
                }
                ParsedContent::Video { thumbnail, .. } => {
                    crate::ui::components::view_media(&self.state.media, thumbnail, 320, 240)
                }
                ParsedContent::Audio { waveform, waveform_width, waveform_height, .. } => {
                    crate::ui::components::view_media(&self.state.media, waveform, *waveform_width, *waveform_height)
                }
            }
        } else {
            text("No file loaded.").size(18).into()
        };

        let full_layout = column![
            header,
            container(preview_body)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        container(full_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
