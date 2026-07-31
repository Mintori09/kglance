use iced::widget::{container, scrollable};
use iced::{Element, Length, Padding};

use crate::ui::theme::{default_root, default_scrollable};

pub fn scroll_pane<'a, Message: 'static>(
    id: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> ScrollPaneBuilder<'a, Message> {
    ScrollPaneBuilder {
        id,
        content: content.into(),
        on_scroll: None,
        height: Length::Fill,
        container_padding: None,
    }
}

pub struct ScrollPaneBuilder<'a, Message> {
    id: &'static str,
    content: Element<'a, Message>,
    on_scroll: Option<Box<dyn Fn(scrollable::Viewport) -> Message + 'static>>,
    height: Length,
    container_padding: Option<Padding>,
}

impl<'a, Message: 'static> ScrollPaneBuilder<'a, Message> {
    pub fn on_scroll<F>(mut self, f: F) -> Self
    where
        F: Fn(scrollable::Viewport) -> Message + 'static,
    {
        self.on_scroll = Some(Box::new(f));
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn container_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.container_padding = Some(padding.into());
        self
    }

    pub fn build(self) -> Element<'a, Message> {
        let inner = if let Some(padding) = self.container_padding {
            container(self.content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(padding)
                .style(default_root)
                .into()
        } else {
            self.content
        };

        let mut scroll = scrollable(inner)
            .id(self.id)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(4).margin(2),
            ))
            .style(default_scrollable)
            .width(Length::Fill)
            .height(self.height);

        if let Some(f) = self.on_scroll {
            scroll = scroll.on_scroll(f);
        }

        scroll.into()
    }
}
