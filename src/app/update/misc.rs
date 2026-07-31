use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_copy_code(app: &mut KglanceApp, code: String) -> Task<Message> {
    let toast = app.show_toast("Copied!");
    Task::batch(vec![iced::clipboard::write(code), toast])
}

pub fn handle_open_link(url: String) -> Task<Message> {
    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    Task::none()
}

pub fn handle_theme_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.theme_dark = !app.state.theme_dark;
    Task::none()
}

pub fn handle_toast_dismissed(app: &mut KglanceApp, id: u64) -> Task<Message> {
    app.state.toasts.retain(|t| t.id != id);
    Task::none()
}

pub fn handle_text_edit(
    app: &mut KglanceApp,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if !matches!(action, iced::widget::text_editor::Action::Edit(_)) {
        app.state.text.content.perform(action);
    }
    Task::none()
}

pub fn handle_markdown_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    app.state.markdown.sidebar_width = width.clamp(140.0, 550.0);
    Task::none()
}

pub fn handle_epub_sidebar_resized(app: &mut KglanceApp, width: f32) -> Task<Message> {
    app.state.epub.sidebar_width = width.clamp(140.0, 550.0);
    Task::none()
}

pub fn handle_sidebar_drag_started(app: &mut KglanceApp, start_x: f32) -> Task<Message> {
    app.state.markdown.sidebar_resizing = true;
    app.state.markdown.sidebar_drag_start_x = start_x;
    app.state.markdown.sidebar_drag_start_width = app.state.markdown.sidebar_width;

    app.state.epub.sidebar_resizing = true;
    app.state.epub.sidebar_drag_start_x = start_x;
    app.state.epub.sidebar_drag_start_width = app.state.epub.sidebar_width;
    Task::none()
}

pub fn handle_sidebar_drag_ended(app: &mut KglanceApp) -> Task<Message> {
    app.state.markdown.sidebar_resizing = false;
    app.state.epub.sidebar_resizing = false;
    Task::none()
}

pub fn handle_mouse_moved(app: &mut KglanceApp, x: f32, _y: f32) -> Task<Message> {
    if app.state.markdown.sidebar_resizing {
        let delta = x - app.state.markdown.sidebar_drag_start_x;
        let new_w = (app.state.markdown.sidebar_drag_start_width + delta).clamp(140.0, 550.0);
        app.state.markdown.sidebar_width = new_w;
    }
    if app.state.epub.sidebar_resizing {
        let delta = x - app.state.epub.sidebar_drag_start_x;
        let new_w = (app.state.epub.sidebar_drag_start_width + delta).clamp(140.0, 550.0);
        app.state.epub.sidebar_width = new_w;
    }
    Task::none()
}

pub fn handle_text_scrolled(app: &mut KglanceApp, y: f32) -> Task<Message> {
    app.state.text.scroll_y = y;
    Task::none()
}
