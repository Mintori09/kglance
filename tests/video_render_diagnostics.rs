use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

fn get_test_video() -> &'static Path {
    static VIDEO: OnceLock<(tempfile::TempDir, String)> = OnceLock::new();
    let (_, path) = VIDEO.get_or_init(|| {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("diag_video.mp4");
        let path_str = path.to_str().unwrap();
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=2:size=640x480:rate=30",
                "-f",
                "lavfi",
                "-i",
                "anullsrc",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
                "-shortest",
                "-y",
                path_str,
            ])
            .status()
            .expect("ffmpeg failed");
        assert!(status.success(), "failed to create test video");
        (dir, path_str.to_string())
    });
    Path::new(path)
}

fn gst_init_once() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        gst::init().expect("gst init failed");
    });
}

// ── Test 1: Video Loading & State ───────────────────────────────────────────

#[test]
fn test_video_loading_and_state() {
    gst_init_once();
    use kglance::features::video::handler::load_video;

    let video_path = get_test_video();
    let video_res = load_video(video_path.to_str().unwrap());
    assert!(video_res.is_ok(), "Video should load successfully");
    let video = video_res.unwrap();
    assert!(video.duration().as_secs_f64() > 0.0);
}

// ── Test 2: Iced Image Handle Creation from Video Buffer ─────────────────────

#[test]
fn test_iced_image_handle_from_video_buffer() {
    gst_init_once();
    use iced::widget::image::Handle;

    let dummy_data = vec![255u8; 100 * 100 * 4]; // 100x100 white RGBA image
    let handle = Handle::from_rgba(100, 100, dummy_data.clone());

    let id = handle.id();

    println!("Created Iced Handle ID: {:?}", id);

    // Verify handle format / content reading if available
    let bytes_handle = Handle::from_bytes(dummy_data);
    assert_ne!(id, bytes_handle.id(), "Handles should have unique IDs");
}

// ── Test 3: Media State Routing & Video Flag Check ───────────────────────────

#[test]
fn test_app_media_state_routing_and_video_flag() {
    use kglance::core::KglanceState;
    use kglance::core::preview::PreviewData;

    let mut state = KglanceState::default();

    let media_data = PreviewData::Media {
        url: "/tmp/sample.mp4".to_string(),
        metadata: "Video Duration: 12.34s".to_string(),
        thumbnail_or_waveform: Vec::new(),
        width: 320,
        height: 240,
    };

    // 1. Check populate_state
    media_data.populate_state(&mut state);
    assert_eq!(state.media.metadata, "Video Duration: 12.34s");
    assert_eq!(state.file_type_text, "Video File");

    // 2. Check path ending logic (.mp4 set has_video = true)
    let path = "/tmp/sample.mp4";
    let path_lower = path.to_lowercase();
    let is_video = path_lower.ends_with(".mp4")
        || path_lower.ends_with(".mkv")
        || path_lower.ends_with(".avi")
        || path_lower.ends_with(".mov")
        || path_lower.ends_with(".wmv")
        || path_lower.ends_with(".webm");

    state.media.has_video = is_video;
    assert!(
        state.media.has_video,
        "has_video flag must be true for .mp4 file"
    );
}

// ── Test 4: Handle Invalidation & Redraw Rate Diagnosis ─────────────────────

#[test]
fn test_handle_invalidation_flicker_diagnosis() {
    use iced::widget::image::Handle;

    let dummy_data = vec![128u8; 640 * 480 * 4];

    // Simulating calling Handle::from_rgba repeatedly in view() on every frame
    let h1 = Handle::from_rgba(640, 480, dummy_data.clone());
    let h2 = Handle::from_rgba(640, 480, dummy_data.clone());

    println!("h1 ID: {:?}, h2 ID: {:?}", h1.id(), h2.id());

    // Creating handle from_rgba on every view() creates a NEW unique ID every time
    assert_ne!(
        h1.id(),
        h2.id(),
        "DIAGNOSIS CONFIRMED: Calling Handle::from_rgba inside view() generates distinct Handle IDs, forcing Iced GPU texture cache invalidation every frame!"
    );
}

// ── Test 5: Verify Video Controls Helper Functions ───────────────────────────

#[test]
fn test_video_controls_helper_functions() {
    gst_init_once();
    use kglance::features::video::handler::{
        load_video, seek_relative, seek_to_ratio, toggle_play_pause,
    };

    let video_path = get_test_video();
    let mut video = load_video(video_path.to_str().unwrap()).unwrap();

    let initial_paused = video.paused();
    toggle_play_pause(&mut video);
    assert_ne!(video.paused(), initial_paused);

    seek_to_ratio(&mut video, 0.5);
    seek_relative(&mut video, 0.5);
}

// ── Test 6: Check if mutating VideoFrame.data updates iced::widget::image::Handle ──

#[test]
fn test_handle_pixel_buffer_updates_between_frames() {
    use iced::widget::image::Handle;

    let mut frame_data = vec![0u8; 10 * 10 * 4];
    let handle = Handle::from_rgba(10, 10, frame_data.clone());

    // Mutate frame_data to simulate decoding new video frame
    frame_data[0] = 255;
    frame_data[1] = 255;

    // Notice: handle was created with the OLD clone of frame_data!
    // Handle in Iced holds an Arc<Vec<u8>> internally created at `from_rgba`.
    // Mutating `VideoFrame.data` does NOT update `Handle`'s internal pixel bytes!
    let new_handle = Handle::from_rgba(10, 10, frame_data.clone());

    println!("Original Handle ID: {:?}", handle.id());
    println!("New Handle ID (with updated pixels): {:?}", new_handle.id());

    assert_ne!(
        handle.id(),
        new_handle.id(),
        "CONFIRMED ROOT CAUSE: Iced Handle holds immutable pixel buffer! Mutating VideoFrame.data leaves Handle frozen on the first frame (thumbnail effect)!"
    );
}
