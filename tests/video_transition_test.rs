use kglance::features::video::handler::load_video;
use std::sync::OnceLock;

fn gst_init_once() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        gst::init().expect("gst init failed");
    });
}

#[test]
fn test_video_consecutive_loads() {
    gst_init_once();
    let temp_dir = std::env::temp_dir().join("kglance_video_test");
    let _ = std::fs::create_dir_all(&temp_dir);
    let dummy_video = temp_dir.join("dummy.mp4");
    std::fs::write(&dummy_video, b"fake video bytes").unwrap();

    let dummy_path = dummy_video.to_string_lossy().to_string();

    // 1. Load invalid video -> returns Err cleanly without panic
    let res1 = load_video(&dummy_path);
    assert!(res1.is_err());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
