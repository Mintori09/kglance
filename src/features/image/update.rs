use crate::app::KglanceApp;
use crate::app::messages::Message;
use crate::core::PreviewData;
use crate::features::image::ViewerController;
use crate::log_debug;
use iced::Task;

use std::sync::OnceLock;
use std::sync::atomic::Ordering;
use tokio::sync::Semaphore;

pub fn decode_semaphore() -> &'static Semaphore {
    static SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();
    SEMAPHORE.get_or_init(|| Semaphore::new(2))
}

pub const MAX_PREVIEW_WIDTH: u32 = 1920;
pub const MAX_PREVIEW_HEIGHT: u32 = 1080;

pub fn calculate_fit_dimensions(width: u32, height: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (width, height);
    }
    if width <= max_w && height <= max_h {
        return (width, height);
    }
    let scale_w = max_w as f32 / width as f32;
    let scale_h = max_h as f32 / height as f32;
    let scale = scale_w.min(scale_h);
    let new_w = ((width as f32 * scale).round() as u32).max(1);
    let new_h = ((height as f32 * scale).round() as u32).max(1);
    (new_w, new_h)
}

pub const MAX_HARDWARE_TEXTURE_SIZE: u32 = 8192;

pub fn fast_downsample_rgba(
    src: &[u8],
    orig_w: u32,
    orig_h: u32,
    target_w: u32,
    target_h: u32,
) -> Vec<u8> {
    if target_w == 0 || target_h == 0 || orig_w == 0 || orig_h == 0 {
        return Vec::new();
    }
    let orig_w_usize = orig_w as usize;
    let orig_h_usize = orig_h as usize;
    let target_w_usize = target_w as usize;
    let target_h_usize = target_h as usize;

    let expected_src_len = orig_w_usize.saturating_mul(orig_h_usize).saturating_mul(4);
    if src.len() < expected_src_len {
        return Vec::new();
    }

    if orig_w == target_w && orig_h == target_h {
        return src[..expected_src_len].to_vec();
    }

    let mut dest = vec![0u8; target_w_usize * target_h_usize * 4];

    #[derive(Clone, Copy)]
    struct SamplePoints {
        pts: [usize; 4],
        len: usize,
    }

    impl SamplePoints {
        #[inline(always)]
        fn from_range(start: usize, end: usize, stride: usize) -> Self {
            let span = end - start;
            if span <= 1 {
                Self {
                    pts: [start * stride, 0, 0, 0],
                    len: 1,
                }
            } else if span == 2 {
                Self {
                    pts: [start * stride, (start + 1) * stride, 0, 0],
                    len: 2,
                }
            } else if span == 3 {
                Self {
                    pts: [
                        start * stride,
                        (start + 1) * stride,
                        (start + 2) * stride,
                        0,
                    ],
                    len: 3,
                }
            } else {
                Self {
                    pts: [
                        start * stride,
                        (start + span / 3) * stride,
                        (start + (2 * span) / 3) * stride,
                        (end - 1) * stride,
                    ],
                    len: 4,
                }
            }
        }
    }

    // Precalculate horizontal mapping coordinates
    let mut x_lut = Vec::with_capacity(target_w_usize);
    for out_x in 0..target_w_usize {
        let x_start = (out_x * orig_w_usize) / target_w_usize;
        let x_end = (((out_x + 1) * orig_w_usize) / target_w_usize)
            .max(x_start + 1)
            .min(orig_w_usize);
        x_lut.push(SamplePoints::from_range(x_start, x_end, 4));
    }

    // Precalculate vertical mapping coordinates
    let mut y_lut = Vec::with_capacity(target_h_usize);
    for out_y in 0..target_h_usize {
        let y_start = (out_y * orig_h_usize) / target_h_usize;
        let y_end = (((out_y + 1) * orig_h_usize) / target_h_usize)
            .max(y_start + 1)
            .min(orig_h_usize);
        y_lut.push(SamplePoints::from_range(y_start, y_end, orig_w_usize * 4));
    }

    for (out_y, y_samples) in y_lut.iter().enumerate() {
        let dest_row_offset = out_y * target_w_usize * 4;
        let dest_row = &mut dest[dest_row_offset..dest_row_offset + target_w_usize * 4];
        let (dest_chunks, _) = dest_row.as_chunks_mut::<4>();

        let y_len = y_samples.len;
        let y_pts = y_samples.pts;

        for (out_px, x_samples) in dest_chunks.iter_mut().zip(x_lut.iter()) {
            let mut r_acc: u32 = 0;
            let mut g_acc: u32 = 0;
            let mut b_acc: u32 = 0;
            let mut a_acc: u32 = 0;

            let x_len = x_samples.len;
            let x_pts = x_samples.pts;

            let mut yi = 0;
            while yi < y_len {
                let y_base = y_pts[yi];
                let mut xi = 0;
                while xi < x_len {
                    let base = y_base + x_pts[xi];
                    r_acc += src[base] as u32;
                    g_acc += src[base + 1] as u32;
                    b_acc += src[base + 2] as u32;
                    a_acc += src[base + 3] as u32;
                    xi += 1;
                }
                yi += 1;
            }

            let count = (y_len * x_len) as u32;
            let half = count / 2;
            out_px[0] = ((r_acc + half) / count) as u8;
            out_px[1] = ((g_acc + half) / count) as u8;
            out_px[2] = ((b_acc + half) / count) as u8;
            out_px[3] = ((a_acc + half) / count) as u8;
        }
    }

    dest
}

pub fn decode_to_rgba_full(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if bytes.is_empty() {
        return None;
    }

    // Fast path: SIMD accelerated JPEG decoding via zune-jpeg
    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        let options = zune_jpeg::zune_core::options::DecoderOptions::default()
            .jpeg_set_out_colorspace(zune_jpeg::zune_core::colorspace::ColorSpace::RGBA);
        let mut decoder =
            zune_jpeg::JpegDecoder::new_with_options(std::io::Cursor::new(bytes), options);
        if let Ok(pixels) = decoder.decode()
            && let Some((w, h)) = decoder.dimensions()
        {
            let (orig_w, orig_h) = (w as u32, h as u32);
            if orig_w > MAX_HARDWARE_TEXTURE_SIZE || orig_h > MAX_HARDWARE_TEXTURE_SIZE {
                let (target_w, target_h) = calculate_fit_dimensions(
                    orig_w,
                    orig_h,
                    MAX_HARDWARE_TEXTURE_SIZE,
                    MAX_HARDWARE_TEXTURE_SIZE,
                );
                let downsampled = fast_downsample_rgba(&pixels, orig_w, orig_h, target_w, target_h);
                return Some((target_w, target_h, downsampled));
            }
            return Some((orig_w, orig_h, pixels));
        }
    }

    // Fallback path: image crate (PNG, WebP, GIF, BMP, or JPEG fallback)
    let img = image::load_from_memory(bytes).ok()?;
    let (orig_w, orig_h) = (img.width(), img.height());
    let raw = img.to_rgba8().into_raw();
    if orig_w > MAX_HARDWARE_TEXTURE_SIZE || orig_h > MAX_HARDWARE_TEXTURE_SIZE {
        let (target_w, target_h) = calculate_fit_dimensions(
            orig_w,
            orig_h,
            MAX_HARDWARE_TEXTURE_SIZE,
            MAX_HARDWARE_TEXTURE_SIZE,
        );
        let downsampled = fast_downsample_rgba(&raw, orig_w, orig_h, target_w, target_h);
        Some((target_w, target_h, downsampled))
    } else {
        Some((orig_w, orig_h, raw))
    }
}

pub fn decode_to_rgba_downsampled(
    bytes: &[u8],
    max_w: u32,
    max_h: u32,
) -> Option<(u32, u32, Vec<u8>)> {
    let (orig_w, orig_h, pixels) = decode_to_rgba_full(bytes)?;
    let (target_w, target_h) = calculate_fit_dimensions(orig_w, orig_h, max_w, max_h);
    if target_w == orig_w && target_h == orig_h {
        Some((orig_w, orig_h, pixels))
    } else {
        let downsampled = fast_downsample_rgba(&pixels, orig_w, orig_h, target_w, target_h);
        Some((target_w, target_h, downsampled))
    }
}

pub fn decode_to_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    decode_to_rgba_full(bytes)
}

pub fn decode_to_rgba_handle_downsampled(
    bytes: &[u8],
    max_w: u32,
    max_h: u32,
) -> Option<(iced::widget::image::Handle, u32, u32)> {
    let (w, h, pixels) = decode_to_rgba_downsampled(bytes, max_w, max_h)?;
    let handle = iced::widget::image::Handle::from_rgba(w, h, pixels);
    Some((handle, w, h))
}

pub fn decode_to_rgba_handle(bytes: &[u8]) -> Option<(iced::widget::image::Handle, u32, u32)> {
    let (w, h, pixels) = decode_to_rgba_full(bytes)?;
    let handle = iced::widget::image::Handle::from_rgba(w, h, pixels);
    Some((handle, w, h))
}

pub fn png_to_rgba_handle(png: Vec<u8>) -> Option<iced::widget::image::Handle> {
    decode_to_rgba_handle(&png)
        .map(|(handle, _, _)| handle)
        .or_else(|| {
            log_debug!("decode_to_rgba failed, falling back to Handle::from_bytes");
            Some(iced::widget::image::Handle::from_bytes(png))
        })
}

pub fn png_to_rgba_handle_with_size(
    png: Vec<u8>,
) -> Option<(iced::widget::image::Handle, u32, u32)> {
    decode_to_rgba_handle(&png).or_else(|| {
        log_debug!("decode_to_rgba failed, falling back to Handle::from_bytes");
        let handle = iced::widget::image::Handle::from_bytes(png);
        Some((handle, 0, 0))
    })
}

pub fn spawn_decode_task(
    bytes: Vec<u8>,
    load_id_val: u64,
    load_id_arc: std::sync::Arc<std::sync::atomic::AtomicU64>,
) -> Task<Message> {
    spawn_decode_task_delayed(bytes, load_id_val, load_id_arc, 0)
}

pub fn spawn_decode_task_delayed(
    bytes: Vec<u8>,
    load_id_val: u64,
    load_id_arc: std::sync::Arc<std::sync::atomic::AtomicU64>,
    delay_ms: u64,
) -> Task<Message> {
    Task::perform(
        async move {
            if delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            if load_id_arc.load(Ordering::Relaxed) != load_id_val {
                return None;
            }

            let _permit = decode_semaphore().acquire().await.ok();

            if load_id_arc.load(Ordering::Relaxed) != load_id_val {
                return None;
            }

            let arc2 = load_id_arc.clone();
            let raw_bytes = bytes;

            tokio::task::spawn_blocking(move || {
                if arc2.load(Ordering::Relaxed) != load_id_val {
                    return None;
                }

                let (orig_w, orig_h, full_pixels) = match decode_to_rgba_full(&raw_bytes) {
                    Some(res) => res,
                    None => {
                        if arc2.load(Ordering::Relaxed) != load_id_val {
                            return None;
                        }
                        let (w, h) = image::ImageReader::new(std::io::Cursor::new(&raw_bytes))
                            .with_guessed_format()
                            .ok()
                            .and_then(|r| r.into_dimensions().ok())
                            .unwrap_or((0, 0));
                        let handle = iced::widget::image::Handle::from_bytes(raw_bytes);
                        return Some((handle, None, w, h));
                    }
                };

                if arc2.load(Ordering::Relaxed) != load_id_val {
                    return None;
                }

                let preview_info = if orig_w > MAX_PREVIEW_WIDTH || orig_h > MAX_PREVIEW_HEIGHT {
                    let (target_w, target_h) = calculate_fit_dimensions(
                        orig_w,
                        orig_h,
                        MAX_PREVIEW_WIDTH,
                        MAX_PREVIEW_HEIGHT,
                    );
                    let downsampled =
                        fast_downsample_rgba(&full_pixels, orig_w, orig_h, target_w, target_h);
                    let handle =
                        iced::widget::image::Handle::from_rgba(target_w, target_h, downsampled);
                    Some((handle, target_w, target_h))
                } else {
                    None
                };

                if arc2.load(Ordering::Relaxed) != load_id_val {
                    return None;
                }

                let full_handle =
                    iced::widget::image::Handle::from_rgba(orig_w, orig_h, full_pixels);
                Some((full_handle, preview_info, orig_w, orig_h))
            })
            .await
            .ok()
            .flatten()
        },
        move |opt| match opt {
            Some((full_handle, preview_info, width, height)) => {
                crate::app::messages::ImageMsg::Decoded {
                    load_id: load_id_val,
                    handle: full_handle,
                    cached_preview: preview_info,
                    width,
                    height,
                }
                .into()
            }
            None => Message::None,
        },
    )
}

pub fn handle_decoded(
    app: &mut KglanceApp,
    load_id: u64,
    handle: iced::widget::image::Handle,
    cached_preview: Option<(iced::widget::image::Handle, u32, u32)>,
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

    if let Some((ref prev_h, _, _)) = cached_preview
        && app.state.image.preview_handle.is_none()
    {
        app.state.image.preview_handle = Some(prev_h.clone());
    }

    // Cache the lightweight ~8MB preview in LRU cache so subsequent visits/navigation are instant (<3ms)
    // without consuming 100MB+ per image!
    if !app.state.file_name.is_empty() {
        let preview = app
            .current_content
            .as_ref()
            .map(|c| std::sync::Arc::new(c.clone()));

        let (cache_handle, cache_w, cache_h) =
            if let Some((prev_h, prev_w, prev_h_val)) = cached_preview {
                (prev_h, prev_w, prev_h_val)
            } else {
                (handle, width, height)
            };

        app.state.cache.put(
            app.state.file_name.clone(),
            crate::core::CachedContent::DecodedImage {
                preview,
                handle: cache_handle,
                width: cache_w,
                height: cache_h,
            },
        );
    }

    let _ = std::mem::take(&mut app.state.image.image_bytes);

    Task::none()
}

pub const FOOTER_HEIGHT: f32 = 50.0;

pub fn handle_zoom(app: &mut KglanceApp, factor: f32, cursor: iced::Point) -> Task<Message> {
    let camera = &mut app.state.image.camera;
    let win = app.state.current_window_size;
    let canvas_height = (win.height - FOOTER_HEIGHT).max(1.0);

    if (camera.zoom - 1.0).abs() < f32::EPSILON
        && camera.offset_x == 0.0
        && camera.offset_y == 0.0
        && app.state.image.display_width > 0
        && app.state.image.display_height > 0
    {
        let fit = (win.width / app.state.image.display_width as f32)
            .min(canvas_height / app.state.image.display_height as f32);
        camera.zoom = fit;
    }

    let viewport = iced::Size::new(win.width, canvas_height);
    ViewerController::zoom(camera, factor, cursor, viewport);
    Task::none()
}

pub fn handle_pan(app: &mut KglanceApp, dx: f32, dy: f32) -> Task<Message> {
    ViewerController::pan(&mut app.state.image.camera, dx, dy);
    Task::none()
}

pub fn handle_fit_to_window(app: &mut KglanceApp) -> Task<Message> {
    let win = app.state.current_window_size;
    let canvas_height = (win.height - FOOTER_HEIGHT).max(1.0);
    let viewport = iced::Size::new(win.width, canvas_height);
    let image_size = iced::Size::new(
        app.state.image.display_width as f32,
        app.state.image.display_height as f32,
    );
    ViewerController::fit_to_window(&mut app.state.image.camera, viewport, image_size);
    Task::none()
}

pub fn handle_double_click(app: &mut KglanceApp) -> Task<Message> {
    ViewerController::reset(&mut app.state.image.camera);
    Task::none()
}

pub fn handle_toggle_info(app: &mut KglanceApp) -> Task<Message> {
    if !app.state.image.exif_content.is_empty() {
        app.state.image.show_info = !app.state.image.show_info;
    }
    Task::none()
}

pub fn handle_close_info(app: &mut KglanceApp) -> Task<Message> {
    app.state.image.show_info = false;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fit_dimensions() {
        let (w, h) = calculate_fit_dimensions(3840, 2160, 1920, 1080);
        assert_eq!((w, h), (1920, 1080));

        let (w2, h2) = calculate_fit_dimensions(800, 600, 1920, 1080);
        assert_eq!((w2, h2), (800, 600));

        let (w3, h3) = calculate_fit_dimensions(0, 0, 1920, 1080);
        assert_eq!((w3, h3), (0, 0));
    }

    #[test]
    fn test_fast_downsample_rgba() {
        let orig_w = 4;
        let orig_h = 4;
        let mut src = vec![0u8; (orig_w * orig_h * 4) as usize];
        for chunk in src.as_chunks_mut::<4>().0 {
            chunk[0] = 255;
            chunk[3] = 255;
        }

        let downsampled = fast_downsample_rgba(&src, orig_w, orig_h, 2, 2);
        assert_eq!(downsampled.len(), (2 * 2 * 4) as usize);
        for chunk in downsampled.as_chunks::<4>().0 {
            assert_eq!(chunk[0], 255);
            assert_eq!(chunk[1], 0);
            assert_eq!(chunk[2], 0);
            assert_eq!(chunk[3], 255);
        }
    }
}
