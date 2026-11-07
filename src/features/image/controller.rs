use iced::{Point, Size};

use crate::features::image::Angle;
use crate::features::image::camera::Camera;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewerAction {
    Zoom { factor: f32, cursor: Point },
    Pan { dx: f32, dy: f32 },
    RotateRight,
    RotateLeft,
    FlipHorizontal,
    FlipVertical,
    FitToWindow,
    ActualSize,
    Reset,
    DoubleClickZoom,
}

pub struct ViewerController;

impl ViewerController {
    pub fn prepare_camera_for_zoom(
        camera: &mut Camera,
        display_width: u32,
        display_height: u32,
        viewport: Size,
    ) {
        if (camera.zoom - 1.0).abs() < f32::EPSILON
            && camera.offset_x == 0.0
            && camera.offset_y == 0.0
            && display_width > 0
            && display_height > 0
        {
            let (vis_w, vis_h) = if camera.rotation.is_perpendicular() {
                (display_height as f32, display_width as f32)
            } else {
                (display_width as f32, display_height as f32)
            };
            if vis_w > 0.0 && vis_h > 0.0 {
                camera.zoom = (viewport.width / vis_w).min(viewport.height / vis_h);
            }
        }
    }

    pub fn zoom(camera: &mut Camera, factor: f32, cursor: Point, viewport: Size) {
        let old_zoom = camera.zoom;
        let new_zoom = (old_zoom * factor).clamp(0.1, 10.0);

        let vp_cx = viewport.width / 2.0;
        let vp_cy = viewport.height / 2.0;

        let img_x = (cursor.x - vp_cx - camera.offset_x) / old_zoom;
        let img_y = (cursor.y - vp_cy - camera.offset_y) / old_zoom;

        camera.zoom = new_zoom;
        camera.offset_x = cursor.x - vp_cx - img_x * new_zoom;
        camera.offset_y = cursor.y - vp_cy - img_y * new_zoom;
    }

    pub fn pan(camera: &mut Camera, dx: f32, dy: f32) {
        camera.offset_x += dx;
        camera.offset_y += dy;
    }

    pub fn rotate_right(camera: &mut Camera) {
        camera.rotation.rotate_clockwise_90();
    }

    pub fn rotate_left(camera: &mut Camera) {
        camera.rotation.rotate_counter_clockwise_90();
    }

    pub fn flip_horizontal(camera: &mut Camera) {
        camera.flip.0 = !camera.flip.0;
    }

    pub fn flip_vertical(camera: &mut Camera) {
        camera.flip.1 = !camera.flip.1;
    }

    pub fn fit_to_window(camera: &mut Camera, viewport: Size, image_size: Size) {
        if image_size.width == 0.0 || image_size.height == 0.0 {
            return;
        }
        let (vis_w, vis_h) = if camera.rotation.is_perpendicular() {
            (image_size.height, image_size.width)
        } else {
            (image_size.width, image_size.height)
        };
        camera.zoom = (viewport.width / vis_w).min(viewport.height / vis_h);
        camera.offset_x = 0.0;
        camera.offset_y = 0.0;
    }

    pub fn actual_size(camera: &mut Camera) {
        camera.zoom = 1.0;
        camera.offset_x = 0.0;
        camera.offset_y = 0.0;
    }

    pub fn reset(camera: &mut Camera) {
        camera.zoom = 1.0;
        camera.offset_x = 0.0;
        camera.offset_y = 0.0;
        camera.rotation = Angle::ZERO;
        camera.flip = (false, false);
    }

    pub fn double_click_zoom(camera: &mut Camera, viewport: Size, image_size: Size) {
        let (vis_w, vis_h) = if camera.rotation.is_perpendicular() {
            (image_size.height, image_size.width)
        } else {
            (image_size.width, image_size.height)
        };
        let fit_zoom = (viewport.width / vis_w).min(viewport.height / vis_h);
        if (camera.zoom - fit_zoom).abs() < 0.01 {
            camera.zoom = 1.0;
        } else if (camera.zoom - 1.0).abs() < 0.01 {
            camera.zoom = 2.0;
        } else {
            Self::fit_to_window(camera, viewport, image_size);
        }
        camera.offset_x = 0.0;
        camera.offset_y = 0.0;
    }

    pub fn apply(action: ViewerAction, camera: &mut Camera, viewport: Size, image_size: Size) {
        match action {
            ViewerAction::Zoom { factor, cursor } => {
                Self::zoom(camera, factor, cursor, viewport);
            }
            ViewerAction::Pan { dx, dy } => {
                Self::pan(camera, dx, dy);
            }
            ViewerAction::RotateRight => {
                Self::rotate_right(camera);
            }
            ViewerAction::RotateLeft => {
                Self::rotate_left(camera);
            }
            ViewerAction::FlipHorizontal => {
                Self::flip_horizontal(camera);
            }
            ViewerAction::FlipVertical => {
                Self::flip_vertical(camera);
            }
            ViewerAction::FitToWindow => {
                Self::fit_to_window(camera, viewport, image_size);
            }
            ViewerAction::ActualSize => {
                Self::actual_size(camera);
            }
            ViewerAction::Reset => {
                Self::reset(camera);
            }
            ViewerAction::DoubleClickZoom => {
                Self::double_click_zoom(camera, viewport, image_size);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_controller_prepare_camera_for_zoom() {
        let mut camera = Camera::new();
        let viewport = Size::new(1000.0, 500.0);
        ViewerController::prepare_camera_for_zoom(&mut camera, 2000, 1000, viewport);
        // visual_w = 2000, visual_h = 1000. fit = min(1000/2000, 500/1000) = 0.5
        assert!((camera.zoom - 0.5).abs() < f32::EPSILON);

        // If zoom is already non-default or offset is non-zero, it should NOT reset zoom
        camera.zoom = 1.2;
        ViewerController::prepare_camera_for_zoom(&mut camera, 2000, 1000, viewport);
        assert!((camera.zoom - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_viewer_controller_rotate_and_flip() {
        let mut camera = Camera::new();
        assert_eq!(camera.rotation, Angle::ZERO);
        assert_eq!(camera.flip, (false, false));

        ViewerController::rotate_right(&mut camera);
        assert_eq!(camera.rotation.to_degrees().round(), 90.0);

        ViewerController::rotate_left(&mut camera);
        assert_eq!(camera.rotation.to_degrees().round(), 0.0);

        ViewerController::flip_horizontal(&mut camera);
        assert_eq!(camera.flip, (true, false));

        ViewerController::flip_vertical(&mut camera);
        assert_eq!(camera.flip, (true, true));

        ViewerController::reset(&mut camera);
        assert_eq!(camera.rotation, Angle::ZERO);
        assert_eq!(camera.flip, (false, false));
    }
}
