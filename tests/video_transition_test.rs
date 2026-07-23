use kglance::ui::handlers::video::{spawn_video_player, PlayerCommand, VideoEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[test]
fn test_video_controller_consecutive_loads() {
    let (cmd_tx, cmd_rx) = mpsc::channel(100);
    let (event_tx, _event_rx) = mpsc::channel(100);
    let controller = spawn_video_player(cmd_rx, event_tx);

    // Test consecutive load commands
    let temp_dir = std::env::temp_dir().join("kglance_video_test");
    let _ = std::fs::create_dir_all(&temp_dir);
    let dummy_video = temp_dir.join("dummy.mp4");
    std::fs::write(&dummy_video, b"fake video bytes").unwrap();

    let dummy_path = dummy_video.to_string_lossy().to_string();

    // 1. Send Stop -> Load -> Play
    assert!(cmd_tx.try_send(PlayerCommand::Stop).is_ok());
    assert!(cmd_tx.try_send(PlayerCommand::Load(dummy_path.clone())).is_ok());
    assert!(cmd_tx.try_send(PlayerCommand::Play).is_ok());

    std::thread::sleep(Duration::from_millis(50));

    // 2. Immediately send consecutive Load again (Video 1 -> Video 2 transition)
    assert!(cmd_tx.try_send(PlayerCommand::Stop).is_ok());
    assert!(cmd_tx.try_send(PlayerCommand::Load(dummy_path.clone())).is_ok());
    assert!(cmd_tx.try_send(PlayerCommand::Play).is_ok());

    std::thread::sleep(Duration::from_millis(50));

    // Lock controller and verify stop clean state
    if let Ok(mut c) = controller.lock() {
        c.stop();
        assert!(!c.is_playing);
        assert!(c.video.is_none());
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}
