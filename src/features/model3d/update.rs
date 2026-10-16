use crate::app::KglanceApp;
use crate::app::messages::Model3dMsg;
use iced::Task;

pub fn handle_model3d_message(app: &mut KglanceApp, msg: Model3dMsg) -> Task<crate::app::Message> {
    match msg {
        Model3dMsg::Rotate {
            delta_yaw,
            delta_pitch,
        } => {
            app.state.model3d.yaw += delta_yaw;
            app.state.model3d.pitch = (app.state.model3d.pitch + delta_pitch).clamp(
                -std::f32::consts::FRAC_PI_2 + 0.05,
                std::f32::consts::FRAC_PI_2 - 0.05,
            );
            Task::none()
        }
        Model3dMsg::Zoom(factor) => {
            app.state.model3d.distance = (app.state.model3d.distance * factor).clamp(
                app.state.model3d.base_distance * 0.1,
                app.state.model3d.base_distance * 10.0,
            );
            Task::none()
        }
        Model3dMsg::DragStarted(point) => {
            app.state.model3d.is_dragging = true;
            app.state.model3d.last_cursor = Some(point);
            Task::none()
        }
        Model3dMsg::DragMoved(point) => {
            if app.state.model3d.is_dragging {
                if let Some(last) = app.state.model3d.last_cursor {
                    let dx = point.x - last.x;
                    let dy = point.y - last.y;
                    app.state.model3d.yaw += dx * 0.01;
                    app.state.model3d.pitch = (app.state.model3d.pitch - dy * 0.01).clamp(
                        -std::f32::consts::FRAC_PI_2 + 0.05,
                        std::f32::consts::FRAC_PI_2 - 0.05,
                    );
                }
                app.state.model3d.last_cursor = Some(point);
            }
            Task::none()
        }
        Model3dMsg::DragEnded => {
            app.state.model3d.is_dragging = false;
            app.state.model3d.last_cursor = None;
            Task::none()
        }
        Model3dMsg::ResetCamera => {
            app.state.model3d.reset_camera();
            Task::none()
        }
        Model3dMsg::ToggleShading => {
            app.state.model3d.shading = match app.state.model3d.shading {
                crate::features::model3d::types::ShadingMode::Smooth => {
                    crate::features::model3d::types::ShadingMode::Flat
                }
                crate::features::model3d::types::ShadingMode::Flat => {
                    crate::features::model3d::types::ShadingMode::Wireframe
                }
                crate::features::model3d::types::ShadingMode::Wireframe => {
                    crate::features::model3d::types::ShadingMode::Smooth
                }
            };
            Task::none()
        }
        Model3dMsg::ToggleAutoRotate => {
            app.state.model3d.auto_rotate = !app.state.model3d.auto_rotate;
            Task::none()
        }
        Model3dMsg::ToggleGrid => {
            app.state.model3d.show_grid = !app.state.model3d.show_grid;
            Task::none()
        }
        Model3dMsg::Tick => {
            if app.state.model3d.auto_rotate && !app.state.model3d.is_dragging {
                app.state.model3d.yaw += 0.01;
            }
            Task::none()
        }
    }
}
