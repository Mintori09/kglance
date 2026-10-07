use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

#[inline]
pub fn active_pdf_state_mut(app: &mut KglanceApp) -> &mut crate::core::PdfState {
    &mut app.state.pdf
}

#[inline]
pub fn active_pdf_state(app: &KglanceApp) -> &crate::core::PdfState {
    &app.state.pdf
}

pub const MAX_CACHED_PAGES: usize = crate::core::types::PageCache::MAX_COUNT;

/// Evicts the furthest pages from `current_page` when total cached pages exceed `MAX_CACHED_PAGES` or byte limit.
pub fn evict_distant_pages(pdf_state: &mut crate::core::PdfState, current_page: usize) {
    pdf_state.pages.evict(current_page);
}

/// Promotes a page from Tier 1 disk cache to Tier 2 UI GPU handle if available.
pub fn promote_page_from_disk_if_cached(
    pdf_state: &mut crate::core::PdfState,
    page_index: usize,
) -> bool {
    if pdf_state.pages.is_cached(page_index) {
        return false;
    }
    if let Some(ref disk_cache) = pdf_state.disk_cache
        && let Ok(cached) = disk_cache.load_page_with_meta(page_index)
    {
        let handle = iced::widget::image::Handle::from_bytes(cached.png_bytes);
        let current_page = pdf_state
            .visible_page
            .load(std::sync::atomic::Ordering::Relaxed);
        pdf_state.pages.insert(
            page_index,
            crate::core::PageCacheEntry {
                width: cached.width,
                height: cached.height,
                handle,
            },
            current_page,
        );
        return true;
    }
    false
}

/// Promotes pages within PRELOAD_RADIUS from disk cache to UI GPU handle, and evicts distant pages.
pub fn promote_window_pages(pdf_state: &mut crate::core::PdfState, current_page: usize) {
    let count = pdf_state.page_count;
    if count == 0 {
        return;
    }
    let radius = crate::features::pdf::lazy_handler::PRELOAD_RADIUS;
    let start = current_page.saturating_sub(radius);
    let end = (current_page + radius).min(count.saturating_sub(1));
    for p in start..=end {
        promote_page_from_disk_if_cached(pdf_state, p);
    }
    evict_distant_pages(pdf_state, current_page);
}

pub fn handle_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let pdf_state = active_pdf_state_mut(app);
    if pdf_state.viewport_height <= 0.0 {
        return Task::none();
    }
    let vh = pdf_state.viewport_height;
    let extent = crate::core::scroll::ViewportExtent {
        content_height: pdf_state.total_content_height,
        viewport_height: vh,
    };
    let max_y = extent.max_scroll_y();

    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => {
            if y.abs() > f32::EPSILON {
                let line_height =
                    (vh * crate::core::scroll::WHEEL_SCROLL_VIEWPORT_FRACTION).clamp(50.0, 240.0);
                let step = -y * line_height;
                pdf_state
                    .smooth_scroll
                    .start_interactive(pdf_state.scroll_y, step, max_y);
                if !pdf_state.smooth_scroll.is_animating() {
                    let new_y = pdf_state.smooth_scroll.position_y();
                    pdf_state.scroll_y = new_y;
                    let count = pdf_state.page_count;
                    if count > 0 && !pdf_state.page_y_offsets.is_empty() {
                        let page_index = crate::features::pdf::viewport::find_visible_page(
                            &pdf_state.page_y_offsets,
                            new_y,
                            vh,
                            0.3,
                        );
                        pdf_state
                            .visible_page
                            .store(page_index, std::sync::atomic::Ordering::Relaxed);
                        promote_window_pages(pdf_state, page_index);
                    }
                    return iced::widget::operation::scroll_to(
                        "pdf_content_scroll",
                        iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
                    );
                }
            }
            Task::none()
        }
        iced::mouse::ScrollDelta::Pixels { x, y } => {
            let now = std::time::Instant::now();
            let scaled_delta_y = -y * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;
            let scaled_delta_x = -x * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;

            pdf_state
                .scroll_controller
                .set_position_y(pdf_state.scroll_y);
            pdf_state.scroll_controller.handle_input(
                crate::core::scroll::ScrollInput::Motion {
                    delta_x: scaled_delta_x,
                    delta_y: scaled_delta_y,
                    time: now,
                },
                extent,
            );

            let new_y = pdf_state.scroll_controller.position_y();
            pdf_state.scroll_y = new_y;
            pdf_state.smooth_scroll.stop(new_y);

            let count = pdf_state.page_count;
            if count > 0 && !pdf_state.page_y_offsets.is_empty() {
                let page_index = crate::features::pdf::viewport::find_visible_page(
                    &pdf_state.page_y_offsets,
                    new_y,
                    vh,
                    0.3,
                );
                pdf_state
                    .visible_page
                    .store(page_index, std::sync::atomic::Ordering::Relaxed);
                promote_window_pages(pdf_state, page_index);
            }

            iced::widget::operation::scroll_to(
                "content_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
            )
        }
    }
}

pub fn handle_smooth_scroll_tick(app: &mut KglanceApp, now: std::time::Instant) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    let extent = crate::core::scroll::ViewportExtent {
        content_height: pdf_state.total_content_height,
        viewport_height: pdf_state.viewport_height,
    };
    let max_y = extent.max_scroll_y();

    // 1. ScrollController check
    if (pdf_state.scroll_controller.is_animating()
        || pdf_state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging)
        && let Some(next_y) = pdf_state.scroll_controller.update(now, extent)
    {
        pdf_state.scroll_y = next_y;
        pdf_state.smooth_scroll.stop(next_y);

        let count = pdf_state.page_count;
        if count > 0 && !pdf_state.page_y_offsets.is_empty() {
            let page_index = crate::features::pdf::viewport::find_visible_page(
                &pdf_state.page_y_offsets,
                next_y,
                pdf_state.viewport_height,
                0.3,
            );
            pdf_state
                .visible_page
                .store(page_index, std::sync::atomic::Ordering::Relaxed);
            promote_window_pages(pdf_state, page_index);
        }

        let is_finished = !pdf_state.scroll_controller.is_animating();
        return if is_finished && next_y >= max_y - 1.0 {
            iced::widget::operation::snap_to(
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                "content_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };
    }

    // 2. SmoothScroller check
    if let Some(next_y) = pdf_state.smooth_scroll.tick(pdf_state.scroll_y, now, max_y) {
        pdf_state.scroll_y = next_y;
        pdf_state.scroll_controller.set_position_y(next_y);

        let count = pdf_state.page_count;
        if count > 0 && !pdf_state.page_y_offsets.is_empty() {
            let page_index = crate::features::pdf::viewport::find_visible_page(
                &pdf_state.page_y_offsets,
                next_y,
                pdf_state.viewport_height,
                0.3,
            );
            pdf_state
                .visible_page
                .store(page_index, std::sync::atomic::Ordering::Relaxed);
            promote_window_pages(pdf_state, page_index);
        }

        let is_finished = !pdf_state.smooth_scroll.is_animating;
        let scroll_task = if is_finished && next_y >= max_y - 1.0 {
            iced::widget::operation::snap_to(
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                "content_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };

        return scroll_task;
    }

    Task::none()
}

pub fn handle_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let view_h = viewport.bounds().height;

    let pdf_state = active_pdf_state_mut(app);
    if pdf_state.scroll_controller.is_animating()
        || pdf_state.smooth_scroll.is_animating
        || pdf_state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging
    {
        return Task::none();
    }

    let count = pdf_state.page_count;
    if count > 0 && !pdf_state.page_y_offsets.is_empty() {
        let page_index = crate::features::pdf::viewport::find_visible_page(
            &pdf_state.page_y_offsets,
            y,
            view_h,
            0.3,
        );

        pdf_state.scroll_y = y;
        if view_h > 0.0 {
            pdf_state.viewport_height = view_h;
        }
        let total_h = viewport.content_bounds().height;
        if total_h > 0.0 {
            pdf_state.total_content_height = total_h;
        }
        pdf_state
            .visible_page
            .store(page_index, std::sync::atomic::Ordering::Relaxed);
        promote_window_pages(pdf_state, page_index);

        if pdf_state.sidebar_visible {
            match pdf_state.sidebar_mode {
                crate::core::types::PdfSidebarMode::Thumbnails => {
                    if let Some(&thumb_y) = pdf_state.thumbnail_y_offsets.get(page_index) {
                        let thumb_end = pdf_state
                            .thumbnail_ends
                            .get(page_index)
                            .copied()
                            .unwrap_or(thumb_y + 100.0);
                        let s_h = if pdf_state.sidebar_viewport_height > 0.0 {
                            pdf_state.sidebar_viewport_height
                        } else {
                            800.0
                        };
                        let item_h = thumb_end - thumb_y;
                        let target_y = (thumb_y - (s_h - item_h) / 2.0).max(0.0);
                        return iced::widget::operation::scroll_to(
                            "pdf_thumb_scroll",
                            iced::widget::operation::AbsoluteOffset {
                                x: 0.0,
                                y: target_y,
                            },
                        );
                    }
                }
                crate::core::types::PdfSidebarMode::Toc => {
                    if let Some(active_pos) =
                        pdf_state.outline.iter().rposition(|e| e.page <= page_index)
                    {
                        let s_h = if pdf_state.sidebar_viewport_height > 0.0 {
                            pdf_state.sidebar_viewport_height
                        } else {
                            800.0
                        };
                        const TOC_ITEM_HEIGHT: f32 = 30.0;
                        let item_y = active_pos as f32 * TOC_ITEM_HEIGHT;
                        let target_y = (item_y - (s_h - TOC_ITEM_HEIGHT) / 2.0).max(0.0);
                        return iced::widget::operation::scroll_to(
                            "pdf_toc_scroll",
                            iced::widget::operation::AbsoluteOffset {
                                x: 0.0,
                                y: target_y,
                            },
                        );
                    }
                }
            }
        }
    }
    Task::none()
}

pub fn handle_sidebar_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let view_h = viewport.bounds().height;

    let pdf_state = active_pdf_state_mut(app);
    pdf_state.sidebar_scroll_y = y;
    if view_h > 0.0 {
        pdf_state.sidebar_viewport_height = view_h;
    }

    if !pdf_state.thumbnail_y_offsets.is_empty() {
        let visible_thumb = crate::features::pdf::geometry::find_visible_thumbnail_page(
            &pdf_state.thumbnail_y_offsets,
            y,
            view_h,
        );
        pdf_state
            .visible_thumb_page
            .store(visible_thumb, std::sync::atomic::Ordering::Relaxed);
    }

    Task::none()
}

pub fn handle_pages_loaded(app: &mut KglanceApp) -> Task<Message> {
    pages_loaded(active_pdf_state_mut(app));
    Task::none()
}

pub fn handle_page_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
    page_text: Option<crate::features::pdf::PdfPageText>,
) -> Task<Message> {
    page_ready(
        active_pdf_state_mut(app),
        index,
        data,
        width,
        height,
        page_text,
    );
    Task::none()
}

pub fn pages_loaded(pdf_state: &mut crate::core::PdfState) {
    pdf_state.active_page_tasks = pdf_state.active_page_tasks.saturating_sub(1);
}

pub fn page_ready(
    pdf_state: &mut crate::core::PdfState,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
    page_text: Option<crate::features::pdf::PdfPageText>,
) {
    if index < pdf_state.pages.len() {
        let handle = iced::widget::image::Handle::from_bytes(data);
        let current_page = pdf_state
            .visible_page
            .load(std::sync::atomic::Ordering::Relaxed);
        pdf_state.pages.insert(
            index,
            crate::core::PageCacheEntry {
                width,
                height,
                handle,
            },
            current_page,
        );
    }
    if let Some(text) = page_text
        && index < pdf_state.page_texts.len()
    {
        pdf_state.page_texts[index] = Some(text);
    }
}

pub fn handle_selection_drag_start(
    app: &mut KglanceApp,
    pos: crate::features::pdf::PdfPosition,
) -> Task<Message> {
    crate::features::pdf::selection::handle_selection_drag_start(active_pdf_state_mut(app), pos);
    Task::none()
}

pub fn handle_selection_drag_update(
    app: &mut KglanceApp,
    pos: crate::features::pdf::PdfPosition,
) -> Task<Message> {
    crate::features::pdf::selection::handle_selection_drag_update(active_pdf_state_mut(app), pos);
    Task::none()
}

pub fn handle_selection_drag_end(app: &mut KglanceApp) -> Task<Message> {
    crate::features::pdf::selection::handle_selection_drag_end(active_pdf_state_mut(app));
    Task::none()
}

pub fn handle_selection_clear(app: &mut KglanceApp) -> Task<Message> {
    crate::features::pdf::selection::handle_selection_clear(active_pdf_state_mut(app));
    Task::none()
}

pub fn handle_select_all(pdf_state: &mut crate::core::PdfState) {
    crate::features::pdf::selection::handle_select_all(pdf_state);
}

pub fn handle_thumb_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);

    if index < pdf_state.thumbnails.len() {
        let handle = iced::widget::image::Handle::from_bytes(data);
        let current_thumb = pdf_state
            .visible_thumb_page
            .load(std::sync::atomic::Ordering::Relaxed);
        pdf_state.thumbnails.insert(
            index,
            crate::core::PageCacheEntry {
                width,
                height,
                handle,
            },
            current_thumb,
        );
    }
    Task::none()
}

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    let win_w = app.state.current_window_size.width;
    let pdf_state = active_pdf_state_mut(app);
    let desired_w = pdf_state.desired_width;
    pdf_state.sidebar_visible = !pdf_state.sidebar_visible;
    let sidebar_w = if pdf_state.sidebar_visible {
        pdf_state.sidebar_width + 1.0
    } else {
        0.0
    };
    let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
    let target_display_w = desired_w.min(max_w);
    crate::features::pdf::view::recalculate_pdf_offsets_for_width(pdf_state, target_display_w);
    let scroll_y = pdf_state.scroll_y;
    let load_task = if pdf_state.sidebar_visible {
        if pdf_state.sidebar_mode == crate::core::PdfSidebarMode::Thumbnails {
            start_thumbnail_loading_if_needed(app)
        } else {
            Task::none()
        }
    } else {
        pdf_state
            .thumb_generation_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Task::none()
    };
    let restore_scroll = iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: scroll_y,
        },
    );
    Task::batch([load_task, restore_scroll])
}

pub fn handle_set_sidebar_mode(
    app: &mut KglanceApp,
    mode: crate::core::PdfSidebarMode,
) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    let old_mode = pdf_state.sidebar_mode;
    pdf_state.sidebar_mode = mode;
    if mode == crate::core::PdfSidebarMode::Thumbnails && pdf_state.sidebar_visible {
        start_thumbnail_loading_if_needed(app)
    } else {
        if old_mode == crate::core::PdfSidebarMode::Thumbnails {
            pdf_state
                .thumb_generation_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Task::none()
    }
}

fn start_thumbnail_loading_if_needed(app: &KglanceApp) -> Task<Message> {
    let pdf = active_pdf_state(app);
    let is_have_pdf = !app.state.file_name.is_empty() && pdf.page_count > 0;

    if is_have_pdf {
        let is_typst = matches!(
            app.current_content,
            Some(crate::core::PreviewData::Typst { .. })
        );
        if is_typst {
            crate::features::typst::handler::lazy_load_typst_thumbnails(
                app.state.file_name.clone(),
                pdf.page_count,
                pdf.visible_thumb_page.clone(),
                pdf.thumb_generation_id.clone(),
            )
        } else {
            crate::features::pdf::lazy_handler::lazy_load_thumbnails(
                app.state.file_name.clone(),
                pdf.page_count,
                pdf.visible_thumb_page.clone(),
                pdf.thumb_generation_id.clone(),
                |page_index, page_data| {
                    crate::app::messages::PdfMsg::ThumbReady(
                        page_index,
                        page_data.data,
                        page_data.width,
                        page_data.height,
                    )
                    .into()
                },
                crate::app::messages::PdfMsg::PagesLoaded(Vec::new()).into(),
            )
        }
    } else {
        Task::none()
    }
}

pub fn handle_thumbnail_clicked(app: &mut KglanceApp, page_index: usize) -> Task<Message> {
    scroll_to_page(app, page_index)
}

pub fn handle_toc_item_clicked(app: &mut KglanceApp, page_index: usize) -> Task<Message> {
    scroll_to_page(app, page_index)
}

fn scroll_to_page(app: &mut KglanceApp, page_index: usize) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);

    let count = pdf_state.page_count;
    if count == 0 {
        return Task::none();
    }
    let target = page_index.min(count - 1);
    pdf_state
        .visible_page
        .store(target, std::sync::atomic::Ordering::Relaxed);
    promote_window_pages(pdf_state, target);

    let target_y =
        crate::features::pdf::viewport::page_scroll_offset(&pdf_state.page_y_offsets, target);

    pdf_state.scroll_y = target_y;
    pdf_state.scroll_controller.set_position_y(target_y);
    pdf_state.smooth_scroll.stop(target_y);

    let main_scroll = iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: target_y,
        },
    );

    if pdf_state.sidebar_visible {
        match pdf_state.sidebar_mode {
            crate::core::types::PdfSidebarMode::Thumbnails => {
                pdf_state
                    .visible_thumb_page
                    .store(target, std::sync::atomic::Ordering::Relaxed);
                if let Some(&thumb_y) = pdf_state.thumbnail_y_offsets.get(target) {
                    let thumb_end = pdf_state
                        .thumbnail_ends
                        .get(target)
                        .copied()
                        .unwrap_or(thumb_y + 100.0);
                    let s_h = if pdf_state.sidebar_viewport_height > 0.0 {
                        pdf_state.sidebar_viewport_height
                    } else {
                        800.0
                    };
                    let item_h = thumb_end - thumb_y;
                    let target_thumb_y = (thumb_y - (s_h - item_h) / 2.0).max(0.0);
                    pdf_state.sidebar_scroll_y = target_thumb_y;
                    let side_scroll = iced::widget::operation::scroll_to(
                        "pdf_thumb_scroll",
                        iced::widget::operation::AbsoluteOffset {
                            x: 0.0,
                            y: target_thumb_y,
                        },
                    );
                    return Task::batch([main_scroll, side_scroll]);
                }
            }
            crate::core::types::PdfSidebarMode::Toc => {
                if let Some(active_pos) = pdf_state.outline.iter().rposition(|e| e.page <= target) {
                    let s_h = if pdf_state.sidebar_viewport_height > 0.0 {
                        pdf_state.sidebar_viewport_height
                    } else {
                        800.0
                    };
                    const TOC_ITEM_HEIGHT: f32 = 30.0;
                    let item_y = active_pos as f32 * TOC_ITEM_HEIGHT;
                    let target_toc_y = (item_y - (s_h - TOC_ITEM_HEIGHT) / 2.0).max(0.0);
                    pdf_state.sidebar_scroll_y = target_toc_y;
                    let side_scroll = iced::widget::operation::scroll_to(
                        "pdf_toc_scroll",
                        iced::widget::operation::AbsoluteOffset {
                            x: 0.0,
                            y: target_toc_y,
                        },
                    );
                    return Task::batch([main_scroll, side_scroll]);
                }
            }
        }
    }

    main_scroll
}

pub fn handle_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    use crate::ui::components::sidebar::{PDF_MAX_SIDEBAR_WIDTH, PDF_MIN_SIDEBAR_WIDTH};

    let win_w = app.state.current_window_size.width;
    let pdf_state = active_pdf_state_mut(app);
    let desired_w = pdf_state.desired_width;
    pdf_state.sidebar_width = width.clamp(PDF_MIN_SIDEBAR_WIDTH, PDF_MAX_SIDEBAR_WIDTH);
    crate::features::pdf::geometry::recalculate_pdf_thumbnail_offsets(pdf_state);
    let sidebar_w = if pdf_state.sidebar_visible {
        pdf_state.sidebar_width + 1.0
    } else {
        0.0
    };
    let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
    let target_display_w = desired_w.min(max_w);
    crate::features::pdf::view::recalculate_pdf_offsets_for_width(pdf_state, target_display_w);
    Task::none()
}

pub fn resize_pdf_preview(
    pdf: &mut crate::core::PdfState,
    window_width: f32,
    direction: f32,
) -> Option<Task<Message>> {
    let old_width = pdf.desired_width;
    let mut new_width = (old_width + direction * 50.0).clamp(300.0, 2400.0);

    if new_width == old_width {
        return Some(Task::none());
    }

    let sidebar_width = if pdf.sidebar_visible {
        pdf.sidebar_width + 1.0
    } else {
        0.0
    };

    let max_width = (window_width - sidebar_width - 40.0).clamp(300.0, 2400.0);

    if new_width > max_width {
        new_width = max_width;
    }

    let scroll_y =
        crate::features::pdf::view::rescale_pdf_and_anchor(pdf, old_width, new_width, max_width);

    Some(iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: scroll_y,
        },
    ))
}

pub fn reset_pdf_width(pdf: &mut crate::core::PdfState, window_width: f32) -> Task<Message> {
    let old_width = pdf.desired_width;
    let new_width = 800.0;

    let sidebar_width = if pdf.sidebar_visible {
        pdf.sidebar_width + 1.0
    } else {
        0.0
    };

    let max_width = (window_width - sidebar_width - 40.0).clamp(300.0, 2400.0);

    let scroll_y =
        crate::features::pdf::view::rescale_pdf_and_anchor(pdf, old_width, new_width, max_width);

    iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: scroll_y,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::test_app;

    #[test]
    fn active_pdf_state_selects_shared_pdf_state() {
        let mut app = test_app(None);
        active_pdf_state_mut(&mut app).sidebar_width = 321.0;
        assert_eq!(app.state.pdf.sidebar_width, 321.0);
    }

    #[test]
    fn evict_distant_pages_works() {
        let mut pdf_state = crate::core::PdfState {
            pages: crate::core::types::PageCache::new(30),
            page_count: 30,
            ..Default::default()
        };

        for i in 0..20 {
            pdf_state.pages.insert(
                i,
                crate::core::PageCacheEntry {
                    width: 10,
                    height: 10,
                    handle: iced::widget::image::Handle::from_rgba(1, 1, vec![0; 4]),
                },
                15,
            );
        }

        assert!(pdf_state.pages.count() <= MAX_CACHED_PAGES);
        // Near pages around 15 are retained
        assert!(pdf_state.pages.get(15).is_some());
        assert!(pdf_state.pages.get(14).is_some());
        assert!(pdf_state.pages.get(16).is_some());
    }

    #[test]
    fn test_disk_cache_instant_promotion() {
        let mut state = crate::core::PdfState {
            page_count: 50,
            pages: crate::core::types::PageCache::new(50),
            ..Default::default()
        };
        let cache =
            std::sync::Arc::new(crate::features::pdf::cache::PdfDiskCache::new(77777).unwrap());
        let _ = cache.save_page_with_meta(10, b"\x89PNG\r\n\x1a\nfake", 800, 1000);
        state.disk_cache = Some(cache);

        assert!(state.pages.get(10).is_none());
        assert!(promote_page_from_disk_if_cached(&mut state, 10));
        assert!(state.pages.get(10).is_some());
    }
}
