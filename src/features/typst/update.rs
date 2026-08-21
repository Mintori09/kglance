use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::features::pdf::update as pdf;
use iced::Task;

pub fn handle_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    pdf::handle_scrolled(app, viewport)
}

pub fn handle_pages_loaded(app: &mut KglanceApp) -> Task<Message> {
    pdf::pages_loaded(&mut app.state.typst.pdf);
    Task::none()
}

pub fn handle_page_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
) -> Task<Message> {
    pdf::page_ready(&mut app.state.typst.pdf, index, data, width, height);
    Task::none()
}

pub fn handle_compile_error(app: &mut KglanceApp) -> Task<Message> {
    app.state.typst.pdf.active_page_tasks = app.state.typst.pdf.active_page_tasks.saturating_sub(1);
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
