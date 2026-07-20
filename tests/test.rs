// tests/test.rs

use kglance::dbus::{run_zbus, send_via_dbus};
use std::sync::Arc;
use std::time::Duration;

async fn setup_mock_daemon() -> tokio::sync::mpsc::Receiver<kglance::dbus::DaemonCommand> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let registry = Arc::new(kglance::parser::ParserRegistry::new());

    tokio::spawn(async move {
        let _ = run_zbus(registry, tx).await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    rx
}

#[tokio::test]
async fn test_send_via_dbus_success() {
    let _rx = setup_mock_daemon().await;

    let temp_dir = std::env::temp_dir().join("kglance-test");
    let _ = std::fs::create_dir_all(&temp_dir);
    let test_path = temp_dir.join("test.txt");
    std::fs::write(&test_path, b"test content").unwrap();

    let result = send_via_dbus(test_path.to_str().unwrap());

    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(
        result.is_ok(),
        "This function must return Ok when daemon is run, but got: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_send_via_dbus_daemon_not_running() {
    let test_path = "/path/to/test/file.png";
    let result = send_via_dbus(test_path);

    assert!(
        result.is_err(),
        "This function must be informed failed daemon is not running"
    );
}
