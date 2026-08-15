use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

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

pub fn handle_toggle_outline(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.outline_visible = !app.state.text.outline_visible;
    Task::none()
}

pub fn handle_symbol_clicked(app: &mut KglanceApp, line_number: usize) -> Task<Message> {
    let font_size = app.state.font_size;
    let line_height = font_size * 1.35;
    let target_y = (line_number.saturating_sub(1) as f32) * line_height;

    operation::scroll_to(
        "content_scroll",
        operation::AbsoluteOffset {
            x: 0.0,
            y: target_y,
        },
    )
}
