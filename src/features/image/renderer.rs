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
        let (disp_w, disp_h) = (image.display_width, image.display_height);
        if disp_w == 0 || disp_h == 0 {
            return;
        }

        let (visual_w, visual_h) = if camera.rotation.is_perpendicular() {
            (disp_h as f32, disp_w as f32)
        } else {
            (disp_w as f32, disp_h as f32)
        };

        let fit_scale = (bounds.width / visual_w).min(bounds.height / visual_h);

        let (effective_zoom, offset_x, offset_y) = if (camera.zoom - 1.0).abs() < f32::EPSILON
            && camera.offset_x == 0.0
            && camera.offset_y == 0.0
        {
            (fit_scale, 0.0, 0.0)
        } else {
            (camera.zoom, camera.offset_x, camera.offset_y)
        };

        let img_w = disp_w as f32 * effective_zoom;
        let img_h = disp_h as f32 * effective_zoom;

        let center_x = bounds.x + bounds.width / 2.0 + offset_x;
        let center_y = bounds.y + bounds.height / 2.0 + offset_y;

        let (img_x, draw_w) = if camera.flip.0 {
            (center_x + img_w / 2.0, -img_w)
        } else {
            (center_x - img_w / 2.0, img_w)
        };

        let (img_y, draw_h) = if camera.flip.1 {
            (center_y + img_h / 2.0, -img_h)
        } else {
            (center_y - img_h / 2.0, img_h)
        };

        let draw_bounds = Rectangle::new(
            iced::Point::new(img_x, img_y),
            iced::Size::new(draw_w, draw_h),
        );

        let rotation = iced::Radians(camera.rotation.to_radians());

        // 1. Draw preview_handle as persistent underlay if present.
        // In Iced's WGPU backend, image data >2MB is uploaded asynchronously on a
        // background worker thread when first encountered, during which iced_wgpu
        // skips drawing the image for 1-2 frames. Drawing preview_handle first
        // ensures that the canvas is NEVER blank or flashing black during this upload window.
        if let Some(ref preview) = image.preview_handle {
            renderer.draw_image(
                image::Image {
                    handle: preview.clone(),
                    filter_method: image::FilterMethod::Linear,
                    rotation,
                    border_radius: iced::border::Radius::default(),
                    opacity: 1.0,
                    snap: false,
                },
                draw_bounds,
                bounds,
            );
        }

        // 2. Draw display_handle (or fallback handle) on top of the underlay.
        let main_handle = image.display_handle.as_ref().or(image.handle.as_ref());

        if let Some(main_handle) = main_handle
            && image.preview_handle.as_ref() != Some(main_handle)
        {
            renderer.draw_image(
                image::Image {
                    handle: main_handle.clone(),
                    filter_method: image::FilterMethod::Linear,
                    rotation,
                    border_radius: iced::border::Radius::default(),
                    opacity: 1.0,
                    snap: false,
                },
                draw_bounds,
                bounds,
            );
        }
    }
}
