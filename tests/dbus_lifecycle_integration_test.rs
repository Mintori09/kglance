use kglance::dbus::{
    DaemonCommand, is_gui_open_at, run_zbus_with_name, send_multiple_via_dbus_to, send_via_dbus_to,
};
use kglance::engine::build_registry;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_dbus_full_lifecycle_ipc() {
    let service_name = format!("org.mintori.Kglance.TestLifecycle_{}", std::process::id());
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let registry = Arc::new(build_registry());
    let is_gui_open = Arc::new(AtomicBool::new(false));
    let is_gui_open_clone = Arc::clone(&is_gui_open);
    let name_clone = service_name.clone();

    tokio::spawn(async move {
        let _ = run_zbus_with_name(&name_clone, registry, tx, is_gui_open_clone).await;
    });

    // Wait for DBus daemon to register on session bus
    let start_wait = std::time::Instant::now();
    while start_wait.elapsed() < Duration::from_secs(3) {
        if is_gui_open_at(&service_name).is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

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
    let res = send_via_dbus_to(&service_name, file1.to_str().unwrap());
    assert!(res.is_ok(), "send_via_dbus failed: {:?}", res.err());

    let received = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("Timeout waiting for OpenWindowWithContent via DBus");

    match received {
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
    let res_mult = send_multiple_via_dbus_to(&service_name, &files);
    assert!(
        res_mult.is_ok(),
        "send_multiple_via_dbus failed: {:?}",
        res_mult.err()
    );

    let received_mult = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("Timeout waiting for OpenWindowWithPlaylist via DBus");

    match received_mult {
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
