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
    if let Some(audio) = &app.audio {
        audio.seek_to_ratio(pct as f64);
        let clamped = pct.clamp(0.0, 1.0);
        app.state.audio.progress = clamped;
        let total_dur = app.state.audio.duration_secs;
        let pos = clamped as f64 * total_dur;
        app.state.audio.position_secs = pos;
        if total_dur > 0.0 {
            let cur_hours = (pos / 3600.0) as u64;
            let cur_mins = ((pos % 3600.0) / 60.0) as u64;
            let cur_secs = (pos % 60.0) as u64;
            let dur_hours = (total_dur / 3600.0) as u64;
            let dur_mins = ((total_dur % 3600.0) / 60.0) as u64;
            let dur_secs = (total_dur % 60.0) as u64;
            app.state.audio.time = if dur_hours > 0 {
                format!(
                    "{cur_hours}:{cur_mins:02}:{cur_secs:02} / {dur_hours}:{dur_mins:02}:{dur_secs:02}"
                )
            } else {
                format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}")
            };
        }
    }
    Task::none()
}

pub fn handle_seek_relative(app: &mut KglanceApp, secs: f32) -> Task<Message> {
    if let Some(audio) = &app.audio {
        audio.seek_relative(secs as f64);
        let pos = audio.position_secs();
        let total_dur = app.state.audio.duration_secs;
        app.state.audio.position_secs = pos;
        if total_dur > 0.0 {
            app.state.audio.progress = (pos / total_dur).clamp(0.0, 1.0) as f32;
            let cur_hours = (pos / 3600.0) as u64;
            let cur_mins = ((pos % 3600.0) / 60.0) as u64;
            let cur_secs = (pos % 60.0) as u64;
            let dur_hours = (total_dur / 3600.0) as u64;
            let dur_mins = ((total_dur % 3600.0) / 60.0) as u64;
            let dur_secs = (total_dur % 60.0) as u64;
            app.state.audio.time = if dur_hours > 0 {
                format!(
                    "{cur_hours}:{cur_mins:02}:{cur_secs:02} / {dur_hours}:{dur_mins:02}:{dur_secs:02}"
                )
            } else {
                format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}")
            };
        }
    }
    Task::none()
}

pub fn handle_tick(app: &mut KglanceApp) -> Task<Message> {
    if let Some(audio) = &mut app.audio {
        if audio.poll_eos() {
            app.state.audio.playing = false;
            app.state.audio.progress = 1.0;
            return Task::none();
        }
        let position = audio.position_secs();
        let duration = audio.duration_secs();
        app.state.audio.playing = audio.is_playing();
        app.state.audio.position_secs = position;
        let total_dur = if duration > 0.0 {
            duration
        } else {
            app.state.audio.duration_secs
        };
        if total_dur > 0.0 {
            app.state.audio.duration_secs = total_dur;
            app.state.audio.progress = (position / total_dur).clamp(0.0, 1.0) as f32;
            let cur_hours = (position / 3600.0) as u64;
            let cur_mins = ((position % 3600.0) / 60.0) as u64;
            let cur_secs = (position % 60.0) as u64;
            let dur_hours = (total_dur / 3600.0) as u64;
            let dur_mins = ((total_dur % 3600.0) / 60.0) as u64;
            let dur_secs = (total_dur % 60.0) as u64;
            app.state.audio.time = if dur_hours > 0 {
                format!(
                    "{cur_hours}:{cur_mins:02}:{cur_secs:02} / {dur_hours}:{dur_mins:02}:{dur_secs:02}"
                )
            } else {
                format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}")
            };
        }
    }
    Task::none()
}

pub fn handle_end_of_stream(app: &mut KglanceApp) -> Task<Message> {
    app.state.audio.playing = false;
    Task::none()
}
