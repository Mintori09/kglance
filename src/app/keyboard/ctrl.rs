use iced::Task;

use super::Message;
use crate::core::{PdfState, PreviewData};

const ZOOM_STEP: f32 = 0.2;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.0;
const FONT_MIN: f32 = 8.0;
const FONT_MAX: f32 = 48.0;

use crate::app::KglanceApp;
use crate::log_error;

impl KglanceApp {
    pub(super) fn handle_ctrl_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        if !modifiers.control() {
            return None;
        }
        let c = match key {
            iced::keyboard::Key::Character(c) => c.as_str(),
            _ => return None,
        };

        match c {
            "c" | "C" => self.handle_ctrl_copy(),
            "," => self.handle_toggle_settings(),
            "a" | "A" => self.handle_ctrl_a(),
            "t" => self.handle_toggle_theme(),
            "+" | "=" => self.handle_zoom_or_font(1.0),
            "-" => self.handle_zoom_or_font(-1.0),
            "0" => self.handle_image_reset(),
            "f" | "F" => self.handle_ctrl_f(),
            "e" | "E" | "i" | "I" | "P" => self.handle_json_shortcut(c),
            "w" | "W" => self.handle_toggle_word_wrap(),
            _ => None,
        }
    }

    fn handle_toggle_word_wrap(&mut self) -> Option<Task<Message>> {
        if !matches!(
            self.current_content,
            Some(PreviewData::Text { .. } | PreviewData::Json { .. } | PreviewData::Typst { .. })
        ) {
            return None;
        }

        self.state.word_wrap = !self.state.word_wrap;

        let mut config = crate::core::config::ConfigManager::load_or_create();
        config.ui.word_wrap = self.state.word_wrap;

        if let Err(err) = crate::core::config::ConfigManager::save(&config) {
            log_error!("failed to save word-wrap preference: {}", err);
        }

        Some(Task::none())
    }

    fn handle_json_shortcut(&mut self, key: &str) -> Option<Task<Message>> {
        if !matches!(self.current_content, Some(PreviewData::Json { .. })) {
            return None;
        }

        let message = match key {
            "e" => crate::app::messages::JsonMsg::ExpandAll,
            "E" => crate::app::messages::JsonMsg::CollapseAll,
            "P" => crate::app::messages::JsonMsg::ToggleFormat,
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
        self.state.app_theme = match self.state.app_theme {
            crate::ui::theme::AppTheme::Dark => crate::ui::theme::AppTheme::Light,
            crate::ui::theme::AppTheme::Light => crate::ui::theme::AppTheme::Nord,
            crate::ui::theme::AppTheme::Nord => crate::ui::theme::AppTheme::Dark,
        };

        Some(Task::none())
    }

    fn handle_image_reset(&mut self) -> Option<Task<Message>> {
        if !matches!(self.current_content, Some(PreviewData::Image { .. })) {
            return None;
        }

        self.state.image.camera.zoom = 1.0;
        self.state.image.camera.offset_x = 0.0;
        self.state.image.camera.offset_y = 0.0;

        Some(Task::none())
    }

    fn handle_ctrl_a(&mut self) -> Option<Task<Message>> {
        if matches!(self.current_content, Some(PreviewData::Text { .. })) {
            use iced::widget::text_editor::Action;
            self.state.text.content.perform(Action::SelectAll);
            Some(Task::none())
        } else if matches!(
            self.current_content,
            Some(PreviewData::Markdown { .. }) | Some(PreviewData::Epub { .. })
        ) {
            Some(crate::features::markdown::update::handle_select_all(self))
        } else {
            None
        }
    }

    fn handle_ctrl_copy(&mut self) -> Option<Task<Message>> {
        let text = match self.current_content {
            Some(PreviewData::Text { .. }) => self.state.text.content.selection(),

            Some(PreviewData::Json { .. }) if !self.state.json.tree_mode => {
                self.state.json.raw_editor.selection()
            }

            Some(PreviewData::Json { .. }) if self.state.json.tree_mode => {
                self.state.json.active_node.and_then(|idx| {
                    crate::features::json::parser::JsonParser::extract_subtree_json(
                        &self.state.json.nodes,
                        idx,
                    )
                })
            }

            Some(PreviewData::Markdown { .. } | PreviewData::Epub { .. }) => {
                let previous = crate::features::markdown::update::active_markdown_state(self)
                    .selected_text
                    .clone();

                crate::features::markdown::update::update_selected_text_from_range(self);

                crate::features::markdown::update::active_markdown_state(self)
                    .selected_text
                    .clone()
                    .or(previous)
            }

            _ => None,
        }?;

        if text.is_empty() {
            return None;
        }

        let toast = self.show_toast("Copied selected!");

        Some(Task::batch(vec![iced::clipboard::write(text), toast]))
    }

    fn resize_pdf_preview(
        pdf: &mut PdfState,
        window_width: f32,
        direction: f32,
    ) -> Option<Task<Message>> {
        let old_width = pdf.desired_width;
        let mut new_width = (old_width + direction * 50.0).clamp(300.0, 2400.0);

        if new_width == old_width {
            return Some(Task::none());
        }

        let sidebar_width = if pdf.sidebar_visible {
            pdf.sidebar_width + 1.0
        } else {
            0.0
        };

        let max_width = (window_width - sidebar_width - 40.0).clamp(300.0, 2400.0);

        if new_width > max_width {
            new_width = max_width;
        }

        let scroll_y = crate::features::pdf::view::rescale_pdf_and_anchor(
            pdf, old_width, new_width, max_width,
        );

        Some(iced::widget::operation::scroll_to(
            "content_scroll",
            iced::widget::operation::AbsoluteOffset {
                x: 0.0,
                y: scroll_y,
            },
        ))
    }

    fn handle_zoom_or_font(&mut self, direction: f32) -> Option<Task<Message>> {
        match self.current_content {
            Some(PreviewData::Image { .. }) => {
                self.state.image.camera.zoom = (self.state.image.camera.zoom
                    + direction * ZOOM_STEP)
                    .clamp(ZOOM_MIN, ZOOM_MAX);

                Some(Task::none())
            }

            Some(PreviewData::Pdf { .. }) => Self::resize_pdf_preview(
                &mut self.state.pdf,
                self.state.current_window_size.width,
                direction,
            ),

            Some(PreviewData::Typst { .. }) => Self::resize_pdf_preview(
                &mut self.state.typst.pdf,
                self.state.current_window_size.width,
                direction,
            ),

            Some(
                PreviewData::Markdown { .. }
                | PreviewData::Text { .. }
                | PreviewData::Epub { .. }
                | PreviewData::Json { .. },
            ) => {
                let old_size = self.state.font_size;
                let new_size = (old_size + direction).clamp(FONT_MIN, FONT_MAX);

                if new_size != old_size {
                    self.state.font_size = new_size;

                    let scroll_task = match self.current_content {
                        Some(PreviewData::Markdown { ref blocks, .. }) => {
                            let content_width = self.state.markdown_content_width();
                            let new_scroll_y =
                                crate::features::markdown::rescale_and_update_markdown_layout(
                                    &mut self.state.markdown,
                                    blocks,
                                    old_size,
                                    new_size,
                                    content_width,
                                );
                            Some(iced::widget::operation::scroll_to(
                                "content_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: new_scroll_y,
                                },
                            ))
                        }
                        Some(PreviewData::Epub { ref chapters, .. }) => {
                            let active_chapter = self.state.epub.active_chapter;
                            chapters.get(active_chapter).map(|chapter| {
                                let epub_content_width = self.state.epub_content_width();
                                let new_scroll_y =
                                    crate::features::markdown::rescale_and_update_markdown_layout(
                                        &mut self.state.epub.markdown_state,
                                        &chapter.blocks,
                                        old_size,
                                        new_size,
                                        epub_content_width,
                                    );
                                iced::widget::operation::scroll_to(
                                    "content_scroll",
                                    iced::widget::operation::AbsoluteOffset {
                                        x: 0.0,
                                        y: new_scroll_y,
                                    },
                                )
                            })
                        }
                        Some(PreviewData::Text { .. }) => {
                            let old_lh = old_size * 1.35;
                            let new_lh = new_size * 1.35;
                            let line_index = (self.state.text.scroll_y / old_lh).max(0.0);
                            self.state.text.scroll_y = (line_index * new_lh).max(0.0);
                            Some(iced::widget::operation::scroll_to(
                                "content_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: self.state.text.scroll_y,
                                },
                            ))
                        }
                        Some(PreviewData::Json { .. }) => {
                            let new_scroll_y = if self.state.json.tree_mode {
                                let old_row_h = (old_size * 1.2).max(18.0) + 4.0;
                                let new_row_h = (new_size * 1.2).max(18.0) + 4.0;
                                let node_index = (self.state.json.scroll_y / old_row_h).max(0.0);
                                (node_index * new_row_h).max(0.0)
                            } else {
                                let old_lh = old_size * 1.35;
                                let new_lh = new_size * 1.35;
                                let line_index = (self.state.json.scroll_y / old_lh).max(0.0);
                                (line_index * new_lh).max(0.0)
                            };
                            self.state.json.scroll_y = new_scroll_y;
                            Some(iced::widget::operation::scroll_to(
                                "content_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: self.state.json.scroll_y,
                                },
                            ))
                        }
                        _ => None,
                    };

                    if let Some(task) = scroll_task {
                        return Some(task);
                    }
                }

                Some(Task::none())
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

    pub(super) fn handle_font_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        match key {
            iced::keyboard::Key::Character(c) if (c == "+" || c == "=") && modifiers.shift() => {
                match self.current_content {
                    Some(PreviewData::Pdf { .. }) => Some(Self::reset_pdf_width(
                        &mut self.state.pdf,
                        self.state.current_window_size.width,
                    )),

                    Some(PreviewData::Typst { .. }) => Some(Self::reset_pdf_width(
                        &mut self.state.typst.pdf,
                        self.state.current_window_size.width,
                    )),

                    Some(PreviewData::Markdown { ref blocks, .. }) => {
                        let old_size = self.state.font_size;
                        let new_size = self.state.default_font_size;
                        self.state.font_size = new_size;

                        let content_width = self.state.markdown_content_width();
                        let new_scroll_y =
                            crate::features::markdown::rescale_and_update_markdown_layout(
                                &mut self.state.markdown,
                                blocks,
                                old_size,
                                new_size,
                                content_width,
                            );
                        Some(iced::widget::operation::scroll_to(
                            "content_scroll",
                            iced::widget::operation::AbsoluteOffset {
                                x: 0.0,
                                y: new_scroll_y,
                            },
                        ))
                    }

                    Some(PreviewData::Epub { ref chapters, .. }) => {
                        let old_size = self.state.font_size;
                        let new_size = self.state.default_font_size;
                        self.state.font_size = new_size;
                        let active_chapter = self.state.epub.active_chapter;
                        if let Some(chapter) = chapters.get(active_chapter) {
                            let epub_content_width = self.state.epub_content_width();
                            let new_scroll_y =
                                crate::features::markdown::rescale_and_update_markdown_layout(
                                    &mut self.state.epub.markdown_state,
                                    &chapter.blocks,
                                    old_size,
                                    new_size,
                                    epub_content_width,
                                );
                            Some(iced::widget::operation::scroll_to(
                                "content_scroll",
                                iced::widget::operation::AbsoluteOffset {
                                    x: 0.0,
                                    y: new_scroll_y,
                                },
                            ))
                        } else {
                            Some(Task::none())
                        }
                    }

                    Some(PreviewData::Text { .. }) => {
                        self.state.font_size = self.state.default_font_size;
                        Some(Task::none())
                    }

                    _ => Some(Task::none()),
                }
            }

            _ => None,
        }
    }

    fn reset_pdf_width(pdf: &mut PdfState, window_width: f32) -> Task<Message> {
        let old_width = pdf.desired_width;
        let new_width = 800.0;

        let sidebar_width = if pdf.sidebar_visible {
            pdf.sidebar_width + 1.0
        } else {
            0.0
        };

        let max_width = (window_width - sidebar_width - 40.0).clamp(300.0, 2400.0);

        let scroll_y = crate::features::pdf::view::rescale_pdf_and_anchor(
            pdf, old_width, new_width, max_width,
        );

        iced::widget::operation::scroll_to(
            "content_scroll",
            iced::widget::operation::AbsoluteOffset {
                x: 0.0,
                y: scroll_y,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::app::test_util::{epub_content, test_app};

    #[test]
    fn ctrl_a_selects_all_for_epub() {
        let mut app = test_app(Some(epub_content(&["a", "b"])));
        let task = app.handle_ctrl_a();
        assert!(task.is_some());
        assert!(app.state.epub.markdown_state.selection_range.is_some());
    }

    #[test]
    fn ctrl_c_copies_epub_selection() {
        let mut app = test_app(Some(epub_content(&["hello"])));
        app.state.epub.markdown_state.selected_text = Some("hello".to_string());
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let _guard = runtime.enter();
        assert!(app.handle_ctrl_copy().is_some());
    }

    #[test]
    fn ctrl_c_copies_markdown_selection_with_inline_math() {
        use crate::app::test_util::markdown_content;
        use crate::core::{SelectionPoint, SelectionRange};

        let src = "- $\\Rightarrow$ **cấu trúc thuật toán ANN, nhu cầu can thiệp của con người thấp hơn và yêu cầu dữ liệu lớn hơn.**";
        let mut app = test_app(Some(markdown_content(src)));

        // Simulating selection on the markdown state
        app.state.markdown.selection_range = Some(SelectionRange {
            start: SelectionPoint {
                block: 1,
                offset: 0,
            },
            end: SelectionPoint {
                block: 1,
                offset: 500,
            },
        });

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let _guard = runtime.enter();

        let task = app.handle_ctrl_copy();
        assert!(
            task.is_some(),
            "handle_ctrl_copy should return Task with clipboard write"
        );
        assert!(
            app.state
                .markdown
                .selected_text
                .as_ref()
                .unwrap()
                .contains("cấu trúc thuật toán ANN")
        );
    }

    #[test]
    fn ctrl_c_preserves_existing_selected_text_fallback() {
        use crate::app::test_util::markdown_content;

        let mut app = test_app(Some(markdown_content("- item")));
        app.state.markdown.selected_text = Some("thuật toán ANN".to_string());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let _guard = runtime.enter();

        let task = app.handle_ctrl_copy();
        assert!(task.is_some());
        assert_eq!(
            app.state.markdown.selected_text.as_deref(),
            Some("thuật toán ANN")
        );
    }

    #[test]
    fn ctrl_c_returns_none_without_selection() {
        let mut app = test_app(Some(epub_content(&["hello"])));
        assert!(app.handle_ctrl_copy().is_none());
    }

    #[test]
    fn ctrl_w_toggles_word_wrap() {
        let mut app = test_app(Some(crate::core::PreviewData::Text {
            content: "hello".to_string(),
            line_numbers: "1".to_string(),
            language: "plaintext".to_string(),
        }));
        let initial_wrap = app.state.word_wrap;
        let key = iced::keyboard::Key::Character("w".into());
        let modifiers = iced::keyboard::Modifiers::CTRL;

        let task = app.handle_ctrl_shortcuts(&key, modifiers);
        assert!(task.is_some());
        assert_eq!(app.state.word_wrap, !initial_wrap);
    }

    #[test]
    fn ctrl_plus_minus_resizes_json_font_size() {
        let mut app = test_app(Some(crate::core::PreviewData::Json {
            nodes: vec![],
            content: "{}".to_string(),
            pretty: "{}".to_string(),
            has_parse_error: false,
        }));
        let initial_size = app.state.font_size;
        let plus_key = iced::keyboard::Key::Character("+".into());
        let minus_key = iced::keyboard::Key::Character("-".into());
        let modifiers = iced::keyboard::Modifiers::CTRL;

        let task = app.handle_ctrl_shortcuts(&plus_key, modifiers);
        assert!(task.is_some());
        assert_eq!(app.state.font_size, initial_size + 1.0);

        let task = app.handle_ctrl_shortcuts(&minus_key, modifiers);
        assert!(task.is_some());
        assert_eq!(app.state.font_size, initial_size);
    }

    #[test]
    fn ctrl_plus_preserves_scroll_position_for_text_and_json() {
        let mut app = test_app(Some(crate::core::PreviewData::Text {
            content: "line 1\nline 2\nline 3".to_string(),
            line_numbers: "1\n2\n3".to_string(),
            language: "plaintext".to_string(),
        }));
        app.state.font_size = 14.0;
        app.state.text.scroll_y = 140.0;

        let plus_key = iced::keyboard::Key::Character("+".into());
        let modifiers = iced::keyboard::Modifiers::CTRL;

        let task = app.handle_ctrl_shortcuts(&plus_key, modifiers);
        assert!(task.is_some());
        assert_eq!(app.state.font_size, 15.0);
        assert_eq!(app.state.text.scroll_y, 140.0 * (15.0 / 14.0));

        let mut json_app = test_app(Some(crate::core::PreviewData::Json {
            nodes: vec![],
            content: "{}".to_string(),
            pretty: "{}".to_string(),
            has_parse_error: false,
        }));
        json_app.state.font_size = 14.0;
        json_app.state.json.scroll_y = 280.0;

        let task = json_app.handle_ctrl_shortcuts(&plus_key, modifiers);
        assert!(task.is_some());
        assert_eq!(json_app.state.font_size, 15.0);
        assert_eq!(json_app.state.json.scroll_y, 280.0 * (15.0 / 14.0));

        // Test tree mode node height preservation
        let mut json_tree_app = test_app(Some(crate::core::PreviewData::Json {
            nodes: vec![],
            content: "{}".to_string(),
            pretty: "{}".to_string(),
            has_parse_error: false,
        }));
        json_tree_app.state.font_size = 14.0;
        json_tree_app.state.json.tree_mode = true;
        json_tree_app.state.json.scroll_y = 220.0;

        let task = json_tree_app.handle_ctrl_shortcuts(&plus_key, modifiers);
        assert!(task.is_some());
        assert_eq!(json_tree_app.state.font_size, 15.0);
        let old_row_h = (14.0f32 * 1.2).max(18.0) + 4.0;
        let new_row_h = (15.0f32 * 1.2).max(18.0) + 4.0;
        assert_eq!(
            json_tree_app.state.json.scroll_y,
            (220.0 / old_row_h) * new_row_h
        );
    }
}
