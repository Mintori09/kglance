use std::io::Read;
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
                    let mut frame_count = 0;
                    let frame_size = (target_w * target_h * 4) as usize;
                    let mut buffer = vec![0u8; frame_size];
                    loop {
                        let next_frame_time = start_pos + (frame_count as f64 / fps);
                        loop {
                            let cur_pos = f64::from_bits(shared_pos.load(Ordering::Relaxed));
                            if cur_pos >= next_frame_time {
                                break;
                            }
                            thread::sleep(Duration::from_millis(2));
                        }
                        if stdout.read_exact(&mut buffer).is_err() {
                            break; // Process killed or EOF
                        }
                        frame_count += 1;
                        let _ = event_tx_clone.try_send(VideoEvent::Frame {
                            data: buffer.clone(),
                            width: target_w,
                            height: target_h,
                        });
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
                                        "-pix_fmt",
                                        "rgba",
                                        "-",
                                    ])
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::null())
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
                                            "-pix_fmt",
                                            "rgba",
                                            "-",
                                        ])
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::null())
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
                                let event_tx_clone = event_tx.clone();
                                let path_clone = current_path.clone();
                                let tw = target_w;
                                let th = target_h;
                                let target_seek = current_pos;
                                thread::spawn(move || {
                                    let output = Command::new("ffmpeg")
                                        .stdin(Stdio::null())
                                        .args([
                                            "-ss",
                                            &target_seek.to_string(),
                                            "-i",
                                            &path_clone,
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
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::null())
                                        .output();
                                    if let Ok(out) = output {
                                        let raw_data = out.stdout;
                                        if raw_data.len() == (tw * th * 4) as usize {
                                            let _ = event_tx_clone.try_send(VideoEvent::Frame {
                                                data: raw_data,
                                                width: tw,
                                                height: th,
                                            });
                                        }
                                    }
                                });
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
                                            "-pix_fmt",
                                            "rgba",
                                            "-",
                                        ])
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::null())
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
                                let event_tx_clone = event_tx.clone();
                                let path_clone = current_path.clone();
                                let tw = target_w;
                                let th = target_h;
                                let target_seek = current_pos;
                                thread::spawn(move || {
                                    let output = Command::new("ffmpeg")
                                        .stdin(Stdio::null())
                                        .args([
                                            "-ss",
                                            &target_seek.to_string(),
                                            "-i",
                                            &path_clone,
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
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::null())
                                        .output();
                                    if let Ok(out) = output {
                                        let raw_data = out.stdout;
                                        if raw_data.len() == (tw * th * 4) as usize {
                                            let _ = event_tx_clone.try_send(VideoEvent::Frame {
                                                data: raw_data,
                                                width: tw,
                                                height: th,
                                            });
                                        }
                                    }
                                });
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

                // Send progress update
                let _ = event_tx.try_send(VideoEvent::Progress {
                    position: current_pos,
                    duration: dur,
                    is_playing,
                });
            }
        }
    });
}
