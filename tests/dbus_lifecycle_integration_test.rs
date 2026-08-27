use kglance::dbus::{DaemonCommand, run_zbus, send_multiple_via_dbus, send_via_dbus};
use kglance::engine::build_registry;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_dbus_full_lifecycle_ipc() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let registry = Arc::new(build_registry());
    let is_gui_open = Arc::new(AtomicBool::new(false));
    let is_gui_open_clone = Arc::clone(&is_gui_open);

    tokio::spawn(async move {
        let _ = run_zbus(registry, tx, is_gui_open_clone).await;
    });

    // Wait for DBus daemon to register on session bus
    tokio::time::sleep(Duration::from_millis(250)).await;

    let dir = tempdir().expect("Failed to create tempdir");
    let file1 = dir.path().join("doc1.txt");
    let file2 = dir.path().join("doc2.txt");

    File::create(&file1)
        .unwrap()
        .write_all(b"Content 1")
        .unwrap();
    File::create(&file2)
        .unwrap()
        .write_all(b"Content 2")
        .unwrap();

    // 1. Send single file via DBus client
    let res = send_via_dbus(file1.to_str().unwrap());
    assert!(res.is_ok(), "send_via_dbus failed: {:?}", res.err());

    match rx.recv().await {
        Some(DaemonCommand::OpenWindowWithContent { path, content }) => {
            assert_eq!(path, file1.to_str().unwrap());
            assert!(matches!(
                content,
                kglance::core::preview::PreviewData::Text { .. }
            ));
        }
        other => panic!("Expected OpenWindowWithContent, got {:?}", other.is_some()),
    }

    // 2. Send multiple files via DBus client
    let files = vec![
        file1.to_str().unwrap().to_string(),
        file2.to_str().unwrap().to_string(),
    ];
    let res_mult = send_multiple_via_dbus(&files);
    assert!(
        res_mult.is_ok(),
        "send_multiple_via_dbus failed: {:?}",
        res_mult.err()
    );

    match rx.recv().await {
        Some(DaemonCommand::OpenWindowWithPlaylist {
            path,
            content,
            playlist,
        }) => {
            assert_eq!(path, file1.to_str().unwrap());
            assert_eq!(playlist.len(), 2);
            assert!(matches!(
                content,
                kglance::core::preview::PreviewData::Text { .. }
            ));
        }
        other => panic!("Expected OpenWindowWithPlaylist, got {:?}", other.is_some()),
    }
}
