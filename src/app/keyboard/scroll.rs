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
}
