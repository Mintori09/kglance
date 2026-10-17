use std::fs::File;
use tempfile::tempdir;

use crate::app::messages::TrashMsg;
use crate::app::test_util::test_app;
use crate::app::update::trash::handle_trash_msg;
use crate::core::types::ToastAction;

#[test]
fn test_request_trash_when_disabled_does_nothing() {
    let mut app = test_app(None);
    app.state.trash_config.enabled = false;
    app.state.file_name = "/tmp/test.txt".to_string();

    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);
    assert!(!app.state.show_trash_confirm_dialog);
}

#[test]
fn test_request_trash_with_confirm_enabled_shows_dialog() {
    let mut app = test_app(None);
    app.state.trash_config.enabled = true;
    app.state.trash_config.confirm_on_delete = true;
    app.state.file_name = "/tmp/test.txt".to_string();

    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);
    assert!(app.state.show_trash_confirm_dialog);

    let _ = handle_trash_msg(&mut app, TrashMsg::CancelTrashDialog);
    assert!(!app.state.show_trash_confirm_dialog);
}

#[test]
fn test_trash_advances_playlist_and_undo_restores() {
    let dir = tempdir().expect("create temp dir");
    let file1 = dir.path().join("a_file.txt");
    let file2 = dir.path().join("b_file.txt");
    File::create(&file1).expect("create file 1");
    File::create(&file2).expect("create file 2");

    let mut app = test_app(None);
    app.state.trash_config.enabled = true;
    app.state.trash_config.confirm_on_delete = false;
    app.state.active_dir = Some(dir.path().to_path_buf());
    app.state.playlist = vec![
        file1.to_string_lossy().into_owned(),
        file2.to_string_lossy().into_owned(),
    ];
    app.state.current_index = 0;
    app.state.file_name = file1.to_string_lossy().into_owned();

    // Execute trash on file1 (optimistically removes from playlist and advances to file2)
    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);

    assert_eq!(app.state.playlist.len(), 1);
    assert_eq!(app.state.playlist[0], file2.to_string_lossy().into_owned());
    assert_eq!(app.state.file_name, file2.to_string_lossy().into_owned());

    // When background trash completes successfully
    let trashed_item = crate::core::trash::TrashedItem {
        original_path: file1.clone(),
        original_index: 0,
    };
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::TrashCompleted {
            file_to_trash: file1.to_string_lossy().into_owned(),
            result: Ok(trashed_item.clone()),
        },
    );

    assert!(!app.state.trash_history.is_empty());

    // Should have toast with Undo action
    let last_toast = app.state.toasts.last().expect("toast exists");
    assert_eq!(last_toast.action, Some(ToastAction::UndoTrash));

    // When undo is triggered, the undo toast should be dismissed immediately
    let _ = handle_trash_msg(&mut app, TrashMsg::UndoTrash);
    assert!(
        !app.state
            .toasts
            .iter()
            .any(|t| t.action == Some(ToastAction::UndoTrash))
    );

    // When restore completes
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::RestoreCompleted {
            restored_item: trashed_item,
            result: Ok(()),
        },
    );

    // File1 should be restored back in playlist at original index 0
    assert_eq!(app.state.playlist.len(), 2);
    assert_eq!(app.state.playlist[0], file1.to_string_lossy().into_owned());
    assert_eq!(app.state.file_name, file1.to_string_lossy().into_owned());
}

#[test]
fn test_trash_last_file_moves_to_previous() {
    let dir = tempdir().expect("create temp dir");
    let file1 = dir.path().join("file1.txt");
    let file2 = dir.path().join("file2.txt");
    let file3 = dir.path().join("file3.txt");
    File::create(&file1).expect("create file 1");
    File::create(&file2).expect("create file 2");
    File::create(&file3).expect("create file 3");

    let mut app = test_app(None);
    app.state.trash_config.enabled = true;
    app.state.trash_config.confirm_on_delete = false;
    app.state.active_dir = Some(dir.path().to_path_buf());
    app.state.playlist = vec![
        file1.to_string_lossy().into_owned(),
        file2.to_string_lossy().into_owned(),
        file3.to_string_lossy().into_owned(),
    ];
    // Previewing the last file (index 2)
    app.state.current_index = 2;
    app.state.file_name = file3.to_string_lossy().into_owned();

    // Delete file3
    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);

    // Should move to previous file (file2, index 1) rather than jumping to index 0!
    assert_eq!(app.state.playlist.len(), 2);
    assert_eq!(app.state.current_index, 1);
    assert_eq!(app.state.file_name, file2.to_string_lossy().into_owned());
}

#[test]
fn test_trash_suppresses_directory_sync_file_removed_toast() {
    let dir = tempdir().expect("create temp dir");
    let file1 = dir.path().join("file1.txt");
    let file2 = dir.path().join("file2.txt");
    File::create(&file1).expect("create file 1");
    File::create(&file2).expect("create file 2");

    let f1_str = file1.to_string_lossy().into_owned();
    let f2_str = file2.to_string_lossy().into_owned();

    let mut app = test_app(None);
    app.state.trash_config.enabled = true;
    app.state.trash_config.confirm_on_delete = false;
    app.state.active_dir = Some(dir.path().to_path_buf());
    app.state.playlist = vec![f1_str.clone(), f2_str.clone()];
    app.state.current_index = 0;
    app.state.file_name = f1_str.clone();

    // User deletes f1
    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);
    assert!(app.state.recently_trashed.contains(&f1_str));

    // Trash completion adds the Undo toast
    let trashed_item = crate::core::trash::TrashedItem {
        original_path: file1.clone(),
        original_index: 0,
    };
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::TrashCompleted {
            file_to_trash: f1_str.clone(),
            result: Ok(trashed_item),
        },
    );
    let initial_toasts_len = app.state.toasts.len();
    assert_eq!(initial_toasts_len, 1);
    assert_eq!(app.state.toasts[0].action, Some(ToastAction::UndoTrash));

    // Now simulated directory sync event arrives from inotify
    let gen_id = app
        .state
        .dir_sync_generation_id
        .load(std::sync::atomic::Ordering::Relaxed);
    let _ = crate::app::update::file::handle_directory_sync_completed(
        &mut app,
        dir.path(),
        vec![f2_str.clone()],
        gen_id,
    );

    // It should NOT add "1 file removed from folder" toast or clobber trash toast
    assert_eq!(app.state.toasts.len(), initial_toasts_len);
    assert_eq!(app.state.file_name, f2_str);
}

#[test]
fn test_trash_single_file_shows_toast_and_undo_recovers() {
    let dir = tempdir().expect("create temp dir");
    let file1 = dir.path().join("lonely_file.txt");
    File::create(&file1).expect("create lonely file");

    let f1_str = file1.to_string_lossy().into_owned();

    let mut app = test_app(None);
    app.state.trash_config.enabled = true;
    app.state.trash_config.confirm_on_delete = false;
    app.state.active_dir = Some(dir.path().to_path_buf());
    app.state.playlist = vec![f1_str.clone()];
    app.state.current_index = 0;
    app.state.file_name = f1_str.clone();

    // User deletes the only file
    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);

    // Playlist should be empty, but window is kept alive for undo
    assert!(app.state.playlist.is_empty());
    assert!(app.state.file_name.is_empty());
    assert!(!app.state.content_ready);

    // Trash completed adds the toast
    let trashed_item = crate::core::trash::TrashedItem {
        original_path: file1.clone(),
        original_index: 0,
    };
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::TrashCompleted {
            file_to_trash: f1_str.clone(),
            result: Ok(trashed_item.clone()),
        },
    );

    assert_eq!(app.state.toasts.len(), 1);
    assert_eq!(app.state.toasts[0].action, Some(ToastAction::UndoTrash));

    // User triggers undo via Toast action
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::ToastActionClicked(ToastAction::UndoTrash),
    );

    // Simulated restore completion
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::RestoreCompleted {
            restored_item: trashed_item,
            result: Ok(()),
        },
    );

    // File should be restored back into playlist and previewed
    assert_eq!(app.state.playlist.len(), 1);
    assert_eq!(app.state.playlist[0], f1_str);
    assert_eq!(app.state.file_name, f1_str);
}

#[test]
fn test_real_file_trash_and_restore_cycle() {
    let dir = tempdir().expect("create temp dir");
    let file1 = dir.path().join("real_trash_me.txt");
    let file2 = dir.path().join("real_keep_me.txt");
    File::create(&file1).expect("create file 1");
    File::create(&file2).expect("create file 2");

    let f1_str = file1.canonicalize().unwrap().to_string_lossy().into_owned();
    let f2_str = file2.canonicalize().unwrap().to_string_lossy().into_owned();

    let mut app = test_app(None);
    app.state.trash_config.enabled = true;
    app.state.trash_config.confirm_on_delete = false;
    app.state.trash_config.undo_timeout_seconds = 6;
    app.state.active_dir = Some(dir.path().to_path_buf());
    app.state.playlist = vec![f1_str.clone(), f2_str.clone()];
    app.state.current_index = 0;
    app.state.file_name = f1_str.clone();

    // Perform optimistic playlist advance
    let _ = handle_trash_msg(&mut app, TrashMsg::RequestTrashCurrent);
    assert_eq!(app.state.file_name, f2_str);
    assert_eq!(app.state.playlist, vec![f2_str.clone()]);

    // Perform actual real trash on file1
    let trash_res = crate::core::trash::move_to_trash(std::path::Path::new(&f1_str), 0);
    let trashed_item = match trash_res {
        Ok(item) => item,
        Err(err) => {
            eprintln!("Skipping real trash assertion (unsupported environment): {err}");
            return;
        }
    };

    assert!(!file1.exists(), "File 1 should be moved to trash");

    // Complete trash
    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::TrashCompleted {
            file_to_trash: f1_str.clone(),
            result: Ok(trashed_item.clone()),
        },
    );

    // Verify toast state
    assert_eq!(app.state.toasts.len(), 1);
    let toast = &app.state.toasts[0];
    assert!(toast.message.contains("real_trash_me.txt"));
    assert_eq!(toast.action, Some(ToastAction::UndoTrash));
    assert_eq!(toast.duration, std::time::Duration::from_secs(6));

    // Now perform restore
    let restore_res = crate::core::trash::restore_trashed_item(&trashed_item);
    assert!(
        restore_res.is_ok(),
        "Restore failed: {:?}",
        restore_res.err()
    );
    assert!(file1.exists(), "File 1 should exist after restore");

    let _ = handle_trash_msg(
        &mut app,
        TrashMsg::RestoreCompleted {
            restored_item: trashed_item,
            result: Ok(()),
        },
    );

    assert_eq!(app.state.playlist.len(), 2);
    assert_eq!(app.state.file_name, f1_str);
}
