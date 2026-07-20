use iced::{Element, Subscription, Task, Theme};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::core::{FilePreviewer, KglanceState, PreviewData};
use crate::dbus::DaemonCommand;
use crate::parsers::ParserRegistry;

#[derive(Debug, Clone)]
pub enum Message {
    // Basic actions
    OpenClicked,
    CopyPathClicked,
    BackClicked,
    CloseRequested,
    FileClicked(usize),

    // Navigation
    PrevPageClicked,
    NextPageClicked,
    PrevFileClicked,
    NextFileClicked,
    HistoryBack,
    HistoryForward,

    // Image viewer
    ImageZoomIn,
    ImageZoomOut,
    ImageRotateLeft,
    ImageRotateRight,
    ImageReset,
    ImageDragMoved { dx: f32, dy: f32 },
    ImageDragStarted { x: f32, y: f32 },
    ImageDragFinished,
    ImageScrollZoom(f32),
    ToggleExifSidebar,

    // Text search & wrap
    SearchQueryChanged(String),
    TextSearchNext,
    TextSearchPrev,
    TextSearchClosed,
    TextWrapToggled,
    CopyContentClicked,

    // Table sorting
    SortByFieldClicked(crate::core::SortField),

    // Media
    PlayPauseClicked,
    SeekClicked(f32),
    SeekRelativeClicked(f32),
    VideoEventReceived(crate::ui::handlers::video::VideoEvent),

    // Color Picker
    ToggleColorPicker,
    ImageHovered { x: f32, y: f32 },
    ImageClicked { x: f32, y: f32 },

    // System
    ThemeToggled,
    FileLoaded { path: String, content: PreviewData },
    WindowEvent(iced::window::Id, iced::window::Event),
    KeyPressed(iced::keyboard::Key),
    PdfPageRendered(crate::parsers::PageData),
}

pub struct KglanceApp {
    pub state: KglanceState,
    pub registry: Arc<ParserRegistry>,
    pub daemon_rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>,
    pub is_daemon: bool,
    pub window_id: Option<iced::window::Id>,
    pub current_content: Option<PreviewData>,
    pub video_tx: Option<tokio::sync::mpsc::Sender<crate::ui::handlers::video::PlayerCommand>>,
    pub video_rx:
        Arc<Mutex<Option<tokio::sync::mpsc::Receiver<crate::ui::handlers::video::VideoEvent>>>>,
}

impl KglanceApp {
    pub fn new(
        registry: Arc<ParserRegistry>,
        daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
        initial_path: Option<&str>,
        is_daemon: bool,
    ) -> (Self, Task<Message>) {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
        crate::ui::handlers::video::spawn_video_player(cmd_rx, event_tx);

        let app = Self {
            state: KglanceState::default(),
            registry,
            daemon_rx: Arc::new(Mutex::new(daemon_rx)),
            is_daemon,
            window_id: None,
            current_content: None,
            video_tx: Some(cmd_tx),
            video_rx: Arc::new(Mutex::new(Some(event_rx))),
        };

        let task = if let Some(path) = initial_path {
            let path_str = path.to_string();
            let reg = app.registry.clone();
            Task::perform(
                async move {
                    let content = FilePreviewer::parse(&*reg, Path::new(&path_str)).ok()?;
                    Some(Message::FileLoaded {
                        path: path_str,
                        content,
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

    fn populate_state_from_preview(&mut self, content: PreviewData) {
        match content {
            PreviewData::Text {
                content: text_content,
                line_numbers,
                language,
            } => {
                self.state.text.content = text_content;
                self.state.text.line_numbers = line_numbers;
                self.state.file_type_text = language;
            }
            PreviewData::Image {
                width: _,
                height: _,
                format_info,
                exif_content,
                ..
            } => {
                self.state.image = crate::core::ImageState::default();
                if let Some(exif_val) = exif_content {
                    self.state.image.exif_content = exif_val;
                }
                self.state.file_type_text = format_info;
            }
            PreviewData::Pdf { page_count, .. } => {
                self.state.pdf = crate::core::PdfState::default();
                self.state.pdf.page_count = page_count;
                self.state.file_type_text = "PDF Document".to_string();
            }
            PreviewData::Folder { rows } => {
                self.state.table.rows = rows;
                self.state.file_type_text = "Folder / Archive".to_string();
            }
            PreviewData::Markdown { .. } => {
                self.state.file_type_text = "Markdown Document".to_string();
            }
            PreviewData::Media { metadata, .. } => {
                self.state.media = crate::core::MediaState::default();
                self.state.media.metadata = metadata.clone();
                self.state.file_type_text = if metadata.contains("Video") {
                    "Video File"
                } else {
                    "Audio File"
                }
                .to_string();
            }
            PreviewData::Error(err) => {
                self.state.file_type_text = format!("Error: {}", err);
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
            Message::CopyPathClicked => iced::clipboard::write(self.state.file_name.clone()),
            Message::FileLoaded { path, content } => {
                self.state.file_name = path.clone();
                self.state.content_ready = true;

                self.populate_state_from_preview(content.clone());
                self.current_content = Some(content);

                let is_video = path.ends_with(".mp4")
                    || path.ends_with(".mkv")
                    || path.ends_with(".avi")
                    || path.ends_with(".mov")
                    || path.ends_with(".wmv")
                    || path.ends_with(".webm");

                if let (Some(tx), true) = (&self.video_tx, is_video) {
                    let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Load(path));
                }

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
            Message::PrevPageClicked => {
                if self.state.pdf.current_page > 0 {
                    self.state.pdf.current_page -= 1;
                    let path = self.state.file_name.clone();
                    let page = self.state.pdf.current_page as u32;
                    Task::perform(
                        async move {
                            crate::parsers::pdf::render_pdf_page(std::path::Path::new(&path), page)
                                .ok()
                        },
                        |opt| {
                            opt.map(Message::PdfPageRendered)
                                .unwrap_or(Message::CloseRequested)
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::NextPageClicked => {
                let next = self.state.pdf.current_page + 1;
                if next < self.state.pdf.page_count {
                    self.state.pdf.current_page = next;
                    let path = self.state.file_name.clone();
                    let page = next as u32;
                    Task::perform(
                        async move {
                            crate::parsers::pdf::render_pdf_page(std::path::Path::new(&path), page)
                                .ok()
                        },
                        |opt| {
                            opt.map(Message::PdfPageRendered)
                                .unwrap_or(Message::CloseRequested)
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::PdfPageRendered(page_data) => {
                self.current_content = Some(PreviewData::Pdf {
                    page_count: self.state.pdf.page_count,
                    current_page: self.state.pdf.current_page,
                    data: page_data.data,
                    width: page_data.width,
                    height: page_data.height,
                });
                Task::none()
            }
            Message::ImageReset => {
                self.state.image = crate::core::ImageState::default();
                Task::none()
            }
            Message::PlayPauseClicked => {
                if let Some(tx) = &self.video_tx {
                    if self.state.media.playing {
                        let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Pause);
                    } else {
                        let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Play);
                    }
                }
                Task::none()
            }
            Message::SeekClicked(percent) => {
                if let Some(tx) = &self.video_tx {
                    let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Seek(
                        percent as f64,
                    ));
                }
                Task::none()
            }
            Message::SeekRelativeClicked(secs) => {
                if let Some(tx) = &self.video_tx {
                    let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::SeekRelative(
                        secs as f64,
                    ));
                }
                Task::none()
            }
            Message::VideoEventReceived(event) => {
                match event {
                    crate::ui::handlers::video::VideoEvent::Frame {
                        data,
                        width,
                        height,
                    } => {
                        self.current_content = Some(PreviewData::Media {
                            url: self.state.file_name.clone(),
                            metadata: self.state.media.metadata.clone(),
                            thumbnail_or_waveform: data,
                            width,
                            height,
                        });
                    }
                    crate::ui::handlers::video::VideoEvent::Progress {
                        position,
                        duration,
                        is_playing,
                    } => {
                        self.state.media.playing = is_playing;
                        if duration > 0.0 {
                            self.state.media.progress = (position / duration) as f32;
                            let cur_mins = (position / 60.0) as u32;
                            let cur_secs = (position % 60.0) as u32;
                            let dur_mins = (duration / 60.0) as u32;
                            let dur_secs = (duration % 60.0) as u32;
                            self.state.media.time =
                                format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}");
                        }
                    }
                }
                Task::none()
            }
            Message::ToggleColorPicker => {
                self.state.image.picker_enabled = !self.state.image.picker_enabled;
                if !self.state.image.picker_enabled {
                    self.state.image.picked_color = None;
                    self.state.image.picked_color_hex = String::new();
                    self.state.image.cursor_color = None;
                    self.state.image.cursor_color_hex = String::new();
                }
                Task::none()
            }
            Message::ImageHovered { x, y } => {
                if !self.state.image.picker_enabled {
                    return Task::none();
                }
                let img_opt = self.current_content.as_ref().and_then(|content| {
                    if let PreviewData::Image { data, .. } = content {
                        image::load_from_memory(data).ok()
                    } else {
                        None
                    }
                });
                if let Some(img) = img_opt {
                    use image::GenericImageView;
                    let ix = (x * img.width() as f32) as u32;
                    let iy = (y * img.height() as f32) as u32;
                    if ix < img.width() && iy < img.height() {
                        let pixel = img.get_pixel(ix, iy);
                        let rgb = (pixel[0], pixel[1], pixel[2]);
                        self.state.image.cursor_color = Some(rgb);
                        self.state.image.cursor_color_hex =
                            format!("#{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2);
                    }
                }
                Task::none()
            }
            Message::ImageClicked { x, y } => {
                if !self.state.image.picker_enabled {
                    return Task::none();
                }
                let img_opt = self.current_content.as_ref().and_then(|content| {
                    if let PreviewData::Image { data, .. } = content {
                        image::load_from_memory(data).ok()
                    } else {
                        None
                    }
                });
                if let Some(img) = img_opt {
                    use image::GenericImageView;
                    let ix = (x * img.width() as f32) as u32;
                    let iy = (y * img.height() as f32) as u32;
                    if ix < img.width() && iy < img.height() {
                        let pixel = img.get_pixel(ix, iy);
                        let rgb = (pixel[0], pixel[1], pixel[2]);
                        self.state.image.picked_color = Some(rgb);
                        let hex = format!("#{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2);
                        self.state.image.picked_color_hex = hex.clone();
                        return iced::clipboard::write(hex);
                    }
                }
                Task::none()
            }
            Message::KeyPressed(key) => {
                use iced::keyboard::key::Named;

                let should_close = match key {
                    iced::keyboard::Key::Named(Named::Escape | Named::Backspace | Named::Space) => {
                        true
                    }
                    iced::keyboard::Key::Character(ref c) if c == " " => true,
                    _ => false,
                };

                if should_close {
                    if self.is_daemon {
                        if let Some(id) = self.window_id {
                            iced::window::change_mode(id, iced::window::Mode::Hidden)
                        } else {
                            Task::none()
                        }
                    } else {
                        iced::exit()
                    }
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let preview_body: Element<'_, Message> = if let Some(content) = &self.current_content {
            match content {
                PreviewData::Text { .. } => crate::ui::views::view_text(&self.state.text),
                PreviewData::Markdown { blocks } => crate::ui::views::view_markdown(blocks),
                PreviewData::Image { data, .. } => {
                    crate::ui::views::view_image(&self.state.image, data, false)
                }
                PreviewData::Pdf {
                    data,
                    width,
                    height,
                    ..
                } => crate::ui::views::view_pdf(&self.state.pdf, data, *width, *height),
                PreviewData::Folder { .. } => crate::ui::views::view_table(&self.state.table),
                PreviewData::Media {
                    thumbnail_or_waveform,
                    width,
                    height,
                    ..
                } => crate::ui::views::view_media(
                    &self.state.media,
                    thumbnail_or_waveform,
                    *width,
                    *height,
                ),
                PreviewData::Error(err) => iced::widget::text(err).size(18).into(),
            }
        } else {
            iced::widget::text("No file loaded.").size(18).into()
        };

        crate::ui::window::view_window(&self.state, preview_body)
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
                                let _ = output.send(Message::FileLoaded { path, content }).await;
                            }
                            DaemonCommand::HidePreview => {
                                let _ = output.send(Message::CloseRequested).await;
                            }
                        }
                    }
                }
            }),
        );

        let video_rx_opt = self.video_rx.clone();
        let video_sub = Subscription::run_with_id(
            "video_subscription",
            iced::stream::channel(100, move |mut output| async move {
                let rx = {
                    let mut guard = video_rx_opt.lock().unwrap();
                    guard.take()
                };
                if let Some(mut rx) = rx {
                    use iced::futures::SinkExt;
                    while let Some(event) = rx.recv().await {
                        let _ = output.send(Message::VideoEventReceived(event)).await;
                    }
                }
            }),
        );

        let event_sub = iced::window::events().map(|(id, event)| Message::WindowEvent(id, event));

        let global_event_sub = iced::event::listen_with(|event, _status, window_id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                Some(Message::KeyPressed(key))
            }
            iced::Event::Window(iced::window::Event::Opened { .. }) => Some(Message::WindowEvent(
                window_id,
                iced::window::Event::Opened {
                    position: None,
                    size: iced::Size::ZERO,
                },
            )),
            _ => None,
        });
        Subscription::batch(vec![dbus_sub, event_sub, video_sub, global_event_sub])
    }

    pub fn theme(&self) -> Theme {
        if self.state.theme_dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
