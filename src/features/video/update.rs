use crate::app::KglanceApp;
use crate::app::messages::Message;
use iced::Task;

pub fn handle_play_pause(app: &mut KglanceApp) -> Task<Message> {
    app.handle_play_pause()
}

pub fn handle_seek(app: &mut KglanceApp, pct: f32) -> Task<Message> {
    app.handle_seek(pct)
}

pub fn handle_seek_relative(app: &mut KglanceApp, secs: f32) -> Task<Message> {
    app.handle_seek_relative(secs)
}

pub fn handle_video_new_frame(app: &mut KglanceApp) -> Task<Message> {
    app.handle_video_new_frame()
}

pub fn handle_video_end_of_stream(app: &mut KglanceApp) -> Task<Message> {
    app.handle_video_end_of_stream()
}

pub fn handle_media_mouse_enter(app: &mut KglanceApp) -> Task<Message> {
    app.handle_media_mouse_enter()
}

pub fn handle_media_mouse_leave(app: &mut KglanceApp) -> Task<Message> {
    app.handle_media_mouse_leave()
}
