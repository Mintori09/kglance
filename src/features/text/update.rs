use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_text_edit(
    app: &mut KglanceApp,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if !matches!(action, iced::widget::text_editor::Action::Edit(_)) {
        app.state.text.content.perform(action);
    }
    Task::none()
}

pub fn handle_text_scrolled(app: &mut KglanceApp, y: f32) -> Task<Message> {
    app.state.text.scroll_y = y;
    app.record_read_position();
    Task::none()
}
