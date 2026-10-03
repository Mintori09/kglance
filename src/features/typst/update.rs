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
    pdf::pages_loaded(&mut app.state.pdf);
    Task::none()
}

pub fn handle_page_ready(
    app: &mut KglanceApp,
    index: usize,
    data: Vec<u8>,
    width: u32,
    height: u32,
) -> Task<Message> {
    pdf::page_ready(&mut app.state.pdf, index, data, width, height);
    Task::none()
}

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

pub fn handle_source_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let vh = viewport.bounds().height;
    let total_h = viewport.content_bounds().height;

    let text_state = &mut app.state.typst.source_text;
    if vh > 0.0 {
        text_state.viewport_height = vh;
    }
    if total_h > 0.0 {
        text_state.total_content_height = total_h;
    }

    let is_animating = text_state.smooth_scroll.is_animating
        || text_state.scroll_controller.is_animating()
        || text_state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging;
    if is_animating {
        return Task::none();
    }

    let delta_y = (text_state.scroll_y - y).abs();
    let delta_vh = (text_state.viewport_height - vh).abs();
    if delta_y < 1.0 && delta_vh < 1.0 {
        return Task::none();
    }

    text_state.smooth_scroll.stop(y);
    text_state.scroll_controller.stop(y);
    text_state.scroll_y = y;

    let theme = app.state.app_theme;
    crate::features::text::update_tokens_for_viewport(&mut app.state.typst.source_text, y, theme);
    Task::none()
}

pub fn handle_source_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let theme = app.state.app_theme;
    crate::features::text::update::handle_text_state_wheel_scrolled(
        &mut app.state.typst.source_text,
        theme,
        "typst_source_scroll",
        delta,
    )
}
