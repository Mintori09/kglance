use std::hash::Hash;
use std::io::Read;

use iced_futures::subscription::{self, Recipe};
use tokio::sync::mpsc;

use crate::app::Message;
use crate::{log_debug, log_error};

use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Load(String),
    Play,
    Pause,
    Seek(f64),
    SeekRelative(f64),
    Stop,
}

#[derive(Debug, Clone)]
pub enum VideoEvent {
    Frame {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Progress {
        position: f64,
        duration: f64,
        is_playing: bool,
    },
}

fn probe_video_info(path: &str) -> Option<(u32, u32, f64, f64)> {
    // Probe resolution and framerate
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate",
            "-of",
            "csv=p=0",
            path,
        ])
        .output()
        .ok()?;

    let s = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = s.trim().split(',').collect();
    if parts.len() >= 3 {
        let w = parts[0].parse::<u32>().unwrap_or(640);
        let h = parts[1].parse::<u32>().unwrap_or(360);

        let fps_parts: Vec<&str> = parts[2].split('/').collect();
        let fps = if fps_parts.len() == 2 {
            let num = fps_parts[0].parse::<f64>().unwrap_or(30.0);
            let den = fps_parts[1].parse::<f64>().unwrap_or(1.0);
            if den > 0.0 { num / den } else { 30.0 }
        } else {
            parts[2].parse::<f64>().unwrap_or(30.0)
        };

        // Probe duration
        let dur_output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "csv=p=0",
                path,
            ])
            .output()
            .ok()?;
        let dur_s = String::from_utf8_lossy(&dur_output.stdout);
        let duration = dur_s.trim().parse::<f64>().unwrap_or(0.0);

        Some((w, h, fps, duration))
    } else {
        None
    }
}

pub fn spawn_video_player(
    mut cmd_rx: tokio::sync::mpsc::Receiver<PlayerCommand>,
    event_tx: tokio::sync::mpsc::Sender<VideoEvent>,
) {
    thread::spawn(move || {
        let mut mpv_opt: Option<mpv::MpvHandler> = None;
        let mut ffmpeg_child: Option<Child> = None;
        let mut current_path = String::new();
        let mut is_playing = false;
        let mut current_pos = 0.0;
        let mut duration = 0.0;
        let mut fps = 30.0;
        let mut target_w = 640;
        let mut target_h = 360;
        let mut ffmpeg_start_pos = 0.0;
        let mut seek_target: Option<f64> = None;
        let mut has_video_stream = false;

        let shared_pos = Arc::new(AtomicU64::new(0.0f64.to_bits()));
        let mut last_progress_sent = std::time::Instant::now();
        let mut last_sent_playing = false;
        let mut last_sent_pos = -1.0;

        let kill_and_reap_ffmpeg = |ffmpeg_child: &mut Option<Child>| {
            if let Some(mut child) = ffmpeg_child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        };

        let spawn_ffmpeg_reader =
            |child: &mut Child,
             start_pos: f64,
             target_w: u32,
             target_h: u32,
             fps: f64,
             shared_pos: Arc<AtomicU64>,
             event_tx: tokio::sync::mpsc::Sender<VideoEvent>| {
                let mut stdout = child
                    .stdout
                    .take()
                    .expect("Failed to open stdout of ffmpeg");
                let event_tx_clone = event_tx.clone();
                thread::spawn(move || {
                    use std::time::Instant;
                    let mut start_instant = Instant::now();
                    let mut frame_count = 0usize;
                    let frame_size = (target_w * target_h * 4) as usize;
                    let frame_duration = 1.0 / fps;

                    loop {
                        let mut buffer = vec![0u8; frame_size];
                        if stdout.read_exact(&mut buffer).is_err() {
                            log_error!(
                                "ffmpeg reader: read_exact failed after {} frames",
                                frame_count
                            );
                            break;
                        }

                        let elapsed_expected = frame_count as f64 * frame_duration;

                        // Drift correction: check position from MPV
                        let cur_pos = f64::from_bits(shared_pos.load(Ordering::Relaxed));
                        let cur_elapsed = (cur_pos - start_pos).max(0.0);
                        let drift = cur_elapsed - elapsed_expected;
                        if drift.abs() > 0.15
                            && let Some(adjusted) =
                                Instant::now().checked_sub(Duration::from_secs_f64(cur_elapsed))
                        {
                            start_instant = adjusted;
                        }

                        let target_time = start_instant + Duration::from_secs_f64(elapsed_expected);
                        let now = Instant::now();
                        if target_time > now {
                            thread::sleep(target_time - now);
                        }

                        frame_count += 1;
                        if frame_count.is_multiple_of(30) {
                            log_debug!("ffmpeg reader: sent {} frames (fps={})", frame_count, fps);
                        }
                        #[allow(clippy::collapsible_if)]
                        if let Err(e) = event_tx_clone.try_send(VideoEvent::Frame {
                            data: buffer,
                            width: target_w,
                            height: target_h,
                        }) {
                            if frame_count <= 5 || frame_count.is_multiple_of(30) {
                                log_debug!(
                                    "ffmpeg reader: frame {} dropped ({:?})",
                                    frame_count,
                                    e
                                );
                            }
                        }
                    }
                });
            };

        loop {
            // Try to receive a command with a timeout depending on play status
            let timeout = if is_playing {
                Duration::from_millis(5)
            } else {
                Duration::from_millis(100)
            };

            // Using blocking try_recv or standard recv with timeout
            let mut cmd = match cmd_rx.try_recv() {
                Ok(c) => Some(c),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    thread::sleep(timeout);
                    cmd_rx.try_recv().ok()
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            };

            if let Some(initial_cmd) = cmd.take() {
                let mut current_cmd = initial_cmd;
                let mut pending_non_seek = None;

                // Coalesce consecutive Seek/SeekRelative commands
                if matches!(
                    current_cmd,
                    PlayerCommand::Seek(_) | PlayerCommand::SeekRelative(_)
                ) {
                    while let Ok(next_cmd) = cmd_rx.try_recv() {
                        match next_cmd {
                            PlayerCommand::Seek(_) | PlayerCommand::SeekRelative(_) => {
                                current_cmd = next_cmd;
                            }
                            _ => {
                                pending_non_seek = Some(next_cmd);
                                break;
                            }
                        }
                    }
                }

                let mut process_cmd = |c: PlayerCommand| match c {
                    PlayerCommand::Load(path) => {
                        kill_and_reap_ffmpeg(&mut ffmpeg_child);
                        mpv_opt = None;

                        current_path = path.clone();
                        is_playing = false;
                        current_pos = 0.0;
                        ffmpeg_start_pos = 0.0;
                        shared_pos.store(0.0f64.to_bits(), Ordering::Relaxed);

                        if let Some((w, h, f, d)) = probe_video_info(&path) {
                            has_video_stream = true;
                            duration = d;
                            fps = f;

                            let max_dim = 1080.0;
                            let scale = (max_dim / (w.max(h) as f64)).min(1.0);
                            target_w = (((w as f64 * scale) as u32) & !1).max(16);
                            target_h = (((h as f64 * scale) as u32) & !1).max(16);
                        } else {
                            has_video_stream = false;
                            duration = 0.0;
                            fps = 30.0;
                            target_w = 640;
                            target_h = 360;
                        }

                        let mut builder =
                            mpv::MpvHandlerBuilder::new().expect("Failed to init MPV builder");
                        let _ = builder.set_option("vo", "null");
                        let _ = builder.set_option("ytdl", "no");
                        let _ = builder.set_option("keep-open", "yes");
                        if let Ok(mut mpv) = builder.build() {
                            let _ = mpv.command(&["loadfile", &path]);
                            let _ = mpv.set_property("pause", true);
                            mpv_opt = Some(mpv);
                        }
                    }
                    PlayerCommand::Play => {
                        if let Some(ref mut mpv) = mpv_opt {
                            if current_pos >= duration - 0.5 {
                                let _ = mpv.command(&["seek", "0.0", "absolute"]);
                                current_pos = 0.0;
                                shared_pos.store(0.0f64.to_bits(), Ordering::Relaxed);
                            }
                            let _ = mpv.set_property("pause", false);
                            is_playing = true;

                            if has_video_stream {
                                kill_and_reap_ffmpeg(&mut ffmpeg_child);
                                ffmpeg_start_pos = current_pos;
                                if let Ok(mut child) = Command::new("ffmpeg")
                                    .stdin(Stdio::null())
                                    .args([
                                        "-ss",
                                        &current_pos.to_string(),
                                        "-i",
                                        current_path.as_str(),
                                        "-vf",
                                        &format!("scale={target_w}:{target_h}"),
                                        "-f",
                                        "rawvideo",
                                        "-vcodec",
                                        "rawvideo",
                                        "-pix_fmt",
                                        "rgba",
                                        "-",
                                    ])
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::inherit())
                                    .spawn()
                                {
                                    spawn_ffmpeg_reader(
                                        &mut child,
                                        ffmpeg_start_pos,
                                        target_w,
                                        target_h,
                                        fps,
                                        shared_pos.clone(),
                                        event_tx.clone(),
                                    );
                                    ffmpeg_child = Some(child);
                                }
                            }
                        }
                    }
                    PlayerCommand::Pause => {
                        if let Some(ref mut mpv) = mpv_opt {
                            let _ = mpv.set_property("pause", true);
                            is_playing = false;
                            kill_and_reap_ffmpeg(&mut ffmpeg_child);
                        }
                    }
                    PlayerCommand::Seek(percent) => {
                        if let Some(ref mut mpv) = mpv_opt {
                            let target = percent * duration;
                            let _ = mpv.command(&["seek", &target.to_string(), "absolute"]);
                            current_pos = target;
                            shared_pos.store(target.to_bits(), Ordering::Relaxed);
                            seek_target = Some(target);

                            if is_playing {
                                if has_video_stream {
                                    kill_and_reap_ffmpeg(&mut ffmpeg_child);
                                    ffmpeg_start_pos = current_pos;
                                    if let Ok(mut child) = Command::new("ffmpeg")
                                        .stdin(Stdio::null())
                                        .args([
                                            "-ss",
                                            &current_pos.to_string(),
                                            "-i",
                                            current_path.as_str(),
                                            "-vf",
                                            &format!("scale={target_w}:{target_h}"),
                                            "-f",
                                            "rawvideo",
                                            "-vcodec",
                                            "rawvideo",
                                            "-pix_fmt",
                                            "rgba",
                                            "-",
                                        ])
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::inherit())
                                        .spawn()
                                    {
                                        spawn_ffmpeg_reader(
                                            &mut child,
                                            ffmpeg_start_pos,
                                            target_w,
                                            target_h,
                                            fps,
                                            shared_pos.clone(),
                                            event_tx.clone(),
                                        );
                                        ffmpeg_child = Some(child);
                                    }
                                }
                            } else if has_video_stream {
                                kill_and_reap_ffmpeg(&mut ffmpeg_child);
                                if let Ok(mut child) = Command::new("ffmpeg")
                                    .stdin(Stdio::null())
                                    .args([
                                        "-ss",
                                        &current_pos.to_string(),
                                        "-i",
                                        current_path.as_str(),
                                        "-vf",
                                        &format!("scale={target_w}:{target_h}"),
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
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::inherit())
                                    .spawn()
                                {
                                    let event_tx_clone = event_tx.clone();
                                    let tw = target_w;
                                    let th = target_h;
                                    let mut stdout =
                                        child.stdout.take().expect("Failed to open stdout");
                                    thread::spawn(move || {
                                        let frame_size = (tw * th * 4) as usize;
                                        let mut buffer = vec![0u8; frame_size];
                                        if stdout.read_exact(&mut buffer).is_ok() {
                                            log_debug!(
                                                "ffmpeg single-frame reader: sent frame ({}x{})",
                                                tw,
                                                th
                                            );
                                            let _ = event_tx_clone.try_send(VideoEvent::Frame {
                                                data: buffer,
                                                width: tw,
                                                height: th,
                                            });
                                        } else {
                                            log_error!(
                                                "ffmpeg single-frame reader: read_exact failed"
                                            );
                                        }
                                    });
                                    ffmpeg_child = Some(child);
                                }
                            }
                        }
                    }
                    PlayerCommand::SeekRelative(seconds) => {
                        if let Some(ref mut mpv) = mpv_opt {
                            let target = (current_pos + seconds).clamp(0.0, duration);
                            let _ = mpv.command(&["seek", &target.to_string(), "absolute"]);
                            current_pos = target;
                            shared_pos.store(target.to_bits(), Ordering::Relaxed);
                            seek_target = Some(target);
                            if is_playing {
                                if has_video_stream {
                                    kill_and_reap_ffmpeg(&mut ffmpeg_child);
                                    ffmpeg_start_pos = current_pos;
                                    if let Ok(mut child) = Command::new("ffmpeg")
                                        .stdin(Stdio::null())
                                        .args([
                                            "-ss",
                                            &current_pos.to_string(),
                                            "-i",
                                            current_path.as_str(),
                                            "-vf",
                                            &format!("scale={target_w}:{target_h}"),
                                            "-f",
                                            "rawvideo",
                                            "-vcodec",
                                            "rawvideo",
                                            "-pix_fmt",
                                            "rgba",
                                            "-",
                                        ])
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::inherit())
                                        .spawn()
                                    {
                                        spawn_ffmpeg_reader(
                                            &mut child,
                                            ffmpeg_start_pos,
                                            target_w,
                                            target_h,
                                            fps,
                                            shared_pos.clone(),
                                            event_tx.clone(),
                                        );
                                        ffmpeg_child = Some(child);
                                    }
                                }
                            } else if has_video_stream {
                                kill_and_reap_ffmpeg(&mut ffmpeg_child);
                                if let Ok(mut child) = Command::new("ffmpeg")
                                    .stdin(Stdio::null())
                                    .args([
                                        "-ss",
                                        &current_pos.to_string(),
                                        "-i",
                                        current_path.as_str(),
                                        "-vf",
                                        &format!("scale={target_w}:{target_h}"),
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
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::inherit())
                                    .spawn()
                                {
                                    let event_tx_clone = event_tx.clone();
                                    let tw = target_w;
                                    let th = target_h;
                                    let mut stdout =
                                        child.stdout.take().expect("Failed to open stdout");
                                    thread::spawn(move || {
                                        let frame_size = (tw * th * 4) as usize;
                                        let mut buffer = vec![0u8; frame_size];
                                        if stdout.read_exact(&mut buffer).is_ok() {
                                            log_debug!(
                                                "ffmpeg single-frame reader: sent frame ({}x{})",
                                                tw,
                                                th
                                            );
                                            let _ = event_tx_clone.try_send(VideoEvent::Frame {
                                                data: buffer,
                                                width: tw,
                                                height: th,
                                            });
                                        } else {
                                            log_error!(
                                                "ffmpeg single-frame reader: read_exact failed"
                                            );
                                        }
                                    });
                                    ffmpeg_child = Some(child);
                                }
                            }
                        }
                    }
                    PlayerCommand::Stop => {
                        kill_and_reap_ffmpeg(&mut ffmpeg_child);
                        mpv_opt = None;
                        is_playing = false;
                        shared_pos.store(0.0f64.to_bits(), Ordering::Relaxed);
                    }
                };

                process_cmd(current_cmd);

                if let Some(non_seek_cmd) = pending_non_seek {
                    process_cmd(non_seek_cmd);
                }
            }

            // Process MPV events and track progress for both audio and video
            if let Some(ref mut mpv) = mpv_opt.as_mut() {
                while let Some(event) = mpv.wait_event(0.0) {
                    match event {
                        mpv::Event::Shutdown | mpv::Event::Idle => {
                            is_playing = false;
                        }
                        _ => {}
                    }
                }

                let raw_pos = mpv.get_property::<f64>("time-pos").unwrap_or(0.0);
                let dur = mpv.get_property::<f64>("duration").unwrap_or(0.0);
                if dur > 0.0 {
                    duration = dur;
                }

                if let Some(target) = seek_target {
                    if (raw_pos - target).abs() < 0.5 {
                        seek_target = None;
                        current_pos = raw_pos;
                    } else {
                        current_pos = target;
                    }
                } else {
                    current_pos = raw_pos;
                }
                shared_pos.store(current_pos.to_bits(), Ordering::Relaxed);

                // Send progress update (rate-limited to avoid channel congestion)
                let now = std::time::Instant::now();
                let should_send = is_playing != last_sent_playing
                    || (current_pos - last_sent_pos).abs() > 0.5
                    || now.duration_since(last_progress_sent) >= Duration::from_millis(100);

                if should_send
                    && event_tx
                        .try_send(VideoEvent::Progress {
                            position: current_pos,
                            duration: dur,
                            is_playing,
                        })
                        .is_ok()
                {
                    last_progress_sent = now;
                    last_sent_playing = is_playing;
                    last_sent_pos = current_pos;
                }
            }
        }
    });
}

use std::sync::Mutex;

pub struct VideoRecipe {
    rx: Arc<Mutex<Option<mpsc::Receiver<VideoEvent>>>>,
}

impl VideoRecipe {
    pub fn new(rx: Arc<Mutex<Option<mpsc::Receiver<VideoEvent>>>>) -> Self {
        Self { rx }
    }
}

impl Recipe for VideoRecipe {
    type Output = Message;

    fn hash(&self, state: &mut subscription::Hasher) {
        "video_subscription".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: subscription::EventStream,
    ) -> iced_futures::BoxStream<Self::Output> {
        let rx = self.rx.lock().unwrap().take();
        match rx {
            Some(mut rx) => iced_futures::boxed_stream(iced::stream::channel(
                100,
                move |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
                    use iced::futures::SinkExt;
                    let mut frame_count = 0usize;
                    while let Some(event) = rx.recv().await {
                        let is_frame = matches!(event, VideoEvent::Frame { .. });
                        if is_frame {
                            frame_count += 1;
                        }
                        if is_frame && frame_count.is_multiple_of(30) {
                            log_debug!(
                                "VideoRecipe: frame {} received from tokio channel",
                                frame_count
                            );
                        }
                        if output
                            .send(Message::VideoEventReceived(event))
                            .await
                            .is_err()
                        {
                            log_error!("VideoRecipe: iced channel closed, stopping stream");
                            break;
                        }
                    }
                },
            )),
            None => iced_futures::boxed_stream(iced::futures::stream::empty()),
        }
    }
}
