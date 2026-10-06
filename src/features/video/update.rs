use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_play_pause(app: &mut KglanceApp) -> Task<Message> {
    if let Some(video) = &mut app.video {
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
            let cur_mins = (position / 60.0) as u32;
            let cur_secs = (position % 60.0) as u32;
            let dur_mins = (duration / 60.0) as u32;
            let dur_secs = (duration % 60.0) as u32;
            app.state.media.time = format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}");
        }
    }
    Task::none()
}

pub fn handle_video_end_of_stream(app: &mut KglanceApp) -> Task<Message> {
    app.state.media.playing = false;
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
