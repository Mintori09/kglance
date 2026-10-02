use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::features::text::anchor::ViewportAnchor;
use crate::features::text::display_map::WrapMode;
use crate::features::text::indexer::IndexResult;
use crate::ui::components::code_viewer::SelectionRange;
use iced::Task;

const CONTENT_SCROLL_ID: &str = "content_scroll";

pub fn handle_selection_changed(
    app: &mut KglanceApp,
    selection: Option<SelectionRange>,
) -> Task<Message> {
    app.state.text.selection = selection;
    Task::none()
}

pub fn handle_copy_requested(_app: &mut KglanceApp, _text: String) -> Task<Message> {
    Task::none()
}

pub fn handle_tokens_ready(app: &mut KglanceApp, result: IndexResult) -> Task<Message> {
    if result.version == app.state.text.cached_tokens_version {
        app.state.text.cached_tokens = result.spans;
        app.state.text.cached_tokens_start_line = result.start_line;
    }
    Task::none()
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.text.search_query = query.clone();
    app.state.text.search_matches.clear();
    app.state.text.search_match_index = 0;

    if query.is_empty() {
        app.state.text.search_info = String::new();
        return Task::none();
    }

    let query_lower = query.to_lowercase();
    let total_lines = app.state.text.document.total_lines();
    for line_idx in 0..total_lines {
        let line_text = app.state.text.document.get_line(line_idx);
        let line_lower = line_text.to_lowercase();
        let mut start_search = 0;
        while let Some(found_byte) = line_lower[start_search..].find(&query_lower) {
            let actual_byte = start_search + found_byte;
            let col = line_text[..actual_byte].chars().count() + 1;
            app.state.text.search_matches.push((line_idx + 1, col));
            start_search = actual_byte + query_lower.len().max(1);
        }
    }

    let match_count = app.state.text.search_matches.len();
    if match_count > 0 {
        app.state.text.search_info = format!("1/{match_count}");
    } else {
        app.state.text.search_info = "0/0".to_string();
    }

    Task::none()
}

pub fn update_tokens_for_viewport(
    text_state: &mut crate::core::TextState,
    y: f32,
    theme: crate::ui::theme::AppTheme,
) {
    let total_lines = text_state.document.total_lines();
    if total_lines == 0 {
        return;
    }

    let (visible_start, visible_end) =
        text_state
            .display_map
            .compute_visible_range(y, text_state.viewport_height, 0);

    let current_start = text_state.cached_tokens_start_line;
    let current_end = current_start + text_state.cached_tokens.len();

    let threshold = 5;
    let needs_update = text_state.cached_tokens.is_empty()
        || (current_start > 0 && visible_start < current_start + threshold)
        || visible_end + threshold >= current_end;

    if needs_update {
        let overscan = 50;
        let start_line = visible_start.saturating_sub(overscan);
        let end_line = (visible_end + overscan).min(total_lines.saturating_sub(1));

        let spans = text_state.syntax_cache.get_or_tokenize_range(
            start_line,
            end_line,
            &text_state.document,
            theme,
        );
        text_state.cached_tokens = spans;
        text_state.cached_tokens_start_line = start_line;
    }
}

pub fn handle_search_next(app: &mut KglanceApp) -> Task<Message> {
    let match_count = app.state.text.search_matches.len();
    if match_count == 0 {
        return Task::none();
    }

    app.state.text.search_match_index = (app.state.text.search_match_index + 1) % match_count;
    let idx = app.state.text.search_match_index;
    app.state.text.search_info = format!("{}/{}", idx + 1, match_count);

    if let Some(&(line_num, _)) = app.state.text.search_matches.get(idx) {
        let line_idx = line_num.saturating_sub(1);
        let target_y = app.state.text.display_map.get_line_y(line_idx);
        let text_state = &mut app.state.text;
        let max_y = crate::core::scroll::max_scroll_y(
            text_state.total_content_height,
            text_state.viewport_height,
        );
        text_state
            .smooth_scroll
            .start_navigation(text_state.scroll_y, target_y, max_y);
        let theme = app.state.app_theme;
        update_tokens_for_viewport(&mut app.state.text, target_y, theme);
    }

    Task::none()
}

pub fn handle_search_prev(app: &mut KglanceApp) -> Task<Message> {
    let match_count = app.state.text.search_matches.len();
    if match_count == 0 {
        return Task::none();
    }

    app.state.text.search_match_index = if app.state.text.search_match_index == 0 {
        match_count - 1
    } else {
        app.state.text.search_match_index - 1
    };
    let idx = app.state.text.search_match_index;
    app.state.text.search_info = format!("{}/{}", idx + 1, match_count);

    if let Some(&(line_num, _)) = app.state.text.search_matches.get(idx) {
        let line_idx = line_num.saturating_sub(1);
        let target_y = app.state.text.display_map.get_line_y(line_idx);
        let text_state = &mut app.state.text;
        let max_y = crate::core::scroll::max_scroll_y(
            text_state.total_content_height,
            text_state.viewport_height,
        );
        text_state
            .smooth_scroll
            .start_navigation(text_state.scroll_y, target_y, max_y);
        let theme = app.state.app_theme;
        update_tokens_for_viewport(&mut app.state.text, target_y, theme);
    }

    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.search_visible = false;
    app.state.text.search_query.clear();
    app.state.text.search_matches.clear();
    app.state.text.search_info.clear();
    Task::none()
}

pub fn handle_wrap_toggled(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.wrap = !app.state.text.wrap;
    app.state.word_wrap = app.state.text.wrap;
    let wrap_mode = if app.state.text.wrap {
        WrapMode::Word
    } else {
        WrapMode::None
    };
    app.state.text.display_map.update_geometry(
        &app.state.text.document,
        app.state.text.display_map.viewport_width,
        app.state.font_size,
        wrap_mode,
    );
    app.state.text.total_content_height = app.state.text.display_map.total_content_height();
    let mut config = crate::core::config::ConfigManager::load_or_create();
    config.ui.word_wrap = app.state.word_wrap;
    let _ = crate::core::config::ConfigManager::save(&config);
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
    let vw = viewport.bounds().width;
    let vh = viewport.bounds().height;
    let total_h = viewport.content_bounds().height;

    handle_text_scrolled_values(app, y, vw, vh, total_h)
}

pub fn handle_text_scrolled_values(
    app: &mut KglanceApp,
    y: f32,
    vw: f32,
    vh: f32,
    total_h: f32,
) -> Task<Message> {
    let text_state = &mut app.state.text;
    if vh > 0.0 {
        text_state.viewport_height = vh;
    }
    if total_h > 0.0 {
        text_state.total_content_height = total_h;
    }
    if vw > 0.0 && (vw - text_state.display_map.viewport_width).abs() > 20.0 {
        let wrap_mode = if text_state.wrap {
            WrapMode::Word
        } else {
            WrapMode::None
        };
        text_state.display_map.update_geometry(
            &text_state.document,
            vw,
            app.state.font_size,
            wrap_mode,
        );
        text_state.total_content_height = text_state.display_map.total_content_height();
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

    // Update Viewport Anchor
    let line_height = app.state.font_size * 1.35;
    text_state.anchor =
        ViewportAnchor::from_scroll_y(y, line_height, text_state.document.total_lines());

    let theme = app.state.app_theme;
    update_tokens_for_viewport(&mut app.state.text, y, theme);

    app.record_read_position();
    Task::none()
}

pub fn handle_text_state_wheel_scrolled(
    text_state: &mut crate::core::TextState,
    theme: crate::ui::theme::AppTheme,
    scroll_id: &'static str,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
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
            update_tokens_for_viewport(text_state, new_y, theme);

            iced::widget::operation::scroll_to(
                scroll_id,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
            )
        }
    }
}

pub fn handle_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let theme = app.state.app_theme;
    handle_text_state_wheel_scrolled(&mut app.state.text, theme, CONTENT_SCROLL_ID, delta)
}

pub fn advance_text_state_smooth_scroll(
    text_state: &mut crate::core::TextState,
    theme: crate::ui::theme::AppTheme,
    scroll_id: &'static str,
    now: std::time::Instant,
) -> (Task<Message>, bool) {
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
        update_tokens_for_viewport(text_state, next_y, theme);

        let scroll_task = if is_finished && next_y >= max_y - 1.0 {
            iced::widget::operation::snap_to(
                scroll_id,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                scroll_id,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                scroll_id,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };

        return (scroll_task, is_finished);
    }

    // 2. SmoothScroller check (keyboard navigation & discrete wheel step)
    if let Some(next_y) = text_state
        .smooth_scroll
        .tick(text_state.scroll_y, now, max_y)
    {
        text_state.scroll_y = next_y;
        text_state.scroll_controller.set_position_y(next_y);
        let is_finished = !text_state.smooth_scroll.is_animating;
        update_tokens_for_viewport(text_state, next_y, theme);

        let scroll_task = if is_finished && next_y >= max_y - 1.0 {
            iced::widget::operation::snap_to(
                scroll_id,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                scroll_id,
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                scroll_id,
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };

        return (scroll_task, is_finished);
    }

    (Task::none(), false)
}

pub fn handle_smooth_scroll_tick(app: &mut KglanceApp, now: std::time::Instant) -> Task<Message> {
    let theme = app.state.app_theme;
    let (task, finished) =
        advance_text_state_smooth_scroll(&mut app.state.text, theme, CONTENT_SCROLL_ID, now);
    if finished {
        app.record_read_position();
    }
    task
}

pub fn handle_toggle_outline(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.outline_visible = !app.state.text.outline_visible;
    Task::none()
}

pub fn handle_symbol_clicked(app: &mut KglanceApp, line_number: usize) -> Task<Message> {
    let line_idx = line_number.saturating_sub(1);
    let target_y = app.state.text.display_map.get_line_y(line_idx);

    let text_state = &mut app.state.text;
    let max_y = crate::core::scroll::max_scroll_y(
        text_state.total_content_height,
        text_state.viewport_height,
    );
    text_state
        .smooth_scroll
        .start_navigation(text_state.scroll_y, target_y, max_y);
    let theme = app.state.app_theme;
    update_tokens_for_viewport(&mut app.state.text, target_y, theme);
    Task::none()
}

pub fn handle_goto_line_toggle(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.goto_line_visible = !app.state.text.goto_line_visible;
    if app.state.text.goto_line_visible {
        app.state.text.search_visible = false;
        app.state.text.goto_line_query.clear();
        iced::widget::operation::focus(crate::ui::components::search_bar::GOTO_LINE_INPUT_ID)
    } else {
        app.state.text.goto_line_query.clear();
        Task::none()
    }
}

pub fn handle_goto_line_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.text.goto_line_query = query;
    Task::none()
}

pub fn handle_goto_line_submitted(app: &mut KglanceApp) -> Task<Message> {
    let query = app.state.text.goto_line_query.trim();
    if let Ok(line) = query.parse::<usize>() {
        let total_lines = app.state.text.document.total_lines().max(1);
        let target_line = line.clamp(1, total_lines);
        let line_idx = target_line - 1;
        let target_y = app.state.text.display_map.get_line_y(line_idx);

        let text_state = &mut app.state.text;
        let max_y = crate::core::scroll::max_scroll_y(
            text_state.total_content_height,
            text_state.viewport_height,
        );
        text_state
            .smooth_scroll
            .start_navigation(text_state.scroll_y, target_y, max_y);
        let theme = app.state.app_theme;
        update_tokens_for_viewport(&mut app.state.text, target_y, theme);

        app.state.text.goto_line_visible = false;
        app.state.text.goto_line_query.clear();
    }
    Task::none()
}

pub fn handle_goto_line_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.goto_line_visible = false;
    app.state.text.goto_line_query.clear();
    Task::none()
}
