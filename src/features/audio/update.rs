use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_play_pause(app: &mut KglanceApp) -> Task<Message> {
    if let Some(audio) = &mut app.audio {
        match audio.toggle_play_pause() {
            Ok(playing) => {
                app.state.audio.playing = playing;
            }
            Err(e) => {
                crate::log_error!("Failed to toggle audio playback: {e}");
                app.state.audio.error = Some(e);
            }
        }
    }
    Task::none()
}

pub fn handle_seek(app: &mut KglanceApp, pct: f32) -> Task<Message> {
    let clamped = pct.clamp(0.0, 1.0);
    let total_dur = app.state.audio.duration_secs;
    let pos = clamped as f64 * total_dur;
    app.state.audio.progress = clamped;
    app.state.audio.position_secs = pos;
    if total_dur > 0.0 {
        app.state.audio.time = format_time_label(pos, total_dur);
    }
    if let Some(audio) = &mut app.audio {
        audio.seek_to_secs(pos);
    }
    Task::none()
}

pub fn handle_seek_relative(app: &mut KglanceApp, secs: f32) -> Task<Message> {
    let total_dur = app.state.audio.duration_secs;
    let cur_pos = app.state.audio.position_secs;
    let new_pos = if total_dur > 0.0 {
        (cur_pos + secs as f64).clamp(0.0, total_dur)
    } else {
        (cur_pos + secs as f64).max(0.0)
    };
    app.state.audio.position_secs = new_pos;
    if total_dur > 0.0 {
        app.state.audio.progress = (new_pos / total_dur).clamp(0.0, 1.0) as f32;
        app.state.audio.time = format_time_label(new_pos, total_dur);
    }
    if let Some(audio) = &mut app.audio {
        audio.seek_to_secs(new_pos);
    }
    Task::none()
}

pub fn handle_volume_change(app: &mut KglanceApp, vol: f32) -> Task<Message> {
    let clamped = vol.clamp(0.0, 1.0);
    app.state.audio.volume = clamped;
    app.state.audio.muted = clamped == 0.0;
    if clamped > 0.0 {
        app.state.audio.volume_before_mute = clamped;
    }
    if let Some(audio) = &app.audio {
        audio.set_volume(clamped as f64);
    }
    Task::none()
}

pub fn handle_toggle_mute(app: &mut KglanceApp) -> Task<Message> {
    if app.state.audio.muted {
        let restored = if app.state.audio.volume_before_mute > 0.0 {
            app.state.audio.volume_before_mute
        } else {
            1.0
        };
        app.state.audio.volume = restored;
        app.state.audio.muted = false;
        if let Some(audio) = &app.audio {
            audio.set_volume(restored as f64);
        }
    } else {
        app.state.audio.volume_before_mute = app.state.audio.volume;
        app.state.audio.volume = 0.0;
        app.state.audio.muted = true;
        if let Some(audio) = &app.audio {
            audio.set_volume(0.0);
        }
    }
    Task::none()
}

pub fn handle_tick(app: &mut KglanceApp) -> Task<Message> {
    if let Some(audio) = &mut app.audio {
        if audio.poll_eos() {
            app.state.audio.playing = false;
            app.state.audio.progress = 1.0;
            let total_dur = app.state.audio.duration_secs;
            app.state.audio.position_secs = total_dur;
            app.state.audio.time = format_time_label(total_dur, total_dur);
            return Task::none();
        }

        app.state.audio.playing = audio.is_playing();
        if let Some(position) = audio.query_position_secs() {
            let duration = audio.duration_secs();
            app.state.audio.position_secs = position;
            let total_dur = if duration > 0.0 {
                duration
            } else {
                app.state.audio.duration_secs
            };
            if total_dur > 0.0 {
                app.state.audio.duration_secs = total_dur;
                app.state.audio.progress = (position / total_dur).clamp(0.0, 1.0) as f32;
                app.state.audio.time = format_time_label(position, total_dur);
            }
        }
    }
    Task::none()
}

pub fn handle_end_of_stream(app: &mut KglanceApp) -> Task<Message> {
    app.state.audio.playing = false;
    Task::none()
}

pub(crate) fn format_time_label(pos: f64, dur: f64) -> String {
    let pos_str = format_seconds(pos);
    let dur_str = format_seconds(dur);
    format!("{pos_str} / {dur_str}")
}

pub(crate) fn format_seconds(secs: f64) -> String {
    let total_secs = secs.max(0.0) as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let rem_secs = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{mins:02}:{rem_secs:02}")
    } else {
        format!("{mins}:{rem_secs:02}")
    }
}
