use iced::Event;
use iced::advanced::image;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::mouse::click::Kind as ClickKind;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Layout, Shell};
use iced::{Element, Length, Point, Rectangle, Size};

use crate::core::ImageState;
use crate::features::image::camera::Camera;

#[derive(Debug, Clone, Copy, Default)]
struct CanvasState {
    drag_start: Option<Point>,
    previous_click: Option<iced::advanced::mouse::Click>,
}

pub struct ImageCanvas<'a, Message> {
    image: &'a ImageState,
    camera: &'a Camera,
    width: Length,
    height: Length,
    on_zoom: Option<Box<dyn Fn(f32, Point) -> Message + 'a>>,
    on_pan: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
    on_double_click: Option<Box<dyn Fn() -> Message + 'a>>,
}

impl<'a, Message> ImageCanvas<'a, Message> {
    pub fn new(image: &'a ImageState, camera: &'a Camera) -> Self {
        Self {
            image,
            camera,
            width: Length::Fill,
            height: Length::Fill,
            on_zoom: None,
            on_pan: None,
            on_double_click: None,
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn on_zoom(mut self, on_zoom: impl Fn(f32, Point) -> Message + 'a) -> Self {
        self.on_zoom = Some(Box::new(on_zoom));
        self
    }

    pub fn on_drag(mut self, on_pan: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_pan = Some(Box::new(on_pan));
        self
    }

    pub fn on_double_click(mut self, on_double_click: impl Fn() -> Message + 'a) -> Self {
        self.on_double_click = Some(Box::new(on_double_click));
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ImageCanvas<'_, Message>
where
    Renderer: renderer::Renderer + image::Renderer<Handle = image::Handle>,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<CanvasState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(CanvasState::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<CanvasState>();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let cursor_position = match cursor {
                    mouse::Cursor::Available(p) if bounds.contains(p) => p,
                    _ => return,
                };

                let factor = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        if *y > 0.0 {
                            1.15
                        } else {
                            1.0 / 1.15
                        }
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => {
                        if *y > 0.0 {
                            1.15
                        } else {
                            1.0 / 1.15
                        }
                    }
                };

                if let Some(ref on_zoom) = self.on_zoom {
                    shell.publish(on_zoom(factor, cursor_position));
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let cursor_position = match cursor {
                    mouse::Cursor::Available(p) if bounds.contains(p) => p,
                    _ => return,
                };

                let click =
                    mouse::Click::new(cursor_position, mouse::Button::Left, state.previous_click);

                state.previous_click = Some(click);

                if click.kind() == ClickKind::Double {
                    if let Some(ref on_double_click) = self.on_double_click {
                        shell.publish(on_double_click());
                    }
                    state.drag_start = None;
                    return;
                }

                state.drag_start = Some(cursor_position);
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.drag_start.is_some() {
                    state.drag_start = None;
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(start) = state.drag_start {
                    let dx = position.x - start.x;
                    let dy = position.y - start.y;
                    state.drag_start = Some(*position);
                    if let Some(ref on_pan) = self.on_pan {
                        shell.publish(on_pan(dx, dy));
                    }
                }
            }

            _ => {}
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use crate::features::image::renderer::{CanvasRenderer, ImageRenderer};
        CanvasRenderer.draw(renderer, self.camera, self.image, layout.bounds());
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<CanvasState>();
        let bounds = layout.bounds();

        if state.drag_start.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::Idle
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ImageCanvas<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + image::Renderer<Handle = image::Handle> + 'a,
{
    fn from(canvas: ImageCanvas<'a, Message>) -> Self {
        Element::new(canvas)
    }
}
