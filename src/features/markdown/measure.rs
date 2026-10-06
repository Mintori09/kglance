use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::scroll::GestureState;
use iced::Task;
use iced::widget::operation;

use super::update::active_markdown_state_mut;

/// Applies a block height measured from the rendered layout, keeping the
/// block at the top of the viewport anchored in place.
pub fn handle_block_measured(
    app: &mut KglanceApp,
    block_index: usize,
    height: f32,
) -> Task<Message> {
    let font_size = app.state.font_size;
    let state = active_markdown_state_mut(app);
    let old_scroll_y = state.scroll_y;
    let new_scroll_y =
        super::apply_measured_block_heights(state, &[(block_index, height)], font_size);

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
