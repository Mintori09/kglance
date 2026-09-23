mod keyboard;
mod media;
pub mod messages;
pub use messages::*;

pub mod update;

mod window;

use crate::core::{FilePreviewer, KglanceState, PreviewData};
use crate::dbus::DaemonCommand;
use crate::features::common::parser::traits::ParserRegistry;
use crate::features::json;
use crate::features::markdown::Block;
use crate::log_debug;
use iced::Subscription;
use iced::window as iced_window;
use iced::{Element, Task, Theme};
use iced_futures::subscription;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;

pub struct KglanceApp {
    pub state: KglanceState,
    pub registry: Arc<ParserRegistry>,
    pub daemon_rx: Arc<Mutex<Option<mpsc::Receiver<DaemonCommand>>>>,
    pub is_daemon: bool,
    pub is_gui_open: Arc<std::sync::atomic::AtomicBool>,
    pub window_id: Option<iced::window::Id>,
    pub is_window_opening: bool,
    pub current_content: Option<PreviewData>,
    pub video: Option<iced_video_player::Video>,
    pub ctrl_held: bool,
    pub shift_held: bool,
    pub pending_g: bool,
    pub pending_home: bool,
    pub file_watcher: Option<crate::core::file_watcher::FileWatcher>,
}

impl Default for KglanceApp {
    fn default() -> Self {
        Self {
            state: KglanceState::default(),
            registry: Arc::new(ParserRegistry::default()),
            daemon_rx: Arc::new(Mutex::new(None)),
            is_daemon: false,
            is_gui_open: Arc::new(AtomicBool::new(false)),
            window_id: None,
            is_window_opening: false,
            current_content: None,
            video: None,
            ctrl_held: false,
            shift_held: false,
            pending_g: false,
            pending_home: false,
            file_watcher: None,
        }
    }
}

impl KglanceApp {
    pub fn new(
        registry: Arc<ParserRegistry>,
        daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
        initial_paths: &[String],
        is_daemon: bool,
        is_gui_open: Arc<std::sync::atomic::AtomicBool>,
    ) -> (Self, Task<Message>) {
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
            word_wrap: config.ui.word_wrap,
            json_tree_view: config.ui.json_tree_view,

            ..Default::default()
        };

        state
            .cache
            .set_max_bytes(config.cache.max_memory_mb.saturating_mul(1024 * 1024));

        state.read_positions = crate::core::ReadPositions::load();
        state.json.tree_mode = config.ui.json_tree_view;
        state
            .markdown
            .scroll_controller
            .apply_config(&config.scroll);
        state.pdf.scroll_controller.apply_config(&config.scroll);
        state.text.scroll_controller.apply_config(&config.scroll);

        if !initial_paths.is_empty() {
            state.playlist = initial_paths.to_vec();
            state.current_index = 0;
        }

        let app = Self {
            state,
            registry,
            daemon_rx: Arc::new(Mutex::new(daemon_rx)),
            is_daemon,
            is_gui_open,
            window_id: None,
            is_window_opening: false,
            current_content: None,
            video: None,
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

    pub(crate) fn restore_read_position_for(&mut self, path: &str) -> Task<Message> {
        let pos = self.state.read_positions.get(path);
        let scroll_y = pos.map(|p| p.scroll_y).unwrap_or(0.0);
        let chapter = pos.map(|p| p.chapter).unwrap_or(0);

        let applies_content_scroll = match &self.current_content {
            Some(crate::core::PreviewData::Text { .. }) => {
                self.state.text.scroll_y = scroll_y;
                true
            }
            Some(crate::core::PreviewData::Markdown { .. }) => {
                self.state.markdown.scroll_y = scroll_y;
                true
            }
            Some(crate::core::PreviewData::Epub { .. }) => {
                let max = self.state.epub.chapters.len().max(1);
                let target_ch = chapter.min(max - 1);
                crate::features::epub::update::ensure_chapter_loaded(self, target_ch);
                self.state.epub.markdown_state.scroll_y = scroll_y;
                true
            }
            _ => false,
        };

        if applies_content_scroll {
            iced::widget::operation::scroll_to(
                "content_scroll",
                iced::widget::operation::AbsoluteOffset {
                    x: 0.0,
                    y: scroll_y,
                },
            )
        } else {
            Task::none()
        }
    }

    pub fn trigger_preload(&mut self) -> Task<Message> {
        if self.state.cache.max_bytes() == 0 {
            return Task::none();
        }

        let is_rapid = self.state.is_rapid_navigating;

        let lookahead = crate::core::preloader::calculate_dynamic_lookahead(
            self.state.cache.current_bytes(),
            self.state.cache.max_bytes(),
        );

        let indices = crate::core::preloader::calculate_preload_window(
            self.state.current_index,
            self.state.playlist.len(),
            lookahead,
        );

        let mut paths_to_preload = Vec::new();
        for idx in indices {
            if idx < self.state.playlist.len() {
                let path = self.state.playlist[idx].clone();
                if self.state.cache.peek(&path).is_none()
                    && !self.state.pending_preloads.contains(&path)
                    && crate::core::preloader::should_preload_file(&path)
                {
                    self.state.pending_preloads.insert(path.clone());
                    paths_to_preload.push(path);
                }
            }
        }

        if paths_to_preload.is_empty() {
            return Task::none();
        }

        crate::log_debug!(
            "trigger_preload: queueing {} files for sequential background preload: {:?}",
            paths_to_preload.len(),
            paths_to_preload
        );

        let reg = self.registry.clone();
        let current_gen = self.state.generation_id.load(Ordering::Relaxed);
        let generation_id = self.state.generation_id.clone();

        let stream = iced::stream::channel(
            10,
            move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                use iced::futures::SinkExt;

                if is_rapid {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    if generation_id.load(Ordering::Relaxed) != current_gen {
                        return;
                    }
                }

                for target_path in paths_to_preload {
                    if generation_id.load(Ordering::Relaxed) != current_gen {
                        break;
                    }

                    let _permit = crate::features::image::decode_semaphore()
                        .acquire()
                        .await
                        .ok();

                    if generation_id.load(Ordering::Relaxed) != current_gen {
                        break;
                    }

                    let reg_clone = reg.clone();
                    let path_clone = target_path.clone();
                    let gen_clone = generation_id.clone();

                    let content = tokio::task::spawn_blocking(move || {
                        if gen_clone.load(Ordering::Relaxed) != current_gen {
                            return None;
                        }
                        let parsed =
                            FilePreviewer::parse(&*reg_clone, Path::new(&path_clone)).ok()?;

                        if gen_clone.load(Ordering::Relaxed) != current_gen {
                            return None;
                        }

                        if let PreviewData::Image {
                            ref data,
                            width,
                            height,
                            ..
                        } = parsed
                        {
                            if gen_clone.load(Ordering::Relaxed) != current_gen {
                                return None;
                            }

                            let (handle, dec_w, dec_h) =
                                crate::features::image::decode_to_rgba_handle_downsampled(
                                    data,
                                    crate::features::image::MAX_PREVIEW_WIDTH,
                                    crate::features::image::MAX_PREVIEW_HEIGHT,
                                )
                                .unwrap_or_else(|| {
                                    (
                                        iced::widget::image::Handle::from_bytes(data.clone()),
                                        width,
                                        height,
                                    )
                                });
                            if gen_clone.load(Ordering::Relaxed) != current_gen {
                                return None;
                            }
                            let preview = std::sync::Arc::new(parsed);
                            return Some((
                                preview.clone(),
                                Some(crate::core::CachedContent::DecodedImage {
                                    preview: Some(preview),
                                    handle,
                                    width: dec_w,
                                    height: dec_h,
                                }),
                            ));
                        }

                        Some((std::sync::Arc::new(parsed), None))
                    })
                    .await
                    .ok()
                    .flatten();

                    drop(_permit);

                    if generation_id.load(Ordering::Relaxed) != current_gen {
                        break;
                    }

                    if let Some((preview_arc, decoded_cache)) = content {
                        let msg = crate::app::messages::NavigationMsg::PreloadCompleted {
                            path: target_path,
                            content: preview_arc,
                            decoded_cache,
                        }
                        .into();

                        if output.send(msg).await.is_err() {
                            break;
                        }
                    }
                }
            },
        );

        Task::run(stream, |msg| msg)
    }

    pub fn invalidate_render_generations(&self) {
        self.state.generation_id.fetch_add(1, Ordering::Relaxed);
        self.state
            .dir_sync_generation_id
            .fetch_add(1, Ordering::Relaxed);
        self.state.pdf.generation_id.fetch_add(1, Ordering::Relaxed);
        self.state
            .typst
            .pdf
            .generation_id
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .markdown
            .generation_id
            .fetch_add(1, Ordering::Relaxed);
    }

    fn save_current_read_position(&mut self) {
        self.record_read_position();

        if self.state.read_positions_dirty {
            let _ = self.state.read_positions.save();
            self.state.read_positions_dirty = false;
        }
    }

    fn prepare_file_tasks(&mut self, path: &str, content: &PreviewData) -> Vec<Task<Message>> {
        let mut tasks = self.prepare_markdown_tasks(content, path);

        if let Some(task) = self.prepare_pdf_task(content, path) {
            tasks.push(task);
        }

        if let Some(task) = self.prepare_typst_task(content, path) {
            tasks.push(task);
        }

        tasks.extend(self.prepare_media_tasks(path));

        if let Some(task) = self.prepare_sibling_scan_task(path) {
            tasks.push(task);
        }

        if let PreviewData::Image {
            data,
            width,
            height,
            ..
        } = content
        {
            if self.state.image.display_handle.is_none() {
                // Either downsampled preview is active (needs upgrade) or uncached (needs decode)
                let is_rapid_navigating = self.state.is_rapid_navigating;

                let delay_ms = if self.state.image.preview_handle.is_some() {
                    // Downsampled preview is already displayed on screen (<3ms).
                    // Debounce upgrade to full resolution (200ms) so rapid navigation skips it.
                    200
                } else if is_rapid_navigating {
                    // Uncached image during rapid navigation: debounce to avoid decoding skipped files
                    150
                } else {
                    0
                };

                let load_id_val = self.state.image.load_id.load(Ordering::Relaxed);
                let load_id_arc = self.state.image.load_id.clone();
                tasks.push(crate::features::image::spawn_decode_task_delayed(
                    data.clone(),
                    load_id_val,
                    load_id_arc,
                    delay_ms,
                ));
            } else if !self.state.cache.contains(path)
                && let Some(ref handle) = self.state.image.display_handle
            {
                self.state.cache.put(
                    path.to_string(),
                    crate::core::CachedContent::DecodedImage {
                        preview: Some(std::sync::Arc::new(content.clone())),
                        handle: handle.clone(),
                        width: *width,
                        height: *height,
                    },
                );
            }
        }

        if let PreviewData::Json {
            nodes,
            pretty,
            has_parse_error,
            ..
        } = content
        {
            let cache_hit = self
                .state
                .cache
                .get(path)
                .and_then(|c| c.as_parsed_json())
                .map(|(n, p, e)| (n.to_vec(), p.to_string(), e));

            if let Some((nodes, pretty, has_error)) = cache_hit {
                log_debug!("JSON parse cache HIT for {}", path);
                json::populate_state(&mut self.state, &nodes, &pretty, has_error);
            } else {
                log_debug!("JSON parse cache MISS for {}, storing in cache", path);
                self.state.cache.put(
                    path.to_string(),
                    crate::core::CachedContent::ParsedJson {
                        preview: Some(std::sync::Arc::new(content.clone())),
                        nodes: nodes.clone(),
                        pretty: pretty.clone(),
                        has_error: *has_parse_error,
                    },
                );

                crate::features::json::populate_state(
                    &mut self.state,
                    nodes,
                    pretty,
                    *has_parse_error,
                );
            }
        }

        match content {
            PreviewData::Image { .. } | PreviewData::Json { .. } => {}
            _ => {
                if !self.state.cache.contains(path) {
                    self.state.cache.put(
                        path.to_string(),
                        crate::core::CachedContent::Preview(std::sync::Arc::new(content.clone())),
                    );
                }
            }
        }

        tasks
    }

    pub fn handle_file_loaded(&mut self, path: String, content: PreviewData) -> Task<Message> {
        self.state.pending_preloads.clear();
        self.invalidate_render_generations();
        self.save_current_read_position();
        self.update_loaded_file_state(&path, &content);

        let mut tasks = self.prepare_file_tasks(&path, &content);
        tasks.push(self.restore_read_position_for(&path));
        tasks.push(self.trigger_preload());

        Task::batch(tasks)
    }

    fn update_loaded_file_state(&mut self, path: &str, content: &PreviewData) {
        self.state.file_name = path.to_string();
        let path_obj = std::path::Path::new(path);
        if path_obj.is_dir() {
            self.state.active_dir = Some(path_obj.to_path_buf());
        } else if let Some(parent) = path_obj.parent() {
            self.state.active_dir = Some(parent.to_path_buf());
        }
        self.state.content_ready = true;

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
        let current_gen = self.state.markdown.generation_id.load(Ordering::Relaxed);
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
                        let gen_arc = Arc::clone(&self.state.markdown.generation_id);
                        tasks.push(Task::perform(
                            async move {
                                if gen_arc.load(Ordering::Relaxed) != current_gen {
                                    return crate::app::messages::MarkdownMsg::MermaidBlockRendered {
                                        generation_id: current_gen,
                                        index: i,
                                        png_bytes: None,
                                    }
                                    .into();
                                }

                                let png = tokio::task::spawn_blocking(move || {
                                    if gen_arc.load(Ordering::Relaxed) != current_gen {
                                        return None;
                                    }
                                    crate::parsers::markdown::render_mermaid_to_png(
                                        &code, None, prefer_cli,
                                    )
                                })
                                .await
                                .ok()
                                .flatten();
                                crate::app::messages::MarkdownMsg::MermaidBlockRendered {
                                    generation_id: current_gen,
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
                        if crate::core::net::is_remote_url(&path_str) {
                            tasks.push(Task::perform(
                                async move {
                                    let bytes =
                                        crate::core::net::fetch_remote_image(&path_str).await;
                                    crate::app::messages::MarkdownMsg::ImageLoaded {
                                        generation_id: current_gen,
                                        index: i,
                                        png_bytes: bytes,
                                    }
                                    .into()
                                },
                                |msg| msg,
                            ));
                        } else {
                            let resolved = crate::core::utils::resolve_path(&path_str, file_path);

                            tasks.push(Task::perform(
                                async move {
                                    let bytes = tokio::task::spawn_blocking(move || {
                                        std::fs::read(&resolved).ok()
                                    })
                                    .await
                                    .ok()
                                    .flatten();
                                    crate::app::messages::MarkdownMsg::ImageLoaded {
                                        generation_id: current_gen,
                                        index: i,
                                        png_bytes: bytes,
                                    }
                                    .into()
                                },
                                |msg| msg,
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        tasks
    }

    fn prepare_pdf_task(&mut self, content: &PreviewData, path: &str) -> Option<Task<Message>> {
        let is_pdf = matches!(content, PreviewData::Pdf { .. });

        let session_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as usize)
            .unwrap_or(0);
        let disk_cache = crate::features::pdf::PdfDiskCache::new(session_id)
            .ok()
            .map(std::sync::Arc::new);
        self.state.pdf.disk_cache = disk_cache.clone();

        if let PreviewData::Pdf {
            data,
            width,
            height,
            ..
        } = content
            && !data.is_empty()
            && !self.state.pdf.pages.is_empty()
        {
            if let Some(ref dc) = disk_cache {
                let _ = dc.save_page_with_meta(0, data, *width, *height);
            }
            let handle = iced::widget::image::Handle::from_bytes(data.clone());
            self.state.pdf.pages.insert(
                0,
                crate::core::PageCacheEntry {
                    width: *width,
                    height: *height,
                    handle,
                },
                0,
            );
        }

        if is_pdf && self.state.pdf.page_count > 0 {
            let page_count = self.state.pdf.page_count;
            let pdf_path = path.to_string();
            let visible_page = self.state.pdf.visible_page.clone();
            let generation_id = self.state.pdf.generation_id.clone();

            let visible_thumb_page = self.state.pdf.visible_thumb_page.clone();
            let thumb_generation_id = self.state.pdf.thumb_generation_id.clone();
            let thumb_task = if self.state.pdf.sidebar_visible
                && self.state.pdf.sidebar_mode == crate::core::types::PdfSidebarMode::Thumbnails
            {
                Some(crate::features::pdf::handler::lazy_load_thumbnails(
                    pdf_path.clone(),
                    page_count,
                    visible_thumb_page,
                    thumb_generation_id,
                ))
            } else {
                None
            };

            if page_count > 1 {
                self.state.pdf.active_page_tasks =
                    self.state.pdf.active_page_tasks.saturating_add(1);
                let page_task = crate::features::pdf::handler::lazy_load_pages(
                    pdf_path,
                    page_count,
                    visible_page,
                    generation_id,
                    disk_cache,
                );
                if let Some(tt) = thumb_task {
                    Some(Task::batch([page_task, tt]))
                } else {
                    Some(page_task)
                }
            } else {
                thumb_task
            }
        } else {
            None
        }
    }

    fn prepare_typst_task(&mut self, content: &PreviewData, path: &str) -> Option<Task<Message>> {
        let is_typst = matches!(content, PreviewData::Typst { .. });

        let session_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as usize)
            .unwrap_or(0);
        let disk_cache = crate::features::pdf::PdfDiskCache::new(session_id)
            .ok()
            .map(std::sync::Arc::new);
        self.state.typst.pdf.disk_cache = disk_cache.clone();

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
            if let Some(ref dc) = disk_cache {
                let _ = dc.save_page_with_meta(0, data, *width, *height);
            }
            let handle = iced::widget::image::Handle::from_bytes(data.clone());
            self.state.typst.pdf.pages.insert(
                0,
                crate::core::PageCacheEntry {
                    width: *width,
                    height: *height,
                    handle,
                },
                0,
            );
        }

        if is_typst && self.state.typst.pdf.page_count > 0 && self.state.typst.error.is_none() {
            let page_count = self.state.typst.pdf.page_count;
            let typst_path = path.to_string();
            let visible_page = self.state.typst.pdf.visible_page.clone();
            let generation_id = self.state.typst.pdf.generation_id.clone();

            let thumb_task = if self.state.typst.pdf.sidebar_visible
                && self.state.typst.pdf.sidebar_mode
                    == crate::core::types::PdfSidebarMode::Thumbnails
            {
                Some(crate::features::pdf::handler::lazy_load_thumbnails(
                    typst_path.clone(),
                    page_count,
                    visible_page.clone(),
                    generation_id.clone(),
                ))
            } else {
                None
            };

            if page_count > 1 {
                self.state.typst.pdf.active_page_tasks =
                    self.state.typst.pdf.active_page_tasks.saturating_add(1);
                let page_task = crate::features::typst::handler::lazy_load_typst_pages(
                    typst_path,
                    page_count,
                    visible_page,
                    generation_id,
                    disk_cache,
                );
                if let Some(tt) = thumb_task {
                    Some(Task::batch([page_task, tt]))
                } else {
                    Some(page_task)
                }
            } else {
                thumb_task
            }
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
        self.state.media.error = None;

        if is_video || is_audio {
            match crate::features::video::handler::load_video(path) {
                Ok(video) => {
                    self.state.media.playing = true;
                    self.video = Some(video);
                }
                Err(e) => {
                    crate::log_error!("Failed to load video: {e}");
                    self.state.media.error = Some(e);
                    self.video = None;
                }
            }
        } else {
            self.video = None;
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
            } else if !self.is_window_opening {
                self.is_window_opening = true;
                vec![self.create_new_window()]
            } else {
                vec![]
            }
        } else if let Some(id) = self.window_id {
            vec![iced::window::gain_focus(id)]
        } else {
            vec![]
        }
    }

    fn prepare_sibling_scan_task(&mut self, path: &str) -> Option<Task<Message>> {
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
        let (preview_body, edge_to_edge) = if let Some(content) = &self.current_content {
            let edge_to_edge = matches!(
                content,
                PreviewData::Media { .. }
                    | PreviewData::Image { .. }
                    | PreviewData::Pdf { .. }
                    | PreviewData::Typst { .. }
                    | PreviewData::Markdown { .. }
                    | PreviewData::Epub { .. }
            );
            let body: Element<'_, Message> = match content {
                PreviewData::Text { .. } => crate::ui::views::view_text(
                    &self.state.text,
                    self.state.app_theme,
                    self.state.font_size,
                    self.state.font_family_mono.as_deref(),
                    self.state.word_wrap,
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
                PreviewData::Image { .. } => crate::ui::views::view_image(
                    &self.state.image,
                    self.state.font_family.as_deref(),
                ),
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
                    self.state.word_wrap,
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
                    self.state.word_wrap,
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
                    self.video.as_ref(),
                    *width,
                    *height,
                ),
                PreviewData::Error(err) => iced::widget::text(err).size(18).into(),
            };
            (body, edge_to_edge)
        } else {
            (iced::widget::text("No file loaded.").size(18).into(), false)
        };

        let is_mod = self.ctrl_held;
        let preview_body =
            crate::ui::components::scroll_pane::ScrollFilter::new(preview_body, is_mod).into();

        crate::ui::window::view_window(&self.state, preview_body, edge_to_edge)
    }

    /// Variant for [`iced::daemon`] which requires a `window::Id` parameter.
    pub fn view_daemon(&self, _window_id: iced::window::Id) -> Element<'_, Message> {
        self.view()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let dbus_sub = subscription::from_recipe(crate::dbus::recipe::DaemonRecipe::new(
            self.daemon_rx.clone(),
        ));

        let event_sub = iced::window::events()
            .map(|(id, event)| crate::app::messages::SystemMsg::WindowEvent(id, event).into());

        let resize_sub = iced_window::resize_events().map(|(_id, size)| {
            crate::app::messages::SystemMsg::WindowResized(size.width, size.height).into()
        });

        let global_event_sub = iced::event::listen_with(|event, _status, _window_id| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(crate::app::messages::SystemMsg::KeyPressed(key, modifiers).into())
            }
            iced::Event::Keyboard(iced::keyboard::Event::KeyReleased {
                key, modifiers, ..
            }) => {
                if matches!(
                    key,
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Shift)
                ) {
                    Some(Message::ShiftHeldChanged(false))
                } else if matches!(
                    key,
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Control)
                ) {
                    Some(Message::CtrlHeldChanged(false))
                } else {
                    Some(Message::ModifiersUpdated(modifiers))
                }
            }
            iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                Some(Message::ModifiersUpdated(modifiers))
            }
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Some(Message::MouseMoved(position.x, position.y))
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                Some(Message::MousePressed(0.0, 0.0))
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                Some(Message::MouseReleased)
            }
            iced::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                let (dx, dy) = match delta {
                    iced::mouse::ScrollDelta::Lines { x, y } => (x * 80.0, y * 80.0),
                    iced::mouse::ScrollDelta::Pixels { x, y } => (
                        x * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER,
                        y * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER,
                    ),
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

        let smooth_scroll_sub = if self.is_smooth_scrollable() {
            let md_state = crate::features::markdown::update::active_markdown_state(self);
            if md_state.smooth_scroll.is_animating
                || md_state.scroll_controller.is_animating()
                || md_state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging
            {
                iced::time::every(std::time::Duration::from_millis(
                    crate::features::markdown::update::SMOOTH_SCROLL_TICK_MS,
                ))
                .map(|_| {
                    crate::app::messages::MarkdownMsg::SmoothScrollTick(std::time::Instant::now())
                        .into()
                })
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        let pdf_scroll_sub = if matches!(
            self.current_content,
            Some(crate::core::PreviewData::Pdf { .. })
        ) {
            let pdf = &self.state.pdf;
            if pdf.smooth_scroll.is_animating
                || pdf.scroll_controller.is_animating()
                || pdf.scroll_controller.state() == crate::core::scroll::GestureState::Dragging
            {
                iced::time::every(std::time::Duration::from_millis(
                    crate::features::markdown::update::SMOOTH_SCROLL_TICK_MS,
                ))
                .map(|_| {
                    crate::app::messages::PdfMsg::SmoothScrollTick(std::time::Instant::now()).into()
                })
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        let text_scroll_sub = if matches!(
            self.current_content,
            Some(crate::core::PreviewData::Text { .. })
        ) {
            let text = &self.state.text;
            if text.smooth_scroll.is_animating
                || text.scroll_controller.is_animating()
                || text.scroll_controller.state() == crate::core::scroll::GestureState::Dragging
            {
                iced::time::every(std::time::Duration::from_millis(
                    crate::features::markdown::update::SMOOTH_SCROLL_TICK_MS,
                ))
                .map(|_| {
                    crate::app::messages::TextMsg::SmoothScrollTick(std::time::Instant::now())
                        .into()
                })
            } else {
                Subscription::none()
            }
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
            global_event_sub,
            file_watcher_sub,
            auto_scroll_sub,
            smooth_scroll_sub,
            pdf_scroll_sub,
            text_scroll_sub,
            read_positions_sub,
        ])
    }

    pub fn theme(&self) -> Theme {
        Theme::from(self.state.app_theme)
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
        let mut state = KglanceState::default();
        if let Some(ref c) = content {
            c.populate_state(&mut state);
        }
        let registry = std::sync::Arc::new(ParserRegistry::new());
        KglanceApp {
            state,
            registry,
            daemon_rx: std::sync::Arc::new(std::sync::Mutex::new(None)),
            is_daemon: false,
            is_gui_open: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            window_id: None,
            is_window_opening: false,
            current_content: content,
            video: None,
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
                file_href: "chapter.xhtml".to_string(),
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

    pub fn text_content(content: &str, lang: &str) -> PreviewData {
        let lines: Vec<String> = (1..=content.lines().count())
            .map(|n| n.to_string())
            .collect();
        PreviewData::Text {
            content: content.to_string(),
            line_numbers: lines.join("\n"),
            language: lang.to_string(),
        }
    }

    #[test]
    fn state_has_read_positions_fields() {
        let st = KglanceState::default();
        assert!(!st.read_positions_dirty);
        assert_eq!(st.read_positions.get("/x").map(|p| p.scroll_y), None);
    }

    #[test]
    fn restore_restores_markdown_scroll() {
        let mut app = test_app(Some(markdown_content("# hi")));
        app.state.file_name = "/tmp/x.md".into();
        app.state.read_positions.insert(
            "/tmp/x.md".into(),
            crate::core::ReadPosition {
                scroll_y: 42.5,
                chapter: 0,
            },
        );
        let _task = app.restore_read_position_for("/tmp/x.md");
        assert_eq!(app.state.markdown.scroll_y, 42.5);
    }
}
