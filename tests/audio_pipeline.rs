use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use kglance::features::audio::{AudioParser, AudioPlayer};
use kglance::features::common::parser::traits::PreviewParser;
use kglance::features::common::parser::types::ParsedContent;

// ── Test audio generation ──────────────────────────────────────────────────

struct AudioTestFixtures {
    _temp_dir: tempfile::TempDir,
    mp3_path: PathBuf,
    wav_path: PathBuf,
    flac_path: PathBuf,
    cover_mp3_path: PathBuf,
    folder_cover_audio_path: PathBuf,
}

fn get_fixtures() -> &'static AudioTestFixtures {
    static FIXTURES: OnceLock<AudioTestFixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| {
        let dir = tempfile::tempdir().expect("Failed to create tempdir");
        let mp3_path = dir.path().join("test_sine.mp3");
        let wav_path = dir.path().join("test_sine.wav");
        let flac_path = dir.path().join("test_sine.flac");
        let cover_mp3_path = dir.path().join("test_cover.mp3");
        let folder_cover_dir = dir.path().join("album_dir");
        std::fs::create_dir_all(&folder_cover_dir).expect("create album_dir");
        let folder_cover_audio_path = folder_cover_dir.join("track.wav");
        let folder_cover_img_path = folder_cover_dir.join("cover.jpg");

        // 1. Generate standard MP3 with tags (3 seconds)
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=stereo:d=3",
                "-metadata",
                "title=Sine Wave",
                "-metadata",
                "artist=Acoustic Labs",
                "-metadata",
                "album=Synthesis Collection",
                "-y",
                mp3_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ffmpeg must be installed");
        assert!(status.success(), "Failed to generate test mp3");

        // 2. Generate WAV (2 seconds)
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=stereo:d=2",
                "-y",
                wav_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ffmpeg must be installed");
        assert!(status.success(), "Failed to generate test wav");

        // 3. Generate FLAC (2 seconds)
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=stereo:d=2",
                "-metadata",
                "title=Low Sine",
                "-metadata",
                "artist=Bass Test",
                "-metadata",
                "album=Frequency Suite",
                "-y",
                flac_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ffmpeg must be installed");
        assert!(status.success(), "Failed to generate test flac");

        // 4. Generate 100x100 PNG image for cover art test
        let temp_img = dir.path().join("temp_cover.png");
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=100x100:d=1",
                "-vframes",
                "1",
                "-y",
                temp_img.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ffmpeg must be installed");
        assert!(status.success(), "Failed to generate cover image");

        // 5. Generate MP3 with embedded cover art
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=stereo:d=2",
                "-i",
                temp_img.to_str().unwrap(),
                "-map",
                "0:a",
                "-map",
                "1:v",
                "-c:a",
                "libmp3lame",
                "-c:v",
                "mjpeg",
                "-id3v2_version",
                "3",
                "-metadata:s:v",
                "title=Album cover",
                "-metadata:s:v",
                "comment=Cover (front)",
                "-metadata",
                "title=Art Track",
                "-metadata",
                "artist=Cover Artist",
                "-y",
                cover_mp3_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ffmpeg must be installed");
        assert!(status.success(), "Failed to generate mp3 with cover");

        // 6. Generate folder-fallback audio + cover.jpg in album_dir
        std::fs::copy(&temp_img, &folder_cover_img_path).expect("copy cover.jpg");
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=stereo:d=1",
                "-y",
                folder_cover_audio_path.to_str().unwrap(),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ffmpeg must be installed");
        assert!(status.success(), "Failed to generate folder audio");

        AudioTestFixtures {
            _temp_dir: dir,
            mp3_path,
            wav_path,
            flac_path,
            cover_mp3_path,
            folder_cover_audio_path,
        }
    })
}

// ── 1. Dependency checks ───────────────────────────────────────────────────

#[test]
fn test_audio_tools_available() {
    let status = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    assert!(
        status.is_ok(),
        "ffmpeg must be installed to run audio tests"
    );
}

// ── 2. Parser Metadata Tests ───────────────────────────────────────────────

#[test]
fn test_audio_parser_supported_extensions() {
    let parser = AudioParser;
    let exts = parser.supported_extensions();
    assert!(exts.contains(&"mp3"));
    assert!(exts.contains(&"wav"));
    assert!(exts.contains(&"flac"));
    assert!(exts.contains(&"ogg"));
    assert!(exts.contains(&"m4a"));
    assert!(exts.contains(&"opus"));
}

#[test]
fn test_audio_parser_mp3_metadata() {
    let fixtures = get_fixtures();
    let parser = AudioParser;
    let parsed = parser
        .parse(&fixtures.mp3_path)
        .expect("Failed to parse MP3");

    match parsed {
        ParsedContent::Audio {
            title,
            artist,
            album,
            duration_secs,
            metadata,
            ..
        } => {
            assert_eq!(title, "Sine Wave");
            assert_eq!(artist, "Acoustic Labs");
            assert_eq!(album, "Synthesis Collection");
            assert_eq!(duration_secs, 3);
            assert!(metadata.contains("MP3"), "Expected MP3 in {metadata}");
            assert!(metadata.contains("kHz"), "Expected kHz in {metadata}");
        }
        other => panic!("Unexpected content type: {other:?}"),
    }
}

#[test]
fn test_audio_parser_flac_metadata() {
    let fixtures = get_fixtures();
    let parser = AudioParser;
    let parsed = parser
        .parse(&fixtures.flac_path)
        .expect("Failed to parse FLAC");

    match parsed {
        ParsedContent::Audio {
            title,
            artist,
            album,
            duration_secs,
            metadata,
            ..
        } => {
            assert_eq!(title, "Low Sine");
            assert_eq!(artist, "Bass Test");
            assert_eq!(album, "Frequency Suite");
            assert_eq!(duration_secs, 2);
            assert!(metadata.contains("FLAC"), "Expected FLAC in {metadata}");
        }
        other => panic!("Unexpected content type: {other:?}"),
    }
}

#[test]
fn test_audio_parser_wav_fallback_title() {
    let fixtures = get_fixtures();
    let parser = AudioParser;
    let parsed = parser
        .parse(&fixtures.wav_path)
        .expect("Failed to parse WAV");

    match parsed {
        ParsedContent::Audio {
            title,
            duration_secs,
            metadata,
            ..
        } => {
            assert_eq!(title, "test_sine");
            assert_eq!(duration_secs, 2);
            assert!(metadata.contains("WAV"), "Expected WAV in {metadata}");
        }
        other => panic!("Unexpected content type: {other:?}"),
    }
}

#[test]
fn test_audio_parser_embedded_cover_art() {
    let fixtures = get_fixtures();
    let parser = AudioParser;
    let parsed = parser
        .parse(&fixtures.cover_mp3_path)
        .expect("Failed to parse MP3 with cover");

    match parsed {
        ParsedContent::Audio {
            title, cover_art, ..
        } => {
            assert_eq!(title, "Art Track");
            assert!(
                cover_art.is_some(),
                "Embedded cover art should be extracted"
            );
            assert!(
                !cover_art.unwrap().is_empty(),
                "Cover art data should not be empty"
            );
        }
        other => panic!("Unexpected content type: {other:?}"),
    }
}

#[test]
fn test_audio_parser_folder_cover_fallback() {
    let fixtures = get_fixtures();
    let parser = AudioParser;
    let parsed = parser
        .parse(&fixtures.folder_cover_audio_path)
        .expect("Failed to parse folder audio");

    match parsed {
        ParsedContent::Audio { cover_art, .. } => {
            assert!(
                cover_art.is_some(),
                "Directory fallback cover art (cover.jpg) should be detected"
            );
            assert!(!cover_art.unwrap().is_empty());
        }
        other => panic!("Unexpected content type: {other:?}"),
    }
}

// ── 3. AudioPlayer GStreamer Lifecycle Tests ───────────────────────────────

#[test]
fn test_audio_player_initialization_and_play_pause() {
    let fixtures = get_fixtures();
    let path = fixtures.mp3_path.to_str().unwrap();

    let mut player = AudioPlayer::new(path, 3.0).expect("Failed to create AudioPlayer");
    assert!(player.is_playing(), "Player should start in playing state");

    let playing_after_toggle = player
        .toggle_play_pause()
        .expect("Failed to toggle play/pause");
    assert!(!playing_after_toggle, "Should be paused after first toggle");
    assert!(!player.is_playing());

    let playing_after_second_toggle = player
        .toggle_play_pause()
        .expect("Failed to toggle play/pause");
    assert!(
        playing_after_second_toggle,
        "Should be playing after second toggle"
    );
    assert!(player.is_playing());

    player.pause().expect("Failed to pause");
    assert!(!player.is_playing());

    player.play().expect("Failed to play");
    assert!(player.is_playing());
}

#[test]
fn test_audio_player_position_advancement() {
    let fixtures = get_fixtures();
    let path = fixtures.mp3_path.to_str().unwrap();

    let player = AudioPlayer::new(path, 3.0).expect("Failed to create AudioPlayer");

    let start = Instant::now();
    let mut pos = 0.0;
    while start.elapsed() < Duration::from_secs(2) {
        pos = player.position_secs();
        if pos > 0.05 {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        pos > 0.05,
        "Playback position should advance, got: {pos:.3}s"
    );

    let dur = player.duration_secs();
    assert!(
        (2.8..=3.2).contains(&dur),
        "Duration should be close to 3s, got: {dur:.3}s"
    );
}

#[test]
fn test_audio_player_seek() {
    let fixtures = get_fixtures();
    let path = fixtures.mp3_path.to_str().unwrap();

    let player = AudioPlayer::new(path, 3.0).expect("Failed to create AudioPlayer");

    // Wait briefly for GStreamer playbin to preroll
    let start_preroll = Instant::now();
    while start_preroll.elapsed() < Duration::from_secs(2) {
        if player.position_secs() > 0.0 || player.is_playing() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Seek to 1.5s
    player.seek_to_secs(1.5);
    let start_seek = Instant::now();
    let mut pos = 0.0;
    while start_seek.elapsed() < Duration::from_secs(2) {
        pos = player.position_secs();
        if (1.2..=2.2).contains(&pos) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        (1.2..=2.2).contains(&pos),
        "Position after seek(1.5s) should be around 1.5s, got: {pos:.3}s"
    );

    // Seek to ratio 0.8 (approx 2.4s)
    player.seek_to_ratio(0.8);
    let start_ratio = Instant::now();
    let mut pos_ratio = 0.0;
    while start_ratio.elapsed() < Duration::from_secs(2) {
        pos_ratio = player.position_secs();
        if (2.0..=2.8).contains(&pos_ratio) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        (2.0..=2.8).contains(&pos_ratio),
        "Position after seek_to_ratio(0.8) should be around 2.4s, got: {pos_ratio:.3}s"
    );

    // Seek relative back 1.0s
    player.seek_relative(-1.0);
    let start_rel = Instant::now();
    let mut pos_rel = pos_ratio;
    while start_rel.elapsed() < Duration::from_secs(2) {
        pos_rel = player.position_secs();
        if pos_rel < pos_ratio {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        pos_rel < pos_ratio,
        "Position after rewind should decrease: {pos_rel:.3}s < {pos_ratio:.3}s"
    );
}

#[test]
fn test_audio_player_seek_clamps() {
    let fixtures = get_fixtures();
    let path = fixtures.mp3_path.to_str().unwrap();

    let player = AudioPlayer::new(path, 3.0).expect("Failed to create AudioPlayer");

    // Negative relative seek should clamp without panicking
    player.seek_relative(-100.0);
    std::thread::sleep(Duration::from_millis(50));
    assert!(player.position_secs() >= 0.0);

    // Beyond duration seek should clamp without panicking
    player.seek_relative(100.0);
    std::thread::sleep(Duration::from_millis(50));
    assert!(player.position_secs() <= player.duration_secs() + 0.5);
}

#[test]
fn test_audio_player_eos_detection() {
    let fixtures = get_fixtures();
    let path = fixtures.folder_cover_audio_path.to_str().unwrap(); // 1.0s duration

    let mut player = AudioPlayer::new(path, 1.0).expect("Failed to create AudioPlayer");

    // Wait briefly for GStreamer playbin to preroll
    let start_preroll = Instant::now();
    while start_preroll.elapsed() < Duration::from_secs(2) {
        if player.position_secs() > 0.0 || player.is_playing() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Seek close to end (0.95s)
    player.seek_to_secs(0.95);

    let start = Instant::now();
    let mut reached_eos = false;
    while start.elapsed() < Duration::from_secs(5) {
        if player.poll_eos() {
            reached_eos = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(reached_eos, "Player should detect EOS (End of Stream)");
    assert!(!player.is_playing(), "Player should be paused after EOS");
}

#[test]
fn test_audio_player_rapid_switch_drop_cleanup() {
    let fixtures = get_fixtures();
    let paths = [
        fixtures.mp3_path.to_str().unwrap(),
        fixtures.wav_path.to_str().unwrap(),
        fixtures.flac_path.to_str().unwrap(),
    ];

    // Rapidly create and replace audio players (simulating fast user navigation in Dolphin)
    let mut current_player: Option<AudioPlayer> = None;
    for &p in paths.iter().cycle().take(10) {
        let player = AudioPlayer::new(p, 2.0).expect("AudioPlayer creation during rapid switch");
        current_player = Some(player);
        std::thread::sleep(Duration::from_millis(20));
    }

    // Verify last player is functioning
    if let Some(mut p) = current_player {
        assert!(p.is_playing());
        p.pause().expect("Pause final player");
    }
}

#[test]
fn test_audio_player_rapid_seek_no_stall() {
    let fixtures = get_fixtures();
    let path = fixtures.mp3_path.to_str().unwrap();

    let mut player = AudioPlayer::new(path, 3.0).expect("Failed to create AudioPlayer");
    assert!(player.is_playing());

    // Rapidly seek to various positions mimicking rapid key presses (e.g. key '3')
    for _ in 0..10 {
        player.seek_to_ratio(0.3);
        let _ = player.poll_eos();
        std::thread::sleep(Duration::from_millis(15));
    }

    assert!(player.is_playing(), "Player should remain in playing state");

    // Allow a small window for audio pipeline to advance from 0.9s (0.3 of 3.0s)
    let start = Instant::now();
    let mut advanced = false;
    while start.elapsed() < Duration::from_secs(2) {
        let _ = player.poll_eos();
        let pos = player.position_secs();
        if pos >= 0.85 {
            advanced = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        advanced,
        "Audio playback should continue advancing without stalling after rapid seeks"
    );
}
