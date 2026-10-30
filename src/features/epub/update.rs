use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

use super::anchor::*;

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.sidebar_visible = !app.state.epub.sidebar_visible;
    if app.state.epub.sidebar_visible {
        app.state.file_tree.visible = false;
    }

    let content_width = app.state.epub_content_width();
    let old_y = app.state.epub.markdown_state.scroll_y;

    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        crate::features::markdown::recompute_markdown_layout(
            &mut app.state.epub.markdown_state,
            &app.state.epub.continuous_blocks,
            app.state.font_size,
            content_width,
        );
        app.state.epub.markdown_state.scroll_y = old_y;
        return operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset { x: 0.0, y: old_y },
        );
    }

    let idx = app.state.epub.active_chapter;
    if let Some(active_ch) = app.state.epub.chapters.get(idx) {
        crate::features::markdown::recompute_markdown_layout(
            &mut app.state.epub.markdown_state,
            &active_ch.blocks,
            app.state.font_size,
            content_width,
        );
        app.state.epub.markdown_state.scroll_y = old_y;
        return operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset { x: 0.0, y: old_y },
        );
    }

    let y = app.state.epub.markdown_state.scroll_y;
    operation::scroll_to("content_scroll", operation::AbsoluteOffset { x: 0.0, y })
}

pub fn ensure_chapter_loaded(app: &mut KglanceApp, idx: usize) {
    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        return;
    }
    if idx < app.state.epub.chapters.len() {
        if app.state.epub.chapters[idx].blocks.is_empty() {
            let path_str = app.state.file_name.clone();
            let chapter_file_href = app.state.epub.chapters[idx].file_href.clone();
            let chapter_anchor = app.state.epub.chapters[idx].anchor.clone();

            let existing_blocks = app
                .state
                .epub
                .chapters
                .iter()
                .find(|ch| ch.file_href == chapter_file_href && !ch.blocks.is_empty())
                .map(|ch| ch.blocks.clone());

            if let Some(blocks) = existing_blocks {
                if let Some(crate::core::PreviewData::Epub {
                    chapters: preview_chapters,
                    ..
                }) = &mut app.current_content
                    && let Some(preview_ch) = preview_chapters.get_mut(idx)
                {
                    preview_ch.blocks = blocks.clone();
                }
                app.state.epub.chapters[idx].blocks = blocks;
            } else if !path_str.is_empty() {
                let path = std::path::Path::new(&path_str);
                if let Ok((new_blocks, new_images, new_styles)) =
                    crate::features::epub::parser::load_chapter_content_and_images_from_epub(
                        path,
                        &chapter_file_href,
                        chapter_anchor.as_deref(),
                    )
                {
                    if let Some(crate::core::PreviewData::Epub {
                        images,
                        chapters: preview_chapters,
                        ..
                    }) = &mut app.current_content
                    {
                        for (k, v) in new_images {
                            images.insert(k, v);
                        }
                        if let Some(preview_ch) = preview_chapters.get_mut(idx) {
                            preview_ch.blocks = new_blocks.clone();
                        }
                    }
                    app.state.epub.markdown_state.block_styles = new_styles;
                    app.state.epub.chapters[idx].blocks = new_blocks;
                }
            }
        }

        app.state.epub.active_chapter = idx;

        if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content {
            crate::features::epub::state::ensure_chapter_images(
                &mut app.state.epub.markdown_state,
                &app.state.epub.chapters,
                idx,
                images,
            );
        }

        if let Some(active_ch) = app.state.epub.chapters.get(idx) {
            let content_width = app.state.epub_content_width();
            crate::features::markdown::recompute_markdown_layout(
                &mut app.state.epub.markdown_state,
                &active_ch.blocks,
                app.state.font_size,
                content_width,
            );
        }
    }
}

pub fn handle_chapter_clicked(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if idx < app.state.epub.chapters.len() {
        if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
            app.state.epub.active_chapter = idx;
            app.record_read_position();

            let target_block_idx = app
                .state
                .epub
                .chapter_target_offsets
                .get(idx)
                .or_else(|| app.state.epub.chapter_block_offsets.get(idx))
                .copied()
                .unwrap_or(0);
            let target_y = app
                .state
                .epub
                .markdown_state
                .block_y_offsets
                .get(target_block_idx)
                .copied()
                .unwrap_or(0.0);

            let max_y = crate::core::scroll::max_scroll_y(
                app.state.epub.markdown_state.total_content_height,
                app.state.epub.markdown_state.viewport_height,
            );
            let clamped_y = if max_y > 0.0 {
                target_y.clamp(0.0, max_y)
            } else {
                target_y
            };

            app.state.epub.markdown_state.scroll_y = clamped_y;
            app.state
                .epub
                .markdown_state
                .scroll_controller
                .stop(clamped_y);
            app.state.epub.markdown_state.smooth_scroll.stop(clamped_y);

            return operation::scroll_to(
                "content_scroll",
                operation::AbsoluteOffset {
                    x: 0.0,
                    y: clamped_y,
                },
            );
        }

        ensure_chapter_loaded(app, idx);
        app.record_read_position();

        let chapter = &app.state.epub.chapters[idx];
        let target_y = if let Some(block_idx) = crate::features::epub::parser::find_block_index(
            &chapter.blocks,
            chapter.anchor.as_deref(),
            Some(&chapter.title),
        ) {
            app.state
                .epub
                .markdown_state
                .block_y_offsets
                .get(block_idx)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let max_y = crate::core::scroll::max_scroll_y(
            app.state.epub.markdown_state.total_content_height,
            app.state.epub.markdown_state.viewport_height,
        );
        let clamped_y = if max_y > 0.0 {
            target_y.clamp(0.0, max_y)
        } else {
            target_y
        };

        app.state.epub.markdown_state.scroll_y = clamped_y;
        app.state
            .epub
            .markdown_state
            .scroll_controller
            .stop(clamped_y);
        app.state.epub.markdown_state.smooth_scroll.stop(clamped_y);

        let scroll_task = operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset {
                x: 0.0,
                y: clamped_y,
            },
        );

        let mut tasks = vec![scroll_task];
        let current_gen = app
            .state
            .epub
            .markdown_state
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let mut all_image_paths = Vec::new();
        for block in &chapter.blocks {
            crate::features::markdown::parser::collect_image_paths(block, &mut all_image_paths);
        }
        all_image_paths.sort();
        all_image_paths.dedup();

        let gen_arc = app.state.epub.markdown_state.generation_id.clone();
        for path_str in all_image_paths {
            if crate::core::net::is_remote_url(&path_str) {
                tasks.push(crate::features::markdown::spawn_image_load_task(
                    path_str,
                    None,
                    gen_arc.clone(),
                    current_gen,
                ));
            }
        }

        return Task::batch(tasks);
    }
    Task::none()
}

pub fn handle_chapter_toggle_collapse(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if app.state.epub.collapsed_chapters.contains(&idx) {
        app.state.epub.collapsed_chapters.remove(&idx);
    } else {
        app.state.epub.collapsed_chapters.insert(idx);
    }
    Task::none()
}

pub fn handle_reading_mode_changed(
    app: &mut KglanceApp,
    mode: crate::core::config::EpubReadingMode,
) -> Task<Message> {
    if app.state.epub.reading_mode == mode && app.state.epub_reading_mode == mode {
        return Task::none();
    }

    let old_mode = app.state.epub.reading_mode;
    app.state.epub_reading_mode = mode;
    app.state.epub.reading_mode = mode;

    if !app.state.file_name.is_empty()
        && let Some(pos) = app.state.read_positions.get(&app.state.file_name)
    {
        app.state.read_positions.insert(
            app.state.file_name.clone(),
            crate::core::ReadPosition {
                reading_mode: Some(mode),
                ..pos
            },
        );
        app.state.read_positions_dirty = true;
    }

    let anchor = match old_mode {
        crate::core::config::EpubReadingMode::Continuous => {
            capture_continuous_anchor(&app.state.epub)
        }
        crate::core::config::EpubReadingMode::SingleChapter => {
            capture_single_chapter_anchor(&app.state.epub)
        }
    };

    let (title_clone, author_clone, chapters_clone) = {
        let Some(crate::core::PreviewData::Epub {
            title,
            author,
            chapters,
            ..
        }) = &app.current_content
        else {
            return Task::none();
        };

        let chapters_clone = if app.state.epub.chapters.iter().any(|c| !c.blocks.is_empty()) {
            app.state.epub.chapters.clone()
        } else {
            chapters.clone()
        };
        (title.clone(), author.clone(), chapters_clone)
    };

    // Reset scroll_y so populate_state does not reuse mismatched coordinates from previous mode
    app.state.epub.markdown_state.scroll_y = 0.0;

    if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content {
        crate::features::epub::state::populate_state(
            &mut app.state,
            &title_clone,
            &author_clone,
            &chapters_clone,
            anchor.chapter_idx,
            images,
        );
    }

    if mode == crate::core::config::EpubReadingMode::SingleChapter
        && app
            .state
            .epub
            .chapters
            .get(anchor.chapter_idx)
            .is_some_and(|c| c.blocks.is_empty())
    {
        ensure_chapter_loaded(app, anchor.chapter_idx);
    }

    let target_y = match mode {
        crate::core::config::EpubReadingMode::Continuous => {
            resolve_continuous_anchor_scroll_y(&app.state.epub, &anchor)
        }
        crate::core::config::EpubReadingMode::SingleChapter => {
            resolve_single_chapter_anchor_scroll_y(&app.state.epub, &anchor)
        }
    };

    app.state.epub.active_chapter = anchor.chapter_idx;
    app.state.epub.scroll_y = target_y;
    app.state.epub.markdown_state.scroll_y = target_y;
    app.state
        .epub
        .markdown_state
        .scroll_controller
        .stop(target_y);
    app.state.epub.markdown_state.smooth_scroll.stop(target_y);

    if let Some(crate::core::PreviewData::Epub { active_chapter, .. }) = &mut app.current_content {
        *active_chapter = anchor.chapter_idx;
    }

    app.record_read_position();

    operation::scroll_to(
        "content_scroll",
        operation::AbsoluteOffset {
            x: 0.0,
            y: target_y,
        },
    )
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.epub.search.query = query.clone();
    app.state.epub.search.active_match_index = 0;
    app.state.epub.markdown_state.search_query = query.clone();
    app.state.epub.markdown_state.search_match_index = 0;

    if query.trim().is_empty() {
        app.state.epub.search.matches.clear();
        app.state.epub.markdown_state.search_match_count = 0;
        app.state.epub.markdown_state.search_match_blocks.clear();
        app.state.epub.markdown_state.search_info.clear();
        return Task::none();
    }

    let matches = super::search::search_epub_chapters(&app.state.epub.chapters, &query);
    let total = matches.len();

    // Map match blocks for current view highlighting
    let mut match_blocks = Vec::new();
    let is_continuous =
        app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous;
    for m in &matches {
        if is_continuous {
            let start = app
                .state
                .epub
                .chapter_block_offsets
                .get(m.chapter_index)
                .copied()
                .unwrap_or(0);
            match_blocks.push(start + m.block_index);
        } else if m.chapter_index == app.state.epub.active_chapter {
            match_blocks.push(m.block_index);
        }
    }

    app.state.epub.search.matches = matches;
    app.state.epub.markdown_state.search_match_count = total;
    app.state.epub.markdown_state.search_match_blocks = match_blocks;
    app.state.epub.markdown_state.search_info = if total > 0 {
        format!("1/{total}")
    } else {
        String::new()
    };

    Task::none()
}

pub fn handle_search_next(app: &mut KglanceApp) -> Task<Message> {
    let total = app.state.epub.search.total_matches();
    if total == 0 {
        return Task::none();
    }
    let next_idx = (app.state.epub.search.active_match_index + 1) % total;
    jump_to_search_match(app, next_idx)
}

pub fn handle_search_prev(app: &mut KglanceApp) -> Task<Message> {
    let total = app.state.epub.search.total_matches();
    if total == 0 {
        return Task::none();
    }
    let prev_idx = if app.state.epub.search.active_match_index == 0 {
        total - 1
    } else {
        app.state.epub.search.active_match_index - 1
    };
    jump_to_search_match(app, prev_idx)
}

pub fn handle_search_clear(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.search.clear();
    app.state.epub.markdown_state.search_query.clear();
    app.state.epub.markdown_state.search_match_count = 0;
    app.state.epub.markdown_state.search_match_index = 0;
    app.state.epub.markdown_state.search_match_blocks.clear();
    app.state.epub.markdown_state.search_info.clear();
    Task::none()
}

pub fn handle_search_result_clicked(app: &mut KglanceApp, match_idx: usize) -> Task<Message> {
    jump_to_search_match(app, match_idx)
}

fn jump_to_search_match(app: &mut KglanceApp, match_idx: usize) -> Task<Message> {
    let Some(m) = app.state.epub.search.matches.get(match_idx).cloned() else {
        return Task::none();
    };

    app.state.epub.search.active_match_index = match_idx;
    app.state.epub.markdown_state.search_match_index = match_idx;
    app.state.epub.markdown_state.search_info = format!(
        "{}/{}",
        match_idx + 1,
        app.state.epub.search.total_matches()
    );

    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        let ch_start = app
            .state
            .epub
            .chapter_block_offsets
            .get(m.chapter_index)
            .copied()
            .unwrap_or(0);
        let target_block = ch_start + m.block_index;
        let raw_block_y = app
            .state
            .epub
            .markdown_state
            .block_y_offsets
            .get(target_block)
            .copied()
            .unwrap_or(0.0);

        let vh = app.state.epub.markdown_state.viewport_height;
        let offset_padding = if vh > 0.0 {
            (vh * 0.25).max(40.0)
        } else {
            40.0
        };
        let target_y = (raw_block_y - offset_padding).max(0.0);

        let max_y = crate::core::scroll::max_scroll_y(
            app.state.epub.markdown_state.total_content_height,
            vh,
        );
        let clamped_y = if max_y > 0.0 {
            target_y.clamp(0.0, max_y)
        } else {
            target_y
        };

        app.state.epub.active_chapter = m.chapter_index;
        let current_y = app.state.epub.markdown_state.scroll_y;
        app.state.epub.markdown_state.scroll_y = clamped_y;
        app.state
            .epub
            .markdown_state
            .scroll_controller
            .stop(clamped_y);
        app.state
            .epub
            .markdown_state
            .smooth_scroll
            .start_navigation(current_y, clamped_y, max_y);

        // Pre-synchronize virtual window to clamped_y so target_block is immediately in DOM
        crate::features::markdown::view::window::sync_window(
            &mut app.state.epub.markdown_state.virtual_window,
            &app.state.epub.markdown_state.block_y_offsets,
            clamped_y,
            vh,
            crate::features::epub::view::content::VIRTUAL_THRESHOLD,
        );

        operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset {
                x: 0.0,
                y: clamped_y,
            },
        )
    } else {
        if m.chapter_index != app.state.epub.active_chapter {
            ensure_chapter_loaded(app, m.chapter_index);
            app.state.epub.active_chapter = m.chapter_index;
        }
        let raw_block_y = app
            .state
            .epub
            .markdown_state
            .block_y_offsets
            .get(m.block_index)
            .copied()
            .unwrap_or(0.0);

        let vh = app.state.epub.markdown_state.viewport_height;
        let offset_padding = if vh > 0.0 {
            (vh * 0.25).max(40.0)
        } else {
            40.0
        };
        let target_y = (raw_block_y - offset_padding).max(0.0);

        let max_y = crate::core::scroll::max_scroll_y(
            app.state.epub.markdown_state.total_content_height,
            vh,
        );
        let clamped_y = if max_y > 0.0 {
            target_y.clamp(0.0, max_y)
        } else {
            target_y
        };

        let current_y = app.state.epub.markdown_state.scroll_y;
        app.state.epub.markdown_state.scroll_y = clamped_y;
        app.state
            .epub
            .markdown_state
            .scroll_controller
            .stop(clamped_y);
        app.state
            .epub
            .markdown_state
            .smooth_scroll
            .start_navigation(current_y, clamped_y, max_y);

        crate::features::markdown::view::window::sync_window(
            &mut app.state.epub.markdown_state.virtual_window,
            &app.state.epub.markdown_state.block_y_offsets,
            clamped_y,
            vh,
            crate::features::epub::view::content::VIRTUAL_THRESHOLD,
        );

        operation::scroll_to(
            "content_scroll",
            operation::AbsoluteOffset {
                x: 0.0,
                y: clamped_y,
            },
        )
    }
}

pub fn handle_open_image_lightbox(app: &mut KglanceApp, path: String) -> Task<Message> {
    let all_images = collect_all_epub_image_paths(app);
    let filename = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&path);

    let current_index = all_images
        .iter()
        .position(|p| {
            p == &path
                || std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(p)
                    == filename
        })
        .unwrap_or(0);

    let Some(image_bytes) = get_epub_image_bytes(app, &path) else {
        return Task::none();
    };

    let image_state = create_lightbox_image_state(&image_bytes);

    app.state.epub.lightbox = Some(crate::features::epub::types::EpubLightboxState {
        active_path: path,
        all_image_paths: all_images,
        current_index,
        image_state,
    });

    Task::none()
}

pub fn handle_close_image_lightbox(app: &mut KglanceApp) -> Task<Message> {
    app.state.epub.lightbox = None;
    Task::none()
}

pub fn handle_next_lightbox_image(app: &mut KglanceApp) -> Task<Message> {
    let (next_idx, next_path) = {
        let Some(lightbox) = &app.state.epub.lightbox else {
            return Task::none();
        };

        if lightbox.all_image_paths.is_empty() {
            return Task::none();
        }

        let idx = (lightbox.current_index + 1) % lightbox.all_image_paths.len();
        let path = lightbox.all_image_paths[idx].clone();
        (idx, path)
    };

    if let Some(bytes) = get_epub_image_bytes(app, &next_path)
        && let Some(lightbox) = &mut app.state.epub.lightbox
    {
        lightbox.active_path = next_path;
        lightbox.current_index = next_idx;
        lightbox.image_state = create_lightbox_image_state(&bytes);
    }

    Task::none()
}

pub fn handle_prev_lightbox_image(app: &mut KglanceApp) -> Task<Message> {
    let (prev_idx, prev_path) = {
        let Some(lightbox) = &app.state.epub.lightbox else {
            return Task::none();
        };

        if lightbox.all_image_paths.is_empty() {
            return Task::none();
        }

        let len = lightbox.all_image_paths.len();
        let idx = if lightbox.current_index == 0 {
            len.saturating_sub(1)
        } else {
            lightbox.current_index - 1
        };
        let path = lightbox.all_image_paths[idx].clone();
        (idx, path)
    };

    if let Some(bytes) = get_epub_image_bytes(app, &prev_path)
        && let Some(lightbox) = &mut app.state.epub.lightbox
    {
        lightbox.active_path = prev_path;
        lightbox.current_index = prev_idx;
        lightbox.image_state = create_lightbox_image_state(&bytes);
    }

    Task::none()
}

pub fn handle_sidebar_tab_changed(
    app: &mut KglanceApp,
    tab: crate::features::epub::types::EpubSidebarTab,
) -> Task<Message> {
    app.state.epub.sidebar_tab = tab;
    Task::none()
}

fn collect_all_epub_image_paths(app: &KglanceApp) -> Vec<String> {
    let mut paths = Vec::new();

    // From active continuous blocks or all chapters
    if app.state.epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
        paths = crate::features::epub::parser::extract_images_from_blocks(
            &app.state.epub.continuous_blocks,
        );
    } else {
        for ch in &app.state.epub.chapters {
            let ch_paths = crate::features::epub::parser::extract_images_from_blocks(&ch.blocks);
            for p in ch_paths {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }
    }

    // Also include any images in PreviewData that might not be directly in blocks
    if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content {
        for k in images.keys() {
            if !paths.contains(k) {
                paths.push(k.clone());
            }
        }
    }

    paths
}

fn get_epub_image_bytes(app: &KglanceApp, path: &str) -> Option<Vec<u8>> {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);

    if let Some(crate::core::PreviewData::Epub { images, .. }) = &app.current_content
        && let Some(bytes) = images.get(path).or_else(|| images.get(filename))
    {
        return Some(bytes.clone());
    }

    // Fallback: extract on-demand from the EPUB archive on disk
    if !app.state.file_name.is_empty() {
        let epub_path = std::path::Path::new(&app.state.file_name);
        if let Ok(file) = std::fs::File::open(epub_path)
            && let Ok(mut archive) = zip::ZipArchive::new(file)
        {
            let normalized_target = path.replace('\\', "/");
            let target_filename = filename.to_lowercase();

            let file_names: Vec<String> = archive
                .file_names()
                .flatten()
                .map(|name| name.to_string())
                .collect();

            for entry_name in file_names {
                let entry_clean = entry_name.replace('\\', "/");
                let entry_file = std::path::Path::new(&entry_clean)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&entry_clean)
                    .to_lowercase();

                let matches = entry_clean == normalized_target
                    || entry_clean.ends_with(&normalized_target)
                    || normalized_target.ends_with(&entry_clean)
                    || entry_file == target_filename;

                if matches && let Ok(mut zf) = archive.by_name(&entry_name) {
                    use std::io::Read;
                    let mut buf = Vec::new();
                    if zf.read_to_end(&mut buf).is_ok() {
                        return Some(buf);
                    }
                }
            }
        }
    }

    None
}

fn create_lightbox_image_state(bytes: &[u8]) -> crate::features::image::ImageState {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    let (display_handle, width, height) =
        if let Some((h, w, h_dim)) = crate::features::image::decode_to_rgba_handle(bytes) {
            (Some(h), w, h_dim)
        } else {
            (
                Some(iced::widget::image::Handle::from_bytes(bytes.to_vec())),
                0,
                0,
            )
        };

    crate::features::image::ImageState {
        handle: display_handle.clone(),
        preview_handle: None,
        image_bytes: bytes.to_vec(),
        exif_content: String::new(),
        exif_parsed_lines: Vec::new(),
        format_info: String::new(),
        load_state: crate::features::image::ImageLoadState::Ready,
        load_id: Arc::new(AtomicU64::new(0)),
        display_handle,
        display_width: width,
        display_height: height,
        camera: crate::features::image::Camera::new(),
        show_info: false,
        animated_gif: None,
        animation_start_time: None,
        prev_frame_handle: None,
    }
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
