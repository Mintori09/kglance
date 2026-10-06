use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::features::video::subtitles::{SubtitleTrack, find_active_subtitle};
use iced::Task;

pub fn handle_play_pause(app: &mut KglanceApp) -> Task<Message> {
    if let Some(video) = &mut app.video {
        // If video ended, seek to start before playing again.
        if app.state.media.video_ended {
            app.state.media.video_ended = false;
            app.state.media.progress = 0.0;
            let _ = video.seek(std::time::Duration::ZERO, true);
        }
        crate::features::video::handler::toggle_play_pause(video);
        app.state.media.playing = !video.paused();
    }
    Task::none()
}

pub fn handle_seek(app: &mut KglanceApp, pct: f32) -> Task<Message> {
    if let Some(video) = &mut app.video {
        crate::features::video::handler::seek_to_ratio(video, pct as f64);
    }
    Task::none()
}

pub fn handle_seek_relative(app: &mut KglanceApp, secs: f32) -> Task<Message> {
    if let Some(video) = &mut app.video {
        crate::features::video::handler::seek_relative(video, secs as f64);
    }
    Task::none()
}

pub fn handle_video_new_frame(app: &mut KglanceApp) -> Task<Message> {
    if let Some(video) = &app.video {
        let position = video.position().as_secs_f64();
        let duration = video.duration().as_secs_f64();
        app.state.media.playing = !video.paused();
        app.state.media.position_secs = position;
        app.state.media.duration_secs = duration;
        if duration > 0.0 {
            app.state.media.progress = (position / duration) as f32;
            app.state.media.time = format!(
                "{} / {}",
                format_duration(position),
                format_duration(duration)
            );
        }

        // Update active subtitle text only when changed to avoid allocating String on every frame
        let new_sub = if app.state.media.subtitles_enabled {
            app.state
                .media
                .active_subtitle_track
                .and_then(|idx| app.state.media.subtitle_tracks.get(idx))
                .and_then(|track| find_active_subtitle(&track.entries, position))
        } else {
            None
        };

        match (new_sub, &app.state.media.current_subtitle_text) {
            (Some(text), Some(curr)) if text == curr.as_str() => {}
            (Some(text), _) => {
                app.state.media.current_subtitle_text = Some(text.to_string());
            }
            (None, None) => {}
            (None, Some(_)) => {
                app.state.media.current_subtitle_text = None;
            }
        }
    }
    Task::none()
}

fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn handle_video_end_of_stream(app: &mut KglanceApp) -> Task<Message> {
    app.state.media.playing = false;
    app.state.media.video_ended = true;
    app.state.media.current_subtitle_text = None;
    Task::none()
}

pub fn handle_media_mouse_enter(app: &mut KglanceApp) -> Task<Message> {
    app.state.media.show_controls = true;
    Task::none()
}

pub fn handle_media_mouse_leave(app: &mut KglanceApp) -> Task<Message> {
    app.state.media.show_controls = false;
    Task::none()
}

pub fn handle_subtitles_loaded(app: &mut KglanceApp, tracks: Vec<SubtitleTrack>) -> Task<Message> {
    if !app.state.media.has_video {
        return Task::none();
    }
    if !tracks.is_empty() {
        app.state.media.subtitle_tracks = tracks;
        app.state.media.active_subtitle_track = Some(0);
        app.state.media.subtitles_enabled = true;
    } else {
        app.state.media.subtitle_tracks.clear();
        app.state.media.active_subtitle_track = None;
        app.state.media.subtitles_enabled = false;
        app.state.media.current_subtitle_text = None;
    }
    Task::none()
}

pub fn handle_toggle_subtitles(app: &mut KglanceApp) -> Task<Message> {
    if !app.state.media.subtitle_tracks.is_empty() {
        app.state.media.subtitles_enabled = !app.state.media.subtitles_enabled;
        if !app.state.media.subtitles_enabled {
            app.state.media.current_subtitle_text = None;
        }
    }
    Task::none()
}

pub fn handle_cycle_subtitle_track(app: &mut KglanceApp) -> Task<Message> {
    let count = app.state.media.subtitle_tracks.len();
    if count > 0 {
        if let Some(current) = app.state.media.active_subtitle_track {
            if current + 1 < count {
                app.state.media.active_subtitle_track = Some(current + 1);
                app.state.media.subtitles_enabled = true;
            } else {
                app.state.media.active_subtitle_track = Some(0);
                app.state.media.subtitles_enabled = false;
                app.state.media.current_subtitle_text = None;
            }
        } else {
            app.state.media.active_subtitle_track = Some(0);
            app.state.media.subtitles_enabled = true;
        }
    }
    Task::none()
}
