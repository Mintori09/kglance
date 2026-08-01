use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::Message;
use crate::core::PreviewData;

const SCROLL_LINE_AMOUNT: f32 = 80.0;
const SCROLL_HALF_PAGE_AMOUNT: f32 = 600.0;

use crate::app::KglanceApp;

impl KglanceApp {
    pub(super) fn handle_scroll_shortcuts(
        &mut self,
        key: &iced::keyboard::Key,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<Task<Message>> {
        use iced::keyboard::Key;

        match key {
            Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: SCROLL_LINE_AMOUNT,
                    },
                ))
            }
            Key::Character(c) if c == "j" => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: SCROLL_LINE_AMOUNT,
                    },
                ))
            }
            Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -SCROLL_LINE_AMOUNT,
                    },
                ))
            }
            Key::Character(c) if c == "k" => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -SCROLL_LINE_AMOUNT,
                    },
                ))
            }
            Key::Named(iced::keyboard::key::Named::PageDown) => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: SCROLL_HALF_PAGE_AMOUNT,
                    },
                ))
            }
            Key::Character(c) if c == "d" => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: SCROLL_HALF_PAGE_AMOUNT,
                    },
                ))
            }
            Key::Named(iced::keyboard::key::Named::PageUp) => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -SCROLL_HALF_PAGE_AMOUNT,
                    },
                ))
            }
            Key::Character(c) if c == "u" => {
                self.reset_scroll_pending();
                Some(operation::scroll_by(
                    "content_scroll",
                    AbsoluteOffset {
                        x: 0.0,
                        y: -SCROLL_HALF_PAGE_AMOUNT,
                    },
                ))
            }
            // gg -> top
            Key::Character(c) if c == "g" && !modifiers.shift() => {
                self.pending_home = false;
                if self.pending_g {
                    self.pending_g = false;
                    Some(operation::snap_to(
                        "content_scroll",
                        RelativeOffset { x: 0.0, y: 0.0 },
                    ))
                } else {
                    self.pending_g = true;
                    None
                }
            }
            // G -> bottom
            Key::Character(c) if c == "G" || (c == "g" && modifiers.shift()) => {
                self.reset_scroll_pending();
                Some(operation::snap_to(
                    "content_scroll",
                    RelativeOffset { x: 0.0, y: 1.0 },
                ))
            }
            // Home (double-tap like gg) -> top
            Key::Named(iced::keyboard::key::Named::Home) => {
                self.pending_g = false;
                if self.pending_home {
                    self.pending_home = false;
                    Some(operation::snap_to(
                        "content_scroll",
                        RelativeOffset { x: 0.0, y: 0.0 },
                    ))
                } else {
                    self.pending_home = true;
                    None
                }
            }
            Key::Named(iced::keyboard::key::Named::End) => {
                self.reset_scroll_pending();
                Some(operation::snap_to(
                    "content_scroll",
                    RelativeOffset { x: 0.0, y: 1.0 },
                ))
            }
            // g t → toggle TOC / sidebar (markdown, epub, pdf, typst)
            Key::Character(c) if c == "t" && self.pending_g => {
                self.reset_scroll_pending();
                if matches!(self.current_content, Some(PreviewData::Markdown { .. })) {
                    Some(Task::done(
                        crate::app::messages::MarkdownMsg::TocToggled.into(),
                    ))
                } else if matches!(self.current_content, Some(PreviewData::Epub { .. })) {
                    Some(Task::done(
                        crate::app::messages::EpubMsg::SidebarToggled.into(),
                    ))
                } else if matches!(
                    self.current_content,
                    Some(PreviewData::Pdf { .. }) | Some(PreviewData::Typst { .. })
                ) {
                    Some(Task::done(
                        crate::app::messages::PdfMsg::SidebarToggled.into(),
                    ))
                } else {
                    None
                }
            }
            _ => {
                self.reset_scroll_pending();
                None
            }
        }
    }

    fn reset_scroll_pending(&mut self) {
        self.pending_g = false;
        self.pending_home = false;
    }
}
