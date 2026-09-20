use kglance::app::KglanceApp;
use kglance::core::PreviewData;
use std::sync::Arc;

#[test]
fn test_close_current_clears_cache_and_preloads() {
    let mut app = KglanceApp::default();
    app.state.file_name = "test.txt".to_string();
    app.state.cache.put(
        "test.txt".to_string(),
        kglance::core::CachedContent::from_preview(Arc::new(PreviewData::Text {
            content: "hello world".to_string(),
            line_numbers: "1".to_string(),
            language: "text".to_string(),
        })),
    );
    app.state.pending_preloads.insert("pending.txt".to_string());
    app.state.image.image_bytes = vec![1, 2, 3, 4];
    app.state.image.display_handle = Some(iced::widget::image::Handle::from_bytes(vec![0; 16]));
    app.state.pdf.page_count = 10;
    app.state.markdown.search_query = "some query".to_string();

    assert!(!app.state.cache.is_empty());
    assert!(!app.state.pending_preloads.is_empty());
    assert!(!app.state.image.image_bytes.is_empty());
    assert!(app.state.image.display_handle.is_some());
    assert_eq!(app.state.pdf.page_count, 10);
    assert!(!app.state.markdown.search_query.is_empty());

    // Call close_current
    let _ = app.handle_close();

    assert!(app.state.cache.is_empty());
    assert!(app.state.pending_preloads.is_empty());
    assert!(app.current_content.is_none());
    assert!(app.state.file_name.is_empty());
    assert!(app.state.image.image_bytes.is_empty());
    assert!(app.state.image.display_handle.is_none());
    assert_eq!(app.state.pdf.page_count, 0);
    assert!(app.state.markdown.search_query.is_empty());
}
