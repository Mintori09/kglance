use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::app::KglanceApp;
use crate::core::PreviewData;

const CONTENT_SCROLL_ID: &str = "content_scroll";
const SCROLL_LINE_AMOUNT: f32 = 80.0;
const SCROLL_HALF_PAGE_AMOUNT: f32 = 600.0;

impl KglanceApp {
    pub(super) fn handle_scroll_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        use iced::keyboard::Key;
        use iced::keyboard::key::Named;

        match key {
            Key::Named(Named::ArrowDown) => Some(self.scroll_by(SCROLL_LINE_AMOUNT)),
            Key::Character(character) if character == "j" => {
                Some(self.scroll_by(SCROLL_LINE_AMOUNT))
            }
            Key::Named(Named::ArrowUp) => Some(self.scroll_by(-SCROLL_LINE_AMOUNT)),
            Key::Character(character) if character == "k" => {
                Some(self.scroll_by(-SCROLL_LINE_AMOUNT))
            }
            Key::Named(Named::PageDown) => Some(self.scroll_by(SCROLL_HALF_PAGE_AMOUNT)),
            Key::Character(character) if character == "d" => {
                Some(self.scroll_by(SCROLL_HALF_PAGE_AMOUNT))
            }
            Key::Named(Named::PageUp) => Some(self.scroll_by(-SCROLL_HALF_PAGE_AMOUNT)),
            Key::Character(character) if character == "u" => {
                Some(self.scroll_by(-SCROLL_HALF_PAGE_AMOUNT))
            }
            Key::Character(character) if character == "g" && !modifiers.shift() => {
                self.handle_g_shortcut()
            }
            Key::Character(character)
                if character == "G" || (character == "g" && modifiers.shift()) =>
            {
                Some(self.snap_to_bottom())
            }
            Key::Named(Named::Home) => self.handle_home_shortcut(),
            Key::Named(Named::End) => Some(self.snap_to_bottom()),
            Key::Character(character) if character == "t" && self.pending_g => {
                self.toggle_sidebar()
            }
            _ => {
                self.reset_scroll_pending();
                None
            }
        }
    }

    fn scroll_by(&mut self, vertical_offset: f32) -> Task<Message> {
        self.reset_scroll_pending();

        operation::scroll_by(
            CONTENT_SCROLL_ID,
            AbsoluteOffset {
                x: 0.0,
                y: vertical_offset,
            },
        )
    }

    fn handle_g_shortcut(&mut self) -> Option<Task<Message>> {
        self.pending_home = false;

        if self.pending_g {
            self.pending_g = false;
            Some(self.snap_to_top())
        } else {
            self.pending_g = true;
            None
        }
    }

    fn handle_home_shortcut(&mut self) -> Option<Task<Message>> {
        self.pending_g = false;

        if self.pending_home {
            self.pending_home = false;
            Some(self.snap_to_top())
        } else {
            self.pending_home = true;
            None
        }
    }

    fn snap_to_top(&mut self) -> Task<Message> {
        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 0.0 })
    }

    fn snap_to_bottom(&mut self) -> Task<Message> {
        self.reset_scroll_pending();

        operation::snap_to(CONTENT_SCROLL_ID, RelativeOffset { x: 0.0, y: 1.0 })
    }

    fn toggle_sidebar(&mut self) -> Option<Task<Message>> {
        self.reset_scroll_pending();

        match self.current_content {
            Some(PreviewData::Markdown { .. }) => Some(Task::done(
                crate::app::messages::MarkdownMsg::TocToggled.into(),
            )),
            Some(PreviewData::Epub { .. }) => Some(Task::done(
                crate::app::messages::EpubMsg::SidebarToggled.into(),
            )),
            Some(PreviewData::Pdf { .. }) | Some(PreviewData::Typst { .. }) => Some(Task::done(
                crate::app::messages::PdfMsg::SidebarToggled.into(),
            )),
            Some(PreviewData::Text { .. }) => Some(Task::done(
                crate::app::messages::TextMsg::ToggleOutline.into(),
            )),
            _ => None,
        }
    }

    fn reset_scroll_pending(&mut self) {
        self.pending_g = false;
        self.pending_home = false;
    }

    pub(super) fn handle_json_tree_navigation(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        use iced::keyboard::Key;
        use iced::keyboard::key::Named;

        if !self.is_json_tree_nav_available() || modifiers.control() || modifiers.alt() {
            return None;
        }

        let visible = crate::features::json::view::tree::visible_node_indices(&self.state.json);
        if visible.is_empty() {
            return None;
        }

        match key {
            Key::Named(Named::ArrowDown) => {
                self.move_json_selection(&visible, 1);
                Some(Task::none())
            }
            Key::Named(Named::ArrowUp) => {
                self.move_json_selection(&visible, -1);
                Some(Task::none())
            }
            Key::Named(Named::ArrowRight) => {
                self.expand_or_enter_json_node();
                Some(Task::none())
            }
            Key::Named(Named::ArrowLeft) => {
                self.collapse_or_exit_json_node();
                Some(Task::none())
            }
            Key::Named(Named::Enter) => {
                if let Some(active) = self.state.json.active_node
                    && self
                        .state
                        .json
                        .nodes
                        .get(active)
                        .is_some_and(|n| n.children_count > 0)
                {
                    return Some(
                        self.update(crate::app::messages::JsonMsg::ToggleNode(active).into()),
                    );
                }
                None
            }
            _ => None,
        }
    }

    fn is_json_tree_nav_available(&self) -> bool {
        matches!(self.current_content, Some(PreviewData::Json { .. }))
            && self.state.json.tree_mode
            && !self.state.json.search_visible
    }

    fn move_json_selection(&mut self, visible: &[usize], delta: isize) {
        let current_pos = self
            .state
            .json
            .active_node
            .and_then(|curr| visible.iter().position(|&idx| idx == curr));

        let new_pos = match current_pos {
            Some(pos) => {
                let target = pos as isize + delta;
                target.clamp(0, (visible.len() - 1) as isize) as usize
            }
            None => {
                if delta > 0 {
                    0
                } else {
                    visible.len() - 1
                }
            }
        };

        if let Some(&node_idx) = visible.get(new_pos) {
            self.state.json.active_node = Some(node_idx);
        }
    }

    fn expand_or_enter_json_node(&mut self) {
        if let Some(active) = self.state.json.active_node
            && let Some(node) = self.state.json.nodes.get(active)
            && node.children_count > 0
        {
            if !self.state.json.expanded.contains(&active) {
                self.state.json.expanded.insert(active);
            } else if active + 1 < self.state.json.nodes.len() {
                self.state.json.active_node = Some(active + 1);
            }
        }
    }

    fn collapse_or_exit_json_node(&mut self) {
        if let Some(active) = self.state.json.active_node {
            if self.state.json.expanded.contains(&active) {
                self.state.json.expanded.remove(&active);
            } else if let Some(parent) = self
                .state
                .json
                .nodes
                .get(active)
                .and_then(|n| n.parent_index)
            {
                self.state.json.active_node = Some(parent);
            }
        }
    }
}
