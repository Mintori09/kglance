use super::*;
use crate::app::KglanceApp;

fn sample_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length & chunk
        0x00, 0x00, 0x00, 0x01, // width = 1
        0x00, 0x00, 0x00, 0x01, // height = 1
        0x08, 0x06, 0x00, 0x00, 0x00, // 8 bit RGBA
        0x1F, 0x15, 0xC4, 0x89, // CRC
        0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, // IDAT
        0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
        0xB4, // data & CRC
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND
    ]
}

fn test_app() -> KglanceApp {
    crate::app::test_util::test_app(None)
}

#[test]
fn test_lightbox_open_close_and_cyclic_navigation() {
    let mut app = test_app();
    let png = sample_png();

    let items = vec![
        LightboxItem {
            id: "img1.png".into(),
            title: Some("Title 1".into()),
            subtitle: "img1.png".into(),
            link_url: None,
            in_memory_bytes: Some(png.clone()),
        },
        LightboxItem {
            id: "img2.png".into(),
            title: None,
            subtitle: "img2.png".into(),
            link_url: Some("https://example.com".into()),
            in_memory_bytes: Some(png.clone()),
        },
    ];

    assert!(app.state.lightbox.is_none());

    // 1. Open at index 0
    let _ = handle_open_lightbox(&mut app, items, 0);
    assert!(app.state.lightbox.is_some());
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 0);
    assert_eq!(lb.items[0].id, "img1.png");

    // 2. Next -> index 1
    let _ = handle_next_lightbox(&mut app);
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 1);
    assert_eq!(lb.items[1].id, "img2.png");

    // 3. Next wraps around -> index 0
    let _ = handle_next_lightbox(&mut app);
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 0);

    // 4. Prev wraps around -> index 1
    let _ = handle_prev_lightbox(&mut app);
    let lb = app.state.lightbox.as_ref().unwrap();
    assert_eq!(lb.current_index, 1);

    // 5. Close -> None
    let _ = handle_close_lightbox(&mut app);
    assert!(app.state.lightbox.is_none());
}
