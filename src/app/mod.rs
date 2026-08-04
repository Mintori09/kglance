mod keyboard;
mod media;
pub mod messages;
pub use messages::*;

pub mod update;

mod window;

use crate::features::markdown::Block;
use iced::Subscription;
use iced::window as iced_window;
use iced::{Element, Task, Theme};
use iced_futures::subscription;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::core::{FilePreviewer, KglanceState, PreviewData};
use crate::dbus::DaemonCommand;
use crate::features::common::parser::traits::ParserRegistry;
use crate::{log_debug, log_info};

pub struct KglanceApp {
    pub state: KglanceState,
    pub registry: Arc<ParserRegistry>,
    pub daemon_rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>,
    pub is_daemon: bool,
    pub window_id: Option<iced::window::Id>,
    pub current_content: Option<PreviewData>,
    pub video_tx: Option<tokio::sync::mpsc::Sender<crate::features::video::handler::PlayerCommand>>,
    pub video_rx: Arc<
        Mutex<Option<tokio::sync::mpsc::Receiver<crate::features::video::handler::VideoEvent>>>,
    >,
    pub video_controller: Option<Arc<Mutex<crate::features::video::handler::VideoController>>>,
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
        let vc = crate::features::video::handler::spawn_video_player(cmd_rx, event_tx);

        let file_watcher = crate::core::file_watcher::FileWatcher::new().ok();

        let config = crate::core::config::ConfigManager::load_or_create();
        let theme_setting = crate::core::config::ConfigManager::get_theme_setting(&config);
        let app_theme = crate::core::config::ConfigManager::resolve_theme(&theme_setting);

        let mut state = KglanceState {
            app_theme,
            theme_setting,
            font_size: config.ui.font_size,
            default_font_size: config.ui.font_size,
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
            prefer_mermaid_cli: config.ui.prefer_mermaid_cli,

            ..Default::default()
        };

        state.read_positions = crate::core::ReadPositions::load();

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

        let task = if !initial_paths.is_empty() {
            let path_str = initial_paths[0].clone();
            crate::app::update::navigation::load_file_task(&app, path_str, |_| {
                crate::app::messages::ActionMsg::CloseRequested.into()
            })
        } else {
            Task::none()
        };

        (app, task)
    }

    pub(crate) fn record_read_position(&mut self) {
        let path = self.state.file_name.clone();
        if path.is_empty() {
            return;
        }
        let pos = match &self.current_content {
            Some(crate::core::PreviewData::Text { .. }) => crate::core::ReadPosition {
                scroll_y: self.state.text.scroll_y,
                chapter: 0,
            },
            Some(crate::core::PreviewData::Markdown { .. }) => crate::core::ReadPosition {
                scroll_y: self.state.markdown.scroll_y,
                chapter: 0,
            },
            Some(crate::core::PreviewData::Epub { .. }) => crate::core::ReadPosition {
                scroll_y: self.state.epub.markdown_state.scroll_y,
                chapter: self.state.epub.active_chapter,
            },
            _ => return,
        };
        self.state.read_positions.insert(path, pos);
        self.state.read_positions_dirty = true;
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

                            Some(
                                crate::app::messages::NavigationMsg::PreloadCompleted {
                                    path: target_path_clone,
                                    content: std::sync::Arc::new(content),
                                }
                                .into(),
                            )
                        },
                        |msg| {
                            msg.unwrap_or(crate::app::messages::SystemMsg::ToastDismissed(0).into())
                        },
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

        self.record_read_position();
        if self.state.read_positions_dirty {
            let _ = self.state.read_positions.save();
            self.state.read_positions_dirty = false;
        }

        self.update_loaded_file_state(&path, &content);

        let mut tasks = self.prepare_markdown_tasks(&content, &path);
        if let Some(pdf_task) = self.prepare_pdf_task(&content, &path) {
            tasks.push(pdf_task);
        }
        if let Some(typst_task) = self.prepare_typst_task(&content, &path) {
            tasks.push(typst_task);
        }
        tasks.extend(self.prepare_media_tasks(&path));
        tasks.extend(self.prepare_window_tasks());
        if let Some(scan_task) = self.prepare_sibling_scan_task(&path) {
            tasks.push(scan_task);
        }

        log_info!(
            "[PERF] handle_file_loaded state+tasks prepared in {:?}",
            t0.elapsed()
        );

        Task::batch(tasks)
    }

    fn update_loaded_file_state(&mut self, path: &str, content: &PreviewData) {
        use crate::features::image::Camera;
        self.state.file_name = path.to_string();
        self.state.content_ready = true;
        self.state.image.camera = Camera::new();

        if let Some(ref watcher) = self.file_watcher {
            let _ = watcher
                .cmd_tx
                .send(crate::core::file_watcher::WatchCommand::Watch(
                    std::path::PathBuf::from(path),
                ));
        }

        content.populate_state(&mut self.state);
        self.current_content = Some(content.clone());
    }

    fn prepare_markdown_tasks(&self, content: &PreviewData, file_path: &str) -> Vec<Task<Message>> {
        let mut tasks = Vec::new();
        if let PreviewData::Markdown { blocks, .. } = content {
            for (i, block) in blocks.iter().enumerate() {
                match block {
                    Block::Mermaid {
                        lines,
                        rendered: None,
                    } => {
                        log_debug!("Spawning async render for Mermaid block[{}]", i);
                        let code = lines.join("\n");
                        let prefer_cli = self.state.prefer_mermaid_cli;
                        tasks.push(Task::perform(
                            async move {
                                let png = tokio::task::spawn_blocking(move || {
                                    crate::parsers::markdown::render_mermaid_to_png(
                                        &code, None, prefer_cli,
                                    )
                                })
                                .await
                                .ok()
                                .flatten();
                                crate::app::messages::MarkdownMsg::MermaidBlockRendered {
                                    index: i,
                                    png_bytes: png,
                                }
                                .into()
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
                            Path::new(file_path)
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
                                crate::app::messages::MarkdownMsg::ImageLoaded {
                                    index: i,
                                    png_bytes: bytes,
                                }
                                .into()
                            },
                            |msg| msg,
                        ));
                    }
                    _ => {}
                }
            }
        }
        tasks
    }

    fn prepare_pdf_task(&mut self, content: &PreviewData, path: &str) -> Option<Task<Message>> {
        let is_pdf = matches!(content, PreviewData::Pdf { .. });

        if let PreviewData::Pdf {
            data,
            width,
            height,
            ..
        } = content
            && !data.is_empty()
            && !self.state.pdf.pages.is_empty()
        {
            let handle = iced::widget::image::Handle::from_rgba(*width, *height, data.clone());
            self.state.pdf.pages[0] = Some(crate::core::PageCacheEntry {
                data: data.clone(),
                width: *width,
                height: *height,
                handle,
            });
        }

        if is_pdf && self.state.pdf.page_count > 1 {
            self.state.pdf.loading = true;
            let page_count = self.state.pdf.page_count;
            let pdf_path = path.to_string();
            let visible_page = self.state.pdf.visible_page.clone();
            Some(crate::features::pdf::handler::lazy_load_pages(
                pdf_path,
                page_count,
                visible_page,
            ))
        } else {
            None
        }
    }

    fn prepare_typst_task(&mut self, content: &PreviewData, path: &str) -> Option<Task<Message>> {
        let is_typst = matches!(content, PreviewData::Typst { .. });

        if let PreviewData::Typst {
            data,
            width,
            height,
            page_count,
            ..
        } = content
            && !data.is_empty()
            && !self.state.typst.pdf.pages.is_empty()
            && *page_count > 0
        {
            let handle = iced::widget::image::Handle::from_rgba(*width, *height, data.clone());
            self.state.typst.pdf.pages[0] = Some(crate::core::PageCacheEntry {
                data: data.clone(),
                width: *width,
                height: *height,
                handle,
            });
        }

        if is_typst && self.state.typst.pdf.page_count > 1 && self.state.typst.error.is_none() {
            self.state.typst.pdf.loading = true;
            let page_count = self.state.typst.pdf.page_count;
            let typst_path = path.to_string();
            let visible_page = self.state.typst.pdf.visible_page.clone();
            Some(crate::features::typst::handler::lazy_load_typst_pages(
                typst_path,
                page_count,
                visible_page,
            ))
        } else {
            None
        }
    }

    fn prepare_media_tasks(&mut self, path: &str) -> Vec<Task<Message>> {
        let mut tasks = Vec::new();
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
                let _ = tx.try_send(crate::features::video::handler::PlayerCommand::Stop);
                let _ = tx.try_send(crate::features::video::handler::PlayerCommand::Load(
                    path.to_string(),
                ));
                let _ = tx.try_send(crate::features::video::handler::PlayerCommand::Play);
            } else {
                let _ = tx.try_send(crate::features::video::handler::PlayerCommand::Stop);
            }
        }

        if is_video {
            let thumb_path = path.to_string();
            tasks.push(Task::perform(
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
                    crate::app::messages::MediaMsg::VideoThumbnailLoaded { data }.into()
                },
                |msg| msg,
            ));
        }

        tasks
    }

    fn prepare_window_tasks(&mut self) -> Vec<Task<Message>> {
        if self.is_daemon {
            if let Some(id) = self.window_id {
                vec![
                    iced::window::set_mode(id, iced::window::Mode::Windowed),
                    iced::window::gain_focus(id),
                ]
            } else {
                vec![self.create_new_window()]
            }
        } else if let Some(id) = self.window_id {
            vec![iced::window::gain_focus(id)]
        } else {
            vec![]
        }
    }

    fn prepare_sibling_scan_task(&mut self, path: &str) -> Option<Task<Message>> {
        // let is_video_or_epub = {
        //     let lower = path.to_lowercase();
        //     lower.ends_with(".epub")
        //         || lower.ends_with(".mp4")
        //         || lower.ends_with(".mkv")
        //         || lower.ends_with(".avi")
        //         || lower.ends_with(".mov")
        //         || lower.ends_with(".webm")
        // };
        //
        // if is_video_or_epub {
        //     self.state.playlist.clear();
        // }

        if self.state.playlist.len() <= 1 {
            let scan_path = path.to_string();
            Some(Task::perform(
                async move {
                    let files = tokio::task::spawn_blocking(move || {
                        crate::core::navigation::scan_sibling_files(&scan_path)
                    })
                    .await
                    .unwrap_or_default();
                    crate::app::messages::NavigationMsg::SiblingFilesLoaded(files).into()
                },
                |msg| msg,
            ))
        } else {
            None
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::update(self, message)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let (preview_body, is_media) = if let Some(content) = &self.current_content {
            let is_media = matches!(content, PreviewData::Media { .. });
            let body: Element<'_, Message> = match content {
                PreviewData::Text { .. } => crate::ui::views::view_text(
                    &self.state.text,
                    self.state.app_theme,
                    self.state.font_size,
                    self.state.font_family_mono.as_deref(),
                ),
                PreviewData::Markdown { blocks, .. } => crate::ui::views::view_markdown(
                    blocks,
                    &self.state.markdown,
                    self.state.font_size,
                    self.state.app_theme,
                    self.state.font_family.as_deref(),
                    self.state.font_family_mono.as_deref(),
                    self.state.max_text_width,
                ),
                PreviewData::Image { .. } => crate::ui::views::view_image(&self.state.image),
                PreviewData::Font { name, metadata, .. } => {
                    crate::ui::views::view_font(name, metadata, self.state.app_theme)
                }
                PreviewData::Pdf { .. } => crate::ui::views::view_pdf(
                    &self.state.pdf,
                    self.state.font_size,
                    self.state.app_theme,
                ),
                PreviewData::Typst { .. } => crate::ui::views::view_typst(
                    &self.state.typst,
                    self.state.app_theme,
                    self.state.font_size,
                    self.state.font_family_mono.as_deref(),
                ),
                PreviewData::Folder { .. } => {
                    crate::ui::views::view_folder(&self.state.folder, self.state.app_theme)
                }
                PreviewData::Spreadsheet { .. } => crate::ui::views::view_spreadsheet(
                    &self.state.spreadsheet,
                    self.state.app_theme,
                ),
                PreviewData::Json { .. } => crate::ui::views::view_json(
                    &self.state.json,
                    self.state.font_size,
                    self.state.app_theme,
                    self.state.font_family_mono.as_deref(),
                ),
                PreviewData::Epub { .. } => crate::ui::views::view_epub(
                    &self.state.epub,
                    self.state.font_size,
                    self.state.app_theme,
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

        let video_sub = subscription::from_recipe(
            crate::features::video::handler::VideoRecipe::new(self.video_rx.clone()),
        );

        let event_sub = iced::window::events()
            .map(|(id, event)| crate::app::messages::SystemMsg::WindowEvent(id, event).into());

        let resize_sub = iced_window::resize_events().map(|(_id, size)| {
            crate::app::messages::SystemMsg::WindowResized(size.width, size.height).into()
        });

        let global_event_sub = iced::event::listen_with(|event, _status, _window_id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(crate::app::messages::SystemMsg::KeyPressed(key, modifiers).into())
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

        let auto_scroll_sub = if crate::features::markdown::update::active_markdown_state(self)
            .auto_scroll_delta
            .is_some()
        {
            iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| crate::app::messages::MarkdownMsg::AutoScrollTick.into())
        } else {
            Subscription::none()
        };

        let read_positions_sub = if self.state.read_positions_dirty {
            iced::time::every(std::time::Duration::from_secs(1))
                .map(|_| crate::app::messages::SystemMsg::ReadPositionsTick.into())
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![
            dbus_sub,
            event_sub,
            resize_sub,
            video_sub,
            global_event_sub,
            file_watcher_sub,
            auto_scroll_sub,
            read_positions_sub,
        ])
    }

    pub fn theme(&self) -> Theme {
        if self.state.app_theme.is_dark() {
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

#[cfg(test)]
pub(crate) mod test_util {
    use super::*;

    pub fn test_app(content: Option<PreviewData>) -> KglanceApp {
        let registry = std::sync::Arc::new(ParserRegistry::new());
        KglanceApp {
            state: KglanceState::default(),
            registry,
            daemon_rx: std::sync::Arc::new(std::sync::Mutex::new(None)),
            is_daemon: false,
            window_id: None,
            current_content: content,
            video_tx: None,
            video_rx: std::sync::Arc::new(std::sync::Mutex::new(None)),
            video_controller: None,
            ctrl_held: false,
            shift_held: false,
            pending_g: false,
            pending_home: false,
            file_watcher: None,
        }
    }

    #[allow(dead_code)]
    pub fn markdown_content(md: &str) -> PreviewData {
        PreviewData::Markdown {
            blocks: crate::parsers::markdown::parse_to_blocks(md),
            raw_text: md.to_string(),
        }
    }

    pub fn epub_content(chapter_texts: &[&str]) -> PreviewData {
        let chapters = chapter_texts
            .iter()
            .map(|t| crate::core::types::EpubChapterInfo {
                title: t.to_string(),
                level: 1,
                anchor: None,
                blocks: crate::parsers::markdown::parse_to_blocks(t),
            })
            .collect();
        PreviewData::Epub {
            title: "Test EPUB".to_string(),
            author: String::new(),
            chapters,
            active_chapter: 0,
            images: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn state_has_read_positions_fields() {
        let st = KglanceState::default();
        assert!(!st.read_positions_dirty);
        assert_eq!(st.read_positions.get("/x").map(|p| p.scroll_y), None);
    }
}
