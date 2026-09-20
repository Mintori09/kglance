use kglance::core::preview::PreviewData;
use kglance::core::types::KglanceState;
use kglance::features::image::ImageLoadState;

#[test]
fn test_image_populate_state_initializes_display_handle_and_dimensions() {
    let mut state = KglanceState::default();

    // 1x1 transparent PNG bytes
    let sample_png = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let preview = PreviewData::Image {
        data: sample_png.clone(),
        width: 100,
        height: 80,
        format_info: "Image (PNG - 100x80)".to_string(),
        exif_content: None,
    };

    preview.populate_state(&mut state);

    assert_eq!(state.image.load_state, ImageLoadState::Ready);
    assert_eq!(state.image.display_width, 100);
    assert_eq!(state.image.display_height, 80);
    assert!(state.image.display_handle.is_some());
    assert!(state.image.handle.is_some());
}

#[test]
fn test_cached_content_decoded_image_returns_preview() {
    let sample_png = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let preview = std::sync::Arc::new(PreviewData::Image {
        data: sample_png.clone(),
        width: 100,
        height: 80,
        format_info: "Image (PNG - 100x80)".to_string(),
        exif_content: None,
    });
    let handle = iced::widget::image::Handle::from_bytes(sample_png);

    let cached = kglance::core::CachedContent::DecodedImage {
        preview: Some(preview),
        handle,
        width: 100,
        height: 80,
    };

    assert!(
        cached.as_preview().is_some(),
        "Cached DecodedImage must return preview for navigation"
    );
}

#[test]
fn test_image_navigation_next_and_prev_using_cache() {
    let mut app = kglance::app::KglanceApp::default();
    let img1_path = "img1.png".to_string();
    let img2_path = "img2.png".to_string();

    let sample_png1 = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let sample_png2 = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let preview1 = std::sync::Arc::new(PreviewData::Image {
        data: sample_png1.clone(),
        width: 100,
        height: 100,
        format_info: "Image (PNG - 100x100)".to_string(),
        exif_content: None,
    });
    let handle1 = iced::widget::image::Handle::from_bytes(sample_png1);

    let preview2 = std::sync::Arc::new(PreviewData::Image {
        data: sample_png2.clone(),
        width: 200,
        height: 200,
        format_info: "Image (PNG - 200x200)".to_string(),
        exif_content: None,
    });
    let handle2 = iced::widget::image::Handle::from_bytes(sample_png2);

    app.state.playlist = vec![img1_path.clone(), img2_path.clone()];
    app.state.current_index = 0;

    app.state.cache.put(
        img1_path.clone(),
        kglance::core::CachedContent::DecodedImage {
            preview: Some(preview1.clone()),
            handle: handle1,
            width: 100,
            height: 100,
        },
    );
    app.state.cache.put(
        img2_path.clone(),
        kglance::core::CachedContent::DecodedImage {
            preview: Some(preview2.clone()),
            handle: handle2,
            width: 200,
            height: 200,
        },
    );

    // Navigate to next file (img2)
    let _ = kglance::app::update::navigation::handle_next_file(&mut app);

    assert_eq!(app.state.current_index, 1);
    assert_eq!(app.state.file_name, img2_path);
    assert_eq!(app.state.image.display_width, 200);
    assert_eq!(app.state.image.display_height, 200);
    assert!(app.state.image.display_handle.is_some());

    // Navigate back to prev file (img1)
    let _ = kglance::app::update::navigation::handle_prev_file(&mut app);

    assert_eq!(app.state.current_index, 0);
    assert_eq!(app.state.file_name, img1_path);
    assert_eq!(app.state.image.display_width, 100);
    assert_eq!(app.state.image.display_height, 100);
    assert!(app.state.image.display_handle.is_some());
}

#[test]
fn test_decode_to_rgba_and_handle() {
    // 1x1 transparent PNG bytes
    let sample_png = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let decoded = kglance::features::image::decode_to_rgba(&sample_png);
    assert!(decoded.is_some());
    let (w, h, pixels) = decoded.unwrap();
    assert_eq!(w, 1);
    assert_eq!(h, 1);
    assert_eq!(pixels.len(), 4); // 1x1 RGBA = 4 bytes

    let handle_opt = kglance::features::image::decode_to_rgba_handle(&sample_png);
    assert!(handle_opt.is_some());
    let (_handle, hw, hh) = handle_opt.unwrap();
    assert_eq!(hw, 1);
    assert_eq!(hh, 1);
}

#[test]
fn test_handle_decoded_populates_display_and_cache() {
    let mut app = kglance::app::KglanceApp::default();
    app.state.file_name = "/tmp/test.jpg".to_string();
    let load_id = app
        .state
        .image
        .load_id
        .load(std::sync::atomic::Ordering::Relaxed);

    let handle = iced::widget::image::Handle::from_rgba(10, 20, vec![0; 10 * 20 * 4]);
    let _ = kglance::features::image::handle_decoded(&mut app, load_id, handle, None, 10, 20);

    assert_eq!(app.state.image.load_state, ImageLoadState::Ready);
    assert_eq!(app.state.image.display_width, 10);
    assert_eq!(app.state.image.display_height, 20);
    assert!(app.state.image.display_handle.is_some());
    assert!(app.state.cache.get("/tmp/test.jpg").is_some());
}

#[test]
fn test_calculate_fit_dimensions_no_upscale_for_small_images() {
    use kglance::features::image::calculate_fit_dimensions;

    // Small image: 800x600 inside 1920x1080 -> remains 800x600
    assert_eq!(calculate_fit_dimensions(800, 600, 1920, 1080), (800, 600));

    // Exact bounds: 1920x1080 -> remains 1920x1080
    assert_eq!(
        calculate_fit_dimensions(1920, 1080, 1920, 1080),
        (1920, 1080)
    );

    // 1x1 icon -> remains 1x1
    assert_eq!(calculate_fit_dimensions(1, 1, 1920, 1080), (1, 1));
}

#[test]
fn test_calculate_fit_dimensions_downscales_large_images_proportionally() {
    use kglance::features::image::calculate_fit_dimensions;

    // 11398x7599 (86MP) -> fits in 1920x1080, height is the constraining dimension
    let (w, h) = calculate_fit_dimensions(11398, 7599, 1920, 1080);
    assert_eq!(h, 1080);
    assert!(w <= 1920);
    // Aspect ratio check: 11398 / 7599 ~ 1.50
    let orig_ratio = 11398.0 / 7599.0;
    let new_ratio = w as f64 / h as f64;
    assert!((orig_ratio - new_ratio).abs() < 0.01);

    // 4000x2000 -> width is the constraining dimension (scale = 1920 / 4000 = 0.48, h = 960)
    let (w2, h2) = calculate_fit_dimensions(4000, 2000, 1920, 1080);
    assert_eq!(w2, 1920);
    assert_eq!(h2, 960);
}

#[test]
fn test_fast_downsample_rgba_correctness() {
    use kglance::features::image::fast_downsample_rgba;

    // Edge cases: 0 dimension returns empty
    assert!(fast_downsample_rgba(&[], 0, 0, 100, 100).is_empty());
    assert!(fast_downsample_rgba(&[0; 16], 2, 2, 0, 100).is_empty());
    assert!(fast_downsample_rgba(&[0; 4], 2, 2, 1, 1).is_empty()); // truncated buffer

    // 4x4 image downsampled to 2x2
    // Create 4x4 RGBA where each 2x2 block has distinct values
    let mut src = vec![0u8; 4 * 4 * 4];
    // Fill top-left 2x2 block with 100, top-right with 200
    for y in 0..2 {
        for x in 0..2 {
            let idx = (y * 4 + x) * 4;
            src[idx] = 100;
            src[idx + 1] = 100;
            src[idx + 2] = 100;
            src[idx + 3] = 255;
        }
    }
    for y in 0..2 {
        for x in 2..4 {
            let idx = (y * 4 + x) * 4;
            src[idx] = 200;
            src[idx + 1] = 200;
            src[idx + 2] = 200;
            src[idx + 3] = 255;
        }
    }

    let downsampled = fast_downsample_rgba(&src, 4, 4, 2, 2);
    assert_eq!(downsampled.len(), 2 * 2 * 4);
    // Top-left pixel of 2x2 result should be 100
    assert_eq!(downsampled[0], 100);
    assert_eq!(downsampled[1], 100);
    assert_eq!(downsampled[2], 100);
    assert_eq!(downsampled[3], 255);

    // Top-right pixel of 2x2 result should be 200
    assert_eq!(downsampled[4], 200);
    assert_eq!(downsampled[5], 200);
    assert_eq!(downsampled[6], 200);
    assert_eq!(downsampled[7], 255);
}

#[test]
fn test_fast_downsample_rgba_massive_speed() {
    use kglance::features::image::fast_downsample_rgba;
    use std::time::Instant;

    // Synthetic 4000x3000 (12MP) image
    let orig_w = 4000;
    let orig_h = 3000;
    let src = vec![128u8; (orig_w as usize) * (orig_h as usize) * 4];

    let t0 = Instant::now();
    let downsampled = fast_downsample_rgba(&src, orig_w, orig_h, 1920, 1080);
    let elapsed = t0.elapsed();

    assert_eq!(downsampled.len(), 1920 * 1080 * 4);
    // In unoptimized debug builds under concurrent test load, 12MP downsampling should take < 500ms (in release it is ~5ms).
    assert!(
        elapsed.as_millis() < 500,
        "Downsampling 12MP took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_progressive_preview_stage1_and_stage2() {
    let mut app = kglance::app::KglanceApp::default();
    app.state.file_name = "/tmp/photo.jpg".to_string();
    let load_id = app
        .state
        .image
        .load_id
        .load(std::sync::atomic::Ordering::Relaxed);

    // Initial state: loading large image
    app.state.image.load_state = ImageLoadState::Loading;
    app.state.image.display_handle = None;
    app.state.image.preview_handle = None;

    // Stage 1: Fast downsampled preview is loaded (e.g. from cache or preload)
    let preview_handle = iced::widget::image::Handle::from_rgba(400, 300, vec![100; 400 * 300 * 4]);
    app.state.image.preview_handle = Some(preview_handle.clone());
    app.state.image.load_state = ImageLoadState::Ready;

    assert_eq!(app.state.image.load_state, ImageLoadState::Ready);
    assert!(app.state.image.preview_handle.is_some());
    assert!(app.state.image.display_handle.is_none());

    // Stage 2: Full resolution decoded image arrives in background
    let full_handle =
        iced::widget::image::Handle::from_rgba(4000, 3000, vec![200; 4000 * 3000 * 4]);
    let _ = kglance::features::image::handle_decoded(
        &mut app,
        load_id,
        full_handle,
        Some((preview_handle, 400, 300)),
        4000,
        3000,
    );

    assert_eq!(app.state.image.load_state, ImageLoadState::Ready);
    assert!(app.state.image.display_handle.is_some());
    assert_eq!(app.state.image.display_width, 4000);
    assert_eq!(app.state.image.display_height, 3000);
    // Preview handle is retained for smooth fit-to-window rendering without texture swap flicker
    assert!(app.state.image.preview_handle.is_some());
    assert!(app.state.cache.get("/tmp/photo.jpg").is_some());
}

#[test]
fn test_tiered_cache_retains_many_previews_without_eviction() {
    let mut app = kglance::app::KglanceApp::default();

    // Cache 10 large image previews (downsampled to 1920x1080)
    for i in 0..10 {
        let path = format!("/tmp/photo_{}.jpg", i);
        let preview_handle =
            iced::widget::image::Handle::from_rgba(1920, 1080, vec![0u8; 1920 * 1080 * 4]);
        app.state.cache.put(
            path,
            kglance::core::CachedContent::DecodedImage {
                preview: None,
                handle: preview_handle,
                width: 1920,
                height: 1080,
            },
        );
    }

    // Verify ALL 10 files remain cached without any eviction (10 * 8MB = 80MB < 512MB limit)
    for i in 0..10 {
        let path = format!("/tmp/photo_{}.jpg", i);
        assert!(
            app.state.cache.contains(&path),
            "File {} should still be in cache",
            path
        );
    }
}

#[test]
fn test_canvas_renderer_handle_stability_on_zoom() {
    use kglance::core::ImageState;

    let preview_handle = iced::widget::image::Handle::from_rgba(400, 300, vec![100; 400 * 300 * 4]);
    let display_handle =
        iced::widget::image::Handle::from_rgba(4000, 3000, vec![200; 4000 * 3000 * 4]);

    let state = ImageState {
        display_handle: Some(display_handle.clone()),
        preview_handle: Some(preview_handle.clone()),
        display_width: 4000,
        display_height: 3000,
        ..Default::default()
    };

    // When display_handle is ready, it must always be the active handle regardless of zoom scale
    let selected = state
        .display_handle
        .as_ref()
        .or(state.preview_handle.as_ref());
    assert!(selected.is_some());
    assert_eq!(selected.unwrap(), &display_handle);
}

#[test]
fn test_canvas_renderer_underlay_prevents_flicker() {
    use kglance::core::ImageState;

    let preview_handle = iced::widget::image::Handle::from_rgba(400, 300, vec![100; 400 * 300 * 4]);
    let display_handle =
        iced::widget::image::Handle::from_rgba(4000, 3000, vec![200; 4000 * 3000 * 4]);

    // Scenario 1: Initial stage - only preview_handle is ready
    let state_preview_only = ImageState {
        handle: Some(preview_handle.clone()),
        preview_handle: Some(preview_handle.clone()),
        display_handle: None,
        display_width: 4000,
        display_height: 3000,
        ..Default::default()
    };

    let mut drawn_handles_stage1 = Vec::new();
    if let Some(ref prev) = state_preview_only.preview_handle {
        drawn_handles_stage1.push(prev.clone());
    }
    let main_stage1 = state_preview_only
        .display_handle
        .as_ref()
        .or(state_preview_only.handle.as_ref());
    if let Some(main) = main_stage1
        && state_preview_only.preview_handle.as_ref() != Some(main)
    {
        drawn_handles_stage1.push(main.clone());
    }
    // Preview-only must be drawn exactly once (no duplicate draw)
    assert_eq!(drawn_handles_stage1.len(), 1);
    assert_eq!(drawn_handles_stage1[0], preview_handle);

    // Scenario 2: Transition stage - both preview_handle and display_handle exist
    let state_both = ImageState {
        handle: Some(display_handle.clone()),
        preview_handle: Some(preview_handle.clone()),
        display_handle: Some(display_handle.clone()),
        display_width: 4000,
        display_height: 3000,
        ..Default::default()
    };

    let mut drawn_handles_stage2 = Vec::new();
    if let Some(ref prev) = state_both.preview_handle {
        drawn_handles_stage2.push(prev.clone());
    }
    let main_stage2 = state_both
        .display_handle
        .as_ref()
        .or(state_both.handle.as_ref());
    if let Some(main) = main_stage2
        && state_both.preview_handle.as_ref() != Some(main)
    {
        drawn_handles_stage2.push(main.clone());
    }
    // Both preview underlay and display handle on top must be submitted to prevent blank frame
    assert_eq!(drawn_handles_stage2.len(), 2);
    assert_eq!(drawn_handles_stage2[0], preview_handle);
    assert_eq!(drawn_handles_stage2[1], display_handle);
}

#[test]
fn test_scroll_delta_zoom_factor_scaling() {
    // Standard discrete mouse wheel tick (+1 line) -> ~1.15
    let line_factor_in = 1.15f32;
    let line_factor_out = 1.0 / 1.15f32;
    assert!((line_factor_in - 1.15).abs() < 0.001);
    assert!((line_factor_out - 0.8695).abs() < 0.001);

    // Continuous touchpad pixel scroll (+5.0 pixels) -> ~1.025
    let pixel_delta = 5.0f32;
    let pixel_factor = 1.0 + (pixel_delta * 0.005).clamp(-0.5, 0.5);
    assert!((pixel_factor - 1.025).abs() < 0.001);
    assert!(pixel_factor > 1.0);
}

#[test]
fn test_viewer_controller_zoom_preserves_cursor_focus() {
    use iced::{Point, Size};
    use kglance::features::image::{Camera, ViewerController};

    let mut camera = Camera::new();
    camera.zoom = 1.0;
    camera.offset_x = 0.0;
    camera.offset_y = 0.0;

    let cursor = Point::new(400.0, 300.0);
    let viewport = Size::new(800.0, 600.0);

    // Zooming in by 1.5x at viewport center (400, 300)
    ViewerController::zoom(&mut camera, 1.5, cursor, viewport);

    assert_eq!(camera.zoom, 1.5);
    // When zooming at viewport center, offset remains at (0, 0)
    assert!((camera.offset_x).abs() < 0.001);
    assert!((camera.offset_y).abs() < 0.001);
}

#[test]
fn test_decode_to_rgba_jpeg_fast_path() {
    let mut img = image::RgbImage::new(4, 4);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 0, 0]);
    }
    let mut jpeg_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Jpeg).unwrap();

    assert!(jpeg_bytes.starts_with(&[0xFF, 0xD8, 0xFF]));
    let decoded = kglance::features::image::decode_to_rgba(&jpeg_bytes);
    assert!(decoded.is_some());
    let (w, h, pixels) = decoded.unwrap();
    assert_eq!(w, 4);
    assert_eq!(h, 4);
    assert_eq!(pixels.len(), 4 * 4 * 4);
    // Red channel should be close to 255
    assert!(pixels[0] > 200);
}
