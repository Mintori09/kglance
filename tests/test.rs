// tests/test.rs

use kglance::dbus::{is_gui_open_at, run_zbus_with_name, send_via_dbus_to};
use kglance::parsers::common::parser::traits::ParserRegistry;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

fn test_service_name(suffix: &str) -> String {
    format!("org.mintori.Kglance.Test_{}_{}", std::process::id(), suffix)
}

async fn setup_mock_daemon(
    service_name: &str,
) -> tokio::sync::mpsc::Receiver<kglance::dbus::DaemonCommand> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    let registry = Arc::new(ParserRegistry::new());
    let name = service_name.to_string();

    tokio::spawn(async move {
        let is_gui_open = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let _ = run_zbus_with_name(&name, registry, tx, is_gui_open).await;
    });

    let start_wait = std::time::Instant::now();
    while start_wait.elapsed() < Duration::from_secs(3) {
        if is_gui_open_at(service_name).is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    rx
}

#[tokio::test]
async fn test_send_via_dbus_success() {
    let service_name = test_service_name("success");
    let _rx = setup_mock_daemon(&service_name).await;

    let dir = tempdir().unwrap();
    let test_path = dir.path().join("test.txt");
    std::fs::write(&test_path, b"test content").unwrap();

    let result = send_via_dbus_to(&service_name, test_path.to_str().unwrap());

    assert!(
        result.is_ok(),
        "This function must return Ok when daemon is run, but got: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_send_via_dbus_daemon_not_running() {
    let non_running_service = test_service_name("not_running");
    let test_path = "/path/to/test/file.png";
    let result = send_via_dbus_to(&non_running_service, test_path);

    assert!(
        result.is_err(),
        "This function must be informed failed daemon is not running"
    );
}
