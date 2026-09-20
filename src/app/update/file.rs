use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::PreviewData;
use iced::Task;
use std::path::Path;

pub fn handle_preload_completed(
    app: &mut KglanceApp,
    path: String,
    content: std::sync::Arc<PreviewData>,
    decoded_cache: Option<crate::core::CachedContent>,
) -> Task<Message> {
    app.state.pending_preloads.remove(&path);
    if app.is_daemon && !app.is_gui_open.load(std::sync::atomic::Ordering::Acquire) {
        crate::log_debug!(
            "Preload completed for {} but daemon window is closed, dropping cache",
            path
        );
        return Task::none();
    }
    if let Some(decoded) = decoded_cache {
        crate::log_debug!("Preload completed & cached (with decoded data): {}", path);
        app.state.cache.put(path, decoded);
    } else {
        crate::log_debug!("Preload completed & cached (preview only): {}", path);
        app.state
            .cache
            .put(path, crate::core::CachedContent::Preview(content));
    }
    Task::none()
}

pub fn handle_file_loaded_msg(
    app: &mut KglanceApp,
    path: String,
    content: PreviewData,
    generation_id: usize,
) -> Task<Message> {
    if generation_id
        != app
            .state
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed)
    {
        crate::log_debug!(
            "Discarding stale FileLoaded for \"{}\" (msg gen: {}, current gen: {})",
            path,
            generation_id,
            app.state
                .generation_id
                .load(std::sync::atomic::Ordering::Relaxed)
        );
        return Task::none();
    }
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
    let file_task = app.handle_file_loaded(path, content);
    let window_tasks = Task::batch(app.prepare_window_tasks());
    Task::batch(vec![file_task, window_tasks])
}

pub fn handle_daemon_update_window(
    app: &mut KglanceApp,
    path: String,
    content: PreviewData,
) -> Task<Message> {
    if app.is_daemon && app.window_id.is_none() {
        return Task::none();
    }
    let current_gen = app
        .state
        .generation_id
        .load(std::sync::atomic::Ordering::Relaxed);

    let file_task = handle_file_loaded_msg(app, path, content, current_gen);
    let window_tasks = Task::batch(app.prepare_window_tasks());
    Task::batch(vec![file_task, window_tasks])
}

pub fn handle_daemon_update_with_playlist(
    app: &mut KglanceApp,
    path: String,
    content: PreviewData,
    playlist: Vec<String>,
) -> Task<Message> {
    if app.is_daemon && app.window_id.is_none() {
        return Task::none();
    }
    handle_daemon_open_with_playlist(app, path, content, playlist)
}

pub fn handle_file_preview_error(app: &mut KglanceApp, path: String) -> Task<Message> {
    let name = Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or(path);
    app.show_toast(format!("\"{}\" cannot be previewed", name))
}

pub fn handle_file_changed(app: &mut KglanceApp, path: String) -> Task<Message> {
    if app.is_daemon && app.window_id.is_none() {
        return Task::none();
    }

    let path_obj = Path::new(&path);
    let mut tasks = Vec::new();

    // 1. Asynchronously scan files of the active preview directory if the changed path belongs to it
    let active_dir = app.state.active_dir.clone().or_else(|| {
        Path::new(&app.state.file_name)
            .parent()
            .map(|p| p.to_path_buf())
    });

    if let Some(parent) = active_dir {
        let is_sibling_or_in_dir = path_obj.parent() == Some(&parent) || path_obj == parent;
        if is_sibling_or_in_dir {
            let parent_dir = parent.clone();
            let sync_gen = app
                .state
                .dir_sync_generation_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            tasks.push(Task::perform(
                async move {
                    let files = tokio::task::spawn_blocking({
                        let dir = parent_dir.clone();
                        move || crate::core::navigation::scan_directory_files(&dir)
                    })
                    .await
                    .unwrap_or_default();
                    crate::app::messages::NavigationMsg::DirectorySyncCompleted {
                        dir: parent_dir,
                        files,
                        generation_id: sync_gen,
                    }
                    .into()
                },
                |msg| msg,
            ));
        }
    }

    // 2. Handle active file changes
    if path == app.state.file_name {
        if !path_obj.exists() {
            let name = path_obj
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(path);
            app.current_content = None;
            app.state.content_ready = false;
            tasks.push(app.show_toast(format!("File deleted: \"{}\"", name)));
            return Task::batch(tasks);
        }

        tasks.push(crate::app::update::navigation::load_file_task(
            app,
            path,
            |err_path| crate::app::messages::SystemMsg::FilePreviewError(err_path).into(),
        ));
    }

    if tasks.is_empty() {
        Task::none()
    } else {
        Task::batch(tasks)
    }
}

pub fn handle_directory_sync_completed(
    app: &mut KglanceApp,
    dir: &Path,
    new_playlist: Vec<String>,
    generation_id: usize,
) -> Task<Message> {
    if app.is_daemon && app.window_id.is_none() {
        return Task::none();
    }

    let current_gen = app
        .state
        .dir_sync_generation_id
        .load(std::sync::atomic::Ordering::Relaxed);
    if generation_id != current_gen {
        crate::log_debug!(
            "Discarding stale DirectorySyncCompleted for {:?} (msg gen: {}, current gen: {})",
            dir,
            generation_id,
            current_gen
        );
        return Task::none();
    }

    // Guard against race condition: only update if active directory still belongs to the scanned directory
    let is_same_dir = app.state.active_dir.as_deref() == Some(dir)
        || (!app.state.file_name.is_empty()
            && Path::new(&app.state.file_name).parent() == Some(dir));
    if !is_same_dir {
        return Task::none();
    }

    if new_playlist == app.state.playlist {
        return Task::none();
    }

    let mut tasks = Vec::new();
    let old_count = app.state.playlist.len();
    let new_count = new_playlist.len();

    let mut file_to_load = None;
    let mut active_file_deleted = false;
    app.state.playlist = new_playlist;
    if let Some(pos) = app
        .state
        .playlist
        .iter()
        .position(|p| p == &app.state.file_name)
    {
        app.state.current_index = pos;
        app.state.active_dir = Some(dir.to_path_buf());
    } else if let Some(first) = app.state.playlist.first() {
        app.state.current_index = 0;
        app.state.file_name = first.clone();
        app.state.active_dir = Some(dir.to_path_buf());
        file_to_load = Some(first.clone());
        active_file_deleted = true;
    } else {
        app.state.current_index = 0;
        app.state.file_name.clear();
        app.state.active_dir = Some(dir.to_path_buf());
        app.current_content = None;
        app.state.content_ready = false;
        active_file_deleted = true;
    }

    if let Some(path) = file_to_load {
        tasks.push(crate::app::update::navigation::load_file_task(
            app,
            path,
            |err_path| crate::app::messages::SystemMsg::FilePreviewError(err_path).into(),
        ));
    } else {
        tasks.push(app.trigger_preload());
    }

    // Synchronize Grid view if currently in Grid mode
    if let crate::core::ViewMode::Grid(thumbnails) = &app.state.view_mode {
        let (updated_thumbnails, thumb_tasks) =
            crate::app::update::grid::sync_grid_thumbnails(thumbnails, &app.state.playlist);
        app.state.view_mode = crate::core::ViewMode::Grid(updated_thumbnails);
        tasks.extend(thumb_tasks);
    }

    // Show toast notification on directory file count change or active file removal
    if active_file_deleted && !app.state.playlist.is_empty() {
        tasks.push(app.show_toast("Current file removed; showing first file".to_string()));
    } else if old_count > 0 && old_count != new_count && !active_file_deleted {
        let toast_msg = if new_count > old_count {
            let diff = new_count - old_count;
            if diff == 1 {
                "1 new file added in folder".to_string()
            } else {
                format!("{} new files added in folder", diff)
            }
        } else {
            let diff = old_count - new_count;
            if diff == 1 {
                "1 file removed from folder".to_string()
            } else {
                format!("{} files removed from folder", diff)
            }
        };
        tasks.push(app.show_toast(toast_msg));
    }

    if tasks.is_empty() {
        Task::none()
    } else {
        Task::batch(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_util::test_app;

    #[test]
    fn test_file_changed_ignored_when_daemon_window_closed() {
        let mut app = test_app(None);
        app.is_daemon = true;
        app.window_id = None;
        app.state.file_name = "/tmp/test.md".to_string();

        let _task = handle_file_changed(&mut app, "/tmp/test.md".to_string());
        assert!(app.window_id.is_none());
        assert!(app.current_content.is_none());
    }

    #[tokio::test]
    async fn test_file_changed_active_file_deleted_resets_content() {
        let mut app = test_app(None);
        app.state.file_name = "/tmp/non_existent_deleted_file_12345.md".to_string();
        app.state.content_ready = true;
        app.current_content = Some(PreviewData::Text {
            content: "deleted soon".to_string(),
            line_numbers: "1".into(),
            language: "".into(),
        });

        let _task = handle_file_changed(
            &mut app,
            "/tmp/non_existent_deleted_file_12345.md".to_string(),
        );
        assert!(!app.state.content_ready);
        assert!(app.current_content.is_none());
    }

    #[tokio::test]
    async fn test_directory_sync_completed_updates_playlist_and_index() {
        let mut app = test_app(None);
        app.state.file_name = "/path/to/b.txt".to_string();
        app.state.playlist = vec!["/path/to/a.txt".to_string(), "/path/to/b.txt".to_string()];
        app.state.current_index = 1;

        let new_playlist = vec![
            "/path/to/0_new.txt".to_string(),
            "/path/to/a.txt".to_string(),
            "/path/to/b.txt".to_string(),
        ];

        let gen_id = app
            .state
            .dir_sync_generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let _task =
            handle_directory_sync_completed(&mut app, Path::new("/path/to"), new_playlist, gen_id);
        assert_eq!(app.state.playlist.len(), 3);
        assert_eq!(app.state.current_index, 2);
    }

    #[tokio::test]
    async fn test_directory_sync_completed_when_active_file_removed_from_playlist() {
        let mut app = test_app(None);
        app.state.file_name = "/path/to/b.txt".to_string();
        app.state.playlist = vec!["/path/to/a.txt".to_string(), "/path/to/b.txt".to_string()];
        app.state.current_index = 1;

        let new_playlist = vec!["/path/to/a.txt".to_string()];

        let gen_id = app
            .state
            .dir_sync_generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let _task =
            handle_directory_sync_completed(&mut app, Path::new("/path/to"), new_playlist, gen_id);
        assert_eq!(app.state.playlist.len(), 1);
        assert_eq!(app.state.current_index, 0);
    }

    #[tokio::test]
    async fn test_directory_sync_completed_when_all_files_removed() {
        let mut app = test_app(None);
        app.state.file_name = "/path/to/b.txt".to_string();
        app.state.playlist = vec!["/path/to/b.txt".to_string()];
        app.state.current_index = 0;
        app.state.content_ready = true;
        app.current_content = Some(PreviewData::Text {
            content: "hello".to_string(),
            line_numbers: "1".into(),
            language: "".into(),
        });

        let gen_id = app
            .state
            .dir_sync_generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let _task =
            handle_directory_sync_completed(&mut app, Path::new("/path/to"), Vec::new(), gen_id);
        assert!(app.state.playlist.is_empty());
        assert_eq!(app.state.current_index, 0);
        assert!(app.state.file_name.is_empty());
        assert_eq!(
            app.state.active_dir,
            Some(std::path::PathBuf::from("/path/to"))
        );
        assert!(!app.state.content_ready);
        assert!(app.current_content.is_none());
    }

    #[tokio::test]
    async fn test_directory_sync_completed_recovers_empty_directory() {
        let mut app = test_app(None);
        app.state.active_dir = Some(std::path::PathBuf::from("/path/to"));
        app.state.file_name = "".to_string();
        app.state.playlist = Vec::new();
        app.state.current_index = 0;
        app.state.content_ready = false;

        let gen_id = app
            .state
            .dir_sync_generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let new_playlist = vec!["/path/to/recovered.txt".to_string()];
        let _task =
            handle_directory_sync_completed(&mut app, Path::new("/path/to"), new_playlist, gen_id);

        assert_eq!(
            app.state.playlist,
            vec!["/path/to/recovered.txt".to_string()]
        );
        assert_eq!(app.state.file_name, "/path/to/recovered.txt");
        assert_eq!(app.state.current_index, 0);
        assert_eq!(
            app.state.active_dir,
            Some(std::path::PathBuf::from("/path/to"))
        );
    }

    #[tokio::test]
    async fn test_directory_sync_completed_discards_stale_generation() {
        let mut app = test_app(None);
        app.state.active_dir = Some(std::path::PathBuf::from("/path/to"));
        app.state.file_name = "/path/to/b.txt".to_string();
        app.state.playlist = vec!["/path/to/b.txt".to_string()];
        app.state.dir_sync_generation_id =
            std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(2));

        let stale_playlist = vec!["/path/to/stale.txt".to_string()];
        let _task =
            handle_directory_sync_completed(&mut app, Path::new("/path/to"), stale_playlist, 1);

        assert_eq!(app.state.playlist, vec!["/path/to/b.txt".to_string()]);
    }

    #[tokio::test]
    async fn test_directory_sync_completed_ignores_outdated_directory() {
        let mut app = test_app(None);
        // Active file has moved to /other/dir/c.txt
        app.state.file_name = "/other/dir/c.txt".to_string();
        app.state.playlist = vec!["/other/dir/c.txt".to_string()];
        app.state.current_index = 0;

        let gen_id = app
            .state
            .dir_sync_generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let old_dir_playlist = vec!["/path/to/a.txt".to_string(), "/path/to/b.txt".to_string()];
        let _task = handle_directory_sync_completed(
            &mut app,
            Path::new("/path/to"),
            old_dir_playlist,
            gen_id,
        );

        // Playlist should not be overwritten by out-of-date sync from old directory
        assert_eq!(app.state.playlist, vec!["/other/dir/c.txt".to_string()]);
    }

    #[test]
    fn test_daemon_update_ignored_when_window_closed() {
        let mut app = test_app(None);
        app.is_daemon = true;
        app.window_id = None;

        let content = PreviewData::Text {
            content: "hello".to_string(),
            line_numbers: "1".into(),
            language: "".into(),
        };

        let _task = handle_daemon_update_window(&mut app, "/tmp/test.txt".to_string(), content);
        assert!(app.window_id.is_none());
        assert!(app.current_content.is_none());
    }
}
