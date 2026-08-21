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

pub const MAX_CACHED_PAGES: usize = 24;

/// Evicts the furthest pages from `current_page` when total cached pages exceed `MAX_CACHED_PAGES`.
/// Visited pages are preserved in cache for instant display when scrolling back.
pub fn evict_distant_pages(pdf_state: &mut crate::core::PdfState, current_page: usize) {
    let cached_indices: Vec<usize> = pdf_state
        .pages
        .iter()
        .enumerate()
        .filter_map(|(idx, page)| page.is_some().then_some(idx))
        .collect();

    if cached_indices.len() > MAX_CACHED_PAGES {
        let mut sorted_by_dist = cached_indices;
        sorted_by_dist
            .sort_by_key(|&idx| std::cmp::Reverse((idx as isize - current_page as isize).abs()));

        let to_evict = sorted_by_dist.len() - MAX_CACHED_PAGES;
        for &idx in sorted_by_dist.iter().take(to_evict) {
            pdf_state.pages[idx] = None;
        }
    }
}

/// Promotes a page from Tier 1 disk cache to Tier 2 UI GPU handle if available.
pub fn promote_page_from_disk_if_cached(
    pdf_state: &mut crate::core::PdfState,
    page_index: usize,
) -> bool {
    if page_index >= pdf_state.pages.len() || pdf_state.pages[page_index].is_some() {
        return false;
    }
    if let Some(ref disk_cache) = pdf_state.disk_cache
        && let Ok(png_bytes) = disk_cache.load_page(page_index)
    {
        let handle = iced::widget::image::Handle::from_bytes(png_bytes);
        pdf_state.pages[page_index] = Some(crate::core::PageCacheEntry {
            width: 0,
            height: 0,
            handle,
        });
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
                    let target_y =
                        (page_index as f32 * (pdf_state.sidebar_width * 1.3) - 100.0).max(0.0);
                    return iced::widget::operation::scroll_to(
                        "pdf_thumb_scroll",
                        iced::widget::operation::AbsoluteOffset {
                            x: 0.0,
                            y: target_y,
                        },
                    );
                }
                crate::core::types::PdfSidebarMode::Toc => {
                    if let Some(active_pos) =
                        pdf_state.outline.iter().rposition(|e| e.page <= page_index)
                    {
                        let target_y = (active_pos as f32 * 28.0 - 100.0).max(0.0);
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
    pdf_state.loading = false;
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
        pdf_state.pages[index] = Some(crate::core::PageCacheEntry {
            width,
            height,
            handle,
        });
        let current_page = pdf_state
            .visible_page
            .load(std::sync::atomic::Ordering::Relaxed);
        evict_distant_pages(pdf_state, current_page);
    }

    let all_loaded = pdf_state.pages.iter().all(|p| p.is_some());
    if all_loaded {
        pdf_state.loading = false;
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
        pdf_state.thumbnails[index] = Some(crate::core::PageCacheEntry {
            width,
            height,
            handle,
        });
    }
    Task::none()
}

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    let pdf_state = active_pdf_state_mut(app);
    pdf_state.sidebar_visible = !pdf_state.sidebar_visible;
    let scroll_y = pdf_state.scroll_y;
    let load_task = if pdf_state.sidebar_visible {
        start_thumbnail_loading_if_needed(app)
    } else {
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
    pdf_state.sidebar_mode = mode;
    if mode == crate::core::PdfSidebarMode::Thumbnails {
        start_thumbnail_loading_if_needed(app)
    } else {
        Task::none()
    }
}

fn start_thumbnail_loading_if_needed(app: &KglanceApp) -> Task<Message> {
    let pdf = active_pdf_state(app);
    if !app.state.file_name.is_empty() && pdf.page_count > 0 {
        crate::features::pdf::lazy_load_thumbnails(
            app.state.file_name.clone(),
            pdf.page_count,
            pdf.visible_page.clone(),
            pdf.generation_id.clone(),
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

    let target_y =
        crate::features::pdf::viewport::page_scroll_offset(&pdf_state.page_y_offsets, target);

    iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: target_y,
        },
    )
}

pub fn handle_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    active_pdf_state_mut(app).sidebar_width = width.clamp(120.0, 500.0);
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
            pages: vec![
                Some(crate::core::PageCacheEntry {
                    width: 10,
                    height: 10,
                    handle: iced::widget::image::Handle::from_rgba(10, 10, vec![0; 400]),
                });
                30
            ],
            page_count: 30,
            ..Default::default()
        };

        // Total cached pages is 30, MAX_CACHED_PAGES is 24.
        // With current_page = 15, the 6 furthest pages (e.g. indices 0, 1, 2, 29, 28, 27) are evicted.
        evict_distant_pages(&mut pdf_state, 15);
        let cached_count = pdf_state.pages.iter().filter(|p| p.is_some()).count();
        assert_eq!(cached_count, MAX_CACHED_PAGES);

        // Near pages around 15 are retained
        assert!(pdf_state.pages[15].is_some());
        assert!(pdf_state.pages[14].is_some());
        assert!(pdf_state.pages[16].is_some());
    }

    #[test]
    fn test_disk_cache_instant_promotion() {
        let mut state = crate::core::PdfState {
            page_count: 50,
            pages: vec![None; 50],
            ..Default::default()
        };
        let cache =
            std::sync::Arc::new(crate::features::pdf::cache::PdfDiskCache::new(77777).unwrap());
        let _ = cache.save_page(10, b"\x89PNG\r\n\x1a\nfake");
        state.disk_cache = Some(cache);

        assert!(state.pages[10].is_none());
        assert!(promote_page_from_disk_if_cached(&mut state, 10));
        assert!(state.pages[10].is_some());
    }
}
