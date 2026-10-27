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

            let chapter = &app.state.epub.chapters[idx];
            let start_block_idx = app
                .state
                .epub
                .chapter_block_offsets
                .get(idx)
                .copied()
                .unwrap_or(0);
            let local_block_offset = crate::features::epub::parser::find_block_index(
                &chapter.blocks,
                chapter.anchor.as_deref(),
                Some(&chapter.title),
            )
            .unwrap_or(0);
            let target_block_idx = start_block_idx + local_block_offset;
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
            tasks.push(crate::features::markdown::spawn_image_load_task(
                path_str,
                None,
                gen_arc.clone(),
                current_gen,
            ));
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

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
