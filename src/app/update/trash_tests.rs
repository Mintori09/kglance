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
