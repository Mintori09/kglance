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
    let win_w = app.state.current_window_size.width;
    let win_h = app.state.current_window_size.height;
    if app.state.selection_menu.is_cursor_inside(
        app.state.last_cursor_position,
        win_w,
        win_h,
        &app.state.selection_menu_config.custom_actions,
    ) {
        return Task::none();
    }
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
                pdf_state.scroll_controller.stop(pdf_state.scroll_y);
                let line_height =
                    (vh * crate::core::scroll::WHEEL_SCROLL_VIEWPORT_FRACTION).clamp(50.0, 240.0);
                let prev_y = pdf_state.scroll_y;
                let step = -y * line_height;
                pdf_state
                    .smooth_scroll
                    .start_interactive(prev_y, step, max_y);
                if !pdf_state.smooth_scroll.is_animating() {
                    let new_y = pdf_state.smooth_scroll.position_y();
                    if (new_y - prev_y).abs() > f32::EPSILON {
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
                            "content_scroll",
                            iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
                        );
                    }
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
    let is_interrupted = pdf_state.scroll_controller.check_interruption(y);
    if !is_interrupted
        && (pdf_state.scroll_controller.is_animating()
            || pdf_state.smooth_scroll.is_animating
            || pdf_state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging)
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

    // Don't overwrite the popup while a background action is loading or streaming.
    if app.state.selection_menu.is_busy() {
        return Task::none();
    }

    if app.state.selection_menu_config.enabled
        && let Some(text) = active_pdf_state(app).selected_text.as_deref()
    {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let pos = app
                .state
                .last_cursor_position
                .unwrap_or(iced::Point::new(100.0, 100.0));
            app.state.selection_menu = crate::core::selection_action::SelectionMenuState::Toolbar {
                pos,
                selected_text: trimmed.to_string(),
            };
        }
    }
    Task::none()
}

pub fn handle_selection_clear(app: &mut KglanceApp) -> Task<Message> {
    if !app.state.selection_menu.is_busy() {
        app.state.selection_menu.hide();
    }
    crate::features::pdf::selection::handle_selection_clear(active_pdf_state_mut(app));
    Task::none()
}

pub fn handle_auto_scroll_tick(app: &mut KglanceApp) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    let Some(delta) = pdf_state.auto_scroll_delta else {
        return Task::none();
    };

    let vh = if pdf_state.viewport_height > 0.0 {
        pdf_state.viewport_height
    } else {
        800.0
    };
    let max_y = (pdf_state.total_content_height - vh).max(0.0);
    let new_y = (pdf_state.scroll_y + delta).clamp(0.0, max_y);

    pdf_state.scroll_y = new_y;
    pdf_state.scroll_controller.stop(new_y);
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

    let win_w = app.state.current_window_size.width;
    let drag_x = app.state.pdf.drag_last_x;
    let drag_y = app.state.pdf.drag_last_y;
    const HEADER_HEIGHT: f32 = 40.0;
    let rel_y = (drag_y - HEADER_HEIGHT).max(0.0);
    let content_y = new_y + rel_y;

    if let Some(pos) = crate::features::pdf::selection::hit_test_pdf_position(
        &app.state.pdf,
        win_w,
        drag_x,
        content_y,
    ) {
        crate::features::pdf::selection::handle_selection_drag_update(&mut app.state.pdf, pos);
    }

    iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
    )
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
    let pdf_state = active_pdf_state_mut(app);
    // If outline exists and user toggles, default to Toc mode if currently in Thumbnails and not visible
    if !pdf_state.sidebar_visible && !pdf_state.outline.is_empty() {
        pdf_state.sidebar_mode = crate::core::PdfSidebarMode::Toc;
    }
    let is_now_visible = !pdf_state.sidebar_visible;
    if is_now_visible {
        app.state.file_tree.visible = false;
    }
    let win_w = app.state.current_window_size.width;
    let pdf_state = active_pdf_state_mut(app);
    let desired_w = pdf_state.desired_width;
    pdf_state.sidebar_visible = is_now_visible;
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

pub fn handle_toggle_outline_node(
    app: &mut KglanceApp,
    page: usize,
    title: String,
) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    pdf_state.outline_state.toggle_expanded(page, &title);
    Task::none()
}

fn scroll_to_page(app: &mut KglanceApp, page_index: usize) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);

    let count = pdf_state.page_count;
    if count == 0 {
        return Task::none();
    }
    let target = page_index.min(count - 1);
    pdf_state.current_page = target;
    pdf_state
        .visible_page
        .store(target, std::sync::atomic::Ordering::Relaxed);
    promote_window_pages(pdf_state, target);

    let target_y =
        crate::features::pdf::viewport::page_scroll_offset(&pdf_state.page_y_offsets, target);

    pdf_state.scroll_y = target_y;
    pdf_state.scroll_controller.stop(target_y);
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

pub fn rescale_pdf_page_width(app: &mut KglanceApp, new_width: f32) -> Option<Task<Message>> {
    let old_width = app.state.pdf.desired_width;
    let clamped_width = new_width.clamp(300.0, 2400.0);

    if (clamped_width - old_width).abs() < f32::EPSILON {
        return Some(Task::none());
    }

    app.state.pdf.desired_width = clamped_width;

    let win_w = app.state.current_window_size.width;
    let sidebar_w = if app.state.pdf.sidebar_visible {
        app.state.pdf.sidebar_width + 1.0
    } else {
        0.0
    };
    let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);

    let scroll_y = crate::features::pdf::view::rescale_pdf_and_anchor(
        &mut app.state.pdf,
        old_width,
        clamped_width,
        max_w,
    );

    app.state
        .toasts
        .retain(|t| !t.message.starts_with("Page Width:"));
    let toast = app.show_toast(format!("Page Width: {clamped_width:.0}px"));

    let scroll_task = iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: scroll_y,
        },
    );

    Some(Task::batch([scroll_task, toast]))
}

pub fn resize_pdf_preview(app: &mut KglanceApp, direction: f32) -> Option<Task<Message>> {
    let old_width = app.state.pdf.desired_width;
    let new_width = (old_width + direction * 50.0).clamp(300.0, 2400.0);
    rescale_pdf_page_width(app, new_width)
}

pub fn reset_pdf_width(app: &mut KglanceApp) -> Task<Message> {
    rescale_pdf_page_width(app, 800.0).unwrap_or_else(Task::none)
}

pub fn handle_goto_page_open(app: &mut KglanceApp) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    let current_page = pdf_state
        .visible_page
        .load(std::sync::atomic::Ordering::Relaxed);
    pdf_state.goto_page.open(current_page);
    iced::widget::operation::focus(crate::features::pdf::goto_dialog::GOTO_PAGE_INPUT_ID)
}

pub fn handle_goto_page_close(app: &mut KglanceApp) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    pdf_state.goto_page.close();
    Task::none()
}

pub fn handle_goto_page_input_changed(app: &mut KglanceApp, input: String) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    pdf_state.goto_page.set_input(input);
    Task::none()
}

pub fn handle_goto_page_submit(app: &mut KglanceApp) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    let total_pages = pdf_state.page_count;
    if let Some(target_page) = pdf_state.goto_page.parse_target_page(total_pages) {
        pdf_state.current_page = target_page;
        pdf_state.goto_page.close();
        scroll_to_page(app, target_page)
    } else {
        pdf_state.goto_page.has_error = true;
        Task::none()
    }
}

pub use crate::features::pdf::search::{
    handle_search_close, handle_search_next, handle_search_open, handle_search_prev,
    handle_search_query_changed,
};

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
