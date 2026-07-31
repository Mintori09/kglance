use iced::Task;

use super::Message;
use crate::core::PreviewData;

use crate::app::KglanceApp;

impl KglanceApp {
    pub(super) fn handle_type_to_search(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        // Type-to-search in spreadsheet: open search bar with typed char
        if matches!(self.current_content, Some(PreviewData::Spreadsheet { .. }))
            && !self.state.spreadsheet.search_visible
            && let iced::keyboard::Key::Character(c) = key
            && !modifiers.control()
            && !modifiers.alt()
            && c != "/"
        {
            self.state.spreadsheet.search_visible = true;
            self.state.spreadsheet.search_query = c.to_string();
            return Some(iced::widget::operation::focus("ss_search_input"));
        }

        // Type-to-search in grid: open search bar with typed char
        if matches!(&self.state.view_mode, crate::core::ViewMode::Grid(_))
            && !self.state.grid_search_visible
            && let iced::keyboard::Key::Character(c) = key
            && !modifiers.control()
            && !modifiers.alt()
            && c != "/"
        {
            self.state.grid_search_visible = true;
            self.state.grid_search_query = c.to_string();
            return Some(iced::widget::operation::focus("grid_search_input"));
        }

        None
    }

    pub(super) fn handle_vim_search_open(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        let iced::keyboard::Key::Character(c) = key else {
            return None;
        };
        if c != "/" {
            return None;
        }

        if matches!(self.current_content, Some(PreviewData::Text { .. }))
            && !self.state.text.search_visible
        {
            self.state.text.search_visible = true;
            Some(iced::widget::operation::focus("txt_search_input"))
        } else if matches!(self.current_content, Some(PreviewData::Markdown { .. }))
            && !self.state.markdown.search_visible
        {
            self.state.markdown.search_visible = true;
            self.state.markdown.search_match_blocks.clear();
            Some(iced::widget::operation::focus("md_search_input"))
        } else if matches!(self.current_content, Some(PreviewData::Json { .. }))
            && !self.state.json.search_visible
            && self.state.json.editing_node.is_none()
        {
            self.state.json.search_visible = true;
            Some(iced::widget::operation::focus("json_search_input"))
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

    pub(super) fn handle_close_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        let should_close = match key {
            iced::keyboard::Key::Named(
                iced::keyboard::key::Named::Escape | iced::keyboard::key::Named::Space,
            ) => true,
            iced::keyboard::Key::Character(c) if c == " " => true,
            _ => false,
        };

        if should_close {
            Some(self.handle_close())
        } else {
            None
        }
    }

    pub(super) fn is_search_active(&self) -> bool {
        if matches!(&self.state.view_mode, crate::core::ViewMode::Grid(_)) {
            return self.state.grid_search_visible;
        }
        match &self.current_content {
            Some(PreviewData::Text { .. }) => self.state.text.search_visible,
            Some(PreviewData::Json { .. }) => {
                self.state.json.search_visible || self.state.json.editing_node.is_some()
            }
            Some(PreviewData::Markdown { .. }) => self.state.markdown.search_visible,
            Some(PreviewData::Spreadsheet { .. }) => self.state.spreadsheet.search_visible,
            _ => false,
        }
    }

    pub(super) fn handle_search_close(&mut self) -> Task<Message> {
        if matches!(&self.state.view_mode, crate::core::ViewMode::Grid(_))
            && self.state.grid_search_visible
        {
            self.state.grid_search_visible = false;
            self.state.grid_search_query.clear();
            return Task::none();
        }
        match &self.current_content {
            Some(PreviewData::Text { .. }) if self.state.text.search_visible => {
                self.state.text.search_visible = false;
                self.state.text.search_query.clear();
                Task::none()
            }
            Some(PreviewData::Json { .. }) => {
                if self.state.json.search_visible {
                    self.state.json.search_visible = false;
                    self.state.json.search_query.clear();
                    return Task::none();
                }
                if self.state.json.editing_node.is_some() {
                    self.state.json.editing_node = None;
                    self.state.json.edit_value.clear();
                }
                Task::none()
            }
            Some(PreviewData::Markdown { .. }) if self.state.markdown.search_visible => {
                self.state.markdown.search_visible = false;
                self.state.markdown.search_query.clear();
                self.state.markdown.search_match_count = 0;
                self.state.markdown.search_match_index = 0;
                self.state.markdown.search_match_blocks.clear();
                self.state.markdown.search_info.clear();
                Task::none()
            }
            Some(PreviewData::Spreadsheet { .. }) if self.state.spreadsheet.search_visible => {
                self.state.spreadsheet.search_visible = false;
                self.state.spreadsheet.search_query.clear();
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
