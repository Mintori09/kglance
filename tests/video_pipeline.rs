use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

// ── Test video generation ────────────────────────────────────────────────

const TEST_W: u32 = 640;
const TEST_H: u32 = 480;
const TEST_FPS: u32 = 30;
const TEST_DURATION_SECS: u32 = 2;

fn get_test_video() -> &'static Path {
    static VIDEO: OnceLock<(tempfile::TempDir, String)> = OnceLock::new();
    let (_, path) = VIDEO.get_or_init(|| {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test_video.mp4");
        let path_str = path.to_str().unwrap();
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                &format!(
                    "testsrc=duration={TEST_DURATION_SECS}:size={TEST_W}x{TEST_H}:rate={TEST_FPS}"
                ),
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
                "-frames:v",
                &(TEST_DURATION_SECS * TEST_FPS).to_string(),
                "-y",
                path_str,
            ])
            .status()
            .expect("ffmpeg not found — install ffmpeg to run these tests");
        assert!(status.success(), "failed to generate test video");
        (dir, path_str.to_string())
    });
    Path::new(path)
}

fn scaled_dims(src_w: u32, src_h: u32) -> (u32, u32) {
    let max_dim = 1080.0;
    let scale = (max_dim / (src_w.max(src_h) as f64)).min(1.0);
    let tw = (((src_w as f64 * scale) as u32) & !1).max(16);
    let th = (((src_h as f64 * scale) as u32) & !1).max(16);
    (tw, th)
}

fn frame_size_bytes(w: u32, h: u32) -> usize {
    (w * h * 4) as usize
}

// ── Helper: run ffmpeg and capture output ───────────────────────────────

fn run_ffmpeg(args: &[&str]) -> Result<(Vec<u8>, String, std::process::ExitStatus), String> {
    let output = Command::new("ffmpeg")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to spawn ffmpeg: {e}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.stdout, stderr, output.status))
}

fn is_valid_rgba_frame(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }
    // Not all zeros (blank frame check)
    if data.iter().all(|&b| b == 0) {
        return false;
    }
    // Alpha channel should be 255 (opaque) for most pixels
    let alpha_pixels: usize = data
        .chunks_exact(4)
        .map(|c| c[3])
        .filter(|&a| a == 255)
        .count();
    alpha_pixels > data.len() / 4 / 2 // at least 50% pixels have alpha=255
}

// ── 1. ffmpeg availability ──────────────────────────────────────────────

#[test]
fn test_ffmpeg_available() {
    let status = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    assert!(status.is_ok(), "ffmpeg must be installed");

    let status = Command::new("ffprobe")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    assert!(status.is_ok(), "ffprobe must be installed");
}

// ── 2. Single frame output (paused seek mode) ──────────────────────────

#[test]
fn test_single_frame_correct_size() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    let (stdout, stderr, status) = run_ffmpeg(&[
        "-ss",
        "0",
        "-i",
        video.to_str().unwrap(),
        "-vf",
        &format!("scale={tw}:{th}"),
        "-vframes",
        "1",
        "-f",
        "rawvideo",
        "-vcodec",
        "rawvideo",
        "-pix_fmt",
        "rgba",
        "-",
    ])
    .expect("ffmpeg single-frame failed");

    assert!(status.success(), "ffmpeg exited with error:\n{stderr}");
    assert_eq!(
        stdout.len(),
        expected_size,
        "frame size mismatch: expected {expected_size}, got {} (w={tw} h={th})\nstderr:\n{stderr}",
        stdout.len()
    );
    assert!(
        is_valid_rgba_frame(&stdout),
        "frame is all zeros or invalid RGBA\nstderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("error"),
        "ffmpeg stderr contains errors:\n{stderr}"
    );
}

#[test]
fn test_single_frame_without_vcodec_flag() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    // Same command WITHOUT -vcodec rawvideo (the original bug scenario)
    let (stdout, stderr, status) = run_ffmpeg(&[
        "-ss",
        "0",
        "-i",
        video.to_str().unwrap(),
        "-vf",
        &format!("scale={tw}:{th}"),
        "-vframes",
        "1",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgba",
        "-",
    ])
    .expect("ffmpeg single-frame failed");

    assert!(
        status.success(),
        "ffmpeg without -vcodec rawvideo failed — this breaks the original code! stderr:\n{stderr}"
    );
    assert_eq!(
        stdout.len(),
        expected_size,
        "frame size mismatch without -vcodec rawvideo — check if your ffmpeg build needs it\nstderr:\n{stderr}"
    );
}

#[test]
fn test_single_frame_different_positions() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    // Seek to 3 different positions and verify frames differ
    let positions = ["0.1", "1.0", "1.5"];
    let mut frames = Vec::new();

    for pos in &positions {
        let (stdout, stderr, status) = run_ffmpeg(&[
            "-ss",
            pos,
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-vframes",
            "1",
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .expect("ffmpeg seek failed");

        assert!(status.success(), "ffmpeg at {pos}s failed:\n{stderr}");
        assert_eq!(stdout.len(), expected_size, "frame size at {pos}s");
        assert!(is_valid_rgba_frame(&stdout), "invalid frame at {pos}s");
        frames.push(stdout);
    }

    // Frames from different positions should differ
    assert_ne!(
        frames[0], frames[1],
        "frames at 0.1s and 1.0s are identical — possible freeze"
    );
    assert_ne!(
        frames[1], frames[2],
        "frames at 1.0s and 1.5s are identical — possible freeze"
    );
}

// ── 3. Continuous frame output (play mode) ──────────────────────────────

#[test]
fn test_continuous_frames_correct_sizes() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    let mut child = Command::new("ffmpeg")
        .args([
            "-re",
            "-ss",
            "0",
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg continuous spawn failed");

    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = std::io::BufReader::with_capacity(expected_size * 10, stdout);

    let mut frames_read = 0;
    let max_frames = 10;
    let mut buffer = vec![0u8; expected_size];

    while frames_read < max_frames {
        match reader.read_exact(&mut buffer) {
            Ok(()) => {
                assert_eq!(
                    buffer.len(),
                    expected_size,
                    "frame {frames_read} has wrong size"
                );
                assert!(
                    is_valid_rgba_frame(&buffer),
                    "frame {frames_read} is invalid (all zeros or bad alpha)"
                );
                frames_read += 1;
            }
            Err(e) => {
                panic!(
                    "read_exact failed at frame {frames_read} after reading {frames_read} frames: {e}"
                );
            }
        }
    }

    // Kill ffmpeg
    let _ = child.kill();
    let _ = child.wait();

    assert_eq!(
        frames_read, max_frames,
        "expected {max_frames} frames but got {frames_read}"
    );
}

#[test]
fn test_continuous_frames_not_stuck() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    let mut child = Command::new("ffmpeg")
        .args([
            "-re",
            "-ss",
            "0",
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg continuous spawn failed");

    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = std::io::BufReader::with_capacity(expected_size * 10, stdout);

    let mut prev_frame = vec![0u8; expected_size];
    let mut buffer = vec![0u8; expected_size];
    let mut changes = 0;
    let total_frames = 15;

    for i in 0..total_frames {
        reader
            .read_exact(&mut buffer)
            .unwrap_or_else(|e| panic!("read_exact failed at frame {i}: {e}"));

        if buffer != prev_frame {
            changes += 1;
        }
        prev_frame.copy_from_slice(&buffer);
    }

    let _ = child.kill();
    let _ = child.wait();

    // At least some frames should differ (flickering = alternating identical frames is bad)
    assert!(
        changes >= total_frames / 2,
        "only {changes}/{total_frames} frames differ — possible stuck frame or flicker between 2 states"
    );
}

#[test]
fn test_continuous_frames_timing() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    let mut child = Command::new("ffmpeg")
        .args([
            "-re",
            "-ss",
            "0",
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg continuous spawn failed");

    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = std::io::BufReader::with_capacity(expected_size * 10, stdout);

    let mut buffer = vec![0u8; expected_size];
    let mut timestamps = Vec::new();
    let max_frames = 15;

    for i in 0..max_frames {
        let t = Instant::now();
        reader
            .read_exact(&mut buffer)
            .unwrap_or_else(|e| panic!("read_exact failed at frame {i}: {e}"));
        timestamps.push(t.elapsed());
    }

    let _ = child.kill();
    let _ = child.wait();

    // Timing sanity: skip first frame (decoder init overhead),
    // subsequent frames should take < 100ms (typically ~33ms for 30fps)
    for (i, dur) in timestamps.iter().enumerate().skip(1) {
        assert!(
            *dur < Duration::from_millis(100),
            "frame {i} took {dur:?} to read — possible decoder block"
        );
    }
}

// ── 4. Pipeline / channel behavior ──────────────────────────────────────

#[test]
fn test_channel_send_receive() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let (tw, th) = scaled_dims(TEST_W, TEST_H);
        let frame_size = frame_size_bytes(tw, th);
        let frame = vec![128u8; frame_size];

        let sent = tx.try_send(frame.clone());
        assert!(sent.is_ok(), "try_send should succeed on empty channel");

        let received = rx.recv().await;
        assert!(received.is_some(), "should receive the frame");
        assert_eq!(
            received.unwrap().len(),
            frame_size,
            "received frame size mismatch"
        );
    });
}

#[test]
fn test_channel_capacity_drops() {
    let rt = tokio::sync::mpsc::channel::<Vec<u8>>(5);
    let (tx, _rx) = rt;

    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let frame_size = frame_size_bytes(tw, th);
    let frame = vec![128u8; frame_size];

    // Fill the channel
    for i in 0..5 {
        assert!(
            tx.try_send(frame.clone()).is_ok(),
            "try_send should succeed at capacity {i}"
        );
    }

    // Next send should fail (channel full) — no crash, frame dropped
    let should_drop = tx.try_send(frame.clone());
    assert!(
        should_drop.is_err(),
        "try_send should return error when channel full — this is expected drop behavior"
    );

    drop(tx);
}

// ── 5. Scale / resolution tests ─────────────────────────────────────────

#[test]
fn test_scaled_dimensions_even() {
    // All scaled dimensions must be even (required by many codecs)
    for w in [640u32, 1920, 1280, 800, 320] {
        for h in [480u32, 1080, 720, 600, 240] {
            let (tw, th) = scaled_dims(w, h);
            assert!(
                tw % 2 == 0,
                "scaled width {tw} from source {w}x{h} must be even"
            );
            assert!(
                th % 2 == 0,
                "scaled height {th} from source {w}x{h} must be even"
            );
            assert!(tw >= 16, "scaled width {tw} too small");
            assert!(th >= 16, "scaled height {th} too small");
            assert!(tw <= w, "scaled width {tw} should not exceed source {w}");
            assert!(th <= h, "scaled height {th} should not exceed source {h}");
        }
    }
}

#[test]
fn test_scaled_dimensions_no_overflow() {
    let max_dim = 1080.0;
    let (tw, th) = scaled_dims(4096, 2160);
    let max_side = tw.max(th) as f64;
    assert!(
        max_side <= max_dim,
        "4K frame scaled to {tw}x{th} — longest side {max_side} exceeds {max_dim}"
    );
}

// ── 6. Frame data integrity ─────────────────────────────────────────────

#[test]
fn test_frame_not_empty() {
    let data = vec![0u8; 100];
    assert!(
        !is_valid_rgba_frame(&data),
        "empty-looking frame should be invalid"
    );

    let data = vec![];
    assert!(!is_valid_rgba_frame(&data), "empty vec should be invalid");

    let data = vec![255u8; 400];
    assert!(
        is_valid_rgba_frame(&data),
        "full opaque frame should be valid"
    );
}

#[test]
fn test_rgba_alpha_channel_integrity() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    // Extract frames at various positions and check alpha channel
    for pos in ["0.0", "0.5", "1.0", "1.5"] {
        let (stdout, _stderr, status) = run_ffmpeg(&[
            "-ss",
            pos,
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-vframes",
            "1",
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .unwrap_or_else(|e| panic!("ffmpeg at {pos}s: {e}"));

        assert!(status.success(), "ffmpeg at {pos}s failed");
        assert_eq!(stdout.len(), expected_size, "frame size at {pos}s");

        // Check alpha: at least 50% of pixels should have alpha=255
        let opaque = stdout.chunks_exact(4).filter(|c| c[3] == 255).count();
        let ratio = opaque as f64 / (stdout.len() / 4) as f64;
        assert!(
            ratio > 0.8,
            "alpha channel corruption at {pos}s: only {:.1}% opaque pixels (expected >80%)",
            ratio * 100.0
        );
    }
}

// ── 7. Error detection ──────────────────────────────────────────────────

#[test]
fn test_ffmpeg_error_detection() {
    // Run ffmpeg with a nonexistent file — stderr should contain errors
    let (_, stderr, status) = run_ffmpeg(&[
        "-i",
        "/nonexistent/video.mp4",
        "-vframes",
        "1",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgba",
        "-",
    ])
    .expect("ffmpeg should run even with bad input");

    assert!(!status.success(), "ffmpeg should fail on nonexistent file");
    assert!(
        !stderr.is_empty(),
        "stderr should contain error messages for nonexistent file"
    );
    assert!(
        stderr.to_lowercase().contains("error")
            || stderr.to_lowercase().contains("cannot")
            || stderr.to_lowercase().contains("not found"),
        "stderr should describe the error, got:\n{stderr}"
    );
}

#[test]
fn test_probe_duration_accuracy() {
    let video = get_test_video();
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            video.to_str().unwrap(),
        ])
        .output()
        .expect("ffprobe not found");

    assert!(output.status.success(), "ffprobe failed");
    let s = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = s.trim().parse().expect("ffprobe output should be a number");
    assert!(
        (duration - TEST_DURATION_SECS as f64).abs() < 0.5,
        "probed duration {duration}s differs from expected {TEST_DURATION_SECS}s by >0.5s"
    );
}

#[test]
fn test_many_seeks_stability() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    // Seek to 20 random positions — no crash, all frames valid
    for i in 0..20 {
        let pos = (i as f64) * 0.1; // 0.0, 0.1, 0.2, ..., 1.9
        let (stdout, stderr, status) = run_ffmpeg(&[
            "-ss",
            &format!("{pos:.1}"),
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-vframes",
            "1",
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .unwrap_or_else(|e| panic!("ffmpeg seek at {pos:.1}s: {e}"));

        assert!(status.success(), "ffmpeg at {pos:.1}s failed:\n{stderr}");
        assert_eq!(stdout.len(), expected_size, "frame size at {pos:.1}s");
        assert!(
            is_valid_rgba_frame(&stdout),
            "invalid frame at {pos:.1}s:\n{stderr}"
        );
    }
}

// ── 8. Without -re flag (regression test) ───────────────────────────────

#[test]
fn test_ffmpeg_no_re_flag() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    // Run WITHOUT -re (the original code's behavior) for 10 frames
    let mut child = Command::new("ffmpeg")
        .args([
            "-ss",
            "0",
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg spawn failed");

    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut buffer = vec![0u8; expected_size];

    // Must be able to read at least 10 frames even without -re
    for i in 0..10 {
        reader
            .read_exact(&mut buffer)
            .unwrap_or_else(|e| panic!("read_exact failed at frame {i} without -re: {e}"));
        assert!(
            is_valid_rgba_frame(&buffer),
            "frame {i} invalid without -re"
        );
    }

    let _ = child.kill();
    let _ = child.wait();
}

// ── 9. Flicker detection: alternating frame pattern ─────────────────────

#[test]
fn test_no_flicker_alternation() {
    let video = get_test_video();
    let (tw, th) = scaled_dims(TEST_W, TEST_H);
    let expected_size = frame_size_bytes(tw, th);

    let mut child = Command::new("ffmpeg")
        .args([
            "-re",
            "-ss",
            "0",
            "-i",
            video.to_str().unwrap(),
            "-vf",
            &format!("scale={tw}:{th}"),
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("ffmpeg spawn failed");

    let stdout = child.stdout.take().expect("take stdout");
    let mut reader = std::io::BufReader::new(stdout);

    let mut frames: Vec<Vec<u8>> = Vec::new();
    let mut buffer = vec![0u8; expected_size];
    let total = 20;

    for _ in 0..total {
        reader.read_exact(&mut buffer).expect("read frame");
        frames.push(buffer.clone());
    }

    let _ = child.kill();
    let _ = child.wait();

    // Detect 2-frame alternation pattern (flicker = ABABAB...)
    for i in 2..total {
        let eq_prev = frames[i] == frames[i - 1];
        let eq_two_before = frames[i] == frames[i - 2];
        let is_alternating = eq_two_before && !eq_prev;
        assert!(
            !is_alternating,
            "flicker detected at frame {}: ABAB pattern (equals i-2 but differs from i-1)",
            i
        );
    }

    // Check overall frame diversity: at most 30% can be duplicates
    let mut unique_count = 0;
    for i in 1..total {
        if frames[i] != frames[i - 1] {
            unique_count += 1;
        }
    }
    let diversity = unique_count as f64 / (total - 1) as f64;
    assert!(
        diversity > 0.5,
        "frame diversity too low: {diversity:.0}% consecutive frames differ — possible flicker/stuck issue"
    );
}
