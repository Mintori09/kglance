use iced::advanced::graphics::core::event::Event;
use iced::advanced::graphics::core::layout::{self, Layout};
use iced::advanced::graphics::core::mouse;
use iced::advanced::graphics::core::renderer;
use iced::advanced::graphics::core::widget::{Tree, Widget, tree};
use iced::advanced::graphics::core::{Clipboard, Shell};
use iced::{Element, Length, Rectangle, Size, Vector, window};

/// Height difference (in pixels) below which a block is considered correctly sized.
const MEASURE_TOLERANCE: f32 = 0.5;

/// Wraps a block and reports its laid-out height when it differs from the
/// height the document model currently assumes.
///
/// It publishes at most once per distinct height, so a rejected measurement
/// cannot cause a message loop.
pub struct Measured<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    block_index: usize,
    known_height: f32,
    on_measured: Box<dyn Fn(usize, f32) -> Message + 'a>,
}

impl<'a, Message, Theme, Renderer> Measured<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        block_index: usize,
        known_height: f32,
        on_measured: impl Fn(usize, f32) -> Message + 'a,
    ) -> Self {
        Self {
            content: content.into(),
            block_index,
            known_height,
            on_measured: Box::new(on_measured),
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Measured<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Option<f32>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(None::<f32>)
    }

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

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            let height = layout.bounds().height;
            let last_reported = tree.state.downcast_mut::<Option<f32>>();
            if (height - self.known_height).abs() > MEASURE_TOLERANCE
                && *last_reported != Some(height)
            {
                *last_reported = Some(height);
                shell.publish((self.on_measured)(self.block_index, height));
            }
        }
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

impl<'a, Message: 'a, Theme: 'a, Renderer: 'a> From<Measured<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(measured: Measured<'a, Message, Theme, Renderer>) -> Self {
        Element::new(measured)
    }
}
