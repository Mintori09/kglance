use iced::Rectangle;
use iced::advanced::image;

use crate::core::ImageState;
use crate::features::image::Camera;

pub trait ImageRenderer {
    fn draw(
        &self,
        renderer: &mut impl image::Renderer<Handle = image::Handle>,
        camera: &Camera,
        image: &ImageState,
        bounds: Rectangle,
    );
}

pub struct CanvasRenderer;

impl ImageRenderer for CanvasRenderer {
    fn draw(
        &self,
        renderer: &mut impl image::Renderer<Handle = image::Handle>,
        camera: &Camera,
        image: &ImageState,
        bounds: Rectangle,
    ) {
        let Some(ref handle) = image.handle else {
            return;
        };

        let img_w = image.width as f32 * camera.zoom;
        let img_h = image.height as f32 * camera.zoom;

        let img_x = bounds.x + bounds.width / 2.0 + camera.offset_x - img_w / 2.0;
        let img_y = bounds.y + bounds.height / 2.0 + camera.offset_y - img_h / 2.0;

        let draw_bounds = Rectangle::new(
            iced::Point::new(img_x, img_y),
            iced::Size::new(img_w, img_h),
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
    }
}
