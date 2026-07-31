use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset};

use super::Message;
use crate::core::{PreviewData, ViewMode};

use crate::ui::handlers::video::VideoEvent;

impl super::KglanceApp {
    pub fn handle_scroll_delta(&mut self, _x: f32, y: f32) -> Task<Message> {
        if self.shift_held && matches!(self.state.view_mode, ViewMode::Grid(_)) {
            let factor = if y > 0.0 { 0.1 } else { -0.1 };
            self.state.grid_scale = (self.state.grid_scale + factor).clamp(0.3, 3.0);
            self.recalc_grid_cols();
            Task::none()
        } else if self.shift_held && matches!(self.current_content, Some(PreviewData::Image { .. }))
        {
            let factor = if y > 0.0 { 0.1 } else { -0.1 };
            self.state.image.camera.zoom = (self.state.image.camera.zoom + factor).clamp(0.1, 10.0);
            Task::none()
        } else if self.shift_held {
            if matches!(
                self.current_content,
                Some(PreviewData::Markdown { .. })
                    | Some(PreviewData::Text { .. })
                    | Some(PreviewData::Epub { .. })
            ) {
                let delta = if y > 0.0 { 1.0 } else { -1.0 };
                self.state.font_size = (self.state.font_size + delta).clamp(8.0, 48.0);
                if let Some(PreviewData::Markdown { ref blocks }) = self.current_content {
                    self.state.markdown.toc = crate::parsers::markdown::extract_toc(
                        blocks,
                        self.state.font_size,
                        &self.state.markdown.cached_image_sizes,
                    );
                }
                Task::none()
            } else {
                operation::scroll_by("content_scroll", AbsoluteOffset { x: -y, y: 0.0 })
            }
        } else {
            Task::none()
        }
    }

    pub fn handle_image_zoom(&mut self, delta: f32) -> Task<Message> {
        self.state.image.camera.zoom = (self.state.image.camera.zoom + delta).clamp(0.1, 10.0);
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
