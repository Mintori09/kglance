use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::scroll::GestureState;
use iced::Task;
use iced::widget::operation;

use super::update::active_markdown_state_mut;

/// Queues a block height measurement to be processed in batch.
pub fn handle_block_measured(
    app: &mut KglanceApp,
    block_index: usize,
    height: f32,
) -> Task<Message> {
    let state = active_markdown_state_mut(app);
    state.pending_measurements.push((block_index, height));
    Task::none()
}

/// Flushes all pending block measurements in a single atomic batch pass,
/// updating layout offsets and anchoring the scroll position.
pub fn flush_pending_block_measurements(app: &mut KglanceApp) -> Task<Message> {
    let font_size = app.state.font_size;
    let state = active_markdown_state_mut(app);

    if state.pending_measurements.is_empty() {
        return Task::none();
    }

    let measurements = std::mem::take(&mut state.pending_measurements);
    let old_scroll_y = state.scroll_y;
    let new_scroll_y = super::apply_measured_block_heights(state, &measurements, font_size);

    let is_animating = state.smooth_scroll.is_animating
        || state.scroll_controller.is_animating()
        || state.scroll_controller.state() == GestureState::Dragging;
    if is_animating || (new_scroll_y - old_scroll_y).abs() < f32::EPSILON {
        return Task::none();
    }

    operation::scroll_to(
        "content_scroll",
        operation::AbsoluteOffset {
            x: 0.0,
            y: new_scroll_y,
        },
    )
}
