use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::FilePreviewer;
use iced::Task;
use std::path::Path;

pub(crate) fn load_file_task(
    app: &KglanceApp,
    path: String,
    on_error: fn(String) -> Message,
) -> Task<Message> {
    let reg = app.registry.clone();
    let path_for_err = path.clone();
    Task::perform(
        async move {
            let content = FilePreviewer::parse(&*reg, Path::new(&path)).ok()?;
            Some(crate::app::messages::SystemMsg::FileLoaded { path, content }.into())
        },
        move |msg| msg.unwrap_or(on_error(path_for_err.clone())),
    )
}

pub fn handle_sibling_files_loaded(app: &mut KglanceApp, files: Vec<String>) -> Task<Message> {
    if !files.is_empty() && app.state.playlist.len() <= 1 {
        let current = app.state.file_name.clone();
        app.state.playlist = files;
        if let Some(pos) = app.state.playlist.iter().position(|p| p == &current) {
            app.state.current_index = pos;
        } else {
            app.state.current_index = 0;
        }
        if matches!(app.state.view_mode, crate::core::ViewMode::Grid(_)) {
            let thumbnails: Vec<crate::core::GridThumbnail> = app
                .state
                .playlist
                .iter()
                .map(|p| {
                    let name = std::path::Path::new(p)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.clone());
                    crate::core::GridThumbnail {
                        path: p.clone(),
                        name,
                        thumbnail_handle: None,
                        is_loading: true,
                    }
                })
                .collect();
            app.state.view_mode = crate::core::ViewMode::Grid(thumbnails);
        }
        return app.trigger_preload();
    }
    Task::none()
}

pub fn handle_next_file(app: &mut KglanceApp) -> Task<Message> {
    if !app.state.playlist.is_empty() {
        let next_idx = (app.state.current_index + 1) % app.state.playlist.len();
        app.state.current_index = next_idx;
        let next_path = app.state.playlist[next_idx].clone();

        if let Some(cached_data) = app.state.cache.get(&next_path).cloned() {
            return app.update(
                crate::app::messages::SystemMsg::FileLoaded {
                    path: next_path,
                    content: (*cached_data).clone(),
                }
                .into(),
            );
        }

        return load_file_task(app, next_path, |path| {
            crate::app::messages::SystemMsg::FilePreviewError(path).into()
        });
    }
    Task::none()
}

pub fn handle_prev_file(app: &mut KglanceApp) -> Task<Message> {
    if !app.state.playlist.is_empty() {
        let prev_idx = if app.state.current_index == 0 {
            app.state.playlist.len() - 1
        } else {
            app.state.current_index - 1
        };
        app.state.current_index = prev_idx;
        let prev_path = app.state.playlist[prev_idx].clone();

        if let Some(cached_data) = app.state.cache.get(&prev_path).cloned() {
            return app.update(
                crate::app::messages::SystemMsg::FileLoaded {
                    path: prev_path,
                    content: (*cached_data).clone(),
                }
                .into(),
            );
        }

        return load_file_task(app, prev_path, |path| {
            crate::app::messages::SystemMsg::FilePreviewError(path).into()
        });
    }
    Task::none()
}

pub fn handle_file_clicked_in_grid(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    let target_path = match &app.state.view_mode {
        crate::core::ViewMode::Grid(thumbnails) => thumbnails.get(idx).map(|t| t.path.clone()),
        _ => None,
    };
    if let Some(target_path) = target_path {
        app.state.current_index = idx;
        app.state.view_mode = crate::core::ViewMode::Detail;
        return load_file_task(app, target_path, |path| {
            crate::app::messages::SystemMsg::FilePreviewError(path).into()
        });
    }
    Task::none()
}

pub fn handle_file_clicked(app: &mut KglanceApp, idx: usize) -> Task<Message> {
    if idx < app.state.folder.rows.len() {
        app.state.folder.selected_index = Some(idx);
    }
    Task::none()
}

pub fn handle_sort_by_field(app: &mut KglanceApp, field: crate::core::SortField) -> Task<Message> {
    let sort = &mut app.state.folder.sort_state;
    if sort.active && sort.field == field {
        match sort.ascending {
            true => sort.ascending = false,
            false => {
                sort.active = false;
                sort.ascending = true;
                crate::core::sort_folder_rows(
                    &mut app.state.folder.rows,
                    &crate::core::SortState {
                        field: crate::core::SortField::Name,
                        ascending: true,
                        active: true,
                    },
                );
            }
        }
    } else {
        sort.field = field;
        sort.ascending = true;
        sort.active = true;
    }
    if sort.active {
        crate::core::sort_folder_rows(&mut app.state.folder.rows, sort);
    }
    Task::none()
}

pub fn handle_toggle_settings(app: &mut KglanceApp) -> Task<Message> {
    if matches!(app.state.view_mode, crate::core::ViewMode::Settings) {
        app.state.view_mode = crate::core::ViewMode::Detail;
    } else {
        app.state.view_mode = crate::core::ViewMode::Settings;
    }
    Task::none()
}
