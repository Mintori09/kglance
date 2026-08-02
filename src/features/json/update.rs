use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;
use iced::widget::operation;

pub fn handle_toggle_mode(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.tree_mode = !app.state.json.tree_mode;
    Task::none()
}

pub fn handle_toggle_node(app: &mut KglanceApp, index: usize) -> Task<Message> {
    if app.state.json.expanded.contains(&index) {
        app.state.json.expanded.remove(&index);
    } else {
        app.state.json.expanded.insert(index);
    }
    Task::none()
}

pub fn handle_scrolled(app: &mut KglanceApp, y: f32) -> Task<Message> {
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
        Task::none()
    } else {
        operation::focus("json_search_input")
    }
}

pub fn handle_search_query_changed(app: &mut KglanceApp, query: String) -> Task<Message> {
    app.state.json.search_query = query;
    Task::none()
}

pub fn handle_search_closed(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.search_visible = false;
    app.state.json.search_query.clear();
    Task::none()
}

pub fn handle_expand_all(app: &mut KglanceApp) -> Task<Message> {
    for (i, node) in app.state.json.nodes.iter().enumerate() {
        if node.children_count > 0 {
            app.state.json.expanded.insert(i);
        }
    }
    Task::none()
}

pub fn handle_collapse_all(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.expanded.clear();
    Task::none()
}

pub fn handle_copy_path(app: &mut KglanceApp, index: usize) -> Task<Message> {
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

pub fn handle_node_clicked(app: &mut KglanceApp, index: usize) -> Task<Message> {
    app.state.json.active_node = Some(index);
    Task::none()
}

pub fn handle_breadcrumb_clicked(_app: &mut KglanceApp, _index: usize) -> Task<Message> {
    operation::scroll_to("json_scroll", operation::AbsoluteOffset { x: 0.0, y: 0.0 })
}

pub fn handle_toggle_format(app: &mut KglanceApp) -> Task<Message> {
    let s = &mut app.state.json;
    let content = if s.raw_pretty {
        s.minified_content.clone()
    } else {
        s.pretty_content.clone()
    };
    s.raw_editor = iced::widget::text_editor::Content::with_text(&content);
    s.raw_pretty = !s.raw_pretty;
    Task::none()
}

pub fn handle_edit_start(app: &mut KglanceApp, index: usize) -> Task<Message> {
    if let Some(node) = app.state.json.nodes.get(index) {
        let val = node.value_preview.clone();
        app.state.json.editing_node = Some(index);
        app.state.json.edit_value = val;
    }
    Task::none()
}

pub fn handle_edit_value(app: &mut KglanceApp, val: String) -> Task<Message> {
    app.state.json.edit_value = val;
    Task::none()
}

pub fn handle_edit_save(app: &mut KglanceApp) -> Task<Message> {
    if let Some(_idx) = app.state.json.editing_node {
        let path_str = app.state.file_name.clone();
        app.state.json.editing_node = None;
        app.state.json.edit_value.clear();
        return crate::app::update::navigation::load_file_task(app, path_str, |_| {
            crate::app::messages::SystemMsg::ToastDismissed(0).into()
        });
    }
    Task::none()
}

pub fn handle_edit_cancel(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.editing_node = None;
    app.state.json.edit_value.clear();
    Task::none()
}

pub fn handle_schema_toggle(app: &mut KglanceApp) -> Task<Message> {
    app.state.json.schema_visible = !app.state.json.schema_visible;
    Task::none()
}
