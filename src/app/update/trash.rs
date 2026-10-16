use iced::Task;
use std::path::PathBuf;

use crate::app::KglanceApp;
use crate::app::messages::{Message, TrashMsg};
use crate::core::types::ToastAction;

pub fn handle_trash_msg(app: &mut KglanceApp, msg: TrashMsg) -> Task<Message> {
    match msg {
        TrashMsg::RequestTrashCurrent => handle_request_trash(app),
        TrashMsg::ConfirmTrashCurrent => {
            app.state.show_trash_confirm_dialog = false;
            execute_trash_current_file(app)
        }
        TrashMsg::CancelTrashDialog => {
            app.state.show_trash_confirm_dialog = false;
            Task::none()
        }
        TrashMsg::UndoTrash => handle_undo_trash(app),
        TrashMsg::ToastActionClicked(action) => match action {
            ToastAction::UndoTrash => handle_undo_trash(app),
        },
    }
}

pub fn handle_request_trash(app: &mut KglanceApp) -> Task<Message> {
    if !app.state.trash_config.enabled {
        return Task::none();
    }

    if app.state.file_name.is_empty() {
        return Task::none();
    }

    if app.state.trash_config.confirm_on_delete {
        app.state.show_trash_confirm_dialog = true;
        Task::none()
    } else {
        execute_trash_current_file(app)
    }
}

pub fn execute_trash_current_file(app: &mut KglanceApp) -> Task<Message> {
    let file_to_trash = app.state.file_name.clone();
    if file_to_trash.is_empty() {
        return Task::none();
    }

    let path = PathBuf::from(&file_to_trash);
    let trash_result = crate::core::trash::move_to_trash(&path);

    let item = match trash_result {
        Ok(item) => item,
        Err(err) => {
            return app.show_toast(format!("Failed to move to Trash: {err}"));
        }
    };

    app.state.trash_history.push(item);

    let file_name_display = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_to_trash.clone());

    let undo_timeout = app.state.trash_config.undo_timeout_seconds;
    let toast_task = app.show_toast_with_action(
        format!("Moved \"{file_name_display}\" to Trash"),
        Some(ToastAction::UndoTrash),
        undo_timeout,
    );

    let nav_task = advance_or_close_after_delete(app, &file_to_trash);

    Task::batch(vec![toast_task, nav_task])
}

fn advance_or_close_after_delete(app: &mut KglanceApp, deleted_path: &str) -> Task<Message> {
    // Remove from playlist
    if let Some(pos) = app.state.playlist.iter().position(|p| p == deleted_path) {
        app.state.playlist.remove(pos);
    }

    // Evict from memory cache
    app.state.cache.remove(deleted_path);

    if app.state.playlist.is_empty() {
        app.state.current_index = 0;
        app.state.file_name.clear();
        app.current_content = None;
        app.state.content_ready = false;

        // If no more files in playlist, close/hide current window
        app.handle_close()
    } else {
        // Move to the next available file (clamp to new bounds)
        let new_len = app.state.playlist.len();
        if app.state.current_index >= new_len {
            app.state.current_index = 0;
        }

        let next_path = app.state.playlist[app.state.current_index].clone();
        app.state.file_name = next_path.clone();

        if let crate::core::ViewMode::Grid(old_thumbnails) = &app.state.view_mode {
            let (updated_thumbnails, thumb_tasks) =
                crate::app::update::grid::sync_grid_thumbnails(old_thumbnails, &app.state.playlist);
            app.state.view_mode = crate::core::ViewMode::Grid(updated_thumbnails);
            let mut tasks = thumb_tasks;
            tasks.push(load_file_after_delete(app, next_path));
            Task::batch(tasks)
        } else {
            load_file_after_delete(app, next_path)
        }
    }
}

fn load_file_after_delete(app: &mut KglanceApp, path: String) -> Task<Message> {
    if let Some(cached) = app.state.cache.get(&path)
        && let Some(cached_data) = cached.as_preview().cloned()
    {
        let gen_id = app
            .state
            .generation_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        return app.update(
            crate::app::messages::SystemMsg::FileLoaded {
                path,
                content: (*cached_data).clone(),
                generation_id: gen_id,
            }
            .into(),
        );
    }

    crate::app::update::navigation::load_file_task(app, path, |err_path| {
        crate::app::messages::SystemMsg::FilePreviewError(err_path).into()
    })
}

pub fn handle_undo_trash(app: &mut KglanceApp) -> Task<Message> {
    let Some(last_trashed) = app.state.trash_history.pop() else {
        return app.show_toast("No item to undo");
    };

    let restore_path = last_trashed.original_path.clone();
    let restore_res = crate::core::trash::restore_trashed_item(&last_trashed);

    match restore_res {
        Ok(()) => {
            let path_str = restore_path.to_string_lossy().into_owned();
            let file_name_display = restore_path
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| path_str.clone());

            // Re-insert into playlist if directory matches
            let in_same_dir =
                restore_path.parent().map(|p| p.to_path_buf()) == app.state.active_dir;
            if in_same_dir && !app.state.playlist.contains(&path_str) {
                app.state.playlist.push(path_str.clone());
                app.state.playlist.sort();
                if let Some(pos) = app.state.playlist.iter().position(|p| p == &path_str) {
                    app.state.current_index = pos;
                }
            }
            app.state.file_name = path_str.clone();

            let toast_task = app.show_toast(format!("Restored \"{file_name_display}\""));
            let load_task =
                crate::app::update::navigation::load_file_task(app, path_str, |err_path| {
                    crate::app::messages::SystemMsg::FilePreviewError(err_path).into()
                });

            Task::batch(vec![toast_task, load_task])
        }
        Err(err) => app.show_toast(format!("Failed to restore: {err}")),
    }
}

#[cfg(test)]
#[path = "trash_tests.rs"]
mod tests;
