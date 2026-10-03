use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

pub fn handle_toggle_mode(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.tree_mode = !app.state.json.tree_mode;

    if !app.state.json.tree_mode {
        let content = if app.state.json.raw_pretty {
            app.state.json.pretty_content.clone()
        } else {
            app.state.json.minified_content.clone()
        };
        let win_w = if app.state.current_window_size.width > 0.0 {
            app.state.current_window_size.width
        } else if app.state.window_width > 0.0 {
            app.state.window_width
        } else {
            1024.0
        };
        app.state.json.raw_text = crate::features::text::create_text_state(
            content,
            "json",
            app.state.font_size,
            app.state.word_wrap,
            app.state.app_theme,
            win_w,
        );
    }

    Task::none()
}

pub fn handle_toggle_node(app: &mut KglanceApp, index: usize) -> Task<Message> {
    if app.state.json.expanded.contains(&index) {
        app.state.json.expanded.remove(&index);
    } else {
        app.state.json.expanded.insert(index);
    }
    app.state.json.active_node = Some(index);
    Task::none()
}

pub fn handle_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let vh = viewport.bounds().height;
    let total_h = viewport.content_bounds().height;

    let state = &mut app.state.json;
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
    let state = &mut app.state.json;
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
                "content_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: new_y },
            )
        }
    }
}

pub fn handle_smooth_scroll_tick(app: &mut KglanceApp, now: std::time::Instant) -> Task<Message> {
    let state = &mut app.state.json;
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
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                "content_scroll",
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
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
            )
        } else if is_finished && next_y <= 1.0 {
            iced::widget::operation::snap_to(
                "content_scroll",
                iced::widget::operation::RelativeOffset { x: 0.0, y: 0.0 },
            )
        } else {
            iced::widget::operation::scroll_to(
                "content_scroll",
                iced::widget::operation::AbsoluteOffset { x: 0.0, y: next_y },
            )
        };

        return scroll_task;
    }

    Task::none()
}

pub fn handle_raw_selection_changed(
    app: &mut KglanceApp,
    selection: Option<crate::ui::components::code_viewer::SelectionRange>,
) -> Task<Message> {
    app.state.json.raw_text.selection = selection;
    Task::none()
}

pub fn handle_raw_scrolled(
    app: &mut KglanceApp,
    viewport: iced::widget::scrollable::Viewport,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let y = viewport.absolute_offset().y;
    let vh = viewport.bounds().height;
    let vw = viewport.bounds().width;
    let total_h = viewport.content_bounds().height;

    let text_state = &mut app.state.json.raw_text;
    if vh > 0.0 {
        text_state.viewport_height = vh;
    }
    if total_h > 0.0 {
        text_state.total_content_height = total_h;
    }
    if vw > 0.0 && (vw - text_state.display_map.viewport_width).abs() > 20.0 {
        let wrap_mode = if app.state.word_wrap {
            crate::features::text::WrapMode::Word
        } else {
            crate::features::text::WrapMode::None
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

    let theme = app.state.app_theme;
    crate::features::text::update_tokens_for_viewport(&mut app.state.json.raw_text, y, theme);
    Task::none()
}

pub fn handle_raw_wheel_scrolled(
    app: &mut KglanceApp,
    delta: iced::mouse::ScrollDelta,
) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    let theme = app.state.app_theme;
    crate::features::text::update::handle_text_state_wheel_scrolled(
        &mut app.state.json.raw_text,
        theme,
        "json_raw_scroll",
        delta,
    )
}

pub fn handle_search_toggle(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.json;
    s.search_visible = !s.search_visible;
    if !s.search_visible {
        s.search_query.clear();
        s.search_matches.clear();
        s.search_match_index = 0;
        Task::none()
    } else {
        recompute_search_matches(s);
        operation::focus("json_search_input")
    }
}

fn recompute_search_matches(s: &mut crate::core::types::JsonState) {
    let q = s.search_query.trim().to_lowercase();
    s.search_matches.clear();
    s.search_match_index = 0;

    if q.is_empty() {
        s.search_info.clear();
        return;
    }

    for (i, node) in s.nodes.iter().enumerate() {
        let key_match = node
            .key
            .as_ref()
            .is_some_and(|k| k.to_lowercase().contains(&q));
        let val_match = node.value_preview.to_lowercase().contains(&q);
        if key_match || val_match {
            s.search_matches.push(i);
        }
    }

    update_search_info(s);
}

fn update_search_info(s: &mut crate::core::types::JsonState) {
    if s.search_query.trim().is_empty() {
        s.search_info.clear();
    } else if s.search_matches.is_empty() {
        s.search_info = "0 matches".to_string();
    } else {
        s.search_info = format!(
            "{}/{} matches",
            s.search_match_index + 1,
            s.search_matches.len()
        );
    }
}

fn expand_to_node(s: &mut crate::core::types::JsonState, target_index: usize) {
    let mut current = s.nodes.get(target_index).and_then(|n| n.parent_index);
    while let Some(parent_idx) = current {
        s.expanded.insert(parent_idx);
        current = s.nodes.get(parent_idx).and_then(|n| n.parent_index);
    }
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.json.search_query = query;
    recompute_search_matches(&mut app.state.json);

    if let Some(&first_match) = app.state.json.search_matches.first() {
        app.state.json.active_node = Some(first_match);
        expand_to_node(&mut app.state.json, first_match);
    }
    Task::none()
}

pub fn handle_search_next(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.json;
    if s.search_matches.is_empty() {
        return Task::none();
    }

    s.search_match_index = (s.search_match_index + 1) % s.search_matches.len();
    update_search_info(s);
    let target = s.search_matches[s.search_match_index];
    s.active_node = Some(target);
    expand_to_node(s, target);
    Task::none()
}

pub fn handle_search_prev(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.json;
    if s.search_matches.is_empty() {
        return Task::none();
    }

    if s.search_match_index == 0 {
        s.search_match_index = s.search_matches.len() - 1;
    } else {
        s.search_match_index -= 1;
    }
    update_search_info(s);
    let target = s.search_matches[s.search_match_index];
    s.active_node = Some(target);
    expand_to_node(s, target);
    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.search_visible = false;
    app.state.json.search_query.clear();
    app.state.json.search_matches.clear();
    app.state.json.search_match_index = 0;
    app.state.json.search_info.clear();
    Task::none()
}

pub fn handle_expand_all(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.json;
    let mut expanded =
        rustc_hash::FxHashSet::with_capacity_and_hasher(s.nodes.len() / 2, Default::default());

    for (i, node) in s.nodes.iter().enumerate() {
        if node.children_count > 0 {
            expanded.insert(i);
        }
    }
    s.expanded = expanded;
    Task::none()
}

pub fn handle_collapse_all(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.expanded.clear();
    Task::none()
}

pub fn handle_copy_path(app: &mut KglanceApp, index: usize) -> Task<Message> {
    let path =
        crate::features::json::view::components::build_json_path(&app.state.json.nodes, index);
    let toast = app.show_toast("Copied JSON Path!");
    Task::batch(vec![iced::clipboard::write(path), toast])
}

pub fn handle_copy_value(app: &mut KglanceApp, index: usize) -> Task<Message> {
    let val = app
        .state
        .json
        .nodes
        .get(index)
        .map(|n| n.value_preview.clone())
        .unwrap_or_default();
    let toast = app.show_toast("Copied value!");
    Task::batch(vec![iced::clipboard::write(val), toast])
}

pub fn handle_copy_key(app: &mut KglanceApp, index: usize) -> Task<Message> {
    let key = app
        .state
        .json
        .nodes
        .get(index)
        .and_then(|n| n.key.clone())
        .unwrap_or_default();
    if key.is_empty() {
        return Task::none();
    }
    let toast = app.show_toast("Copied Key!");
    Task::batch(vec![iced::clipboard::write(key), toast])
}

pub fn handle_copy_subtree(app: &mut KglanceApp, index: usize) -> Task<Message> {
    let json_text = crate::features::json::parser::JsonParser::extract_subtree_json(
        &app.state.json.nodes,
        index,
    )
    .unwrap_or_default();
    if json_text.is_empty() {
        return Task::none();
    }
    let toast = app.show_toast("Copied JSON subtree!");
    Task::batch(vec![iced::clipboard::write(json_text), toast])
}

pub fn handle_node_clicked(app: &mut KglanceApp, index: usize) -> Task<Message> {
    app.state.json.active_node = Some(index);
    Task::none()
}

pub fn handle_breadcrumb_clicked(app: &mut KglanceApp, index: usize) -> Task<Message> {
    app.state.json.active_node = Some(index);
    if app
        .state
        .json
        .nodes
        .get(index)
        .is_some_and(|n| n.children_count > 0)
    {
        app.state.json.expanded.insert(index);
    }
    Task::none()
}

pub fn handle_toggle_format(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.json;
    s.raw_pretty = !s.raw_pretty;

    let content = if s.raw_pretty {
        s.pretty_content.clone()
    } else {
        if s.minified_content.is_empty() {
            s.minified_content = serde_json::from_str::<serde_json::Value>(&s.pretty_content)
                .ok()
                .and_then(|v| serde_json::to_string(&v).ok())
                .unwrap_or_else(|| s.pretty_content.clone());
        }
        s.minified_content.clone()
    };

    let win_w = if app.state.current_window_size.width > 0.0 {
        app.state.current_window_size.width
    } else if app.state.window_width > 0.0 {
        app.state.window_width
    } else {
        1024.0
    };

    s.raw_text = crate::features::text::create_text_state(
        content,
        "json",
        app.state.font_size,
        app.state.word_wrap,
        app.state.app_theme,
        win_w,
    );

    Task::none()
}

pub const FONT_MIN: f32 = 8.0;
pub const FONT_MAX: f32 = 48.0;

pub fn zoom_json_font(app: &mut KglanceApp, direction: f32) -> Option<Task<Message>> {
    let target = app.state.font_size + direction;
    rescale_json_font(app, target)
}

pub fn rescale_json_font(app: &mut KglanceApp, new_size: f32) -> Option<Task<Message>> {
    let old_size = app.state.font_size;
    let new_size = new_size.clamp(FONT_MIN, FONT_MAX);
    if (new_size - old_size).abs() < f32::EPSILON {
        return Some(Task::none());
    }
    app.state.font_size = new_size;

    if app.state.json.tree_mode {
        let old_row_h = (old_size * 1.2).max(18.0) + 4.0;
        let new_row_h = (new_size * 1.2).max(18.0) + 4.0;
        let node_index = if old_row_h > 0.0 {
            (app.state.json.scroll_y / old_row_h).max(0.0)
        } else {
            0.0
        };
        let new_scroll_y = (node_index * new_row_h).max(0.0);
        app.state.json.scroll_y = new_scroll_y;
        Some(iced::widget::operation::scroll_to(
            "content_scroll",
            iced::widget::operation::AbsoluteOffset {
                x: 0.0,
                y: new_scroll_y,
            },
        ))
    } else {
        let win_w = app.state.current_window_size.width;
        let theme = app.state.app_theme;
        let word_wrap = app.state.word_wrap;
        let new_scroll_y = crate::features::text::rescale_text_geometry(
            &mut app.state.json.raw_text,
            old_size,
            new_size,
            word_wrap,
            win_w,
            theme,
        );
        app.state.json.scroll_y = new_scroll_y;
        Some(iced::widget::operation::scroll_to(
            "json_raw_scroll",
            iced::widget::operation::AbsoluteOffset {
                x: 0.0,
                y: new_scroll_y,
            },
        ))
    }
}
