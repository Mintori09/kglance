use std::sync::atomic::Ordering;

use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset};

use super::Message;
use crate::core::PreviewData;
use crate::ui::handlers::video::VideoEvent;

impl super::KglanceApp {
    pub fn handle_scroll_delta(&mut self, _x: f32, y: f32) -> Task<Message> {
        if self.ctrl_held.load(Ordering::Relaxed) {
            if matches!(self.current_content, Some(PreviewData::Image { .. })) {
                let factor = if y > 0.0 { 0.1 } else { -0.1 };
                self.state.image.zoom = (self.state.image.zoom + factor).clamp(0.1, 10.0);
                Task::none()
            } else if matches!(
                self.current_content,
                Some(PreviewData::Markdown { .. }) | Some(PreviewData::Text { .. })
            ) {
                let delta = if y > 0.0 { 1.0 } else { -1.0 };
                let old = self.state.font_size;
                let new = (old + delta).clamp(8.0, 48.0);
                let ratio = new / old;
                self.state.font_size = new;
                let target = self.state.scroll_offset * ratio;
                self.state.pending_font_target = Some(target);
                operation::scroll_to("content_scroll", AbsoluteOffset { x: 0.0, y: target })
            } else {
                Task::none()
            }
        } else if self.shift_held.load(Ordering::Relaxed) {
            operation::scroll_by("content_scroll", AbsoluteOffset { x: -y, y: 0.0 })
        } else {
            operation::scroll_by("content_scroll", AbsoluteOffset { x: 0.0, y: -y })
        }
    }

    pub fn handle_image_zoom(&mut self, delta: f32) -> Task<Message> {
        self.state.image.zoom = (self.state.image.zoom + delta).clamp(0.1, 10.0);
        Task::none()
    }

    pub fn handle_play_pause(&mut self) -> Task<Message> {
        if let Some(tx) = &self.video_tx {
            if self.state.media.playing {
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Pause);
            } else {
                let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Play);
            }
        }
        Task::none()
    }

    pub fn handle_seek(&self, percent: f32) -> Task<Message> {
        if let Some(tx) = &self.video_tx {
            let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::Seek(
                percent as f64,
            ));
        }
        Task::none()
    }

    pub fn handle_seek_relative(&self, secs: f32) -> Task<Message> {
        if let Some(tx) = &self.video_tx {
            let _ = tx.try_send(crate::ui::handlers::video::PlayerCommand::SeekRelative(
                secs as f64,
            ));
        }
        Task::none()
    }

    pub fn handle_video_event(&mut self, event: VideoEvent) -> Task<Message> {
        match event {
            VideoEvent::Frame {
                data,
                width,
                height,
            } => {
                let first_frame = self.state.media.frame_data.is_empty();
                self.state.media.frame_data = data;
                self.state.media.frame_width = width;
                self.state.media.frame_height = height;
                if first_frame && let Some(id) = self.window_id {
                    let w = (width as f32 * 1.1) as u32;
                    let h = (height as f32 * 1.1 + 50.0) as u32;
                    let max_w = 1600u32;
                    let max_h = 1000u32;
                    let cw = w.min(max_w).max(400);
                    let ch = h.min(max_h).max(300);
                    return iced::window::resize(id, iced::Size::new(cw as f32, ch as f32));
                }
            }
            VideoEvent::Progress {
                position,
                duration,
                is_playing,
            } => {
                self.state.media.playing = is_playing;
                self.state.media.position_secs = position;
                self.state.media.duration_secs = duration;
                if duration > 0.0 {
                    self.state.media.progress = (position / duration) as f32;
                    let cur_mins = (position / 60.0) as u32;
                    let cur_secs = (position % 60.0) as u32;
                    let dur_mins = (duration / 60.0) as u32;
                    let dur_secs = (duration % 60.0) as u32;
                    self.state.media.time =
                        format!("{cur_mins}:{cur_secs:02} / {dur_mins}:{dur_secs:02}");
                }
            }
        }
        Task::none()
    }

    pub fn handle_media_mouse_enter(&mut self) -> Task<Message> {
        self.state.media.show_controls = true;
        Task::none()
    }

    pub fn handle_media_mouse_leave(&mut self) -> Task<Message> {
        self.state.media.show_controls = false;
        Task::none()
    }
}
