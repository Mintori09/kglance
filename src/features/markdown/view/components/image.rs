use super::style::STYLE;
use crate::app::{KglanceApp, Message};
use crate::core::PreviewData;
use crate::features::image::{png_to_rgba_handle, png_to_rgba_handle_with_size};
use crate::features::markdown::Block;
use crate::log_debug;
use iced::widget::{button, column, container, image};
use iced::{Border, Color, Element, Length, Shadow, Task};

pub(crate) fn render_inline_image<'a>(
    index: usize,
    link_url: Option<&'a str>,
    state: &'a crate::core::MarkdownState,
) -> Element<'a, Message> {
    let Some(handle) = state.cached_image_handles.get(&index) else {
        return container(column![]).into();
    };

    let img = image(handle.clone()).height(Length::Shrink);
    let img = match state.cached_image_sizes.get(&index) {
        Some((width, _)) if *width as f32 > STYLE.image.max_width => {
            img.width(Length::Fixed(STYLE.image.max_width))
        }
        _ => img.width(Length::Shrink),
    };

    let content: Element<'a, Message> = if let Some(url) = link_url {
        let url_clone = url.to_string();
        button(img)
            .on_press(crate::app::messages::SystemMsg::OpenLink(url_clone).into())
            .style(
                |_theme: &iced::Theme, _status: button::Status| button::Style {
                    background: None,
                    text_color: Color::TRANSPARENT,
                    border: Border {
                        width: 0.0,
                        color: Color::TRANSPARENT,
                        radius: 0.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                },
            )
            .padding(0)
            .into()
    } else {
        img.into()
    };

    container(content)
        .center_x(Length::Fill)
        .padding(STYLE.image.padding)
        .width(Length::Fill)
        .into()
}

pub fn handle_mermaid_rendered(
    app: &mut KglanceApp,
    generation_id: usize,
    index: usize,
    png_bytes: Option<Vec<u8>>,
) -> Task<Message> {
    if generation_id
        != app
            .state
            .markdown
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed)
    {
        log_debug!(
            "Discarding stale MermaidBlockRendered[{}] (msg gen: {}, current gen: {})",
            index,
            generation_id,
            app.state
                .markdown
                .generation_id
                .load(std::sync::atomic::Ordering::Relaxed)
        );
        return Task::none();
    }

    log_debug!(
        "MermaidBlockRendered[{}] png={}",
        index,
        if png_bytes.is_some() { "Some" } else { "None" }
    );
    if let Some(PreviewData::Markdown { blocks, .. }) = app.current_content.as_mut()
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
    generation_id: usize,
    index: usize,
    png_bytes: Option<Vec<u8>>,
) -> Task<Message> {
    if generation_id
        != app
            .state
            .markdown
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed)
    {
        log_debug!(
            "Discarding stale MarkdownImageLoaded[{}] (msg gen: {}, current gen: {})",
            index,
            generation_id,
            app.state
                .markdown
                .generation_id
                .load(std::sync::atomic::Ordering::Relaxed)
        );
        return Task::none();
    }

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
            if let Some(PreviewData::Markdown { ref blocks, .. }) = app.current_content {
                let content_width = app.state.markdown_content_width();
                crate::features::markdown::recompute_markdown_layout(
                    &mut app.state.markdown,
                    blocks,
                    app.state.font_size,
                    content_width,
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
