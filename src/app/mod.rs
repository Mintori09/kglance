mod keyboard;
mod media;
mod window;

use iced::{Element, Subscription, Task, Theme};
use iced_futures::subscription;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::core::{FilePreviewer, KglanceState, PreviewData};
use crate::dbus::DaemonCommand;
use crate::log_debug;
use crate::parsers::ParserRegistry;
use crate::parsers::markdown::Block;

#[derive(Debug, Clone)]
pub enum Message {
    // Basic actions
    OpenClicked,
    CopyPathClicked,
    BackClicked,
    CloseRequested,
    FileClicked(usize),

    // Navigation
    PrevFileClicked,
    NextFileClicked,
    HistoryBack,
    HistoryForward,

    // Image viewer
    ImageZoom(f32),
    ScrollDelta {
        x: f32,
        y: f32,
    },
    CtrlHeldChanged(bool),
    ShiftHeldChanged(bool),
    ModifiersUpdated(iced::keyboard::Modifiers),

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
    MediaMouseEnter,
    MediaMouseLeave,

    // System
    ThemeToggled,
    /// Open the preview window immediately (loading state) before content is ready.
    DaemonOpenWindow {
        path: String,
    },
    FileLoaded {
        path: String,
        content: PreviewData,
    },
    WindowEvent(iced::window::Id, iced::window::Event),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    PdfPagesLoaded(Vec<Option<(Vec<u8>, u32, u32)>>),
    PdfPageReady(usize, Vec<u8>, u32, u32),
    PdfScrolled(iced::widget::scrollable::Viewport),

    // Spreadsheet
    SheetTabClicked(usize),
    SpreadsheetColumnClicked(usize),

    // Async Mermaid rendering
    MermaidBlockRendered {
        index: usize,
        png_bytes: Option<Vec<u8>>,
    },
    // Async inline image loading
    MarkdownImageLoaded {
        index: usize,
        png_bytes: Option<Vec<u8>>,
    },
}

fn png_to_rgba_handle(png: Vec<u8>) -> Option<iced::widget::image::Handle> {
    match image::load_from_memory(&png) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(iced::widget::image::Handle::from_rgba(
                width,
                height,
                rgba.into_raw(),
            ))
        }
        Err(_) => {
            log_debug!("image::load_from_memory failed, falling back to Handle::from_bytes");
            Some(iced::widget::image::Handle::from_bytes(png))
        }
    }
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
    pub ctrl_held: bool,
    pub shift_held: bool,
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
            ctrl_held: false,
            shift_held: false,
        };

        // Daemon starts with no window — window is opened on demand via DaemonOpenWindow.
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

    pub fn handle_file_loaded(&mut self, path: String, content: PreviewData) -> Task<Message> {
        self.state.file_name = path.clone();
        self.state.content_ready = true;

        let mermaid_tasks: Vec<Task<Message>> = {
            let mut tasks = Vec::new();
            if let PreviewData::Markdown { ref blocks } = content {
                for (i, block) in blocks.iter().enumerate() {
                    match block {
                        Block::Mermaid {
                            lines,
                            rendered: None,
                        } => {
                            log_debug!("Spawning async render for Mermaid block[{}]", i);
                            let code = lines.join("\n");
                            tasks.push(Task::perform(
                                async move {
                                    let png = tokio::task::spawn_blocking(move || {
                                        crate::parsers::markdown::render_mermaid_to_png(&code)
                                    })
                                    .await
                                    .ok()
                                    .flatten();
                                    Message::MermaidBlockRendered {
                                        index: i,
                                        png_bytes: png,
                                    }
                                },
                                |msg| msg,
                            ));
                        }
                        Block::Image { path, .. } => {
                            log_debug!("Spawning async load for Image block[{}]: {}", i, path);
                            let path_str = path.clone();
                            let resolved = if Path::new(&path_str).is_absolute() {
                                std::path::PathBuf::from(&path_str)
                            } else {
                                Path::new(&self.state.file_name)
                                    .parent()
                                    .unwrap_or(Path::new("."))
                                    .join(&path_str)
                            };
                            tasks.push(Task::perform(
                                async move {
                                    let bytes = tokio::task::spawn_blocking(move || {
                                        std::fs::read(&resolved).ok()
                                    })
                                    .await
                                    .ok()
                                    .flatten();
                                    Message::MarkdownImageLoaded {
                                        index: i,
                                        png_bytes: bytes,
                                    }
                                },
                                |msg| msg,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            log_debug!("Spawning {} mermaid render tasks", tasks.len());
            tasks
        };

        content.populate_state(&mut self.state);
        log_debug!(
            "After populate_state: {} handles cached",
            self.state.markdown.cached_mermaid_handles.len()
        );
        self.current_content = Some(content.clone());

        let is_pdf = matches!(content, PreviewData::Pdf { .. });

        if let PreviewData::Pdf {
            ref data,
            width,
            height,
            ..
        } = content
            && !data.is_empty()
            && !self.state.pdf.pages.is_empty()
        {
            let handle = iced::widget::image::Handle::from_rgba(width, height, data.clone());
            self.state.pdf.pages[0] = Some((data.clone(), width, height));
            self.state.pdf.cached_handles[0] = Some(handle);
        }

        let pdf_task: Option<Task<Message>> = if is_pdf && self.state.pdf.page_count > 1 {
            self.state.pdf.loading = true;
            let page_count = self.state.pdf.page_count;
            let pdf_path = path.clone();
            let visible_page = self.state.pdf.visible_page.clone();
            Some(crate::core::handlers::pdf::lazy_load_pages(
                pdf_path,
                page_count,
                visible_page,
            ))
        } else {
            None
        };

        let path_lower = path.to_lowercase();
        let is_video = path_lower.ends_with(".mp4")
            || path_lower.ends_with(".mkv")
            || path_lower.ends_with(".avi")
            || path_lower.ends_with(".mov")
            || path_lower.ends_with(".wmv")
            || path_lower.ends_with(".webm");

        let is_audio = path_lower.ends_with(".mp3")
            || path_lower.ends_with(".wav")
            || path_lower.ends_with(".flac")
            || path_lower.ends_with(".ogg")
            || path_lower.ends_with(".aac")
            || path_lower.ends_with(".m4a")
            || path_lower.ends_with(".opus");

        self.state.media.has_video = is_video;

        if let Some(tx) = &self.video_tx {
            if is_video || is_audio {
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Load(path));
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Play);
            } else {
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Stop);
            }
        }

        let mut window_tasks: Vec<Task<Message>> = if self.is_daemon {
            if let Some(id) = self.window_id {
                vec![
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
                    iced::window::gain_focus(id),
                ]
            } else {
                vec![]
            }
        } else if let Some(id) = self.window_id {
            vec![iced::window::gain_focus(id)]
        } else {
            vec![]
        };

        if let PreviewData::Image { width, height, .. } = &content
            && let Some(id) = self.window_id
        {
            let size = crate::ui::handlers::image::calculate_window_size(*width, *height);
            window_tasks.push(iced::window::resize(id, size));
        }

        Task::batch(
            mermaid_tasks
                .into_iter()
                .chain(pdf_task)
                .chain(window_tasks),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileClicked(idx) => {
                if idx < self.state.table.rows.len() {
                    self.state.table.selected_index = Some(idx);
                }
                Task::none()
            }
            Message::CloseRequested => self.handle_close(),
            Message::OpenClicked => self.handle_open_clicked(),
            Message::CopyPathClicked => self.handle_copy_path(),
            Message::DaemonOpenWindow { path } => self.handle_daemon_open_window(path),
            Message::FileLoaded { path, content } => self.handle_file_loaded(path, content),
            Message::WindowEvent(id, event) => self.handle_window_event(id, event),
            Message::ImageZoom(delta) => self.handle_image_zoom(delta),
            Message::CtrlHeldChanged(held) => self.handle_ctrl_changed(held),
            Message::ShiftHeldChanged(held) => self.handle_shift_changed(held),
            Message::ModifiersUpdated(modifiers) => self.handle_modifiers_changed(modifiers),
            Message::ScrollDelta { x, y } => self.handle_scroll_delta(x, y),
            Message::PdfScrolled(viewport) => {
                let y = viewport.absolute_offset().y;
                let content_h = viewport.content_bounds().height;
                let count = self.state.pdf.page_count;
                if count > 0 && content_h > 0.0 {
                    let mut page_index = ((y / content_h) * count as f32) as usize;
                    if page_index >= count {
                        page_index = count - 1;
                    }
                    self.state
                        .pdf
                        .visible_page
                        .store(page_index, std::sync::atomic::Ordering::Relaxed);
                }
                Task::none()
            }
            Message::PdfPagesLoaded(_) => {
                self.state.pdf.loading = false;
                Task::none()
            }
            Message::PdfPageReady(index, data, width, height) => {
                if index < self.state.pdf.pages.len() {
                    let handle =
                        iced::widget::image::Handle::from_rgba(width, height, data.clone());
                    self.state.pdf.pages[index] = Some((data, width, height));
                    self.state.pdf.cached_handles[index] = Some(handle);
                }
                let all_loaded = self.state.pdf.pages.iter().all(|p| p.is_some());
                if all_loaded {
                    self.state.pdf.loading = false;
                }
                Task::none()
            }
            Message::MermaidBlockRendered { index, png_bytes } => {
                log_debug!(
                    "MermaidBlockRendered[{}] png={}",
                    index,
                    if png_bytes.is_some() { "Some" } else { "None" }
                );
                if let Some(PreviewData::Markdown { blocks }) = self.current_content.as_mut()
                    && let Some(Block::Mermaid { rendered, .. }) = blocks.get_mut(index)
                {
                    *rendered = png_bytes.clone();
                }
                if let Some(png) = png_bytes {
                    if let Some(handle) = png_to_rgba_handle(png) {
                        self.state
                            .markdown
                            .cached_mermaid_handles
                            .insert(index, handle);
                        log_debug!(
                            "Inserted handle at index {}, cache size={}",
                            index,
                            self.state.markdown.cached_mermaid_handles.len()
                        );
                    } else {
                        log_debug!("png_to_rgba_handle returned None for block[{}]", index);
                    }
                }
                Task::none()
            }
            Message::MarkdownImageLoaded { index, png_bytes } => {
                log_debug!(
                    "MarkdownImageLoaded[{}] bytes={}",
                    index,
                    if png_bytes.is_some() { "Some" } else { "None" }
                );
                if let Some(bytes) = png_bytes {
                    if let Some(handle) = png_to_rgba_handle(bytes) {
                        self.state
                            .markdown
                            .cached_image_handles
                            .insert(index, handle);
                        log_debug!(
                            "Inserted image handle at index {}, cache size={}",
                            index,
                            self.state.markdown.cached_image_handles.len()
                        );
                    } else {
                        log_debug!(
                            "png_to_rgba_handle returned None for image block[{}]",
                            index
                        );
                    }
                }
                Task::none()
            }
            Message::PlayPauseClicked => self.handle_play_pause(),
            Message::SeekClicked(percent) => self.handle_seek(percent),
            Message::SeekRelativeClicked(secs) => self.handle_seek_relative(secs),
            Message::VideoEventReceived(event) => self.handle_video_event(event),
            Message::MediaMouseEnter => self.handle_media_mouse_enter(),
            Message::MediaMouseLeave => self.handle_media_mouse_leave(),
            Message::KeyPressed(key, modifiers) => self.handle_key_pressed(key, modifiers),
            Message::SheetTabClicked(index) => {
                if index < self.state.spreadsheet.sheets.len() {
                    self.state.spreadsheet.active_sheet = index;
                    self.state.spreadsheet.sort_col = None;
                    self.state.spreadsheet.sort_ascending = None;
                }
                Task::none()
            }
            Message::SpreadsheetColumnClicked(col) => {
                let sort = &mut self.state.spreadsheet;
                if sort.sort_col == Some(col) {
                    sort.sort_ascending = match sort.sort_ascending {
                        None => Some(true),
                        Some(true) => Some(false),
                        Some(false) => None,
                    };
                    if sort.sort_ascending.is_none() {
                        sort.sort_col = None;
                    }
                } else {
                    sort.sort_col = Some(col);
                    sort.sort_ascending = Some(true);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let preview_body: Element<'_, Message> = if let Some(content) = &self.current_content {
            match content {
                PreviewData::Text { .. } => {
                    crate::ui::views::view_text(&self.state.text, self.state.theme_dark)
                }
                PreviewData::Markdown { blocks } => {
                    crate::ui::views::view_markdown(blocks, &self.state.markdown)
                }
                PreviewData::Image { .. } => crate::ui::views::view_image(&self.state.image),
                PreviewData::Pdf { .. } => crate::ui::views::view_pdf(&self.state.pdf),
                PreviewData::Folder { .. } => {
                    crate::ui::views::view_table(&self.state.table, self.state.theme_dark)
                }
                PreviewData::Spreadsheet { .. } => {
                    crate::ui::views::view_spreadsheet(&self.state.spreadsheet)
                }
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

        let is_media = matches!(self.current_content, Some(PreviewData::Media { .. }));
        crate::ui::window::view_window(&self.state, preview_body, is_media)
    }

    /// Variant for [`iced::daemon`] which requires a `window::Id` parameter.
    pub fn view_daemon(&self, _window_id: iced::window::Id) -> Element<'_, Message> {
        self.view()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let dbus_sub = subscription::from_recipe(crate::dbus::recipe::DaemonRecipe::new(
            self.daemon_rx.lock().unwrap().take(),
        ));

        let video_sub = subscription::from_recipe(crate::ui::handlers::video::VideoRecipe::new(
            self.video_rx.lock().unwrap().take(),
        ));

        let event_sub = iced::window::events().map(|(id, event)| Message::WindowEvent(id, event));

        let global_event_sub = iced::event::listen_with(|event, _status, _window_id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyPressed(key, modifiers))
            }
            iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::ModifiersUpdated(modifiers))
            }
            iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                let (dx, dy) = match delta {
                    iced::mouse::ScrollDelta::Lines { x, y } => (x * 40.0, y * 40.0),
                    iced::mouse::ScrollDelta::Pixels { x, y } => (x, y),
                };
                Some(Message::ScrollDelta { x: dx, y: dy })
            }
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

    /// Variant for [`iced::daemon`] which requires a `window::Id` parameter.
    pub fn theme_daemon(&self, _window_id: iced::window::Id) -> Theme {
        self.theme()
    }
}
