use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

const CONTENT_SCROLL_ID: &str = "content_scroll";

pub fn handle_sheet_tab_clicked(app: &mut KglanceApp, index: usize) -> Task<Message> {
    if index < app.state.spreadsheet.sheets.len() {
        app.state.spreadsheet.active_sheet = index;
        app.state.spreadsheet.sort_col = None;
        app.state.spreadsheet.sort_ascending = None;
        app.state.spreadsheet.scroll_y = 0.0;
        app.state.spreadsheet.smooth_scroll.stop(0.0);
        app.state.spreadsheet.scroll_controller.stop(0.0);
        let total_rows = app
            .state
            .spreadsheet
            .sheets
            .get(index)
            .map_or(0, |s| s.rows.len());
        app.state.spreadsheet.total_content_height =
            total_rows as f32 * crate::features::csv::view::ROW_STEP;
    }
    Task::none()
}

pub fn handle_column_clicked(app: &mut KglanceApp, col: usize) -> Task<Message> {
    let sort = &mut app.state.spreadsheet;
    if sort.sort_col == Some(col) {
        sort.sort_ascending = match sort.sort_ascending {
            None => Some(true),
            Some(true) => Some(false),
            Some(false) => None,
        };
        if sort.sort_ascending.is_none() {
            sort.sort_col = None;
        }
    } else {
        sort.sort_col = Some(col);
        sort.sort_ascending = Some(true);
    }
    Task::none()
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.spreadsheet.search_query = query;
    app.state.spreadsheet.scroll_y = 0.0;
    app.state.spreadsheet.smooth_scroll.stop(0.0);
    app.state.spreadsheet.scroll_controller.stop(0.0);
    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.spreadsheet.search_visible = false;
    app.state.spreadsheet.search_query.clear();
    Task::none()
}

pub fn handle_spreadsheet_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let vh = viewport.bounds().height;
    let total_h = viewport.content_bounds().height;

    let state = &mut app.state.spreadsheet;
    if vh > 0.0 {
        state.viewport_height = vh;
    }
    if total_h > 0.0 {
        state.total_content_height = total_h;
    }

    if state.smooth_scroll.is_animating
        || state.scroll_controller.is_animating()
        || state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging
    {
        return Task::none();
    }

    let delta_y = (state.scroll_y - y).abs();
    if delta_y < 1.0 {
        return Task::none();
    }

    state.smooth_scroll.stop(y);
    state.scroll_controller.stop(y);
    state.scroll_y = y;
    Task::none()
}

pub fn handle_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let state = &mut app.state.spreadsheet;
    if state.viewport_height <= 0.0 {
        return Task::none();
    }
    let vh = state.viewport_height;
    let extent = crate::core::scroll::ViewportExtent {
        content_height: state.total_content_height,
        viewport_height: vh,
    };
    let max_y = extent.max_scroll_y();

    match delta {
        iced::mouse::ScrollDelta::Lines { y, .. } => {
            if y.abs() > f32::EPSILON {
                let line_height =
                    (vh * crate::core::scroll::WHEEL_SCROLL_VIEWPORT_FRACTION).clamp(50.0, 240.0);
                let step = -y * line_height;
                state
                    .smooth_scroll
                    .start_interactive(state.scroll_y, step, max_y);
            }
            Task::none()
        }
        iced::mouse::ScrollDelta::Pixels { x, y } => {
            let now = std::time::Instant::now();
            let scaled_delta_y = -y * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;
            let scaled_delta_x = -x * crate::core::scroll::TOUCHPAD_SCROLL_MULTIPLIER;

            state.scroll_controller.set_position_y(state.scroll_y);
            state.scroll_controller.handle_input(
                crate::core::scroll::ScrollInput::Motion {
                    delta_x: scaled_delta_x,
                    delta_y: scaled_delta_y,
                    time: now,
                },
                extent,
            );

            let new_y = state.scroll_controller.position_y();
            state.scroll_y = new_y;
            state.smooth_scroll.stop(new_y);

            iced::widget::operation::scroll_to(
                CONTENT_SCROLL_ID,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
            )
        }
    }
}

pub fn handle_smooth_scroll_tick(app: &mut KglanceApp, now: std::time::Instant) -> Task<Message> {
    let state = &mut app.state.spreadsheet;
    let extent = crate::core::scroll::ViewportExtent {
        content_height: state.total_content_height,
        viewport_height: state.viewport_height,
    };
    let max_y = extent.max_scroll_y();

    // 1. ScrollController check
    if (state.scroll_controller.is_animating()
        || state.scroll_controller.state() == crate::core::scroll::GestureState::Dragging)
        && let Some(next_y) = state.scroll_controller.update(now, extent)
    {
        state.scroll_y = next_y;
        state.smooth_scroll.stop(next_y);
        let is_finished = !state.scroll_controller.is_animating();

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
    if let Some(next_y) = state.smooth_scroll.tick(state.scroll_y, now, max_y) {
        state.scroll_y = next_y;
        state.scroll_controller.set_position_y(next_y);
        let is_finished = !state.smooth_scroll.is_animating;

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
