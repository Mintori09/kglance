mod keyboard;
mod media;

mod window;

use iced::widget::operation;
use iced::{Element, Subscription, Task, Theme};
use iced_futures::subscription;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::core::{FilePreviewer, KglanceState, PreviewData};
use crate::dbus::DaemonCommand;
use crate::parsers::ParserRegistry;
use crate::parsers::markdown::Block;
use crate::{log_debug, log_info};

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
    SiblingFilesLoaded(Vec<String>),
    ToggleViewMode,
    FileClickedInGrid(usize),
    GridThumbnailLoaded {
        index: usize,
        handle: Option<iced::widget::image::Handle>,
    },
    PreloadCompleted {
        path: String,
        content: std::sync::Arc<PreviewData>,
    },

    // Image viewer
    ImageZoom(f32),
    ImagePanDelta(f32, f32),
    ImageDoubleClick,
    ScrollDelta {
        x: f32,
        y: f32,
    },
    CtrlHeldChanged(bool),
    ShiftHeldChanged(bool),
    ModifiersUpdated(iced::keyboard::Modifiers),

    // Text editor actions (selection, cursor movement)
    TextEdit(iced::widget::text_editor::Action),

    // Text search & wrap
    SearchQueryChanged(String),
    TextSearchNext,
    TextSearchPrev,
    TextSearchClosed,
    TextWrapToggled,
    CopyContentClicked,
    CopyCode(String),

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
    DaemonOpenWithPlaylist {
        path: String,
        content: PreviewData,
        playlist: Vec<String>,
    },

    FileLoaded {
        path: String,
        content: PreviewData,
    },
    WindowEvent(iced::window::Id, iced::window::Event),
    KeyPressed(iced::keyboard::Key, iced::keyboard::Modifiers),
    PdfPagesLoaded(Vec<Option<(Vec<u8>, u32, u32)>>),
    PdfPageReady(usize, Vec<u8>, u32, u32),
    PdfThumbReady(usize, Vec<u8>, u32, u32),
    PdfScrolled(iced::widget::scrollable::Viewport),

    // Spreadsheet
    SheetTabClicked(usize),
    SpreadsheetColumnClicked(usize),
    SpreadsheetSearchQueryChanged(String),
    SpreadsheetSearchClosed,

    // Grid search
    GridSearchQueryChanged(String),
    GridSearchClosed,

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
    // Async video thumbnail
    VideoThumbnailLoaded {
        data: Vec<u8>,
    },

    // Toast
    ToastDismissed(u64),

    // Link
    OpenLink(String),

    // Preview error
    FilePreviewError(String),

    // Hot reload: file changed on disk
    FileChanged(String),

    // Markdown TOC & Scroll
    TocToggled,
    TocHeadingClicked(usize),
    TocToggleCollapse(usize),
    MarkdownScrolled(f32),
    TextScrolled(f32),
    MarkdownSearchToggle,
    MarkdownSearchQueryChanged(String),
    MarkdownSearchNext,
    MarkdownSearchPrev,
    MarkdownSearchClosed,

    // EPUB Chapter & Sidebar
    EpubSidebarToggled,
    EpubChapterClicked(usize),
    EpubChapterToggleCollapse(usize),

    // Resize TOC / Sidebar Drag Events
    MarkdownSidebarResized(f32),
    EpubSidebarResized(f32),
    SidebarDragStarted(f32),
    SidebarDragEnded,
    MouseMoved(f32, f32),

    // JSON Tree View
    JsonToggleMode,
    JsonToggleNode(usize),
    JsonScrolled(f32),
    JsonRawEdit(iced::widget::text_editor::Action),
    JsonSearchToggle,
    JsonSearchQueryChanged(String),
    JsonSearchClosed,
    JsonExpandAll,
    JsonCollapseAll,
    JsonCopyPath(usize),
    JsonNodeClicked(usize),
    JsonBreadcrumbClicked(usize),
    JsonToggleFormat,
    JsonEditStart(usize),
    JsonEditValue(String),
    JsonEditSave,
    JsonEditCancel,
    JsonSchemaToggle,
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

fn png_to_rgba_handle_with_size(png: Vec<u8>) -> Option<(iced::widget::image::Handle, u32, u32)> {
    match image::load_from_memory(&png) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some((
                iced::widget::image::Handle::from_rgba(width, height, rgba.into_raw()),
                width,
                height,
            ))
        }
        Err(_) => {
            log_debug!("image::load_from_memory failed, falling back to Handle::from_bytes");
            let handle = iced::widget::image::Handle::from_bytes(png);
            Some((handle, 0, 0))
        }
    }
}

fn markdown_block_y_offset(
    blocks: &[Block],
    target_index: usize,
    font_size: f32,
    image_sizes: &std::collections::HashMap<usize, (u32, u32)>,
) -> f32 {
    let mut y: f32 = 15.0;
    for (i, block) in blocks.iter().enumerate() {
        if i == target_index {
            return y;
        }
        y += crate::parsers::markdown::estimated_block_height(block, font_size, i, image_sizes);
    }
    0.0
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
    pub video_controller: Option<Arc<Mutex<crate::ui::handlers::video::VideoController>>>,
    pub ctrl_held: bool,
    pub shift_held: bool,
    pub pending_g: bool,
    pub pending_home: bool,
    pub file_watcher: Option<crate::core::file_watcher::FileWatcher>,
}

impl KglanceApp {
    pub fn new(
        registry: Arc<ParserRegistry>,
        daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
        initial_paths: &[String],
        is_daemon: bool,
    ) -> (Self, Task<Message>) {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
        let vc = crate::ui::handlers::video::spawn_video_player(cmd_rx, event_tx);

        let file_watcher = crate::core::file_watcher::FileWatcher::new().ok();

        let config = crate::core::config::ConfigManager::load_or_create();
        let theme_dark = crate::core::config::ConfigManager::get_theme(&config) != "Light";

        let mut state = KglanceState {
            theme_dark,
            font_size: config.ui.font_size,
            font_family: config.ui.font_family,
            font_family_mono: config.ui.font_family_mono,
            epub_font_family: config.ui.epub_font_family,
            max_text_width: config.ui.max_text_width,
            window_default_size: iced::Size::new(
                config.ui.default_width as f32,
                config.ui.default_height as f32,
            ),
            window_min_size: iced::Size::new(
                config.ui.min_width as f32,
                config.ui.min_height as f32,
            ),
            ..Default::default()
        };

        if !initial_paths.is_empty() {
            state.playlist = initial_paths.to_vec();
            state.current_index = 0;
        }

        let app = Self {
            state,
            registry,
            daemon_rx: Arc::new(Mutex::new(daemon_rx)),
            is_daemon,
            window_id: None,
            current_content: None,
            video_tx: Some(cmd_tx),
            video_rx: Arc::new(Mutex::new(Some(event_rx))),
            video_controller: Some(vc),
            ctrl_held: false,
            shift_held: false,
            pending_g: false,
            pending_home: false,
            file_watcher,
        };

        // Daemon starts with no window — window is opened on demand when a preview request arrives.
        let task = if !initial_paths.is_empty() {
            let path_str = initial_paths[0].clone();
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

    pub fn trigger_preload(&mut self) -> Task<Message> {
        let indices = crate::core::preloader::calculate_preload_window(
            self.state.current_index,
            self.state.playlist.len(),
        );

        let mut tasks = Vec::new();
        for idx in indices {
            if idx < self.state.playlist.len() {
                let path = self.state.playlist[idx].clone();
                if self.state.cache.peek(&path).is_none()
                    && !self.state.pending_preloads.contains(&path)
                    && crate::core::preloader::should_preload_file(&path)
                {
                    self.state.pending_preloads.insert(path.clone());
                    let reg = self.registry.clone();
                    let target_path = path.clone();

                    let target_path_clone = path.clone();

                    tasks.push(Task::perform(
                        async move {
                            let content = tokio::task::spawn_blocking(move || {
                                FilePreviewer::parse(&*reg, Path::new(&target_path)).ok()
                            })
                            .await
                            .ok()
                            .flatten()?;

                            Some(Message::PreloadCompleted {
                                path: target_path_clone,
                                content: std::sync::Arc::new(content),
                            })
                        },
                        |msg| msg.unwrap_or(Message::ToastDismissed(0)),
                    ));
                }
            }
        }
        Task::batch(tasks)
    }

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

    pub fn handle_file_loaded(&mut self, path: String, content: PreviewData) -> Task<Message> {
        let t0 = Instant::now();
        let _scan_path = path.clone();
        self.state.file_name = path.clone();
        self.state.content_ready = true;
        self.state.image.camera = crate::preview::image::Camera::new();

        if let Some(ref watcher) = self.file_watcher {
            let _ = watcher
                .cmd_tx
                .send(crate::core::file_watcher::WatchCommand::Watch(
                    std::path::PathBuf::from(&path),
                ));
        }

        let mut mermaid_tasks: Vec<Task<Message>> = {
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
                                        crate::parsers::markdown::render_mermaid_to_png(&code, None)
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
            tasks
        };

        content.populate_state(&mut self.state);
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
            self.state.pdf.pages[0] = Some(crate::core::PageCacheEntry {
                data: data.clone(),
                width,
                height,
                handle,
            });
        }

        let pdf_task: Option<Task<Message>> = if is_pdf && self.state.pdf.page_count > 1 {
            self.state.pdf.loading = true;
            let page_count = self.state.pdf.page_count;
            let pdf_path = path.clone();
            let visible_page = self.state.pdf.visible_page.clone();
            Some(crate::ui::handlers::pdf::lazy_load_pages(
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

        let thumb_path = if is_video { Some(path.clone()) } else { None };

        if let Some(tx) = &self.video_tx {
            if is_video || is_audio {
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Stop);
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Load(
                    path.clone(),
                ));
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Play);
            } else {
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Stop);
            }
        }

        if let Some(thumb_path) = thumb_path {
            mermaid_tasks.push(Task::perform(
                async move {
                    let data = tokio::task::spawn_blocking(move || {
                        crate::parsers::video::extract_video_thumbnail(std::path::Path::new(
                            &thumb_path,
                        ))
                    })
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                    Message::VideoThumbnailLoaded { data }
                },
                |msg| msg,
            ));
        }

        // Build window management tasks.
        // In daemon mode with no existing window, open one now alongside content.
        let window_tasks: Vec<Task<Message>> = if self.is_daemon {
            if let Some(id) = self.window_id {
                // Window already exists — un-hide and focus it.
                vec![
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
                    iced::window::gain_focus(id),
                ]
            } else {
                // First preview: open window now. Content is already in self.current_content
                // so the first rendered frame will show content immediately.
                vec![self.create_new_window()]
            }
        } else if let Some(id) = self.window_id {
            vec![iced::window::gain_focus(id)]
        } else {
            vec![]
        };

        log_info!(
            "[PERF] handle_file_loaded state+tasks prepared in {:?}",
            t0.elapsed()
        );

        let is_video_or_epub = {
            let lower = path.to_lowercase();
            lower.ends_with(".epub")
                || lower.ends_with(".mp4")
                || lower.ends_with(".mkv")
                || lower.ends_with(".avi")
                || lower.ends_with(".mov")
                || lower.ends_with(".webm")
        };

        if is_video_or_epub {
            self.state.playlist.clear();
        }

        let scan_task: Option<Task<Message>> =
            if !is_video_or_epub && self.state.playlist.len() <= 1 {
                let scan_path = path.clone();
                Some(Task::perform(
                    async move {
                        let files = tokio::task::spawn_blocking(move || {
                            crate::core::navigation::scan_sibling_files(&scan_path)
                        })
                        .await
                        .unwrap_or_default();
                        Message::SiblingFilesLoaded(files)
                    },
                    |msg| msg,
                ))
            } else {
                None
            };

        Task::batch(
            mermaid_tasks
                .into_iter()
                .chain(pdf_task)
                .chain(window_tasks)
                .chain(scan_task),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PreloadCompleted { path, content } => {
                self.state.pending_preloads.remove(&path);
                self.state.cache.put(path, content);
                Task::none()
            }

            Message::SiblingFilesLoaded(files) => {
                if !files.is_empty() && self.state.playlist.len() <= 1 {
                    let current = self.state.file_name.clone();
                    self.state.playlist = files;
                    if let Some(pos) = self.state.playlist.iter().position(|p| p == &current) {
                        self.state.current_index = pos;
                    } else {
                        self.state.current_index = 0;
                    }
                    if matches!(self.state.view_mode, crate::core::ViewMode::Grid(_)) {
                        let thumbnails: Vec<crate::core::GridThumbnail> = self
                            .state
                            .playlist
                            .iter()
                            .map(|p| {
                                let name = std::path::Path::new(p)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| p.clone());
                                crate::core::GridThumbnail {
                                    path: p.clone(),
                                    name,
                                    thumbnail_handle: None,
                                    is_loading: true,
                                }
                            })
                            .collect();
                        self.state.view_mode = crate::core::ViewMode::Grid(thumbnails);
                    }
                    return self.trigger_preload();
                }
                Task::none()
            }

            Message::NextFileClicked => {
                if !self.state.playlist.is_empty() {
                    let next_idx = (self.state.current_index + 1) % self.state.playlist.len();
                    self.state.current_index = next_idx;
                    let next_path = self.state.playlist[next_idx].clone();

                    if let Some(cached_data) = self.state.cache.get(&next_path).cloned() {
                        return self.update(Message::FileLoaded {
                            path: next_path,
                            content: (*cached_data).clone(),
                        });
                    }

                    let reg = self.registry.clone();
                    let path_for_err = next_path.clone();
                    return Task::perform(
                        async move {
                            let content =
                                FilePreviewer::parse(&*reg, Path::new(&next_path)).ok()?;
                            Some(Message::FileLoaded {
                                path: next_path,
                                content,
                            })
                        },
                        move |msg| msg.unwrap_or(Message::FilePreviewError(path_for_err.clone())),
                    );
                }
                Task::none()
            }
            Message::PrevFileClicked => {
                if !self.state.playlist.is_empty() {
                    let prev_idx = if self.state.current_index == 0 {
                        self.state.playlist.len() - 1
                    } else {
                        self.state.current_index - 1
                    };
                    self.state.current_index = prev_idx;
                    let prev_path = self.state.playlist[prev_idx].clone();

                    if let Some(cached_data) = self.state.cache.get(&prev_path).cloned() {
                        return self.update(Message::FileLoaded {
                            path: prev_path,
                            content: (*cached_data).clone(),
                        });
                    }

                    let reg = self.registry.clone();
                    let path_for_err = prev_path.clone();
                    return Task::perform(
                        async move {
                            let content =
                                FilePreviewer::parse(&*reg, Path::new(&prev_path)).ok()?;
                            Some(Message::FileLoaded {
                                path: prev_path,
                                content,
                            })
                        },
                        move |msg| msg.unwrap_or(Message::FilePreviewError(path_for_err.clone())),
                    );
                }
                Task::none()
            }

            Message::GridThumbnailLoaded { index, handle } => {
                if let crate::core::ViewMode::Grid(ref mut thumbnails) = self.state.view_mode
                    && let Some(item) = thumbnails.get_mut(index)
                {
                    item.thumbnail_handle = handle;
                    item.is_loading = false;
                }
                Task::none()
            }
            Message::ToggleViewMode => {
                match self.state.view_mode {
                    crate::core::ViewMode::Detail => {
                        let mut tasks = Vec::new();
                        let thumbnails: Vec<crate::core::GridThumbnail> = self
                            .state
                            .playlist
                            .iter()
                            .enumerate()
                            .map(|(idx, p)| {
                                let name = Path::new(p)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| p.clone());

                                        let path_for_task = p.clone();
                                        tasks.push(Task::perform(
                                            async move {
                                                let handle = tokio::task::spawn_blocking(move || {
                                                    if let Some(thumb_path) = crate::ui::views::grid::get_freedesktop_thumbnail_path(&path_for_task) {
                                                        return Some(iced::widget::image::Handle::from_path(thumb_path));
                                                    }
                                                    let lower = path_for_task.to_lowercase();
                                                    if lower.ends_with(".png")
                                                        || lower.ends_with(".jpg")
                                                        || lower.ends_with(".jpeg")
                                                        || lower.ends_with(".webp")
                                                        || lower.ends_with(".bmp")
                                                        || lower.ends_with(".gif")
                                                    {
                                                        return Some(iced::widget::image::Handle::from_path(&path_for_task));
                                                    }
                                                    if (lower.ends_with(".mp4")
                                                        || lower.ends_with(".mkv")
                                                        || lower.ends_with(".avi")
                                                        || lower.ends_with(".mov")
                                                        || lower.ends_with(".webm"))
                                                        && let Some(bytes) = crate::parsers::video::extract_video_thumbnail(std::path::Path::new(&path_for_task))
                                                    {
                                                        return Some(iced::widget::image::Handle::from_bytes(bytes));
                                                    }
                                                    None
                                                })
                                                .await
                                                .ok()
                                                .flatten();

                                                Message::GridThumbnailLoaded {
                                                    index: idx,
                                                    handle,
                                                }
                                            },
                                            |msg| msg,
                                        ));

                                crate::core::GridThumbnail {
                                    path: p.clone(),
                                    name,
                                    thumbnail_handle: None,
                                    is_loading: true,
                                }
                            })
                            .collect();
                        self.state.view_mode = crate::core::ViewMode::Grid(thumbnails);
                        return Task::batch(tasks);
                    }
                    crate::core::ViewMode::Grid(_) => {
                        self.state.view_mode = crate::core::ViewMode::Detail;
                    }
                }
                Task::none()
            }

            Message::FileClickedInGrid(idx) => {
                let target_path = match &self.state.view_mode {
                    crate::core::ViewMode::Grid(thumbnails) => {
                        thumbnails.get(idx).map(|t| t.path.clone())
                    }
                    _ => None,
                };
                if let Some(target_path) = target_path {
                    self.state.current_index = idx;
                    self.state.view_mode = crate::core::ViewMode::Detail;
                    let reg = self.registry.clone();
                    let path_for_err = target_path.clone();
                    return Task::perform(
                        async move {
                            let content =
                                FilePreviewer::parse(&*reg, Path::new(&target_path)).ok()?;
                            Some(Message::FileLoaded {
                                path: target_path,
                                content,
                            })
                        },
                        move |msg| msg.unwrap_or(Message::FilePreviewError(path_for_err.clone())),
                    );
                }
                Task::none()
            }
            Message::FileClicked(idx) => {
                if idx < self.state.table.rows.len() {
                    self.state.table.selected_index = Some(idx);
                }
                Task::none()
            }
            Message::SortByFieldClicked(field) => {
                let sort = &mut self.state.table.sort_state;
                if sort.active && sort.field == field {
                    match sort.ascending {
                        true => sort.ascending = false,
                        false => {
                            sort.active = false;
                            sort.ascending = true;
                            crate::core::sort_table_rows(
                                &mut self.state.table.rows,
                                &crate::core::SortState {
                                    field: crate::core::SortField::Name,
                                    ascending: true,
                                    active: true,
                                },
                            );
                        }
                    }
                } else {
                    sort.field = field;
                    sort.ascending = true;
                    sort.active = true;
                }
                if sort.active {
                    crate::core::sort_table_rows(&mut self.state.table.rows, sort);
                }
                Task::none()
            }
            Message::CloseRequested => self.handle_close(),
            Message::OpenClicked => self.handle_open_clicked(),
            Message::CopyPathClicked => self.handle_copy_path(),
            Message::CopyCode(code) => {
                let toast = self.show_toast("Copied!");
                Task::batch(vec![iced::clipboard::write(code), toast])
            }
            Message::DaemonOpenWindow { path } => self.handle_daemon_open_window(path),
            Message::DaemonOpenWithPlaylist {
                path,
                content,
                playlist,
            } => {
                if !playlist.is_empty() {
                    self.state.playlist = playlist;
                    if let Some(pos) = self.state.playlist.iter().position(|p| p == &path) {
                        self.state.current_index = pos;
                    } else {
                        self.state.current_index = 0;
                    }
                }
                self.handle_file_loaded(path, content)
            }

            Message::FileLoaded { path, content } => {
                if !self.state.playlist.contains(&path) {
                    self.state.playlist.clear();
                }
                self.handle_file_loaded(path, content)
            }
            Message::WindowEvent(id, event) => self.handle_window_event(id, event),
            Message::ImageZoom(delta) => self.handle_image_zoom(delta),
            Message::ImagePanDelta(dx, dy) => {
                use crate::preview::image::ViewerController;
                ViewerController::pan(&mut self.state.image.camera, dx, dy);
                Task::none()
            }
            Message::ImageDoubleClick => {
                use crate::preview::image::ViewerController;
                ViewerController::reset(&mut self.state.image.camera);
                Task::none()
            }
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
                    self.state.pdf.pages[index] = Some(crate::core::PageCacheEntry {
                        data,
                        width,
                        height,
                        handle,
                    });
                }
                let all_loaded = self.state.pdf.pages.iter().all(|p| p.is_some());
                if all_loaded {
                    self.state.pdf.loading = false;
                }
                Task::none()
            }
            Message::PdfThumbReady(index, data, width, height) => {
                if index < self.state.pdf.thumbnails.len() {
                    let handle =
                        iced::widget::image::Handle::from_rgba(width, height, data.clone());
                    self.state.pdf.thumbnails[index] = Some(crate::core::PageCacheEntry {
                        data,
                        width,
                        height,
                        handle,
                    });
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
                    if let Some((handle, w, h)) = png_to_rgba_handle_with_size(bytes) {
                        self.state
                            .markdown
                            .cached_image_handles
                            .insert(index, handle);
                        self.state.markdown.cached_image_sizes.insert(index, (w, h));
                        if let Some(PreviewData::Markdown { ref blocks }) = self.current_content {
                            self.state.markdown.toc = crate::parsers::markdown::extract_toc(
                                blocks,
                                self.state.font_size,
                                &self.state.markdown.cached_image_sizes,
                            );
                        }
                        log_debug!(
                            "Inserted image handle at index {}, size={}x{}, cache size={}",
                            index,
                            w,
                            h,
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
            Message::VideoThumbnailLoaded { data } => {
                if let Some(PreviewData::Media {
                    ref mut thumbnail_or_waveform,
                    ..
                }) = self.current_content
                {
                    *thumbnail_or_waveform = data;
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
            Message::SpreadsheetSearchQueryChanged(query) => {
                self.state.spreadsheet.search_query = query;
                Task::none()
            }
            Message::SpreadsheetSearchClosed => {
                self.state.spreadsheet.search_visible = false;
                self.state.spreadsheet.search_query.clear();
                Task::none()
            }
            Message::GridSearchQueryChanged(query) => {
                self.state.grid_search_query = query;
                Task::none()
            }
            Message::GridSearchClosed => {
                self.state.grid_search_visible = false;
                self.state.grid_search_query.clear();
                Task::none()
            }
            Message::OpenLink(url) => {
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                Task::none()
            }
            Message::ThemeToggled => {
                self.state.theme_dark = !self.state.theme_dark;
                Task::none()
            }
            Message::ToastDismissed(id) => {
                self.state.toasts.retain(|t| t.id != id);
                Task::none()
            }
            Message::TextEdit(action) => {
                if !matches!(action, iced::widget::text_editor::Action::Edit(_)) {
                    self.state.text.content.perform(action);
                }
                Task::none()
            }
            Message::FilePreviewError(path) => {
                let name = Path::new(&path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(path);
                self.show_toast(format!("\"{}\" cannot be previewed", name))
            }
            Message::TocToggled => {
                self.state.markdown.toc_visible = !self.state.markdown.toc_visible;
                Task::none()
            }
            Message::TocToggleCollapse(idx) => {
                if self.state.markdown.collapsed_headings.contains(&idx) {
                    self.state.markdown.collapsed_headings.remove(&idx);
                } else {
                    self.state.markdown.collapsed_headings.insert(idx);
                }
                Task::none()
            }
            Message::TocHeadingClicked(idx) => {
                let y = self
                    .state
                    .markdown
                    .toc
                    .iter()
                    .find(|e| e.block_index == idx)
                    .map(|e| e.y_offset)
                    .unwrap_or(0.0);
                iced::widget::operation::scroll_to(
                    "content_scroll",
                    iced::widget::operation::AbsoluteOffset { x: 0.0, y },
                )
            }
            Message::MarkdownScrolled(y) => {
                self.state.markdown.scroll_y = y;
                let toc = &self.state.markdown.toc;
                if let Some(active_pos) = toc.iter().rposition(|e| e.y_offset <= y + 50.0) {
                    let target_y = (active_pos as f32 * 28.0 - 100.0).max(0.0);
                    iced::widget::operation::scroll_to(
                        "toc_scroll",
                        iced::widget::operation::AbsoluteOffset {
                            x: 0.0,
                            y: target_y,
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::MarkdownSearchToggle => {
                let s = &mut self.state.markdown;
                s.search_visible = !s.search_visible;
                if !s.search_visible {
                    s.search_query.clear();
                    s.search_match_count = 0;
                    s.search_match_index = 0;
                    s.search_match_blocks.clear();
                    s.search_info.clear();
                    Task::none()
                } else {
                    operation::focus("md_search_input")
                }
            }
            Message::MarkdownSearchQueryChanged(query) => {
                let s = &mut self.state.markdown;
                s.search_query = query.clone();
                s.search_match_index = 0;
                if query.is_empty() {
                    s.search_match_count = 0;
                    s.search_match_blocks.clear();
                    s.search_info.clear();
                } else if let Some(PreviewData::Markdown { blocks }) = &self.current_content {
                    let q = query.to_lowercase();
                    let mut count = 0;
                    let mut match_blocks = Vec::new();
                    for (bi, block) in blocks.iter().enumerate() {
                        let text = match block {
                            Block::Heading { content, .. } | Block::Paragraph(content) => {
                                crate::parsers::markdown::flatten_inlines(content)
                            }
                            Block::CodeBlock { code, .. } => code.clone(),
                            Block::Quote(inner) => inner
                                .iter()
                                .map(|ib| match ib {
                                    Block::Heading { content, .. } | Block::Paragraph(content) => {
                                        crate::parsers::markdown::flatten_inlines(content)
                                    }
                                    _ => String::new(),
                                })
                                .collect::<Vec<_>>()
                                .join(" "),
                            Block::List { items, .. } => items
                                .iter()
                                .flat_map(|item| {
                                    let own =
                                        crate::parsers::markdown::flatten_inlines(&item.content);
                                    let sub: String = item
                                        .sub_blocks
                                        .iter()
                                        .map(|lb| match lb {
                                            Block::Heading { content, .. }
                                            | Block::Paragraph(content) => {
                                                crate::parsers::markdown::flatten_inlines(content)
                                            }
                                            _ => String::new(),
                                        })
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    vec![own, sub]
                                })
                                .collect::<Vec<_>>()
                                .join(" "),
                            Block::Table(tbl) => tbl
                                .rows
                                .iter()
                                .flat_map(|r| r.iter())
                                .map(|cell| {
                                    crate::parsers::markdown::flatten_inlines(&cell.content)
                                })
                                .collect::<Vec<_>>()
                                .join(" "),
                            _ => String::new(),
                        };
                        let n = text.to_lowercase().matches(&q).count();
                        for _ in 0..n {
                            match_blocks.push(bi);
                        }
                        count += n;
                    }
                    s.search_match_count = count;
                    s.search_match_blocks = match_blocks;
                    s.search_info = if count > 0 {
                        format!("1/{}", count)
                    } else {
                        String::new()
                    };
                }
                Task::none()
            }
            Message::MarkdownSearchNext => {
                let s = &mut self.state.markdown;
                if s.search_match_count > 0 {
                    s.search_match_index = (s.search_match_index + 1) % s.search_match_count;
                    s.search_info =
                        format!("{}/{}", s.search_match_index + 1, s.search_match_count);
                    let block_idx = s.search_match_blocks[s.search_match_index];
                    if let Some(PreviewData::Markdown { blocks }) = &self.current_content {
                        let y = markdown_block_y_offset(
                            blocks,
                            block_idx,
                            self.state.font_size,
                            &self.state.markdown.cached_image_sizes,
                        );
                        return iced::widget::operation::scroll_to(
                            "content_scroll",
                            iced::widget::operation::AbsoluteOffset { x: 0.0, y },
                        );
                    }
                }
                Task::none()
            }
            Message::MarkdownSearchPrev => {
                let s = &mut self.state.markdown;
                if s.search_match_count > 0 {
                    s.search_match_index = if s.search_match_index == 0 {
                        s.search_match_count - 1
                    } else {
                        s.search_match_index - 1
                    };
                    s.search_info =
                        format!("{}/{}", s.search_match_index + 1, s.search_match_count);
                    let block_idx = s.search_match_blocks[s.search_match_index];
                    if let Some(PreviewData::Markdown { blocks }) = &self.current_content {
                        let y = markdown_block_y_offset(
                            blocks,
                            block_idx,
                            self.state.font_size,
                            &self.state.markdown.cached_image_sizes,
                        );
                        return iced::widget::operation::scroll_to(
                            "content_scroll",
                            iced::widget::operation::AbsoluteOffset { x: 0.0, y },
                        );
                    }
                }
                Task::none()
            }
            Message::MarkdownSearchClosed => {
                let s = &mut self.state.markdown;
                s.search_visible = false;
                s.search_query.clear();
                s.search_match_count = 0;
                s.search_match_index = 0;
                s.search_match_blocks.clear();
                s.search_info.clear();
                Task::none()
            }
            Message::EpubSidebarToggled => {
                self.state.epub.sidebar_visible = !self.state.epub.sidebar_visible;
                Task::none()
            }
            Message::EpubChapterClicked(idx) => {
                if idx < self.state.epub.chapters.len() {
                    self.state.epub.active_chapter = idx;
                    let chapter = &self.state.epub.chapters[idx];
                    let font_size = self.state.font_size;
                    let mut target_y: f32 = 0.0;

                    // If chapter has an anchor or title, try to find matching block offset
                    if let Some(ref anc) = chapter.anchor {
                        let mut y_accum: f32 = 0.0;
                        for (b_idx, block) in chapter.blocks.iter().enumerate() {
                            let text_flat = match block {
                                crate::parsers::markdown::Block::Heading { content, .. }
                                | crate::parsers::markdown::Block::Paragraph(content) => {
                                    crate::parsers::markdown::flatten_inlines(content)
                                }
                                _ => String::new(),
                            };
                            if text_flat.contains(anc) || text_flat.contains(&chapter.title) {
                                target_y = y_accum;
                                break;
                            }
                            y_accum += crate::parsers::markdown::estimated_block_height(
                                block,
                                font_size,
                                b_idx,
                                &self.state.markdown.cached_image_sizes,
                            );
                        }
                    } else {
                        // Search by chapter title in blocks
                        let mut y_accum: f32 = 0.0;
                        for (b_idx, block) in chapter.blocks.iter().enumerate() {
                            let text_flat = match block {
                                crate::parsers::markdown::Block::Heading { content, .. }
                                | crate::parsers::markdown::Block::Paragraph(content) => {
                                    crate::parsers::markdown::flatten_inlines(content)
                                }
                                _ => String::new(),
                            };
                            if !chapter.title.is_empty() && text_flat.contains(&chapter.title) {
                                target_y = y_accum;
                                break;
                            }
                            y_accum += crate::parsers::markdown::estimated_block_height(
                                block,
                                font_size,
                                b_idx,
                                &self.state.markdown.cached_image_sizes,
                            );
                        }
                    }

                    if target_y > 0.0 {
                        return iced::widget::operation::scroll_to(
                            "content_scroll",
                            iced::widget::operation::AbsoluteOffset {
                                x: 0.0,
                                y: target_y,
                            },
                        );
                    } else {
                        return iced::widget::operation::snap_to(
                            "content_scroll",
                            iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
                        );
                    }
                }
                Task::none()
            }
            Message::EpubChapterToggleCollapse(idx) => {
                if self.state.epub.collapsed_chapters.contains(&idx) {
                    self.state.epub.collapsed_chapters.remove(&idx);
                } else {
                    self.state.epub.collapsed_chapters.insert(idx);
                }
                Task::none()
            }
            Message::MarkdownSidebarResized(width) => {
                self.state.markdown.sidebar_width = width.clamp(140.0, 550.0);
                Task::none()
            }
            Message::EpubSidebarResized(width) => {
                self.state.epub.sidebar_width = width.clamp(140.0, 550.0);
                Task::none()
            }
            Message::SidebarDragStarted(start_x) => {
                self.state.markdown.sidebar_resizing = true;
                self.state.markdown.sidebar_drag_start_x = start_x;
                self.state.markdown.sidebar_drag_start_width = self.state.markdown.sidebar_width;

                self.state.epub.sidebar_resizing = true;
                self.state.epub.sidebar_drag_start_x = start_x;
                self.state.epub.sidebar_drag_start_width = self.state.epub.sidebar_width;
                Task::none()
            }
            Message::SidebarDragEnded => {
                self.state.markdown.sidebar_resizing = false;
                self.state.epub.sidebar_resizing = false;
                Task::none()
            }
            Message::MouseMoved(x, _y) => {
                if self.state.markdown.sidebar_resizing {
                    let delta = x - self.state.markdown.sidebar_drag_start_x;
                    let new_w =
                        (self.state.markdown.sidebar_drag_start_width + delta).clamp(140.0, 550.0);
                    self.state.markdown.sidebar_width = new_w;
                }
                if self.state.epub.sidebar_resizing {
                    let delta = x - self.state.epub.sidebar_drag_start_x;
                    let new_w =
                        (self.state.epub.sidebar_drag_start_width + delta).clamp(140.0, 550.0);
                    self.state.epub.sidebar_width = new_w;
                }
                Task::none()
            }
            Message::JsonToggleMode => {
                self.state.json.tree_mode = !self.state.json.tree_mode;
                Task::none()
            }
            Message::JsonToggleNode(index) => {
                if self.state.json.expanded.contains(&index) {
                    self.state.json.expanded.remove(&index);
                } else {
                    self.state.json.expanded.insert(index);
                }
                Task::none()
            }
            Message::JsonScrolled(y) => {
                self.state.json.scroll_y = y;
                Task::none()
            }
            Message::JsonRawEdit(action) => {
                if !matches!(action, iced::widget::text_editor::Action::Edit(_)) {
                    self.state.json.raw_editor.perform(action);
                }
                Task::none()
            }
            Message::JsonSearchToggle => {
                let s = &mut self.state.json;
                s.search_visible = !s.search_visible;
                if !s.search_visible {
                    s.search_query.clear();
                    Task::none()
                } else {
                    operation::focus("json_search_input")
                }
            }
            Message::JsonSearchQueryChanged(query) => {
                self.state.json.search_query = query;
                Task::none()
            }
            Message::JsonSearchClosed => {
                self.state.json.search_visible = false;
                self.state.json.search_query.clear();
                Task::none()
            }
            Message::JsonExpandAll => {
                for (i, node) in self.state.json.nodes.iter().enumerate() {
                    if node.children_count > 0 {
                        self.state.json.expanded.insert(i);
                    }
                }
                Task::none()
            }
            Message::JsonCollapseAll => {
                self.state.json.expanded.clear();
                Task::none()
            }
            Message::JsonCopyPath(index) => {
                let val = self
                    .state
                    .json
                    .nodes
                    .get(index)
                    .map(|n| n.value_preview.clone())
                    .unwrap_or_default();
                let toast = self.show_toast("Copied value!");
                Task::batch(vec![iced::clipboard::write(val), toast])
            }
            Message::JsonNodeClicked(index) => {
                self.state.json.active_node = Some(index);
                Task::none()
            }
            Message::JsonBreadcrumbClicked(_index) => iced::widget::operation::scroll_to(
                "json_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: 0.0 },
            ),
            Message::JsonToggleFormat => {
                let s = &mut self.state.json;
                let content = if s.raw_pretty {
                    s.minified_content.clone()
                } else {
                    s.pretty_content.clone()
                };
                s.raw_editor = iced::widget::text_editor::Content::with_text(&content);
                s.raw_pretty = !s.raw_pretty;
                Task::none()
            }
            Message::JsonEditStart(index) => {
                if let Some(node) = self.state.json.nodes.get(index) {
                    let val = node.value_preview.clone();
                    self.state.json.editing_node = Some(index);
                    self.state.json.edit_value = val;
                }
                Task::none()
            }
            Message::JsonEditValue(val) => {
                self.state.json.edit_value = val;
                Task::none()
            }
            Message::JsonEditSave => {
                if let Some(_idx) = self.state.json.editing_node {
                    let path_str = self.state.file_name.clone();
                    let reg = self.registry.clone();
                    self.state.json.editing_node = None;
                    self.state.json.edit_value.clear();
                    // Re-read and re-parse the file to get fresh content
                    return Task::perform(
                        async move {
                            let content =
                                FilePreviewer::parse(&*reg, std::path::Path::new(&path_str))
                                    .ok()?;
                            Some(Message::FileLoaded {
                                path: path_str,
                                content,
                            })
                        },
                        |msg| msg.unwrap_or(Message::ToastDismissed(0)),
                    );
                }
                Task::none()
            }
            Message::JsonEditCancel => {
                self.state.json.editing_node = None;
                self.state.json.edit_value.clear();
                Task::none()
            }
            Message::JsonSchemaToggle => {
                self.state.json.schema_visible = !self.state.json.schema_visible;
                Task::none()
            }
            Message::TextScrolled(y) => {
                self.state.text.scroll_y = y;
                Task::none()
            }
            Message::FileChanged(path) => {
                if path == self.state.file_name {
                    let path_obj = Path::new(&path);
                    if !path_obj.exists() {
                        let name = path_obj
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or(path);
                        self.current_content = None;
                        self.state.content_ready = false;
                        return self.show_toast(format!("File deleted: \"{}\"", name));
                    }

                    let prev_md_y = self.state.markdown.scroll_y;
                    let prev_txt_y = self.state.text.scroll_y;

                    if let Ok(content) = FilePreviewer::parse(&*self.registry, path_obj) {
                        let is_md = matches!(content, PreviewData::Markdown { .. });
                        let is_txt = matches!(content, PreviewData::Text { .. });

                        let load_task = self.handle_file_loaded(path, content);

                        let target_y = if is_md {
                            prev_md_y
                        } else if is_txt {
                            prev_txt_y
                        } else {
                            0.0
                        };

                        if target_y > 0.0 {
                            let scroll_task = iced::widget::operation::scroll_to(
                                "content_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: target_y,
                                },
                            );
                            return Task::batch(vec![load_task, scroll_task]);
                        } else {
                            return load_task;
                        }
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let (preview_body, is_media) = if let Some(content) = &self.current_content {
            let is_media = matches!(content, PreviewData::Media { .. });
            let body: Element<'_, Message> = match content {
                PreviewData::Text { .. } => crate::ui::views::view_text(
                    &self.state.text,
                    self.state.theme_dark,
                    self.state.font_size,
                    self.state.font_family_mono.as_deref(),
                ),
                PreviewData::Markdown { blocks } => crate::ui::views::view_markdown(
                    blocks,
                    &self.state.markdown,
                    self.state.font_size,
                    self.state.theme_dark,
                    self.state.font_family_mono.as_deref(),
                    self.state.max_text_width,
                ),
                PreviewData::Image { .. } => crate::ui::views::view_image(&self.state.image),
                PreviewData::Font { name, metadata, .. } => {
                    crate::ui::views::view_font(name, metadata, self.state.theme_dark)
                }
                PreviewData::Pdf { .. } => crate::ui::views::view_pdf(&self.state.pdf),
                PreviewData::Folder { .. } => {
                    crate::ui::views::view_table(&self.state.table, self.state.theme_dark)
                }
                PreviewData::Spreadsheet { .. } => {
                    crate::ui::views::view_spreadsheet(&self.state.spreadsheet)
                }
                PreviewData::Json { .. } => crate::ui::views::view_json(
                    &self.state.json,
                    self.state.font_size,
                    self.state.theme_dark,
                    self.state.font_family_mono.as_deref(),
                ),
                PreviewData::Epub { .. } => crate::ui::views::view_epub(
                    &self.state.epub,
                    self.state.font_size,
                    self.state.theme_dark,
                    self.state
                        .epub_font_family
                        .as_deref()
                        .or(self.state.font_family.as_deref()),
                    self.state.font_family_mono.as_deref(),
                    self.state.max_text_width,
                ),
                PreviewData::Media {
                    thumbnail_or_waveform,
                    width,
                    height,
                    ..
                } => crate::ui::views::view_media(
                    &self.state.media,
                    thumbnail_or_waveform,
                    self.video_controller.as_ref().unwrap(),
                    *width,
                    *height,
                ),
                PreviewData::Error(err) => iced::widget::text(err).size(18).into(),
            };
            (body, is_media)
        } else {
            (iced::widget::text("No file loaded.").size(18).into(), false)
        };

        crate::ui::window::view_window(&self.state, preview_body, is_media)
    }

    /// Variant for [`iced::daemon`] which requires a `window::Id` parameter.
    pub fn view_daemon(&self, _window_id: iced::window::Id) -> Element<'_, Message> {
        self.view()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let dbus_sub = subscription::from_recipe(crate::dbus::recipe::DaemonRecipe::new(
            self.daemon_rx.clone(),
        ));

        let video_sub = subscription::from_recipe(crate::ui::handlers::video::VideoRecipe::new(
            self.video_rx.clone(),
        ));

        let event_sub = iced::window::events().map(|(id, event)| Message::WindowEvent(id, event));

        let global_event_sub = iced::event::listen_with(|event, _status, _window_id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyPressed(key, modifiers))
            }
            iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::ModifiersUpdated(modifiers))
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Some(Message::MouseMoved(position.x, position.y))
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                Some(Message::SidebarDragEnded)
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

        let file_watcher_sub = if let Some(ref watcher) = self.file_watcher {
            subscription::from_recipe(crate::core::file_watcher::FileWatcherRecipe::new(
                watcher.events.clone(),
            ))
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![
            dbus_sub,
            event_sub,
            video_sub,
            global_event_sub,
            file_watcher_sub,
        ])
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
