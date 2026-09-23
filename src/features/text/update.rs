use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

const CONTENT_SCROLL_ID: &str = "content_scroll";

pub fn handle_text_edit(
    app: &mut KglanceApp,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if !matches!(action, iced::widget::text_editor::Action::Edit(_)) {
        app.state.text.content.perform(action);
    }
    Task::none()
}

pub fn handle_text_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let vh = viewport.bounds().height;
    let total_h = viewport.content_bounds().height;

    let text_state = &mut app.state.text;
    if vh > 0.0 {
        text_state.viewport_height = vh;
    }
    if total_h > 0.0 {
        text_state.total_content_height = total_h;
    }

    if text_state.smooth_scroll.is_animating || text_state.scroll_controller.is_animating() {
        return Task::none();
    }

    let delta_y = (text_state.scroll_y - y).abs();
    if delta_y < 1.0 {
        return Task::none();
    }

    text_state.smooth_scroll.stop(y);
    text_state.scroll_controller.stop(y);
    text_state.scroll_y = y;
    app.record_read_position();
    Task::none()
}

pub fn handle_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let text_state = &mut app.state.text;
    if text_state.viewport_height <= 0.0 {
        return Task::none();
    }
    let vh = text_state.viewport_height;
    let extent = crate::core::scroll::ViewportExtent {
        content_height: text_state.total_content_height,
        viewport_height: vh,
    };
    let max_y = extent.max_scroll_y();

    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => {
            if y.abs() > f32::EPSILON {
                let line_height =
                    (vh * crate::core::scroll::WHEEL_SCROLL_VIEWPORT_FRACTION).clamp(50.0, 240.0);
                let step = -y * line_height;
                text_state
                    .smooth_scroll
                    .start_interactive(text_state.scroll_y, step, max_y);
            }
            Task::none()
        }
        iced::mouse::ScrollDelta::Pixels { x, y } => {
            let now = std::time::Instant::now();
            let scaled_delta_y = -y * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;
            let scaled_delta_x = -x * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;

            text_state
                .scroll_controller
                .set_position_y(text_state.scroll_y);
            text_state.scroll_controller.handle_input(
                crate::core::scroll::ScrollInput::Motion {
                    delta_x: scaled_delta_x,
                    delta_y: scaled_delta_y,
                    time: now,
                },
                extent,
            );

            let new_y = text_state.scroll_controller.position_y();
            text_state.scroll_y = new_y;
            text_state.smooth_scroll.stop(new_y);

            iced::widget::operation::scroll_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
            )
        }
    }
}

pub fn handle_smooth_scroll_tick(app: &mut KglanceApp, now: std::time::Instant) -> Task<Message> {
    let text_state = &mut app.state.text;
    let extent = crate::core::scroll::ViewportExtent {
        content_height: text_state.total_content_height,
        viewport_height: text_state.viewport_height,
    };
    let max_y = extent.max_scroll_y();

    // 1. ScrollController check
    if (text_state.scroll_controller.is_animating()
        || text_state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging)
        && let Some(next_y) = text_state.scroll_controller.update(now, extent)
    {
        text_state.scroll_y = next_y;
        text_state.smooth_scroll.stop(next_y);
        let is_finished = !text_state.scroll_controller.is_animating();
        if is_finished {
            app.record_read_position();
        }

        let scroll_task = if is_finished && next_y >= max_y - 1.0 {
            iced::widget::operation::snap_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };

        return scroll_task;
    }

    // 2. SmoothScroller check (keyboard navigation & discrete wheel step)
    if let Some(next_y) = text_state
        .smooth_scroll
        .tick(text_state.scroll_y, now, max_y)
    {
        text_state.scroll_y = next_y;
        text_state.scroll_controller.set_position_y(next_y);
        let is_finished = !text_state.smooth_scroll.is_animating;
        if is_finished {
            app.record_read_position();
        }

        let scroll_task = if is_finished && next_y >= max_y - 1.0 {
            iced::widget::operation::snap_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };

        return scroll_task;
    }

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

    let text_state = &mut app.state.text;
    let max_y = crate::core::scroll::max_scroll_y(
        text_state.total_content_height,
        text_state.viewport_height,
    );
    text_state
        .smooth_scroll
        .start_navigation(text_state.scroll_y, target_y, max_y);
    Task::none()
}
