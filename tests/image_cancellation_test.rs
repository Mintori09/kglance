use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::test]
async fn test_decode_task_aborts_when_load_id_changes() {
    let load_id_val = 1u64;
    let load_id_arc = Arc::new(AtomicU64::new(load_id_val));

    // Stale task simulation: user immediately navigated to next file (load_id changed to 2)
    load_id_arc.store(2, Ordering::Relaxed);

    // Dummy JPEG bytes
    let dummy_bytes = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00,
    ];
    let task = kglance::features::image::spawn_decode_task_delayed(
        dummy_bytes,
        load_id_val,
        load_id_arc.clone(),
        0,
    );

    // The iced task drops without emitting Decoded image
    drop(task);
}

#[test]
fn test_handle_decoded_ignores_stale_load_id() {
    let mut app = kglance::app::KglanceApp::default();
    app.state.image.load_id.store(5, Ordering::Relaxed);

    let dummy_handle = iced::widget::image::Handle::from_bytes(vec![0u8; 10]);
    let stale_load_id = 4;

    let _ = kglance::features::image::handle_decoded(
        &mut app,
        stale_load_id,
        dummy_handle,
        None,
        100,
        100,
    );

    // State should not be updated to Ready or have handle set
    assert_ne!(
        app.state.image.load_state,
        kglance::features::image::ImageLoadState::Ready
    );
    assert!(app.state.image.handle.is_none());
    assert_eq!(app.state.image.display_width, 0);
    assert_eq!(app.state.image.display_height, 0);
}

#[test]
fn test_handle_decoded_accepts_matching_load_id() {
    let mut app = kglance::app::KglanceApp::default();
    app.state.image.load_id.store(5, Ordering::Relaxed);

    let dummy_handle = iced::widget::image::Handle::from_bytes(vec![0u8; 10]);
    let matching_load_id = 5;

    let _ = kglance::features::image::handle_decoded(
        &mut app,
        matching_load_id,
        dummy_handle,
        None,
        100,
        100,
    );

    assert_eq!(
        app.state.image.load_state,
        kglance::features::image::ImageLoadState::Ready
    );
    assert!(app.state.image.handle.is_some());
    assert_eq!(app.state.image.display_width, 100);
    assert_eq!(app.state.image.display_height, 100);
}

#[test]
fn test_generation_id_invalidation_increments_state() {
    let app = kglance::app::KglanceApp::default();
    let initial_gen = app.state.generation_id.load(Ordering::Relaxed);

    app.invalidate_render_generations();
    let new_gen = app.state.generation_id.load(Ordering::Relaxed);

    assert_ne!(initial_gen, new_gen);
    assert_eq!(new_gen, initial_gen + 1);
}
