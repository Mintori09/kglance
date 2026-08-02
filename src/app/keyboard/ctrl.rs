use iced::Task;

use super::Message;
use crate::core::PreviewData;
use crate::parsers::markdown::extract_toc;

const ZOOM_STEP: f32 = 0.2;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.0;
const FONT_MIN: f32 = 8.0;
const FONT_MAX: f32 = 48.0;

use crate::app::KglanceApp;

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
                self.state.theme_dark = !self.state.theme_dark;
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
            Some(crate::app::update::markdown::handle_select_all(self))
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
            let selected_text = crate::app::update::markdown::active_markdown_state(self)
                .selected_text
                .clone();
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
    fn ctrl_c_returns_none_without_selection() {
        let mut app = test_app(Some(epub_content(&["hello"])));
        assert!(app.handle_ctrl_copy().is_none());
    }
}
