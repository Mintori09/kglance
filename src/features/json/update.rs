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
        app.state.json.raw_text = crate::features::text::create_text_state(
            content,
            "json",
            app.state.font_size,
            app.state.text.wrap,
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

pub fn handle_scrolled(app: &mut KglanceApp, y: f32) -> Task<Message> {
    if app.ctrl_held {
        return Task::none();
    }
    app.state.json.scroll_y = y;
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
    let total_h = viewport.content_bounds().height;

    let text_state = &mut app.state.json.raw_text;
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

    s.raw_text = crate::features::text::create_text_state(
        content,
        "json",
        app.state.font_size,
        app.state.text.wrap,
    );

    Task::none()
}
