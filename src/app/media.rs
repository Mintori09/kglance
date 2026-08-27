use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset};

use super::Message;
use crate::core::{PreviewData, ViewMode};

impl super::KglanceApp {
    pub fn handle_scroll_delta(&mut self, x: f32, y: f32) -> Task<Message> {
        let is_mod = self.ctrl_held;
        let scroll_val = if y.abs() > f32::EPSILON {
            y
        } else if x.abs() > f32::EPSILON {
            x
        } else {
            return Task::none();
        };

        if is_mod && matches!(self.state.view_mode, ViewMode::Grid(_)) {
            let factor = if scroll_val > 0.0 { 0.1 } else { -0.1 };
            self.state.grid_scale = (self.state.grid_scale + factor).clamp(0.3, 3.0);
            self.recalc_grid_cols();
            Task::none()
        } else if is_mod && matches!(self.current_content, Some(PreviewData::Image { .. })) {
            let factor = if scroll_val > 0.0 { 1.15 } else { 1.0 / 1.15 };
            self.state.image.camera.zoom = (self.state.image.camera.zoom * factor).clamp(0.1, 10.0);
            Task::none()
        } else if is_mod && matches!(self.current_content, Some(PreviewData::Pdf { .. })) {
            let delta = if scroll_val > 0.0 { 50.0 } else { -50.0 };
            let old_desired = self.state.pdf.desired_width;
            let next_desired = (old_desired + delta).clamp(300.0, 2400.0);
            if (next_desired - old_desired).abs() > f32::EPSILON {
                let win_w = self.state.current_window_size.width;
                let sidebar_w = if self.state.pdf.sidebar_visible {
                    self.state.pdf.sidebar_width + 1.0
                } else {
                    0.0
                };
                let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
                let new_scroll_y = crate::features::pdf::view::rescale_pdf_and_anchor(
                    &mut self.state.pdf,
                    old_desired,
                    next_desired,
                    max_w,
                );
                operation::scroll_to(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: new_scroll_y,
                    },
                )
            } else {
                operation::scroll_to(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: self.state.pdf.scroll_y,
                    },
                )
            }
        } else if is_mod && matches!(self.current_content, Some(PreviewData::Typst { .. })) {
            let delta = if scroll_val > 0.0 { 50.0 } else { -50.0 };
            let old_desired = self.state.typst.pdf.desired_width;
            let next_desired = (old_desired + delta).clamp(300.0, 2400.0);
            if (next_desired - old_desired).abs() > f32::EPSILON {
                let win_w = self.state.current_window_size.width;
                let sidebar_w = if self.state.typst.pdf.sidebar_visible {
                    self.state.typst.pdf.sidebar_width + 1.0
                } else {
                    0.0
                };
                let max_w = (win_w - sidebar_w - 40.0).clamp(300.0, 2400.0);
                let new_scroll_y = crate::features::pdf::view::rescale_pdf_and_anchor(
                    &mut self.state.typst.pdf,
                    old_desired,
                    next_desired,
                    max_w,
                );
                operation::scroll_to(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: new_scroll_y,
                    },
                )
            } else {
                operation::scroll_to(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: self.state.typst.pdf.scroll_y,
                    },
                )
            }
        } else if is_mod {
            if matches!(
                self.current_content,
                Some(PreviewData::Markdown { .. })
                    | Some(PreviewData::Text { .. })
                    | Some(PreviewData::Epub { .. })
                    | Some(PreviewData::Json { .. })
            ) {
                let delta = if scroll_val > 0.0 { 1.0 } else { -1.0 };
                let old_font_size = self.state.font_size;
                let next_font_size = (old_font_size + delta).clamp(8.0, 48.0);
                if (next_font_size - old_font_size).abs() > f32::EPSILON {
                    self.state.font_size = next_font_size;
                    if let Some(PreviewData::Markdown { ref blocks, .. }) = self.current_content {
                        self.state.markdown.toc = crate::parsers::markdown::extract_toc(
                            blocks,
                            self.state.font_size,
                            &self.state.markdown.cached_image_sizes,
                        );
                        let (offsets, total_h) = crate::features::markdown::compute_block_y_offsets(
                            blocks,
                            self.state.font_size,
                            &self.state.markdown.cached_image_sizes,
                        );
                        self.state.markdown.block_y_offsets = offsets;
                        self.state.markdown.total_content_height = total_h;
                    }

                    if let Some(PreviewData::Epub { ref chapters, .. }) = self.current_content {
                        let active_chapter = self.state.epub.active_chapter;
                        if let Some(chapter) = chapters.get(active_chapter) {
                            self.state.epub.markdown_state.toc =
                                crate::parsers::markdown::extract_toc(
                                    &chapter.blocks,
                                    self.state.font_size,
                                    &self.state.markdown.cached_image_sizes,
                                );
                            let (offsets, total_h) =
                                crate::features::markdown::compute_block_y_offsets(
                                    &chapter.blocks,
                                    self.state.font_size,
                                    &self.state.markdown.cached_image_sizes,
                                );
                            self.state.epub.markdown_state.block_y_offsets = offsets;
                            self.state.epub.markdown_state.total_content_height = total_h;
                        }
                    }

                    match self.current_content {
                        Some(PreviewData::Markdown { ref blocks, .. }) => {
                            let new_scroll_y = crate::parsers::markdown::rescale_markdown_scroll_y(
                                blocks,
                                self.state.markdown.scroll_y,
                                old_font_size,
                                next_font_size,
                                &self.state.markdown.cached_image_sizes,
                            );
                            self.state.markdown.scroll_y = new_scroll_y;
                            operation::scroll_to(
                                "content_scroll",
                                AbsoluteOffset {
                                    x: 0.0,
                                    y: new_scroll_y,
                                },
                            )
                        }
                        Some(PreviewData::Epub { ref chapters, .. }) => {
                            let active_chapter = self.state.epub.active_chapter;
                            if let Some(chapter) = chapters.get(active_chapter) {
                                let new_scroll_y =
                                    crate::parsers::markdown::rescale_markdown_scroll_y(
                                        &chapter.blocks,
                                        self.state.epub.markdown_state.scroll_y,
                                        old_font_size,
                                        next_font_size,
                                        &self.state.markdown.cached_image_sizes,
                                    );
                                self.state.epub.markdown_state.scroll_y = new_scroll_y;
                                operation::scroll_to(
                                    "content_scroll",
                                    AbsoluteOffset {
                                        x: 0.0,
                                        y: new_scroll_y,
                                    },
                                )
                            } else {
                                Task::none()
                            }
                        }
                        Some(PreviewData::Text { .. }) => {
                            let old_lh = old_font_size * 1.35;
                            let new_lh = next_font_size * 1.35;
                            let line_index = (self.state.text.scroll_y / old_lh).max(0.0);
                            self.state.text.scroll_y = (line_index * new_lh).max(0.0);
                            operation::scroll_to(
                                "content_scroll",
                                AbsoluteOffset {
                                    x: 0.0,
                                    y: self.state.text.scroll_y,
                                },
                            )
                        }
                        Some(PreviewData::Json { .. }) => {
                            let new_scroll_y = if self.state.json.tree_mode {
                                let old_row_h = (old_font_size * 1.2).max(18.0) + 4.0;
                                let new_row_h = (next_font_size * 1.2).max(18.0) + 4.0;
                                let node_index = (self.state.json.scroll_y / old_row_h).max(0.0);
                                (node_index * new_row_h).max(0.0)
                            } else {
                                let old_lh = old_font_size * 1.35;
                                let new_lh = next_font_size * 1.35;
                                let line_index = (self.state.json.scroll_y / old_lh).max(0.0);
                                (line_index * new_lh).max(0.0)
                            };
                            self.state.json.scroll_y = new_scroll_y;
                            operation::scroll_to(
                                "content_scroll",
                                AbsoluteOffset {
                                    x: 0.0,
                                    y: self.state.json.scroll_y,
                                },
                            )
                        }
                        _ => Task::none(),
                    }
                } else {
                    match self.current_content {
                        Some(PreviewData::Markdown { .. }) => operation::scroll_to(
                            "content_scroll",
                            AbsoluteOffset {
                                x: 0.0,
                                y: self.state.markdown.scroll_y,
                            },
                        ),
                        Some(PreviewData::Epub { .. }) => operation::scroll_to(
                            "content_scroll",
                            AbsoluteOffset {
                                x: 0.0,
                                y: self.state.epub.markdown_state.scroll_y,
                            },
                        ),
                        Some(PreviewData::Text { .. }) => operation::scroll_to(
                            "content_scroll",
                            AbsoluteOffset {
                                x: 0.0,
                                y: self.state.text.scroll_y,
                            },
                        ),
                        Some(PreviewData::Json { .. }) => operation::scroll_to(
                            "content_scroll",
                            AbsoluteOffset {
                                x: 0.0,
                                y: self.state.json.scroll_y,
                            },
                        ),
                        _ => Task::none(),
                    }
                }
            } else {
                operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: -scroll_val,
                        y: 0.0,
                    },
                )
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
        if let Some(video) = &mut self.video {
            crate::features::video::handler::toggle_play_pause(video);
            self.state.media.playing = !video.paused();
        }
        Task::none()
    }

    pub fn handle_seek(&mut self, percent: f32) -> Task<Message> {
        if let Some(video) = &mut self.video {
            crate::features::video::handler::seek_to_ratio(video, percent as f64);
        }
        Task::none()
    }

    pub fn handle_seek_relative(&mut self, secs: f32) -> Task<Message> {
        if let Some(video) = &mut self.video {
            crate::features::video::handler::seek_relative(video, secs as f64);
        }
        Task::none()
    }

    pub fn handle_video_new_frame(&mut self) -> Task<Message> {
        if let Some(video) = &self.video {
            let position = video.position().as_secs_f64();
            let duration = video.duration().as_secs_f64();
            self.state.media.playing = !video.paused();
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
        Task::none()
    }

    pub fn handle_video_end_of_stream(&mut self) -> Task<Message> {
        self.state.media.playing = false;
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
