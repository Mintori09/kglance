use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    let y = viewport.absolute_offset().y;
    let content_h = viewport.content_bounds().height;

    let is_typst = matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    );
    let pdf_state = if is_typst {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };

    let count = pdf_state.page_count;
    if count > 0 && content_h > 0.0 {
        let mut page_index = ((y / content_h) * count as f32) as usize;
        if page_index >= count {
            page_index = count - 1;
        }
        pdf_state
            .visible_page
            .store(page_index, std::sync::atomic::Ordering::Relaxed);

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
    app.state.pdf.loading = false;
    Task::none()
}

pub fn handle_page_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
) -> Task<Message> {
    let pdf_state = if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };

    if index < pdf_state.pages.len() {
        let handle = iced::widget::image::Handle::from_rgba(width, height, data.clone());
        pdf_state.pages[index] = Some(crate::core::PageCacheEntry {
            data,
            width,
            height,
            handle,
        });
    }
    let all_loaded = pdf_state.pages.iter().all(|p| p.is_some());
    if all_loaded {
        pdf_state.loading = false;
    }
    Task::none()
}

pub fn handle_thumb_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
) -> Task<Message> {
    let pdf_state = if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };

    if index < pdf_state.thumbnails.len() {
        let handle = iced::widget::image::Handle::from_rgba(width, height, data.clone());
        pdf_state.thumbnails[index] = Some(crate::core::PageCacheEntry {
            data,
            width,
            height,
            handle,
        });
    }
    Task::none()
}

pub fn handle_sidebar_toggled(app: &mut KglanceApp) -> Task<Message> {
    let pdf_state = if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };
    pdf_state.sidebar_visible = !pdf_state.sidebar_visible;
    if pdf_state.sidebar_visible {
        start_thumbnail_loading_if_needed(app)
    } else {
        Task::none()
    }
}

pub fn handle_set_sidebar_mode(
    app: &mut KglanceApp,
    mode: crate::core::PdfSidebarMode,
) -> Task<Message> {
    let pdf_state = if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };
    pdf_state.sidebar_mode = mode;
    if mode == crate::core::PdfSidebarMode::Thumbnails {
        start_thumbnail_loading_if_needed(app)
    } else {
        Task::none()
    }
}

fn start_thumbnail_loading_if_needed(app: &KglanceApp) -> Task<Message> {
    if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        if !app.state.file_name.is_empty() && app.state.typst.pdf.page_count > 0 {
            crate::ui::handlers::pdf::lazy_load_thumbnails(
                app.state.file_name.clone(),
                app.state.typst.pdf.page_count,
                app.state.typst.pdf.visible_page.clone(),
            )
        } else {
            Task::none()
        }
    } else if !app.state.file_name.is_empty() && app.state.pdf.page_count > 0 {
        crate::ui::handlers::pdf::lazy_load_thumbnails(
            app.state.file_name.clone(),
            app.state.pdf.page_count,
            app.state.pdf.visible_page.clone(),
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
    let pdf_state = if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };

    let count = pdf_state.page_count;
    if count == 0 {
        return Task::none();
    }
    let target = page_index.min(count - 1);
    pdf_state
        .visible_page
        .store(target, std::sync::atomic::Ordering::Relaxed);
    let relative_y = target as f32 / count as f32;
    iced::widget::operation::snap_to(
        "content_scroll",
        iced::widget::operation::RelativeOffset {
            x: 0.0,
            y: relative_y,
        },
    )
}

pub fn handle_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    let pdf_state = if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        &mut app.state.typst.pdf
    } else {
        &mut app.state.pdf
    };
    pdf_state.sidebar_width = width.clamp(120.0, 500.0);
    Task::none()
}
