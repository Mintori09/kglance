use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_compile_error(app: &mut KglanceApp) -> Task<Message> {
    app.state.pdf.active_page_tasks = app.state.pdf.active_page_tasks.saturating_sub(1);
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
