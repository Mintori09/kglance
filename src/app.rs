use iced::widget::operation::{self, AbsoluteOffset};
use iced::{Element, Subscription, Task, Theme};
use iced_futures::subscription::{self, Recipe};
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::core::{FilePreviewer, KglanceState, PreviewData};
use crate::dbus::DaemonCommand;
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

    fn populate_state_from_preview(&mut self, content: PreviewData) {
        match content {
            PreviewData::Text {
                content: text_content,
                line_numbers,
                language,
            } => {
                self.state.text.content =
                    iced::widget::text_editor::Content::with_text(&text_content);
                self.state.text.extension = language.clone();
                self.state.text.line_numbers = line_numbers;
                self.state.file_type_text = language;
            }
            PreviewData::Image {
                data,
                width,
                height,
                format_info,
                exif_content,
            } => {
                self.state.image = crate::core::ImageState {
                    handle: Some(iced::widget::image::Handle::from_bytes(data.clone())),
                    image_bytes: data,
                    image_width: width,
                    image_height: height,
                    exif_content: exif_content.unwrap_or_default(),
                    ..Default::default()
                };
                self.state.file_type_text = format_info;
            }
            PreviewData::Pdf { page_count, .. } => {
                self.state.pdf = crate::core::PdfState::default();
                self.state.pdf.page_count = page_count;
                self.state.pdf.pages = vec![None; page_count];
                self.state.pdf.cached_handles = vec![None; page_count];
                self.state.file_type_text = "PDF Document".to_string();
            }
            PreviewData::Folder { rows, total_size } => {
                self.state.table.rows = rows;
                self.state.table.total_size = total_size;
                self.state.table.folder_path = self.state.file_name.clone();
                self.state.table.selected_index = None;
                self.state.file_type_text = "Folder / Archive".to_string();
            }
            PreviewData::Markdown { blocks } => {
                self.state.markdown = crate::core::types::MarkdownState::default();
                self.state.markdown.cached_mermaid_handles = blocks
                    .iter()
                    .map(|block| {
                        if let Block::Mermaid {
                            rendered: Some(png),
                            ..
                        } = block
                        {
                            Some(iced::widget::image::Handle::from_bytes(png.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                self.state.file_type_text = "Markdown Document".to_string();
            }
            PreviewData::Spreadsheet {
                sheets,
                active_sheet,
            } => {
                self.state.spreadsheet.sheets = sheets;
                self.state.spreadsheet.active_sheet = active_sheet;
                self.state.file_type_text = "Spreadsheet".to_string();
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
            Message::FileClicked(idx) => {
                if idx < self.state.table.rows.len() {
                    self.state.table.selected_index = Some(idx);
                }
                Task::none()
            }
            Message::CloseRequested => {
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
            Message::OpenClicked => {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&self.state.file_name)
                    .spawn();
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
            Message::CopyPathClicked => iced::clipboard::write(self.state.file_name.clone()),
            Message::DaemonOpenWindow { path } => {
                // Window open signal arrived before content — show loading state immediately.
                self.state.file_name = path;
                self.state.content_ready = false;
                self.current_content = None;

                if let Some(id) = self.window_id {
                    // Window already exists from a previous preview — un-hide and focus.
                    Task::batch(vec![
                        iced::window::set_mode(id, iced::window::Mode::Windowed),
                        iced::window::gain_focus(id),
                    ])
                } else {
                    // First open — create the window now (overlaps with file parsing).
                    let settings = iced::window::Settings {
                        size: iced::Size::new(1024.0, 768.0),
                        min_size: Some(iced::Size::new(800.0, 600.0)),
                        exit_on_close_request: false,
                        decorations: false,
                        ..Default::default()
                    };
                    let (id, open_task) = iced::window::open(settings);
                    // Store ID immediately so subsequent messages can reference the window.
                    // SAFETY: id is pre-allocated by Iced before the window is mapped.
                    // We shadow self.window_id here via a returned task side-effect;
                    // the Opened event from event_sub will also call self.window_id = Some(id).
                    let _ = id; // stored via WindowEvent::Opened handler
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
            Message::FileLoaded { path, content } => {
                self.state.file_name = path.clone();
                self.state.content_ready = true;

                // Collect async Mermaid rendering tasks before content is consumed
                let mermaid_tasks: Vec<Task<Message>> = {
                    let mut tasks = Vec::new();
                    if let PreviewData::Markdown { ref blocks } = content {
                        for (i, block) in blocks.iter().enumerate() {
                            if let Block::Mermaid {
                                lines,
                                rendered: None,
                            } = block
                            {
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
                        }
                    }
                    tasks
                };

                self.populate_state_from_preview(content.clone());
                self.current_content = Some(content.clone());

                let is_pdf = matches!(content, PreviewData::Pdf { .. });

                // Populate page 0 from initial data and build its handle immediately
                if let PreviewData::Pdf {
                    ref data,
                    width,
                    height,
                    ..
                } = content
                    && !data.is_empty()
                    && !self.state.pdf.pages.is_empty()
                {
                    let handle =
                        iced::widget::image::Handle::from_rgba(width, height, data.clone());
                    self.state.pdf.pages[0] = Some((data.clone(), width, height));
                    self.state.pdf.cached_handles[0] = Some(handle);
                }

                // Lazy load PDF pages one-by-one in background tasks
                let pdf_task: Option<Task<Message>> = if is_pdf && self.state.pdf.page_count > 1 {
                    self.state.pdf.loading = true;
                    let page_count = self.state.pdf.page_count;
                    let path = path.clone();
                    let visible_page = self.state.pdf.visible_page.clone();
                    let stream = iced::stream::channel(
                        10,
                        move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                            use iced::futures::SinkExt;
                            let mut rendered = vec![false; page_count];
                            if page_count > 0 {
                                rendered[0] = true;
                            } // index 0 unused, 1-indexed

                            loop {
                                let curr = visible_page.load(std::sync::atomic::Ordering::Relaxed);

                                // Find unrendered pages
                                let mut unrendered: Vec<usize> =
                                    (1..page_count).filter(|&i| !rendered[i]).collect();
                                if unrendered.is_empty() {
                                    break;
                                }

                                // Sort by distance to current view
                                unrendered.sort_by_key(|&i| (i as isize - curr as isize).abs());

                                // Take up to 4 pages to render concurrently
                                let to_render: Vec<usize> =
                                    unrendered.into_iter().take(4).collect();
                                let mut tasks = Vec::new();

                                for &i in &to_render {
                                    rendered[i] = true;
                                    let p = path.clone();
                                    tasks.push(tokio::task::spawn_blocking(move || {
                                        let p_path = std::path::Path::new(&p);
                                        crate::parsers::pdf::render_pdf_page(p_path, i as u32).ok()
                                    }));
                                }

                                let results = iced::futures::future::join_all(tasks).await;
                                for (idx, res) in to_render.iter().zip(results) {
                                    if let Some(d) = res.ok().flatten() {
                                        let _ = output
                                            .send(Message::PdfPageReady(
                                                *idx, d.data, d.width, d.height,
                                            ))
                                            .await;
                                    }
                                }
                            }
                            let _ = output.send(Message::PdfPagesLoaded(vec![])).await;
                        },
                    );
                    Some(Task::run(stream, |msg| msg))
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
                        let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Load(
                            path.clone(),
                        ));
                        let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Play);
                    } else {
                        let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Stop);
                    }
                }

                // Show window and focus.
                // DaemonOpenWindow already opened the window; here we just ensure it's visible
                // and focused when content arrives (window may have been closed between events).
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
            Message::WindowEvent(id, event) => {
                match event {
                    iced::window::Event::Opened { .. } => {
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
            Message::ImageZoom(delta) => {
                self.state.image.zoom = (self.state.image.zoom + delta).clamp(0.1, 10.0);
                Task::none()
            }
            Message::CtrlHeldChanged(held) => {
                self.ctrl_held = held;
                Task::none()
            }
            Message::ShiftHeldChanged(held) => {
                self.shift_held = held;
                Task::none()
            }
            Message::ModifiersUpdated(modifiers) => {
                self.ctrl_held = modifiers.control();
                self.shift_held = modifiers.shift();
                Task::none()
            }
            Message::ScrollDelta { x: _x, y } => {
                use iced::widget::operation;
                use iced::widget::operation::AbsoluteOffset;
                if self.ctrl_held {
                    // Ctrl+scroll: zoom image
                    if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                        let factor = if y > 0.0 { 0.1 } else { -0.1 };
                        self.state.image.zoom = (self.state.image.zoom + factor).clamp(0.1, 10.0);
                    }
                    Task::none()
                } else if self.shift_held {
                    // Shift+scroll: horizontal scroll
                    operation::scroll_by("content_scroll", AbsoluteOffset { x: -y, y: 0.0 })
                } else {
                    // Normal scroll: vertical
                    operation::scroll_by("content_scroll", AbsoluteOffset { x: 0.0, y: -y })
                }
            }
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
                // Batch load fallback — unused in lazy path but kept for compatibility
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
                // Clear loading flag when last page arrives
                let all_loaded = self.state.pdf.pages.iter().all(|p| p.is_some());
                if all_loaded {
                    self.state.pdf.loading = false;
                }
                Task::none()
            }
            Message::MermaidBlockRendered { index, png_bytes } => {
                if let Some(PreviewData::Markdown { blocks }) = self.current_content.as_mut()
                    && let Some(Block::Mermaid { rendered, .. }) = blocks.get_mut(index)
                {
                    *rendered = png_bytes;
                }
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
                        let first_frame = self.state.media.frame_data.is_empty();
                        self.state.media.frame_data = data;
                        self.state.media.frame_width = width;
                        self.state.media.frame_height = height;
                        if first_frame && let Some(id) = self.window_id {
                            let w = (width as f32 * 1.1) as u32;
                            let h = (height as f32 * 1.1 + 50.0) as u32;
                            let max_w = 1600u32;
                            let max_h = 1000u32;
                            let cw = w.min(max_w).max(400);
                            let ch = h.min(max_h).max(300);
                            return iced::window::resize(id, iced::Size::new(cw as f32, ch as f32));
                        }
                    }
                    crate::ui::handlers::video::VideoEvent::Progress {
                        position,
                        duration,
                        is_playing,
                    } => {
                        self.state.media.playing = is_playing;
                        self.state.media.position_secs = position;
                        self.state.media.duration_secs = duration;
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
            Message::MediaMouseEnter => {
                self.state.media.show_controls = true;
                Task::none()
            }
            Message::MediaMouseLeave => {
                self.state.media.show_controls = false;
                Task::none()
            }
            Message::KeyPressed(key, modifiers) => {
                use iced::keyboard::key::Named;
                let ctrl = modifiers.control();

                // Handle folder preview specific keyboard selection
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

                // Ctrl+C: copy file path
                if ctrl {
                    if let iced::keyboard::Key::Character(ref c) = key {
                        match c.as_str() {
                            "c" => return iced::clipboard::write(self.state.file_name.clone()),
                            "+" | "=" => {
                                if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                                    self.state.image.zoom =
                                        (self.state.image.zoom + 0.2).clamp(0.1, 10.0);
                                    return Task::none();
                                }
                            }
                            "-" => {
                                if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                                    self.state.image.zoom =
                                        (self.state.image.zoom - 0.2).clamp(0.1, 10.0);
                                    return Task::none();
                                }
                            }
                            _ => {}
                        }
                    }
                    // Ctrl+0: reset zoom
                    if let iced::keyboard::Key::Character(ref c) = key
                        && c == "0"
                        && matches!(self.current_content, Some(PreviewData::Image { .. }))
                    {
                        self.state.image.zoom = 1.0;
                        return Task::none();
                    }
                }

                let scroll_amount = 80.0;
                let half_page_scroll_amout = 600.0;
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
                            y: -half_page_scroll_amout,
                        },
                    )),
                    iced::keyboard::Key::Named(Named::PageDown) => Some(operation::scroll_by(
                        "content_scroll",
                        AbsoluteOffset {
                            x: 0.0,
                            y: -half_page_scroll_amout,
                        },
                    )),

                    iced::keyboard::Key::Character(c) if c == "d" => Some(operation::scroll_by(
                        "content_scroll",
                        AbsoluteOffset {
                            x: 0.0,
                            y: -half_page_scroll_amout,
                        },
                    )),
                    iced::keyboard::Key::Character(c) if c == "u" => Some(operation::scroll_by(
                        "content_scroll",
                        AbsoluteOffset {
                            x: 0.0,
                            y: half_page_scroll_amout,
                        },
                    )),
                    _ => None,
                };
                if let Some(task) = scroll_task {
                    return task;
                }

                let should_close = match &key {
                    iced::keyboard::Key::Named(Named::Escape | Named::Backspace | Named::Space) => {
                        true
                    }
                    iced::keyboard::Key::Character(c)
                        if c == " " || c == "\u{8}" || c == "\u{7f}" =>
                    {
                        true
                    }
                    _ => false,
                };

                if should_close {
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
                } else {
                    Task::none()
                }
            }
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
        let dbus_sub = subscription::from_recipe(DaemonRecipe {
            rx: self.daemon_rx.lock().unwrap().take(),
        });

        let video_sub = subscription::from_recipe(VideoRecipe {
            rx: self.video_rx.lock().unwrap().take(),
        });

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

struct DaemonRecipe {
    rx: Option<mpsc::Receiver<DaemonCommand>>,
}

impl Recipe for DaemonRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "dbus_subscription".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx = self.rx;
        match rx {
            Some(mut rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    while let Some(cmd) = rx.recv().await {
                        match cmd {
                            DaemonCommand::OpenWindow { path } => {
                                let _ = output.send(Message::DaemonOpenWindow { path }).await;
                            }
                            DaemonCommand::ShowPreview { path, content } => {
                                let _ = output.send(Message::FileLoaded { path, content }).await;
                            }
                            DaemonCommand::HidePreview => {
                                let _ = output.send(Message::CloseRequested).await;
                            }
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}

struct VideoRecipe {
    rx: Option<mpsc::Receiver<crate::ui::handlers::video::VideoEvent>>,
}

impl Recipe for VideoRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "video_subscription".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx = self.rx;
        match rx {
            Some(mut rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    while let Some(event) = rx.recv().await {
                        let _ = output.send(Message::VideoEventReceived(event)).await;
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
