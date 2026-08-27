use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

pub fn handle_toggle_mode(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.tree_mode = !app.state.json.tree_mode;

    if !app.state.json.tree_mode {
        let s = &mut app.state.json;
        let content = if s.raw_pretty {
            &s.pretty_content
        } else {
            &s.minified_content
        };
        s.raw_editor = iced::widget::text_editor::Content::with_text(content);
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

pub fn handle_raw_edit(
    app: &mut KglanceApp,
    action: iced::widget::text_editor::Action,
) -> Task<Message> {
    if !matches!(action, iced::widget::text_editor::Action::Edit(_)) {
        app.state.json.raw_editor.perform(action);
    }
    Task::none()
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
        &s.pretty_content
    } else {
        if s.minified_content.is_empty() {
            s.minified_content = serde_json::from_str::<serde_json::Value>(&s.pretty_content)
                .ok()
                .and_then(|v| serde_json::to_string(&v).ok())
                .unwrap_or_else(|| s.pretty_content.clone());
        }
        &s.minified_content
    };

    s.raw_editor = iced::widget::text_editor::Content::with_text(content);

    Task::none()
}
