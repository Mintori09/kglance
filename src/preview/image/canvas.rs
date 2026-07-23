use iced::advanced::image;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::mouse::click::Kind as ClickKind;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Layout, Shell};
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Size};

use crate::core::ImageState;
use crate::preview::image::camera::Camera;

#[derive(Debug, Clone, Copy, Default)]
struct CanvasState {
    drag_start: Option<Point>,
    previous_click: Option<iced::advanced::mouse::Click>,
}

pub struct ImageCanvas<'a, Message> {
    image: &'a ImageState,
    camera: &'a Camera,
    on_drag: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
    on_drag_start: Option<Box<dyn Fn(Point) -> Message + 'a>>,
    on_drag_end: Option<Box<dyn Fn() -> Message + 'a>>,
    on_double_click: Option<Box<dyn Fn() -> Message + 'a>>,
    width: Length,
    height: Length,
}

impl<'a, Message> ImageCanvas<'a, Message> {
    pub fn new(image: &'a ImageState, camera: &'a Camera) -> Self {
        Self {
            image,
            camera,
            on_drag: None,
            on_drag_start: None,
            on_drag_end: None,
            on_double_click: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    pub fn on_drag(mut self, f: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_drag = Some(Box::new(f));
        self
    }

    pub fn on_drag_start(mut self, f: impl Fn(Point) -> Message + 'a) -> Self {
        self.on_drag_start = Some(Box::new(f));
        self
    }

    pub fn on_drag_end(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn on_double_click(mut self, f: impl Fn() -> Message + 'a) -> Self {
        self.on_double_click = Some(Box::new(f));
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ImageCanvas<'a, Message>
where
    Renderer: image::Renderer<Handle = image::Handle> + 'a,
    Message: Clone,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<CanvasState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(CanvasState::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
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

        match event {
            Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                let Some(pos) = cursor.position_over(layout.bounds()) else {
                    return;
                };

                if let Some(ref f) = self.on_double_click {
                    let new_click = iced::advanced::mouse::Click::new(
                        pos,
                        iced::mouse::Button::Left,
                        state.previous_click,
                    );
                    if new_click.kind() == ClickKind::Double {
                        shell.publish(f());
                    }
                    state.previous_click = Some(new_click);
                }

                state.drag_start = Some(pos);
                if let Some(ref f) = self.on_drag_start {
                    shell.publish(f(pos));
                }
                shell.capture_event();
            }
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                if state.drag_start.is_some() {
                    state.drag_start = None;
                    if let Some(ref f) = self.on_drag_end {
                        shell.publish(f());
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                if let Some(origin) = state.drag_start {
                    let dx = position.x - origin.x;
                    let dy = position.y - origin.y;
                    if let Some(ref f) = self.on_drag {
                        shell.publish(f(dx, dy));
                    }
                    state.drag_start = Some(*position);
                    shell.capture_event();
                }
            }
            _ => {}
        }
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
            mouse::Interaction::None
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
        let bounds = layout.bounds();

        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let Some(ref handle) = self.image.handle else {
            return;
        };

        let zoom = self.camera.zoom;
        let (img_w, img_h) = if self.image.width == 0 || self.image.height == 0 {
            (bounds.width, bounds.height)
        } else if (zoom - 1.0).abs() < 0.001 {
            let scale_w = bounds.width / self.image.width as f32;
            let scale_h = bounds.height / self.image.height as f32;
            let fit_zoom = scale_w.min(scale_h);
            (
                self.image.width as f32 * fit_zoom,
                self.image.height as f32 * fit_zoom,
            )
        } else {
            (
                self.image.width as f32 * zoom,
                self.image.height as f32 * zoom,
            )
        };

        let vp_cx = bounds.x + bounds.width / 2.0;
        let vp_cy = bounds.y + bounds.height / 2.0;

        let draw_x = vp_cx + self.camera.offset_x - img_w / 2.0;
        let draw_y = vp_cy + self.camera.offset_y - img_h / 2.0;

        let draw_bounds = Rectangle::new(Point::new(draw_x, draw_y), Size::new(img_w, img_h));

        renderer.with_layer(bounds, |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default(),
                    shadow: Default::default(),
                    snap: false,
                },
                Color::from_rgb(0.08, 0.08, 0.08),
            );

            renderer.draw_image(
                image::Image {
                    handle: handle.clone(),
                    filter_method: image::FilterMethod::Linear,
                    rotation: iced::Radians(0.0),
                    border_radius: iced::border::Radius::default(),
                    opacity: 1.0,
                    snap: true,
                },
                draw_bounds,
                bounds,
            );
        });
    }
}

impl<'a, Message, Theme, Renderer> From<ImageCanvas<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = image::Handle> + 'a,
    Theme: 'a,
    Message: 'a + Clone,
{
    fn from(canvas: ImageCanvas<'a, Message>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(canvas)
    }
}
