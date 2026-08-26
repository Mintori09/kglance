use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::PreviewData;
use crate::log_debug;
use iced::Task;

use std::sync::atomic::Ordering;

pub fn png_to_rgba_handle(png: Vec<u8>) -> Option<iced::widget::image::Handle> {
    match image::load_from_memory(&png) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(iced::widget::image::Handle::from_rgba(
                width,
                height,
                rgba.into_raw(),
            ))
        }
        Err(_) => {
            log_debug!("image::load_from_memory failed, falling back to Handle::from_bytes");
            Some(iced::widget::image::Handle::from_bytes(png))
        }
    }
}

pub fn png_to_rgba_handle_with_size(
    png: Vec<u8>,
) -> Option<(iced::widget::image::Handle, u32, u32)> {
    match image::load_from_memory(&png) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some((
                iced::widget::image::Handle::from_rgba(width, height, rgba.into_raw()),
                width,
                height,
            ))
        }
        Err(_) => {
            log_debug!("image::load_from_memory failed, falling back to Handle::from_bytes");
            let handle = iced::widget::image::Handle::from_bytes(png);
            Some((handle, 0, 0))
        }
    }
}

pub fn spawn_decode_task(
    bytes: Vec<u8>,
    load_id_val: u64,
    load_id_arc: std::sync::Arc<std::sync::atomic::AtomicU64>,
) -> Task<Message> {
    Task::perform(
        async move {
            if load_id_arc.load(Ordering::Relaxed) != load_id_val {
                return None;
            }

            let arc2 = load_id_arc.clone();
            let result = tokio::task::spawn_blocking(move || {
                if arc2.load(Ordering::Relaxed) != load_id_val {
                    return None;
                }

                let (width, height) = image::ImageReader::new(std::io::Cursor::new(&bytes))
                    .with_guessed_format()
                    .ok()
                    .and_then(|r| r.into_dimensions().ok())
                    .unwrap_or((0, 0));

                let handle = iced::widget::image::Handle::from_bytes(bytes);

                if arc2.load(Ordering::Relaxed) != load_id_val {
                    return None;
                }

                Some((handle, width, height))
            })
            .await
            .ok()
            .flatten()?;

            Some(result)
        },
        move |opt| match opt {
            Some((handle, width, height)) => crate::app::messages::ImageMsg::Decoded {
                load_id: load_id_val,
                handle,
                width,
                height,
            }
            .into(),
            None => Message::None,
        },
    )
}

pub fn handle_decoded(
    app: &mut KglanceApp,
    load_id: u64,
    handle: iced::widget::image::Handle,
    width: u32,
    height: u32,
) -> Task<Message> {
    let current = app.state.image.load_id.load(Ordering::Relaxed);
    if load_id != current {
        log_debug!(
            "Dropping stale ImageDecoded (task={}, current={})",
            load_id,
            current,
        );
        return Task::none();
    }

    app.state.image.handle = Some(handle.clone());
    app.state.image.load_state = crate::features::image::ImageLoadState::Ready;

    app.state.image.display_handle = Some(handle.clone());
    app.state.image.display_width = width;
    app.state.image.display_height = height;

    app.state.image.camera = crate::features::image::Camera::new();

    if !app.state.file_name.is_empty() {
        app.state.cache.put(
            app.state.file_name.clone(),
            crate::core::CachedContent::DecodedImage {
                handle,
                width,
                height,
            },
        );
    }

    let _ = std::mem::take(&mut app.state.image.image_bytes);

    Task::none()
}

pub fn handle_zoom(app: &mut KglanceApp, factor: f32, cursor: iced::Point) -> Task<Message> {
    use crate::features::image::ViewerController;

    let camera = &mut app.state.image.camera;
    let win = app.state.current_window_size;

    if (camera.zoom - 1.0).abs() < f32::EPSILON
        && camera.offset_x == 0.0
        && camera.offset_y == 0.0
        && app.state.image.display_width > 0
        && app.state.image.display_height > 0
    {
        let fit = (win.width / app.state.image.display_width as f32)
            .min(win.height / app.state.image.display_height as f32)
            .min(1.0);
        camera.zoom = fit;
    }

    let viewport = iced::Size::new(win.width, win.height);
    ViewerController::zoom(camera, factor, cursor, viewport);
    Task::none()
}

pub fn handle_pan(app: &mut KglanceApp, dx: f32, dy: f32) -> Task<Message> {
    use crate::features::image::ViewerController;
    ViewerController::pan(&mut app.state.image.camera, dx, dy);
    Task::none()
}

pub fn handle_fit_to_window(app: &mut KglanceApp) -> Task<Message> {
    use crate::features::image::ViewerController;
    let win = app.state.current_window_size;
    let viewport = iced::Size::new(win.width, win.height);
    let image_size = iced::Size::new(
        app.state.image.display_width as f32,
        app.state.image.display_height as f32,
    );
    ViewerController::fit_to_window(&mut app.state.image.camera, viewport, image_size);
    Task::none()
}

pub fn handle_double_click(app: &mut KglanceApp) -> Task<Message> {
    use crate::features::image::ViewerController;
    ViewerController::reset(&mut app.state.image.camera);
    Task::none()
}

pub fn handle_video_thumbnail_loaded(app: &mut KglanceApp, data: Vec<u8>) -> Task<Message> {
    if let Some(PreviewData::Media {
        ref mut thumbnail_or_waveform,
        ..
    }) = app.current_content
    {
        *thumbnail_or_waveform = data;
    }
    Task::none()
}
