use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn active_pdf_state_mut(app: &mut KglanceApp) -> &mut crate::core::PdfState {
    if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    }
}

pub fn active_pdf_state(app: &KglanceApp) -> &crate::core::PdfState {
    if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &app.state.typst.pdf
    } else {
        &app.state.pdf
    }
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

pub fn handle_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    let y = viewport.absolute_offset().y;
    let view_h = viewport.bounds().height;

    let pdf_state = active_pdf_state_mut(app);

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
        pdf_state
            .visible_page
            .store(page_index, std::sync::atomic::Ordering::Relaxed);

        let start = page_index.saturating_sub(2);
        let end = (page_index + 2).min(count.saturating_sub(1));
        for p in start..=end {
            promote_page_from_disk_if_cached(pdf_state, p);
        }
        evict_distant_pages(pdf_state, page_index);

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
) -> Task<Message> {
    page_ready(active_pdf_state_mut(app), index, data, width, height);
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

    let start = target.saturating_sub(2);
    let end = (target + 2).min(count.saturating_sub(1));
    for p in start..=end {
        promote_page_from_disk_if_cached(pdf_state, p);
    }
    evict_distant_pages(pdf_state, target);

    let target_y =
        crate::features::pdf::viewport::page_scroll_offset(&pdf_state.page_y_offsets, target);

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
    let win_w = app.state.current_window_size.width;
    let pdf_state = active_pdf_state_mut(app);
    let desired_w = pdf_state.desired_width;
    pdf_state.sidebar_width = width.clamp(120.0, 500.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::test_app;
    use crate::core::PreviewData;

    #[test]
    fn active_pdf_state_selects_regular_pdf_by_default() {
        let mut app = test_app(None);
        active_pdf_state_mut(&mut app).sidebar_width = 321.0;
        assert_eq!(app.state.pdf.sidebar_width, 321.0);
        assert_eq!(app.state.typst.pdf.sidebar_width, 220.0);
    }

    #[test]
    fn active_pdf_state_selects_typst_pdf_for_typst_content() {
        let content = Some(PreviewData::Typst {
            page_count: 1,
            current_page: 0,
            data: Vec::new(),
            width: 0,
            height: 0,
            source: String::new(),
            error: None,
            outline: Vec::new(),
            page_dimensions: Vec::new(),
        });
        let mut app = test_app(content);
        active_pdf_state_mut(&mut app).sidebar_width = 432.0;
        assert_eq!(app.state.typst.pdf.sidebar_width, 432.0);
        assert_eq!(app.state.pdf.sidebar_width, 220.0);
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
