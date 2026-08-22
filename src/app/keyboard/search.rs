use iced::Task;

use super::Message;
use crate::app::KglanceApp;
use crate::core::{PreviewData, ViewMode};

const SPREADSHEET_SEARCH_INPUT_ID: &str = "ss_search_input";
const GRID_SEARCH_INPUT_ID: &str = "grid_search_input";
const TEXT_SEARCH_INPUT_ID: &str = "txt_search_input";
const MARKDOWN_SEARCH_INPUT_ID: &str = "md_search_input";
const JSON_SEARCH_INPUT_ID: &str = "json_search_input";
const SEARCH_TRIGGER: &str = "/";

impl KglanceApp {
    pub(super) fn handle_type_to_search(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        if self.should_start_type_to_search(key, modifiers) {
            if self.is_spreadsheet() {
                return Some(self.open_spreadsheet_search(key));
            }

            if self.is_grid_view() {
                return Some(self.open_grid_search(key));
            }
        }

        None
    }

    pub(super) fn handle_vim_search_open(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        if !Self::is_search_trigger(key) {
            return None;
        }

        if self.is_text_search_available() {
            return Some(self.open_text_search());
        }

        if self.is_markdown_search_available() {
            return Some(self.open_markdown_search());
        }

        if self.is_json_search_available() {
            return Some(self.open_json_search());
        }

        if self.is_spreadsheet() {
            return Some(self.open_spreadsheet_search_without_query());
        }

        if self.is_grid_view() {
            return Some(self.open_grid_search_without_query());
        }

        None
    }

    pub(super) fn handle_close_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
    ) -> Option<Task<Message>> {
        if Self::is_close_shortcut(key) {
            Some(self.handle_close())
        } else {
            None
        }
    }

    pub(super) fn is_search_active(&self) -> bool {
        if self.is_grid_view() {
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
        if self.is_grid_search_active() {
            self.close_grid_search();
            return Task::none();
        }

        match &self.current_content {
            Some(PreviewData::Text { .. }) if self.state.text.search_visible => {
                self.close_text_search();
            }
            Some(PreviewData::Json { .. }) => {
                self.close_json_search_or_editing();
            }
            Some(PreviewData::Markdown { .. }) if self.state.markdown.search_visible => {
                self.close_markdown_search();
            }
            Some(PreviewData::Spreadsheet { .. }) if self.state.spreadsheet.search_visible => {
                self.close_spreadsheet_search();
            }
            _ => {}
        }

        Task::none()
    }

    fn should_start_type_to_search(
        &self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> bool {
        matches!(key, iced::keyboard::Key::Character(character)
            if !modifiers.control()
                && !modifiers.alt()
                && character != SEARCH_TRIGGER)
            && ((self.is_spreadsheet() && !self.state.spreadsheet.search_visible)
                || (self.is_grid_view() && !self.state.grid_search_visible))
    }

    fn is_search_trigger(key: &iced::keyboard::Key) -> bool {
        matches!(
            key,
            iced::keyboard::Key::Character(character) if character == SEARCH_TRIGGER
        )
    }

    fn is_spreadsheet(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Spreadsheet { .. }))
    }

    fn is_grid_view(&self) -> bool {
        matches!(&self.state.view_mode, ViewMode::Grid(_))
    }

    fn is_text_search_available(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Text { .. }))
            && !self.state.text.search_visible
    }

    fn is_markdown_search_available(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Markdown { .. }))
            && !self.state.markdown.search_visible
    }

    fn is_json_search_available(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Json { .. }))
            && !self.state.json.search_visible
            && self.state.json.editing_node.is_none()
    }

    fn is_grid_search_active(&self) -> bool {
        self.is_grid_view() && self.state.grid_search_visible
    }

    fn open_spreadsheet_search(&mut self, key: &iced::keyboard::Key) -> Task<Message> {
        let iced::keyboard::Key::Character(character) = key else {
            return Task::none();
        };

        self.state.spreadsheet.search_visible = true;
        self.state.spreadsheet.search_query = character.to_string();

        iced::widget::operation::focus(SPREADSHEET_SEARCH_INPUT_ID)
    }

    fn open_grid_search(&mut self, key: &iced::keyboard::Key) -> Task<Message> {
        let iced::keyboard::Key::Character(character) = key else {
            return Task::none();
        };

        self.state.grid_search_visible = true;
        self.state.grid_search_query = character.to_string();

        iced::widget::operation::focus(GRID_SEARCH_INPUT_ID)
    }

    fn open_spreadsheet_search_without_query(&mut self) -> Task<Message> {
        self.state.spreadsheet.search_visible = true;
        iced::widget::operation::focus(SPREADSHEET_SEARCH_INPUT_ID)
    }

    fn open_grid_search_without_query(&mut self) -> Task<Message> {
        self.state.grid_search_visible = true;
        iced::widget::operation::focus(GRID_SEARCH_INPUT_ID)
    }

    fn open_text_search(&mut self) -> Task<Message> {
        self.state.text.search_visible = true;
        iced::widget::operation::focus(TEXT_SEARCH_INPUT_ID)
    }

    fn open_markdown_search(&mut self) -> Task<Message> {
        self.state.markdown.search_visible = true;
        self.state.markdown.search_match_blocks.clear();
        iced::widget::operation::focus(MARKDOWN_SEARCH_INPUT_ID)
    }

    fn open_json_search(&mut self) -> Task<Message> {
        self.state.json.search_visible = true;
        iced::widget::operation::focus(JSON_SEARCH_INPUT_ID)
    }

    fn close_grid_search(&mut self) {
        self.state.grid_search_visible = false;
        self.state.grid_search_query.clear();
    }

    fn close_text_search(&mut self) {
        self.state.text.search_visible = false;
        self.state.text.search_query.clear();
    }

    fn close_json_search_or_editing(&mut self) {
        if self.state.json.search_visible {
            self.state.json.search_visible = false;
            self.state.json.search_query.clear();
        }

        if self.state.json.editing_node.is_some() {
            self.state.json.editing_node = None;
            self.state.json.edit_value.clear();
        }
    }

    fn close_markdown_search(&mut self) {
        self.state.markdown.search_visible = false;
        self.state.markdown.search_query.clear();
        self.state.markdown.search_match_count = 0;
        self.state.markdown.search_match_index = 0;
        self.state.markdown.search_match_blocks.clear();
        self.state.markdown.search_info.clear();
    }

    fn close_spreadsheet_search(&mut self) {
        self.state.spreadsheet.search_visible = false;
        self.state.spreadsheet.search_query.clear();
    }

    fn is_close_shortcut(key: &iced::keyboard::Key) -> bool {
        match key {
            iced::keyboard::Key::Named(
                iced::keyboard::key::Named::Escape | iced::keyboard::key::Named::Space,
            ) => true,
            iced::keyboard::Key::Character(character) => character == " ",
            _ => false,
        }
    }
}
