use iced::Task;

use super::Message;
use crate::app::KglanceApp;
use crate::core::PreviewData;

impl KglanceApp {
    pub(super) fn handle_ctrl_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        if !(modifiers.control() || modifiers.command() || self.ctrl_held) {
            return None;
        }
        let c = match key {
            iced::keyboard::Key::Character(c) => c.as_str(),
            _ => return None,
        };

        match c {
            "c" | "C" | "\u{3}" => self.handle_ctrl_copy(),
            "," => self.handle_toggle_settings(),
            "a" | "A" | "\u{1}" => self.handle_ctrl_a(),
            "t" | "T" | "\u{14}" => self.handle_toggle_theme(),
            "+" | "=" => self.handle_zoom_or_font(1.0),
            "-" => self.handle_zoom_or_font(-1.0),
            "0" => crate::features::image::update::handle_image_reset(self),
            "f" | "F" | "\u{6}" => self.handle_ctrl_f(),
            "g" | "G" | "\u{7}" => self.handle_ctrl_g(),
            "e" | "E" | "\u{5}" | "i" | "I" | "\t" | "P" | "p" | "\u{10}" => {
                self.handle_json_shortcut(c)
            }
            "w" | "W" | "\u{17}" => crate::features::text::update::handle_toggle_word_wrap(self),
            _ => None,
        }
    }

    fn handle_json_shortcut(&mut self, key: &str) -> Option<Task<Message>> {
        if !matches!(self.current_content, Some(PreviewData::Json { .. })) {
            return None;
        }

        let message = match key {
            "e" | "\u{5}" => crate::app::messages::JsonMsg::ExpandAll,
            "E" => crate::app::messages::JsonMsg::CollapseAll,
            "P" | "p" | "\u{10}" => crate::app::messages::JsonMsg::ToggleFormat,
            _ => return None,
        };

        Some(Task::done(message.into()))
    }

    fn handle_toggle_settings(&mut self) -> Option<Task<Message>> {
        Some(Task::done(
            crate::app::messages::NavigationMsg::ToggleSettingsClicked.into(),
        ))
    }

    fn handle_toggle_theme(&mut self) -> Option<Task<Message>> {
        Some(Task::done(
            crate::app::messages::SystemMsg::ThemeToggled.into(),
        ))
    }

    fn active_code_viewer_mut(&mut self) -> Option<&mut crate::core::TextState> {
        match self.current_content {
            Some(PreviewData::Text { .. }) => Some(&mut self.state.text),
            Some(PreviewData::Json { .. }) if !self.state.json.tree_mode => {
                Some(&mut self.state.json.raw_text)
            }
            Some(PreviewData::Typst { .. }) if self.state.typst.show_source => {
                Some(&mut self.state.typst.source_text)
            }
            _ => None,
        }
    }

    fn active_code_viewer(&self) -> Option<&crate::core::TextState> {
        match self.current_content {
            Some(PreviewData::Text { .. }) => Some(&self.state.text),
            Some(PreviewData::Json { .. }) if !self.state.json.tree_mode => {
                Some(&self.state.json.raw_text)
            }
            Some(PreviewData::Typst { .. }) if self.state.typst.show_source => {
                Some(&self.state.typst.source_text)
            }
            _ => None,
        }
    }

    fn handle_ctrl_a(&mut self) -> Option<Task<Message>> {
        if let Some(viewer) = self.active_code_viewer_mut() {
            viewer.select_all();
            return Some(Task::none());
        }
        match &self.current_content {
            Some(PreviewData::Markdown { .. } | PreviewData::Epub { .. }) => {
                Some(crate::features::markdown::update::handle_select_all(self))
            }
            Some(PreviewData::Pdf { .. }) => {
                crate::features::pdf::update::handle_select_all(&mut self.state.pdf);
                Some(Task::none())
            }
            Some(PreviewData::Typst { .. }) if !self.state.typst.show_source => {
                crate::features::pdf::update::handle_select_all(&mut self.state.pdf);
                Some(Task::none())
            }
            Some(PreviewData::Spreadsheet { .. }) => {
                let total_rows = self.state.spreadsheet.display_indices.len();
                let total_cols = self
                    .state
                    .spreadsheet
                    .sheets
                    .get(self.state.spreadsheet.active_sheet)
                    .map_or(0, |s| s.columns.len());
                if total_rows > 0 && total_cols > 0 {
                    self.state.spreadsheet.selection = Some(crate::features::sheet::CellRange {
                        start: crate::features::sheet::CellCoord { row: 0, col: 0 },
                        end: crate::features::sheet::CellCoord {
                            row: total_rows - 1,
                            col: total_cols - 1,
                        },
                    });
                }
                Some(Task::none())
            }
            _ => None,
        }
    }

    pub(super) fn handle_ctrl_copy(&mut self) -> Option<Task<Message>> {
        let (text, html) = if let Some(viewer) = self.active_code_viewer() {
            (viewer.selected_text(), None)
        } else {
            match self.current_content {
                Some(PreviewData::Spreadsheet { .. }) => {
                    (self.state.spreadsheet.selected_text(), None)
                }

                Some(PreviewData::Json { .. }) if self.state.json.tree_mode => {
                    let txt = self.state.json.active_node.and_then(|idx| {
                        crate::features::json::parser::JsonParser::extract_subtree_json(
                            &self.state.json.nodes,
                            idx,
                        )
                    });
                    (txt, None)
                }

                Some(PreviewData::Pdf { .. }) => (
                    self.state.pdf.selected_text.clone(),
                    self.state.pdf.selected_html.clone(),
                ),

                Some(PreviewData::Typst { .. }) if !self.state.typst.show_source => (
                    self.state.pdf.selected_text.clone(),
                    self.state.pdf.selected_html.clone(),
                ),

                Some(PreviewData::Markdown { .. } | PreviewData::Epub { .. }) => {
                    let previous = crate::features::markdown::update::active_markdown_state(self)
                        .selected_text
                        .clone();

                    crate::features::markdown::update::update_selected_text_from_range(self);

                    let txt = crate::features::markdown::update::active_markdown_state(self)
                        .selected_text
                        .clone()
                        .or(previous);
                    (txt, None)
                }

                _ => (None, None),
            }
        };

        let text = text?;
        if text.is_empty() {
            return None;
        }

        let toast = self.show_toast("Copied selected!");

        Some(Task::batch(vec![
            crate::core::clipboard::copy_to_clipboard(text, html),
            toast,
        ]))
    }

    fn handle_zoom_or_font(&mut self, direction: f32) -> Option<Task<Message>> {
        match self.current_content {
            Some(PreviewData::Image { .. }) => {
                crate::features::image::update::handle_image_zoom_step(self, direction)
            }

            Some(PreviewData::Pdf { .. }) => crate::features::pdf::update::resize_pdf_preview(
                &mut self.state.pdf,
                self.state.current_window_size.width,
                direction,
            ),

            Some(PreviewData::Typst { .. }) => {
                if self.state.typst.show_source {
                    let new_size = self.state.font_size + direction;
                    crate::features::text::update::rescale_typst_font(self, new_size)
                } else {
                    crate::features::pdf::update::resize_pdf_preview(
                        &mut self.state.pdf,
                        self.state.current_window_size.width,
                        direction,
                    )
                }
            }

            Some(PreviewData::Json { .. }) => {
                crate::features::json::update::zoom_json_font(self, direction)
            }

            Some(PreviewData::Text { .. }) => {
                crate::features::text::update::zoom_text_font(self, direction)
            }

            Some(PreviewData::Markdown { .. } | PreviewData::Epub { .. }) => {
                crate::features::markdown::update::zoom_markdown_font(self, direction)
            }

            _ => None,
        }
    }

    fn handle_ctrl_f(&mut self) -> Option<Task<Message>> {
        match self.current_content {
            Some(PreviewData::Json { .. }) => {
                self.state.json.search_visible = !self.state.json.search_visible;

                if self.state.json.search_visible {
                    Some(iced::widget::operation::focus("json_search_input"))
                } else {
                    self.state.json.search_query.clear();
                    Some(Task::none())
                }
            }

            Some(PreviewData::Text { .. }) => {
                self.state.text.search_visible = !self.state.text.search_visible;

                if self.state.text.search_visible {
                    self.state.text.goto_line_visible = false;
                    Some(iced::widget::operation::focus("txt_search_input"))
                } else {
                    self.state.text.search_query.clear();
                    Some(Task::none())
                }
            }

            Some(PreviewData::Markdown { .. }) => {
                self.state.markdown.search_visible = !self.state.markdown.search_visible;

                if self.state.markdown.search_visible {
                    Some(iced::widget::operation::focus("md_search_input"))
                } else {
                    self.state.markdown.search_query.clear();
                    self.state.markdown.search_match_count = 0;
                    self.state.markdown.search_match_index = 0;
                    self.state.markdown.search_match_blocks.clear();
                    self.state.markdown.search_info.clear();
                    Some(Task::none())
                }
            }

            Some(PreviewData::Spreadsheet { .. }) => {
                self.state.spreadsheet.search_visible = !self.state.spreadsheet.search_visible;

                if self.state.spreadsheet.search_visible {
                    Some(iced::widget::operation::focus("ss_search_input"))
                } else {
                    self.state.spreadsheet.search_query.clear();
                    Some(Task::none())
                }
            }

            None => match &self.state.view_mode {
                crate::core::ViewMode::Grid(_) => {
                    self.state.grid_search_visible = !self.state.grid_search_visible;

                    if self.state.grid_search_visible {
                        Some(iced::widget::operation::focus("grid_search_input"))
                    } else {
                        self.state.grid_search_query.clear();
                        Some(Task::none())
                    }
                }

                _ => None,
            },

            _ => None,
        }
    }

    fn handle_ctrl_g(&mut self) -> Option<Task<Message>> {
        match self.current_content {
            Some(PreviewData::Text { .. }) => {
                Some(crate::features::text::update::handle_goto_line_toggle(self))
            }
            _ => None,
        }
    }

    pub(super) fn handle_font_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        match key {
            iced::keyboard::Key::Character(c) if (c == "+" || c == "=") && modifiers.shift() => {
                let default_size = self.state.default_font_size;
                match self.current_content {
                    Some(PreviewData::Pdf { .. }) => {
                        Some(crate::features::pdf::update::reset_pdf_width(
                            &mut self.state.pdf,
                            self.state.current_window_size.width,
                        ))
                    }

                    Some(PreviewData::Text { .. }) => {
                        crate::features::text::update::rescale_text_font(self, default_size)
                    }

                    Some(PreviewData::Json { .. }) => {
                        crate::features::json::update::rescale_json_font(self, default_size)
                    }

                    Some(PreviewData::Markdown { .. } | PreviewData::Epub { .. }) => {
                        crate::features::markdown::update::rescale_markdown_font(self, default_size)
                    }

                    Some(PreviewData::Typst { .. }) => {
                        if self.state.typst.show_source {
                            crate::features::text::update::rescale_typst_font(self, default_size)
                        } else {
                            Some(crate::features::pdf::update::reset_pdf_width(
                                &mut self.state.pdf,
                                self.state.current_window_size.width,
                            ))
                        }
                    }

                    _ => Some(Task::none()),
                }
            }

            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "ctrl_tests.rs"]
mod tests;
