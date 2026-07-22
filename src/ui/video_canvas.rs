use crate::core::MediaState;
use iced::advanced::Layout;
use iced::advanced::image;
use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::{Border, Color, ContentFit, Element, Length, Point, Rectangle, Size};

pub struct VideoCanvas<'a> {
    state: &'a MediaState,
    content_fit: ContentFit,
    width_len: Length,
    height_len: Length,
}

impl<'a> VideoCanvas<'a> {
    pub fn new(state: &'a MediaState) -> Self {
        Self {
            state,
            content_fit: ContentFit::Contain,
            width_len: Length::Fill,
            height_len: Length::Fill,
        }
    }

    pub fn content_fit(mut self, fit: ContentFit) -> Self {
        self.content_fit = fit;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width_len = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height_len = height.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for VideoCanvas<'a>
where
    Renderer: image::Renderer<Handle = image::Handle>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width_len,
            height: self.height_len,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width_len, self.height_len)
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

        let Some(ref handle) = self.state.video_handle else {
            return;
        };

        if self.state.frame_width == 0 || self.state.frame_height == 0 {
            return;
        }

        let raw = Size::new(
            self.state.frame_width as f32,
            self.state.frame_height as f32,
        );
        let fit = self.content_fit.fit(raw, bounds.size());

        let draw_bounds = Rectangle::new(
            Point::new(
                bounds.center_x() - fit.width / 2.0,
                bounds.center_y() - fit.height / 2.0,
            ),
            Size::new(fit.width, fit.height),
        );

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

impl<'a, Message, Theme, Renderer> From<VideoCanvas<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = image::Handle> + 'a,
    Theme: 'a,
    Message: 'a,
{
    fn from(canvas: VideoCanvas<'a>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(canvas)
    }
}
