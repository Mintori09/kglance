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
    let path_for_parse = path.clone();
    let gen_id = app
        .state
        .generation_id
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        + 1;
    let generation_id = app.state.generation_id.clone();
    let gen_check = app.state.generation_id.clone();
    Task::perform(
        async move {
            let content = tokio::task::spawn_blocking(move || {
                if generation_id.load(std::sync::atomic::Ordering::Relaxed) != gen_id {
                    return None;
                }
                FilePreviewer::parse(&*reg, Path::new(&path_for_parse)).ok()
            })
            .await
            .ok()
            .flatten()?;
            Some(
                crate::app::messages::SystemMsg::FileLoaded {
                    path,
                    content,
                    generation_id: gen_id,
                }
                .into(),
            )
        },
        move |msg| {
            if let Some(m) = msg {
                m
            } else if gen_check.load(std::sync::atomic::Ordering::Relaxed) == gen_id {
                on_error(path_for_err.clone())
            } else {
                crate::app::Message::None
            }
        },
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
    if app.state.playlist.is_empty() {
        return Task::none();
    }

    let next_idx = (app.state.current_index + 1) % app.state.playlist.len();
    app.state.current_index = next_idx;
    let next_path = app.state.playlist[next_idx].clone();

    if let Some(cached_data) = app.state.cache.get(&next_path) {
        let gen_id = app
            .state
            .generation_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let msg = crate::app::messages::SystemMsg::FileLoaded {
            path: next_path,
            content: (**cached_data).clone(),
            generation_id: gen_id,
        };
        return Task::done(msg.into());
    }

    load_file_task(app, next_path, |path| {
        crate::app::messages::SystemMsg::FilePreviewError(path).into()
    })
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
            let gen_id = app
                .state
                .generation_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            return app.update(
                crate::app::messages::SystemMsg::FileLoaded {
                    path: prev_path,
                    content: (*cached_data).clone(),
                    generation_id: gen_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::preview::PreviewData;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn dummy_preview_data() -> PreviewData {
        PreviewData::Text {
            content: String::from("mock data"),
            line_numbers: "1".into(),
            language: "txt".into(),
        }
    }

    fn create_test_app(
        playlist: Vec<String>,
        current_index: usize,
        cached_files: Vec<(String, PreviewData)>,
    ) -> KglanceApp {
        let mut app = KglanceApp::default();

        app.state.playlist = playlist;
        app.state.current_index = current_index;
        app.state.generation_id = Arc::new(AtomicUsize::new(0));

        for (path, data) in cached_files {
            // LruCache dùng .put() thay vì .insert()
            app.state.cache.put(path, Arc::new(data));
        }

        app
    }

    #[test]
    fn test_handle_next_file_empty_playlist() {
        let mut app = create_test_app(vec![], 0, vec![]);

        let _task = handle_next_file(&mut app);

        assert_eq!(app.state.current_index, 0);
        assert_eq!(app.state.generation_id.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_handle_next_file_wraparound_index() {
        let file1 = "file1.png".to_string();
        let file2 = "file2.png".to_string();
        let playlist = vec![file1, file2];

        let mut app = create_test_app(playlist, 1, vec![]);

        let _task = handle_next_file(&mut app);

        assert_eq!(app.state.current_index, 0);
        assert_eq!(app.state.generation_id.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_handle_next_file_cache_hit() {
        let file1 = "file1.png".to_string();
        let file2 = "file2.png".to_string();

        let playlist = vec![file1.clone(), file2.clone()];
        let cache = vec![(file2.clone(), dummy_preview_data())];

        let mut app = create_test_app(playlist, 0, cache);

        let _task = handle_next_file(&mut app);

        assert_eq!(app.state.current_index, 1);
        assert_eq!(app.state.generation_id.load(Ordering::Relaxed), 1);
        assert!(app.state.cache.contains(&file2));
    }

    #[test]
    fn test_handle_next_file_cache_miss() {
        let file1 = "file1.png".to_string();
        let file2 = "file2.png".to_string();

        let playlist = vec![file1.clone(), file2.clone()];

        let mut app = create_test_app(playlist, 0, vec![]);

        let _task = handle_next_file(&mut app);

        assert_eq!(app.state.current_index, 1);
        assert_eq!(app.state.generation_id.load(Ordering::Relaxed), 1);
        assert!(!app.state.cache.contains(&file2));
    }
}
