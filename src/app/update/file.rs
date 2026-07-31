use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::{FilePreviewer, PreviewData};
use iced::Task;
use std::path::Path;

pub fn handle_preload_completed(
    app: &mut KglanceApp,
    path: String,
    content: std::sync::Arc<PreviewData>,
) -> Task<Message> {
    app.state.pending_preloads.remove(&path);
    app.state.cache.put(path, content);
    Task::none()
}

pub fn handle_file_loaded_msg(
    app: &mut KglanceApp,
    path: String,
    content: PreviewData,
) -> Task<Message> {
    if !app.state.playlist.contains(&path) {
        app.state.playlist.clear();
    }
    app.handle_file_loaded(path, content)
}

pub fn handle_daemon_open_with_playlist(
    app: &mut KglanceApp,
    path: String,
    content: PreviewData,
    playlist: Vec<String>,
) -> Task<Message> {
    if !playlist.is_empty() {
        app.state.playlist = playlist;
        if let Some(pos) = app.state.playlist.iter().position(|p| p == &path) {
            app.state.current_index = pos;
        } else {
            app.state.current_index = 0;
        }
    }
    app.handle_file_loaded(path, content)
}

pub fn handle_file_preview_error(app: &mut KglanceApp, path: String) -> Task<Message> {
    let name = Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(path);
    app.show_toast(format!("\"{}\" cannot be previewed", name))
}

pub fn handle_file_changed(app: &mut KglanceApp, path: String) -> Task<Message> {
    if path == app.state.file_name {
        let path_obj = Path::new(&path);
        if !path_obj.exists() {
            let name = path_obj
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(path);
            app.current_content = None;
            app.state.content_ready = false;
            return app.show_toast(format!("File deleted: \"{}\"", name));
        }

        let prev_md_y = app.state.markdown.scroll_y;
        let prev_txt_y = app.state.text.scroll_y;

        if let Ok(content) = FilePreviewer::parse(&*app.registry, path_obj) {
            let is_md = matches!(content, PreviewData::Markdown { .. });
            let is_txt = matches!(content, PreviewData::Text { .. });

            let load_task = app.handle_file_loaded(path, content);

            let target_y = if is_md {
                prev_md_y
            } else if is_txt {
                prev_txt_y
            } else {
                0.0
            };

            if target_y > 0.0 {
                let scroll_task = iced::widget::operation::scroll_to(
                    "content_scroll",
                    iced::widget::operation::AbsoluteOffset {
                        x: 0.0,
                        y: target_y,
                    },
                );
                return Task::batch(vec![load_task, scroll_task]);
            } else {
                return load_task;
            }
        }
    }
    Task::none()
}
