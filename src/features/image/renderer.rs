use iced::Rectangle;
use iced::advanced::image;

use crate::core::ImageState;
use crate::features::image::camera::Camera;

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
        let handle = image
            .preview_handle
            .as_ref()
            .or(image.display_handle.as_ref());
        let Some(handle) = handle else {
            return;
        };

        let (disp_w, disp_h) = (image.display_width, image.display_height);
        if disp_w == 0 || disp_h == 0 {
            return;
        }

        let (effective_zoom, offset_x, offset_y) = if (camera.zoom - 1.0).abs() < f32::EPSILON
            && camera.offset_x == 0.0
            && camera.offset_y == 0.0
        {
            let fit_scale = (bounds.width / disp_w as f32).min(bounds.height / disp_h as f32);
            (fit_scale, 0.0, 0.0)
        } else {
            (camera.zoom, camera.offset_x, camera.offset_y)
        };

        let img_w = disp_w as f32 * effective_zoom;
        let img_h = disp_h as f32 * effective_zoom;

        let img_x = bounds.x + bounds.width / 2.0 + offset_x - img_w / 2.0;
        let img_y = bounds.y + bounds.height / 2.0 + offset_y - img_h / 2.0;

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
