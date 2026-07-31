use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::PreviewData;
use crate::log_debug;
use crate::parsers::markdown::Block;
use iced::Task;

fn png_to_rgba_handle(png: Vec<u8>) -> Option<iced::widget::image::Handle> {
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

fn png_to_rgba_handle_with_size(png: Vec<u8>) -> Option<(iced::widget::image::Handle, u32, u32)> {
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

pub fn handle_zoom(app: &mut KglanceApp, delta: f32) -> Task<Message> {
    app.handle_image_zoom(delta)
}

pub fn handle_pan(app: &mut KglanceApp, dx: f32, dy: f32) -> Task<Message> {
    use crate::preview::image::ViewerController;
    ViewerController::pan(&mut app.state.image.camera, dx, dy);
    Task::none()
}

pub fn handle_double_click(app: &mut KglanceApp) -> Task<Message> {
    use crate::preview::image::ViewerController;
    ViewerController::reset(&mut app.state.image.camera);
    Task::none()
}

pub fn handle_mermaid_rendered(
    app: &mut KglanceApp,
    index: usize,
    png_bytes: Option<Vec<u8>>,
) -> Task<Message> {
    log_debug!(
        "MermaidBlockRendered[{}] png={}",
        index,
        if png_bytes.is_some() { "Some" } else { "None" }
    );
    if let Some(PreviewData::Markdown { blocks }) = app.current_content.as_mut()
        && let Some(Block::Mermaid { rendered, .. }) = blocks.get_mut(index)
    {
        *rendered = png_bytes.clone();
    }
    if let Some(png) = png_bytes {
        if let Some(handle) = png_to_rgba_handle(png) {
            app.state
                .markdown
                .cached_mermaid_handles
                .insert(index, handle);
            log_debug!(
                "Inserted handle at index {}, cache size={}",
                index,
                app.state.markdown.cached_mermaid_handles.len()
            );
        } else {
            log_debug!("png_to_rgba_handle returned None for block[{}]", index);
        }
    }
    Task::none()
}

pub fn handle_markdown_image_loaded(
    app: &mut KglanceApp,
    index: usize,
    png_bytes: Option<Vec<u8>>,
) -> Task<Message> {
    log_debug!(
        "MarkdownImageLoaded[{}] bytes={}",
        index,
        if png_bytes.is_some() { "Some" } else { "None" }
    );
    if let Some(bytes) = png_bytes {
        if let Some((handle, w, h)) = png_to_rgba_handle_with_size(bytes) {
            app.state
                .markdown
                .cached_image_handles
                .insert(index, handle);
            app.state.markdown.cached_image_sizes.insert(index, (w, h));
            if let Some(PreviewData::Markdown { ref blocks }) = app.current_content {
                app.state.markdown.toc = crate::parsers::markdown::extract_toc(
                    blocks,
                    app.state.font_size,
                    &app.state.markdown.cached_image_sizes,
                );
            }
            log_debug!(
                "Inserted image handle at index {}, size={}x{}, cache size={}",
                index,
                w,
                h,
                app.state.markdown.cached_image_handles.len()
            );
        } else {
            log_debug!(
                "png_to_rgba_handle returned None for image block[{}]",
                index
            );
        }
    }
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
