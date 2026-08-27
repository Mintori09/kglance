use crate::ui::theme::{default_root, default_scrollable};
use iced::advanced::graphics::core::event::Event;
use iced::advanced::graphics::core::layout::{self, Layout};
use iced::advanced::graphics::core::mouse;
use iced::advanced::graphics::core::renderer;
use iced::advanced::graphics::core::widget::{Tree, Widget};
use iced::advanced::graphics::core::{Clipboard, Shell};
use iced::widget::{container, scrollable};
use iced::{Element, Length, Padding};
use iced::{Rectangle, Size, Vector};

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

pub struct ScrollFilter<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    filter_wheel: bool,
}

impl<'a, Message, Theme, Renderer> ScrollFilter<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        filter_wheel: bool,
    ) -> Self {
        Self {
            content: content.into(),
            filter_wheel,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ScrollFilter<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.filter_wheel && matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. })) {
            return;
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message: 'a, Theme: 'a, Renderer: 'a> From<ScrollFilter<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(filter: ScrollFilter<'a, Message, Theme, Renderer>) -> Self {
        Element::new(filter)
    }
}
