use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Duration;

pub enum PlayerCommand {
    Load(String),
    Play,
    Pause,
    Seek(f64),
    SeekRelative(f64),
    Stop,
}

pub struct VideoPlayerController {
    sender: Sender<PlayerCommand>,
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

impl VideoPlayerController {
    pub fn spawn(ui_weak: slint::Weak<crate::ui::generated::PreviewWindow>) -> Self {
        let (tx, rx) = channel::<PlayerCommand>();
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
            let mut frame_count = 0;
            let mut ffmpeg_start_pos = 0.0;
            let mut seek_target: Option<f64> = None;

            let kill_and_reap_ffmpeg = |ffmpeg_child: &mut Option<Child>| {
                if let Some(mut child) = ffmpeg_child.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            };

            loop {
                // If playing, we poll commands with low timeout to keep video updating
                let mut cmd = if is_playing {
                    rx.recv_timeout(Duration::from_millis(5)).ok()
                } else {
                    rx.recv_timeout(Duration::from_millis(100)).ok()
                };

                if let Some(initial_cmd) = cmd.take() {
                    let mut current_cmd = initial_cmd;
                    let mut pending_non_seek = None;

                    // Coalesce consecutive Seek/SeekRelative commands
                    if matches!(
                        current_cmd,
                        PlayerCommand::Seek(_) | PlayerCommand::SeekRelative(_)
                    ) {
                        while let Ok(next_cmd) = rx.try_recv() {
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

                    let process_cmd =
                        |c: PlayerCommand,
                         mpv_opt: &mut Option<mpv::MpvHandler>,
                         ffmpeg_child: &mut Option<Child>,
                         current_path: &mut String,
                         is_playing: &mut bool,
                         current_pos: &mut f64,
                         duration: &mut f64,
                         fps: &mut f64,
                         target_w: &mut u32,
                         target_h: &mut u32,
                         frame_count: &mut u64,
                         ffmpeg_start_pos: &mut f64,
                         seek_target: &mut Option<f64>| {
                            match c {
                                PlayerCommand::Load(path) => {
                                    kill_and_reap_ffmpeg(ffmpeg_child);
                                    *mpv_opt = None;

                                    *current_path = path.clone();
                                    *is_playing = false;
                                    *current_pos = 0.0;
                                    *frame_count = 0;
                                    *ffmpeg_start_pos = 0.0;

                                    if let Some((w, h, f, d)) = probe_video_info(&path) {
                                        *duration = d;
                                        *fps = f;

                                        let max_dim = 720.0;
                                        let scale = (max_dim / (w.max(h) as f64)).min(1.0);
                                        *target_w = (((w as f64 * scale) as u32) & !1).max(16);
                                        *target_h = (((h as f64 * scale) as u32) & !1).max(16);
                                    } else {
                                        *duration = 0.0;
                                        *fps = 30.0;
                                        *target_w = 640;
                                        *target_h = 360;
                                    }

                                    let mut builder = mpv::MpvHandlerBuilder::new()
                                        .expect("Failed to init MPV builder");
                                    let _ = builder.set_option("vo", "null");
                                    let _ = builder.set_option("ytdl", "no");
                                    let _ = builder.set_option("keep-open", "yes");
                                    if let Ok(mut mpv) = builder.build() {
                                        let _ = mpv.command(&["loadfile", &path]);
                                        let _ = mpv.set_property("pause", true);
                                        *mpv_opt = Some(mpv);
                                    }
                                }
                                PlayerCommand::Play => {
                                    if let Some(mpv) = mpv_opt {
                                        if *current_pos >= *duration - 0.5 {
                                            let _ = mpv.command(&["seek", "0.0", "absolute"]);
                                            *current_pos = 0.0;
                                        }
                                        let _ = mpv.set_property("pause", false);
                                        *is_playing = true;

                                        kill_and_reap_ffmpeg(ffmpeg_child);
                                        *frame_count = 0;
                                        *ffmpeg_start_pos = *current_pos;
                                        *ffmpeg_child = Command::new("ffmpeg")
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
                                            .ok();
                                    }
                                }
                                PlayerCommand::Pause => {
                                    if let Some(mpv) = mpv_opt {
                                        let _ = mpv.set_property("pause", true);
                                        *is_playing = false;
                                        kill_and_reap_ffmpeg(ffmpeg_child);
                                    }
                                }
                                PlayerCommand::Seek(percent) => {
                                    if let Some(mpv) = mpv_opt {
                                        let target = percent * *duration;
                                        let _ =
                                            mpv.command(&["seek", &target.to_string(), "absolute"]);
                                        *current_pos = target;
                                        *seek_target = Some(target);

                                        if *is_playing {
                                            kill_and_reap_ffmpeg(ffmpeg_child);
                                            *frame_count = 0;
                                            *ffmpeg_start_pos = *current_pos;
                                            *ffmpeg_child = Command::new("ffmpeg")
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
                                                .ok();
                                        } else {
                                            let ui_weak_clone = ui_weak.clone();
                                            let path_clone = current_path.clone();
                                            let tw = *target_w;
                                            let th = *target_h;
                                            let target_seek = *current_pos;
                                            thread::spawn(move || {
                                                let output = Command::new("ffmpeg")
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
                                                        let _ = slint::invoke_from_event_loop(
                                                            move || {
                                                                if let Some(ui) =
                                                                    ui_weak_clone.upgrade()
                                                                {
                                                                    let slint_img = crate::ui::helpers::make_image_from_rgba(&raw_data, tw, th);
                                                                    ui.set_preview_image(slint_img);
                                                                }
                                                            },
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                PlayerCommand::SeekRelative(seconds) => {
                                    if let Some(mpv) = mpv_opt {
                                        let target = (*current_pos + seconds).clamp(0.0, *duration);
                                        let _ =
                                            mpv.command(&["seek", &target.to_string(), "absolute"]);
                                        *current_pos = target;
                                        *seek_target = Some(target);
                                        if *is_playing {
                                            kill_and_reap_ffmpeg(ffmpeg_child);
                                            *frame_count = 0;
                                            *ffmpeg_start_pos = *current_pos;
                                            *ffmpeg_child = Command::new("ffmpeg")
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
                                                .ok();
                                        } else {
                                            let ui_weak_clone = ui_weak.clone();
                                            let path_clone = current_path.clone();
                                            let tw = *target_w;
                                            let th = *target_h;
                                            let target_seek = *current_pos;
                                            thread::spawn(move || {
                                                let output = Command::new("ffmpeg")
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
                                                        let _ = slint::invoke_from_event_loop(
                                                            move || {
                                                                if let Some(ui) =
                                                                    ui_weak_clone.upgrade()
                                                                {
                                                                    let slint_img = crate::ui::helpers::make_image_from_rgba(&raw_data, tw, th);
                                                                    ui.set_preview_image(slint_img);
                                                                }
                                                            },
                                                        );
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                                PlayerCommand::Stop => {
                                    kill_and_reap_ffmpeg(ffmpeg_child);
                                    *mpv_opt = None;
                                    *is_playing = false;
                                }
                            }
                        };

                    process_cmd(
                        current_cmd,
                        &mut mpv_opt,
                        &mut ffmpeg_child,
                        &mut current_path,
                        &mut is_playing,
                        &mut current_pos,
                        &mut duration,
                        &mut fps,
                        &mut target_w,
                        &mut target_h,
                        &mut frame_count,
                        &mut ffmpeg_start_pos,
                        &mut seek_target,
                    );

                    if let Some(non_seek_cmd) = pending_non_seek {
                        process_cmd(
                            non_seek_cmd,
                            &mut mpv_opt,
                            &mut ffmpeg_child,
                            &mut current_path,
                            &mut is_playing,
                            &mut current_pos,
                            &mut duration,
                            &mut fps,
                            &mut target_w,
                            &mut target_h,
                            &mut frame_count,
                            &mut ffmpeg_start_pos,
                            &mut seek_target,
                        );
                    }
                }

                // If playing and ffmpeg is active, read the next frame based on synchronization
                if let (true, Some(mpv)) = (is_playing, mpv_opt.as_mut()) {
                    // Empty event queue
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

                    // Synchronize video frames to audio clock
                    let next_frame_time = ffmpeg_start_pos + (frame_count as f64 / fps);
                    if current_pos >= next_frame_time {
                        let stdout_opt = ffmpeg_child.as_mut().and_then(|c| c.stdout.as_mut());
                        if let Some(stdout) = stdout_opt {
                            let frame_size = (target_w * target_h * 4) as usize;
                            let mut buffer = vec![0u8; frame_size];
                            if stdout.read_exact(&mut buffer).is_ok() {
                                frame_count += 1;
                                let ui_weak_clone = ui_weak.clone();
                                let raw_data = buffer;
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_weak_clone.upgrade() {
                                        let slint_img = crate::ui::helpers::make_image_from_rgba(
                                            &raw_data, target_w, target_h,
                                        );
                                        ui.set_preview_image(slint_img);
                                    }
                                });
                            }
                        }
                    }

                    // Update seekbar / timestamps
                    let ui_weak_clone = ui_weak.clone();
                    let pos = current_pos;
                    let playing = is_playing;
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_video_playing(playing);
                            if dur > 0.0 {
                                ui.set_video_progress((pos / dur) as f32);
                                let cur_mins = (pos / 60.0) as u32;
                                let cur_secs = (pos % 60.0) as u32;
                                let dur_mins = (dur / 60.0) as u32;
                                let dur_secs = (dur % 60.0) as u32;
                                ui.set_video_time(
                                    format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}")
                                        .into(),
                                );
                            }
                        }
                    });
                }
            }
        });

        Self { sender: tx }
    }

    pub fn load(&self, path: String) {
        let _ = self.sender.send(PlayerCommand::Load(path));
    }

    pub fn play(&self) {
        let _ = self.sender.send(PlayerCommand::Play);
    }

    pub fn pause(&self) {
        let _ = self.sender.send(PlayerCommand::Pause);
    }

    pub fn seek(&self, percent: f64) {
        let _ = self.sender.send(PlayerCommand::Seek(percent));
    }

    pub fn seek_relative(&self, seconds: f64) {
        let _ = self.sender.send(PlayerCommand::SeekRelative(seconds));
    }

    pub fn stop(&self) {
        let _ = self.sender.send(PlayerCommand::Stop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_video_player_spawn_and_commands() {
        let weak_ui = slint::Weak::default();
        let player = VideoPlayerController::spawn(weak_ui);

        // Verify that commands can be sent and do not panic the controller thread
        player.load("nonexistent_video.mp4".to_string());
        player.play();
        player.seek(0.5);
        player.seek_relative(5.0);
        player.pause();
        player.stop();

        // Give a moment for the thread to process commands
        std::thread::sleep(Duration::from_millis(50));
    }
}
