use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    let y = viewport.absolute_offset().y;
    let content_h = viewport.content_bounds().height;
    let count = app.state.typst.pdf.page_count;
    if count > 0 && content_h > 0.0 {
        let mut page_index = ((y / content_h) * count as f32) as usize;
        if page_index >= count {
            page_index = count - 1;
        }
        app.state
            .typst
            .pdf
            .visible_page
            .store(page_index, std::sync::atomic::Ordering::Relaxed);
    }
    Task::none()
}

pub fn handle_pages_loaded(app: &mut KglanceApp) -> Task<Message> {
    app.state.typst.pdf.loading = false;
    Task::none()
}

pub fn handle_page_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
) -> Task<Message> {
    if index < app.state.typst.pdf.pages.len() {
        let handle = iced::widget::image::Handle::from_rgba(width, height, data.clone());
        app.state.typst.pdf.pages[index] = Some(crate::core::PageCacheEntry {
            data,
            width,
            height,
            handle,
        });
    }
    let all_loaded = app.state.typst.pdf.pages.iter().all(|p| p.is_some());
    if all_loaded {
        app.state.typst.pdf.loading = false;
    }
    Task::none()
}

pub fn handle_compile_error(app: &mut KglanceApp) -> Task<Message> {
    app.state.typst.pdf.loading = false;
    if app.state.typst.error.is_none() {
        app.state.typst.error = Some("Failed to compile Typst document".to_string());
    }
    app.state.typst.show_source = true;
    Task::none()
}

pub fn handle_toggle_source(app: &mut KglanceApp) -> Task<Message> {
    app.state.typst.show_source = !app.state.typst.show_source;
    Task::none()
}
