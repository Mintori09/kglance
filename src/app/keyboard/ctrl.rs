use iced::Task;

use super::Message;
use crate::core::PreviewData;

const ZOOM_STEP: f32 = 0.2;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.0;
const FONT_MIN: f32 = 8.0;
const FONT_MAX: f32 = 48.0;

use crate::app::KglanceApp;
use crate::features::markdown::parser::extract_toc;

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
            "," => Some(Task::done(
                crate::app::messages::NavigationMsg::ToggleSettingsClicked.into(),
            )),
            "a" | "A" => self.handle_ctrl_a(),
            "t" => {
                self.state.app_theme = match self.state.app_theme {
                    crate::ui::theme::AppTheme::Dark => crate::ui::theme::AppTheme::Light,
                    crate::ui::theme::AppTheme::Light => crate::ui::theme::AppTheme::Nord,
                    _ => crate::ui::theme::AppTheme::Dark,
                };
                Some(Task::none())
            }
            "+" | "=" => self.handle_zoom_or_font(1.0),
            "-" => self.handle_zoom_or_font(-1.0),
            "0" if matches!(self.current_content, Some(PreviewData::Image { .. })) => {
                self.state.image.camera.zoom = 1.0;
                self.state.image.camera.offset_x = 0.0;
                self.state.image.camera.offset_y = 0.0;
                Some(Task::none())
            }
            "f" | "F" => self.handle_ctrl_f(),
            "e" if matches!(self.current_content, Some(PreviewData::Json { .. })) => {
                Some(Task::done(crate::app::messages::JsonMsg::ExpandAll.into()))
            }
            "E" if matches!(self.current_content, Some(PreviewData::Json { .. })) => Some(
                Task::done(crate::app::messages::JsonMsg::CollapseAll.into()),
            ),
            "i" | "I" if matches!(self.current_content, Some(PreviewData::Json { .. })) => Some(
                Task::done(crate::app::messages::JsonMsg::SchemaToggle.into()),
            ),
            "P" if matches!(self.current_content, Some(PreviewData::Json { .. })) => Some(
                Task::done(crate::app::messages::JsonMsg::ToggleFormat.into()),
            ),
            "w" | "W"
                if matches!(
                    self.current_content,
                    Some(PreviewData::Text { .. })
                        | Some(PreviewData::Json { .. })
                        | Some(PreviewData::Typst { .. })
                ) =>
            {
                self.state.word_wrap = !self.state.word_wrap;
                let mut config = crate::core::config::ConfigManager::load_or_create();
                config.ui.word_wrap = self.state.word_wrap;
                let _ = crate::core::config::ConfigManager::save(&config);
                Some(Task::none())
            }
            _ => None,
        }
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
        if matches!(self.current_content, Some(PreviewData::Text { .. })) {
            let selection = self.state.text.content.selection().unwrap_or_default();
            if selection.is_empty() {
                None
            } else {
                let toast = self.show_toast("Copied! selected");
                Some(Task::batch(vec![iced::clipboard::write(selection), toast]))
            }
        } else if matches!(self.current_content, Some(PreviewData::Json { .. }))
            && !self.state.json.tree_mode
        {
            let selection = self.state.json.raw_editor.selection().unwrap_or_default();
            if selection.is_empty() {
                None
            } else {
                let toast = self.show_toast("Copied selected!");
                Some(Task::batch(vec![iced::clipboard::write(selection), toast]))
            }
        } else if matches!(
            self.current_content,
            Some(PreviewData::Markdown { .. }) | Some(PreviewData::Epub { .. })
        ) {
            let previous_selected = crate::features::markdown::update::active_markdown_state(self)
                .selected_text
                .clone();
            crate::features::markdown::update::update_selected_text_from_range(self);
            let selected_text = crate::features::markdown::update::active_markdown_state(self)
                .selected_text
                .clone()
                .or(previous_selected);
            if let Some(text) = selected_text
                && !text.is_empty()
            {
                let toast = self.show_toast("Copied selected!");
                Some(Task::batch(vec![iced::clipboard::write(text), toast]))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn handle_zoom_or_font(&mut self, direction: f32) -> Option<Task<Message>> {
        if matches!(self.current_content, Some(PreviewData::Image { .. })) {
            let next_zoom =
                (self.state.image.camera.zoom + direction * ZOOM_STEP).clamp(ZOOM_MIN, ZOOM_MAX);
            if (next_zoom - self.state.image.camera.zoom).abs() > f32::EPSILON {
                self.state.image.camera.zoom = next_zoom;
            }
            Some(Task::none())
        } else if matches!(
            self.current_content,
            Some(PreviewData::Markdown { .. })
                | Some(PreviewData::Text { .. })
                | Some(PreviewData::Epub { .. })
                | Some(PreviewData::Pdf { .. })
                | Some(PreviewData::Typst { .. })
        ) {
            let next_font_size = (self.state.font_size + direction).clamp(FONT_MIN, FONT_MAX);
            if (next_font_size - self.state.font_size).abs() > f32::EPSILON {
                self.state.font_size = next_font_size;
                if let Some(PreviewData::Markdown { ref blocks, .. }) = self.current_content {
                    self.state.markdown.toc = extract_toc(
                        blocks,
                        self.state.font_size,
                        &self.state.markdown.cached_image_sizes,
                    );
                }
            }
            Some(Task::none())
        } else {
            None
        }
    }

    fn handle_ctrl_f(&mut self) -> Option<Task<Message>> {
        if matches!(self.current_content, Some(PreviewData::Json { .. })) {
            self.state.json.search_visible = !self.state.json.search_visible;
            if !self.state.json.search_visible {
                self.state.json.search_query.clear();
                Some(Task::none())
            } else {
                Some(iced::widget::operation::focus("json_search_input"))
            }
        } else if matches!(self.current_content, Some(PreviewData::Text { .. })) {
            self.state.text.search_visible = !self.state.text.search_visible;
            if !self.state.text.search_visible {
                self.state.text.search_query.clear();
                Some(Task::none())
            } else {
                Some(iced::widget::operation::focus("txt_search_input"))
            }
        } else if matches!(self.current_content, Some(PreviewData::Markdown { .. })) {
            self.state.markdown.search_visible = !self.state.markdown.search_visible;
            if !self.state.markdown.search_visible {
                self.state.markdown.search_query.clear();
                self.state.markdown.search_match_count = 0;
                self.state.markdown.search_match_index = 0;
                self.state.markdown.search_match_blocks.clear();
                self.state.markdown.search_info.clear();
                Some(Task::none())
            } else {
                Some(iced::widget::operation::focus("md_search_input"))
            }
        } else if matches!(self.current_content, Some(PreviewData::Spreadsheet { .. })) {
            self.state.spreadsheet.search_visible = true;
            Some(iced::widget::operation::focus("ss_search_input"))
        } else if matches!(&self.state.view_mode, crate::core::ViewMode::Grid(_)) {
            self.state.grid_search_visible = true;
            Some(iced::widget::operation::focus("grid_search_input"))
        } else {
            None
        }
    }

    pub(super) fn handle_font_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        match key {
            iced::keyboard::Key::Character(c) if (c == "+" || c == "=") && modifiers.shift() => {
                self.state.font_size = self.state.default_font_size;
                if let Some(PreviewData::Markdown { ref blocks, .. }) = self.current_content {
                    self.state.markdown.toc = extract_toc(
                        blocks,
                        self.state.font_size,
                        &self.state.markdown.cached_image_sizes,
                    );
                }
                Some(Task::none())
            }
            _ => None,
        }
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
}
