use super::style::STYLE;
use crate::app::{KglanceApp, Message};
use crate::core::PreviewData;
use crate::features::image::png_to_rgba_handle;
use crate::features::markdown::Block;
use crate::log_debug;
use iced::widget::{button, column, container, image};
use iced::{Border, Color, Element, Length, Shadow, Task};

pub(crate) fn render_inline_image<'a>(
    path: &'a str,
    link_url: Option<&'a str>,
    explicit_width: Option<u32>,
    explicit_height: Option<u32>,
    state: &'a crate::core::MarkdownState,
    is_epub: bool,
) -> Element<'a, Message> {
    let Some(handle) = state.cached_image_handles.get(path) else {
        return container(column![]).into();
    };

    let cached_size = state.cached_image_sizes.get(path).copied();
    let (display_w, display_h) =
        crate::parsers::markdown::layout_constants::compute_image_display_size(
            cached_size,
            explicit_width,
            explicit_height,
            STYLE.image.max_width,
        );

    let img = image(handle.clone());
    let img = match display_h {
        Some(h) => img.height(Length::Fixed(h)),
        None => img.height(Length::Shrink),
    };
    let img = match display_w {
        Some(w) => img.width(Length::Fixed(w)),
        None => img.width(Length::Shrink),
    };

    let image_card = container(img).style(move |_theme: &iced::Theme| container::Style {
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            width: 1.0,
            radius: STYLE.image.border_radius.into(),
        },
        ..Default::default()
    });

    let content: Element<'a, Message> = if is_epub {
        let path_clone = path.to_string();
        button(image_card)
            .on_press(crate::app::messages::EpubMsg::OpenImageLightbox(path_clone).into())
            .style(|_theme: &iced::Theme, status: button::Status| {
                let border_color = match status {
                    button::Status::Hovered => Color::from_rgba(0.4, 0.7, 1.0, 0.4),
                    button::Status::Pressed => Color::from_rgba(0.4, 0.7, 1.0, 0.7),
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: None,
                    text_color: Color::TRANSPARENT,
                    border: Border {
                        width: 1.0,
                        color: border_color,
                        radius: STYLE.image.border_radius.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                }
            })
            .padding(0)
            .into()
    } else if let Some(url) = link_url {
        let url_clone = url.to_string();
        button(image_card)
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
        image_card.into()
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
    path: String,
    payload: Option<crate::features::image::DecodedImagePayload>,
) -> Task<Message> {
    let is_epub = matches!(app.current_content, Some(PreviewData::Epub { .. }));
    let target_gen = if is_epub {
        app.state
            .epub
            .markdown_state
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed)
    } else {
        app.state
            .markdown
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed)
    };

    if generation_id != target_gen {
        log_debug!(
            "Discarding stale MarkdownImageLoaded[{}] (msg gen: {}, current gen: {})",
            path,
            generation_id,
            target_gen
        );
        return Task::none();
    }

    log_debug!(
        "MarkdownImageLoaded[{}] payload={}",
        path,
        if payload.is_some() { "Some" } else { "None" }
    );
    if let Some(payload) = payload {
        let (handle, w, h, anim_opt) = (
            payload.handle,
            payload.width,
            payload.height,
            payload.animated_gif,
        );

        let size_changed = if is_epub {
            let old_size = app
                .state
                .epub
                .markdown_state
                .cached_image_sizes
                .get(&path)
                .copied();
            app.state
                .epub
                .markdown_state
                .cached_image_handles
                .insert(path.clone(), handle);
            app.state
                .epub
                .markdown_state
                .cached_image_sizes
                .insert(path.clone(), (w, h));
            if let Some(anim) = anim_opt {
                app.state
                    .epub
                    .markdown_state
                    .animated_gifs
                    .insert(path.clone(), anim);
                if app.state.epub.markdown_state.animation_start_time.is_none() {
                    app.state.epub.markdown_state.animation_start_time =
                        Some(std::time::Instant::now());
                }
            }
            old_size != Some((w, h))
        } else {
            let old_size = app.state.markdown.cached_image_sizes.get(&path).copied();
            app.state
                .markdown
                .cached_image_handles
                .insert(path.clone(), handle);
            app.state
                .markdown
                .cached_image_sizes
                .insert(path.clone(), (w, h));
            if let Some(anim) = anim_opt {
                app.state.markdown.animated_gifs.insert(path.clone(), anim);
                if app.state.markdown.animation_start_time.is_none() {
                    app.state.markdown.animation_start_time = Some(std::time::Instant::now());
                }
            }
            old_size != Some((w, h))
        };

        if size_changed {
            if is_epub {
                let content_width = app.state.epub_content_width();
                let font_size = app.state.font_size;
                let epub = &mut app.state.epub;
                let blocks =
                    if epub.reading_mode == crate::core::config::EpubReadingMode::Continuous {
                        epub.continuous_blocks.as_slice()
                    } else {
                        epub.chapters
                            .get(epub.active_chapter)
                            .map_or([].as_slice(), |c| c.blocks.as_slice())
                    };
                crate::features::markdown::recompute_markdown_layout(
                    &mut epub.markdown_state,
                    blocks,
                    font_size,
                    content_width,
                );
            } else if let Some(PreviewData::Markdown { ref blocks, .. }) = app.current_content {
                let content_width = app.state.markdown_content_width();
                crate::features::markdown::recompute_markdown_layout(
                    &mut app.state.markdown,
                    blocks,
                    app.state.font_size,
                    content_width,
                );
            }
        }
        log_debug!(
            "Inserted image handle for path {}, size={}x{}, cache size={}",
            path,
            w,
            h,
            if is_epub {
                app.state.epub.markdown_state.cached_image_handles.len()
            } else {
                app.state.markdown.cached_image_handles.len()
            }
        );
    }
    Task::none()
}

pub fn handle_markdown_animation_tick(app: &mut KglanceApp) -> Task<Message> {
    let now = std::time::Instant::now();
    let is_epub = matches!(app.current_content, Some(PreviewData::Epub { .. }));
    let md = if is_epub {
        &mut app.state.epub.markdown_state
    } else {
        &mut app.state.markdown
    };

    let Some(start_time) = md.animation_start_time else {
        return Task::none();
    };

    if md.animated_gifs.is_empty() {
        return Task::none();
    }

    let elapsed_ms = now.saturating_duration_since(start_time).as_millis() as u64;

    for (path, anim) in &md.animated_gifs {
        let current_handle = anim.current_frame(elapsed_ms);
        if let Some(cached) = md.cached_image_handles.get_mut(path)
            && cached.id() != current_handle.id()
        {
            *cached = current_handle.clone();
        }
    }

    Task::none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::codecs::gif::GifEncoder;
    use ::image::{Delay, Frame, RgbaImage};

    fn create_test_gif() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut buf);
            let frame1 = Frame::from_parts(
                RgbaImage::new(20, 20),
                0,
                0,
                Delay::from_numer_denom_ms(50, 1),
            );
            let frame2 = Frame::from_parts(
                RgbaImage::new(20, 20),
                0,
                0,
                Delay::from_numer_denom_ms(50, 1),
            );
            encoder.encode_frame(frame1).unwrap();
            encoder.encode_frame(frame2).unwrap();
        }
        buf
    }

    #[test]
    fn test_handle_markdown_image_loaded_animated_gif() {
        let mut app = KglanceApp {
            current_content: Some(PreviewData::Markdown {
                blocks: vec![],
                raw_text: String::new(),
            }),
            ..Default::default()
        };
        let gif_bytes = create_test_gif();
        let payload = crate::features::image::decode_image_bytes(gif_bytes);
        let gen_id = app
            .state
            .markdown
            .generation_id
            .load(std::sync::atomic::Ordering::Relaxed);
        let path = "test_animation.gif".to_string();

        let _ = handle_markdown_image_loaded(&mut app, gen_id, path.clone(), payload);

        assert!(app.state.markdown.cached_image_handles.contains_key(&path));
        assert_eq!(
            app.state.markdown.cached_image_sizes.get(&path),
            Some(&(20, 20))
        );
        assert!(app.state.markdown.animated_gifs.contains_key(&path));
        assert!(app.state.markdown.animation_start_time.is_some());

        let initial_handle_id = app.state.markdown.cached_image_handles[&path].id();

        // Advance animation_start_time so elapsed time points to frame 1 (at 60ms)
        app.state.markdown.animation_start_time =
            Some(std::time::Instant::now() - std::time::Duration::from_millis(60));
        let _ = handle_markdown_animation_tick(&mut app);

        let new_handle_id = app.state.markdown.cached_image_handles[&path].id();
        assert_ne!(
            initial_handle_id, new_handle_id,
            "Handle should advance to next frame on tick"
        );
    }
}
