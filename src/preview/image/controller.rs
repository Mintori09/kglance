use iced::{Point, Size};

use crate::preview::image::camera::Camera;

#[derive(Debug, Clone, Copy)]
pub enum ViewerAction {
    Zoom { factor: f32, cursor: Point },
    Pan { dx: f32, dy: f32 },
    FitToWindow,
    ActualSize,
    Reset,
    DoubleClickZoom,
}

pub struct ViewerController;

impl ViewerController {
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

    pub fn fit_to_window(camera: &mut Camera, viewport: Size, image_size: Size) {
        if image_size.width == 0.0 || image_size.height == 0.0 {
            return;
        }
        camera.zoom = (viewport.width / image_size.width).min(viewport.height / image_size.height);
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
    }

    pub fn double_click_zoom(camera: &mut Camera, viewport: Size, image_size: Size) {
        let fit_zoom = (viewport.width / image_size.width).min(viewport.height / image_size.height);
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
