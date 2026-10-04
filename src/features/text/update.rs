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

pub fn handle_selection_drag_started(
    app: &mut KglanceApp,
    pos: crate::ui::components::code_viewer::TextPosition,
) -> Task<Message> {
    app.state.text.is_dragging_selection = true;
    app.state.text.drag_start = Some(pos);
    Task::none()
}

pub fn handle_selection_drag_ended(app: &mut KglanceApp) -> Task<Message> {
    app.state.text.is_dragging_selection = false;
    app.state.text.auto_scroll_delta = None;
    app.state.text.drag_start = None;
    Task::none()
}

pub fn handle_auto_scroll(
    app: &mut KglanceApp,
    delta: Option<f32>,
    cursor: iced::Point,
) -> Task<Message> {
    app.state.text.auto_scroll_delta = delta;
    app.state.text.drag_last_cursor = cursor;
    Task::none()
}

pub fn handle_auto_scroll_tick(app: &mut KglanceApp) -> Task<Message> {
    let Some(delta) = app.state.text.auto_scroll_delta else {
        return Task::none();
    };

    let text_state = &mut app.state.text;
    let max_y = crate::core::scroll::max_scroll_y(
        text_state.total_content_height,
        text_state.viewport_height,
    );
    let new_scroll_y = (text_state.scroll_y + delta).clamp(0.0, max_y);
    let scroll_delta = (new_scroll_y - text_state.scroll_y).abs();

    if scroll_delta < 0.1 {
        if let Some(start_pos) = text_state.drag_start {
            if delta > 0.0 {
                let total_lines = text_state.document.total_lines();
                if total_lines > 0 {
                    let last_line = total_lines.saturating_sub(1);
                    let last_col = text_state.document.get_line(last_line).chars().count();
                    text_state.selection = Some(SelectionRange::new(
                        start_pos,
                        crate::ui::components::code_viewer::TextPosition::new(last_line, last_col),
                    ));
                }
            } else if delta < 0.0 {
                text_state.selection = Some(SelectionRange::new(
                    start_pos,
                    crate::ui::components::code_viewer::TextPosition::new(0, 0),
                ));
            }
        }
        return Task::none();
    }

    text_state.scroll_y = new_scroll_y;
    text_state.smooth_scroll.stop(new_scroll_y);
    text_state.scroll_controller.stop(new_scroll_y);

    let theme = app.state.app_theme;
    update_tokens_for_viewport(&mut app.state.text, new_scroll_y, theme);

    if let Some(start_pos) = app.state.text.drag_start {
        let cursor_pt = app.state.text.drag_last_cursor;
        let rel_x = cursor_pt.x.max(0.0);
        let rel_y = (cursor_pt.y + new_scroll_y).max(0.0);
        let current_pos = crate::ui::components::code_viewer::selection::hit_test_position(
            &app.state.text.document,
            Some(&app.state.text.display_map),
            app.state.text.wrap,
            app.state.font_size,
            iced::Point::new(rel_x, rel_y),
        );
        app.state.text.selection = Some(SelectionRange::new(start_pos, current_pos));
    }

    iced::widget::operation::scroll_to(
        CONTENT_SCROLL_ID,
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: new_scroll_y,
        },
    )
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

    if text_state.cached_tokens_start_line == 0 && text_state.cached_tokens.len() >= total_lines {
        return;
    }

    let (visible_start_vrow, visible_end_vrow) =
        text_state
            .display_map
            .compute_visible_range(y, text_state.viewport_height, 0);

    let (visible_start, _) = text_state
        .display_map
        .visual_row_to_line(visible_start_vrow);
    let (visible_end, _) = text_state
        .display_map
        .visual_row_to_line(visible_end_vrow.saturating_sub(1));

    let current_start = text_state.cached_tokens_start_line;
    let current_end = current_start + text_state.cached_tokens.len();

    let threshold = 5;
    let needs_update = text_state.cached_tokens.is_empty()
        || (current_start > 0 && visible_start < current_start + threshold)
        || visible_end + threshold >= current_end;

    if needs_update {
        let overscan = 80;
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

    let text_state = &mut app.state.text;
    let target_y = text_state
        .display_map
        .get_line_y(text_state.anchor.logical_line)
        + text_state.anchor.visual_offset_px;
    let max_y = crate::core::scroll::max_scroll_y(
        text_state.total_content_height,
        text_state.viewport_height,
    );
    let new_scroll_y = target_y.clamp(0.0, max_y);

    text_state.smooth_scroll.stop(new_scroll_y);
    text_state.scroll_controller.stop(new_scroll_y);
    text_state.scroll_y = new_scroll_y;

    let theme = app.state.app_theme;
    update_tokens_for_viewport(&mut app.state.text, new_scroll_y, theme);

    let mut config = crate::core::config::ConfigManager::load_or_create();
    config.ui.word_wrap = app.state.word_wrap;
    let _ = crate::core::config::ConfigManager::save(&config);

    iced::widget::operation::scroll_to(
        CONTENT_SCROLL_ID,
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: new_scroll_y,
        },
    )
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

pub fn handle_toggle_word_wrap(app: &mut KglanceApp) -> Option<Task<Message>> {
    if !matches!(
        app.current_content,
        Some(
            crate::core::PreviewData::Text { .. }
                | crate::core::PreviewData::Json { .. }
                | crate::core::PreviewData::Typst { .. }
        )
    ) {
        return None;
    }

    app.state.word_wrap = !app.state.word_wrap;
    app.state.text.wrap = app.state.word_wrap;
    let wrap_mode = if app.state.word_wrap {
        WrapMode::Word
    } else {
        WrapMode::None
    };
    let win_w = if app.state.current_window_size.width > 0.0 {
        app.state.current_window_size.width
    } else if app.state.window_width > 0.0 {
        app.state.window_width
    } else {
        1024.0
    };
    let sidebar_w = if app.state.text.outline_visible && app.state.text.sidebar_width > 0.0 {
        app.state.text.sidebar_width
    } else {
        0.0
    };
    let available_w = (win_w - sidebar_w).max(200.0);

    app.state.text.display_map.update_geometry(
        &app.state.text.document,
        available_w,
        app.state.font_size,
        wrap_mode,
    );
    app.state.text.total_content_height = app.state.text.display_map.total_content_height();

    let theme = app.state.app_theme;
    let text_scroll_y = app.state.text.scroll_y;
    update_tokens_for_viewport(&mut app.state.text, text_scroll_y, theme);

    if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Json { .. })
    ) {
        app.state.json.raw_text.wrap = app.state.word_wrap;
        app.state.json.raw_text.display_map.update_geometry(
            &app.state.json.raw_text.document,
            win_w,
            app.state.font_size,
            wrap_mode,
        );
        app.state.json.raw_text.total_content_height =
            app.state.json.raw_text.display_map.total_content_height();
        let json_scroll_y = app.state.json.raw_text.scroll_y;
        update_tokens_for_viewport(&mut app.state.json.raw_text, json_scroll_y, theme);
    } else if matches!(
        app.current_content,
        Some(crate::core::PreviewData::Typst { .. })
    ) {
        app.state.typst.source_text.wrap = app.state.word_wrap;
        app.state.typst.source_text.display_map.update_geometry(
            &app.state.typst.source_text.document,
            win_w,
            app.state.font_size,
            wrap_mode,
        );
        app.state.typst.source_text.total_content_height = app
            .state
            .typst
            .source_text
            .display_map
            .total_content_height();
        let typst_scroll_y = app.state.typst.source_text.scroll_y;
        update_tokens_for_viewport(&mut app.state.typst.source_text, typst_scroll_y, theme);
    }

    let mut config = crate::core::config::ConfigManager::load_or_create();
    config.ui.word_wrap = app.state.word_wrap;

    if let Err(err) = crate::core::config::ConfigManager::save(&config) {
        crate::log_error!("failed to save word-wrap preference: {err}");
    }

    Some(Task::none())
}

pub fn rescale_text_geometry(
    text_state: &mut crate::core::TextState,
    old_font_size: f32,
    new_font_size: f32,
    word_wrap: bool,
    fallback_window_width: f32,
    theme: crate::ui::theme::AppTheme,
) -> f32 {
    let wrap_mode = if word_wrap {
        WrapMode::Word
    } else {
        WrapMode::None
    };
    let vp_w = if text_state.display_map.viewport_width > 0.0 {
        text_state.display_map.viewport_width
    } else {
        fallback_window_width.max(800.0)
    };
    text_state
        .display_map
        .update_geometry(&text_state.document, vp_w, new_font_size, wrap_mode);
    text_state.total_content_height = text_state.display_map.total_content_height();

    let old_lh = old_font_size * 1.35;
    let new_lh = new_font_size * 1.35;
    let line_index = if old_lh > 0.0 {
        (text_state.scroll_y / old_lh).max(0.0)
    } else {
        0.0
    };
    let new_scroll_y = (line_index * new_lh).max(0.0);
    text_state.scroll_y = new_scroll_y;

    update_tokens_for_viewport(text_state, new_scroll_y, theme);
    new_scroll_y
}

pub const FONT_MIN: f32 = 8.0;
pub const FONT_MAX: f32 = 48.0;

pub fn zoom_text_font(app: &mut KglanceApp, direction: f32) -> Option<Task<Message>> {
    let target = app.state.font_size + direction;
    rescale_text_font(app, target)
}

pub fn rescale_text_font(app: &mut KglanceApp, new_size: f32) -> Option<Task<Message>> {
    let old_size = app.state.font_size;
    let new_size = new_size.clamp(FONT_MIN, FONT_MAX);
    if (new_size - old_size).abs() < f32::EPSILON {
        return Some(Task::none());
    }
    app.state.font_size = new_size;

    let win_w = app.state.current_window_size.width;
    let theme = app.state.app_theme;
    let word_wrap = app.state.word_wrap;
    let new_scroll_y = rescale_text_geometry(
        &mut app.state.text,
        old_size,
        new_size,
        word_wrap,
        win_w,
        theme,
    );
    Some(iced::widget::operation::scroll_to(
        "content_scroll",
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: new_scroll_y,
        },
    ))
}

pub fn rescale_text_state_font(
    text_state: &mut crate::core::TextState,
    app_font_size: &mut f32,
    new_size: f32,
    word_wrap: bool,
    fallback_window_width: f32,
    theme: crate::ui::theme::AppTheme,
    scroll_id: &'static str,
) -> Option<Task<Message>> {
    let old_size = *app_font_size;
    let new_size = new_size.clamp(FONT_MIN, FONT_MAX);
    if (new_size - old_size).abs() < f32::EPSILON {
        return Some(Task::none());
    }
    *app_font_size = new_size;

    let new_scroll_y = rescale_text_geometry(
        text_state,
        old_size,
        new_size,
        word_wrap,
        fallback_window_width,
        theme,
    );
    Some(iced::widget::operation::scroll_to(
        scroll_id,
        iced::widget::operation::AbsoluteOffset {
            x: 0.0,
            y: new_scroll_y,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::text::create_text_state;
    use crate::ui::theme::AppTheme;

    #[test]
    fn test_update_tokens_for_viewport_with_word_wrap() {
        // Create 200 long lines that wrap to 5 rows each when viewport is narrow
        let long_line =
            "let very_long_variable_name_with_lots_of_data = \"some long string value\";\n";
        let content: String = (0..200).map(|_| long_line).collect();

        let mut state = create_text_state(
            content,
            "rs",
            14.0,
            true, // word_wrap enabled
            AppTheme::Dark,
            200.0, // narrow viewport forcing wrapping
        );

        // Scroll to mid-document (e.g. scroll_y = 5000.0)
        let scroll_y = 5000.0;
        state.viewport_height = 600.0;
        update_tokens_for_viewport(&mut state, scroll_y, AppTheme::Dark);

        // Verify that tokens were fetched for the logical lines at this scroll position
        let (start_vrow, end_vrow) = state.display_map.compute_visible_range(scroll_y, 600.0, 0);
        let (expected_start_line, _) = state.display_map.visual_row_to_line(start_vrow);
        let (expected_end_line, _) = state
            .display_map
            .visual_row_to_line(end_vrow.saturating_sub(1));

        let token_start = state.cached_tokens_start_line;
        let token_end = token_start + state.cached_tokens.len();

        assert!(
            token_start <= expected_start_line,
            "token_start ({token_start}) should be <= expected_start_line ({expected_start_line})"
        );
        assert!(
            token_end >= expected_end_line,
            "token_end ({token_end}) should be >= expected_end_line ({expected_end_line})"
        );
        assert!(
            !state.cached_tokens.is_empty(),
            "cached_tokens must not be empty"
        );
    }

    #[test]
    fn test_text_drag_and_auto_scroll() {
        let mut app = KglanceApp::default();
        let content: String = (0..100)
            .map(|i| format!("line {i} with some content\n"))
            .collect();
        app.state.text = create_text_state(content, "txt", 14.0, false, AppTheme::Dark, 800.0);
        app.state.text.viewport_height = 400.0;

        let start_pos = crate::ui::components::code_viewer::TextPosition::new(10, 2);
        let _ = handle_selection_drag_started(&mut app, start_pos);
        assert!(app.state.text.is_dragging_selection);
        assert_eq!(app.state.text.drag_start, Some(start_pos));

        // Auto scroll downwards
        let _ = handle_auto_scroll(&mut app, Some(20.0), iced::Point::new(100.0, 450.0));
        assert_eq!(app.state.text.auto_scroll_delta, Some(20.0));

        let _ = handle_auto_scroll_tick(&mut app);
        assert!(app.state.text.scroll_y > 0.0);
        assert!(app.state.text.selection.is_some());
        let sel = app.state.text.selection.unwrap();
        assert_eq!(sel.start, start_pos);

        // Drag ended
        let _ = handle_selection_drag_ended(&mut app);
        assert!(!app.state.text.is_dragging_selection);
        assert_eq!(app.state.text.auto_scroll_delta, None);
        assert_eq!(app.state.text.drag_start, None);
    }
}
