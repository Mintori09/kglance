use super::*;
use crate::app::test_util::{epub_content, test_app};
use crate::features::markdown::update::handle_selection_drag_start;

#[test]
fn drag_start_sets_resizing_and_clears_anchor() {
    let mut app = test_app(None);
    let _ = handle_sidebar_drag_started(&mut app);
    assert!(app.state.markdown.sidebar_resizing);
    assert!(app.state.markdown.sidebar_drag_start_x.is_none());
    assert!(app.state.epub.sidebar_resizing);
    assert!(app.state.epub.sidebar_drag_start_x.is_none());
    assert!(app.state.text.sidebar_resizing);
    assert!(app.state.text.sidebar_drag_start_x.is_none());
    assert!(app.state.pdf.sidebar_resizing);
    assert!(app.state.pdf.sidebar_drag_start_x.is_none());
}

#[test]
fn mouse_moved_anchors_on_first_move_without_resizing() {
    let mut app = test_app(None);
    app.state.epub.sidebar_width = 200.0;
    let _ = handle_sidebar_drag_started(&mut app);
    let _ = handle_mouse_moved(&mut app, 250.0, 100.0);
    assert_eq!(app.state.epub.sidebar_drag_start_x, Some(250.0));
    assert_eq!(app.state.epub.sidebar_drag_start_width, 200.0);
    assert_eq!(app.state.epub.sidebar_width, 200.0);
}

#[test]
fn mouse_moved_applies_delta_after_anchor() {
    let mut app = test_app(None);
    app.state.epub.sidebar_width = 200.0;
    let _ = handle_sidebar_drag_started(&mut app);
    let _ = handle_mouse_moved(&mut app, 250.0, 100.0);
    let _ = handle_mouse_moved(&mut app, 270.0, 100.0);
    assert_eq!(app.state.epub.sidebar_width, 220.0);
    assert_eq!(app.state.epub.sidebar_drag_start_x, Some(250.0));
}

#[test]
fn mouse_moved_resizes_text_sidebar() {
    let mut app = test_app(None);
    app.state.text.sidebar_width = 250.0;
    let _ = handle_sidebar_drag_started(&mut app);
    let _ = handle_mouse_moved(&mut app, 300.0, 100.0);
    let _ = handle_mouse_moved(&mut app, 350.0, 100.0);
    assert_eq!(app.state.text.sidebar_width, 300.0);
    assert_eq!(app.state.text.sidebar_drag_start_x, Some(300.0));
}

#[test]
fn mouse_moved_clamps_epub_width() {
    let mut app = test_app(None);
    app.state.epub.sidebar_width = 200.0;
    let _ = handle_sidebar_drag_started(&mut app);
    let _ = handle_mouse_moved(&mut app, 0.0, 100.0);
    let _ = handle_mouse_moved(&mut app, -1000.0, 100.0);
    assert_eq!(app.state.epub.sidebar_width, 140.0);
}

#[test]
fn drag_end_clears_resizing() {
    let mut app = test_app(None);
    let _ = handle_sidebar_drag_started(&mut app);
    let _ = handle_sidebar_drag_ended(&mut app);
    assert!(!app.state.markdown.sidebar_resizing);
    assert!(!app.state.epub.sidebar_resizing);
    assert!(!app.state.text.sidebar_resizing);
    assert!(!app.state.pdf.sidebar_resizing);
}

#[test]
fn epub_mouse_moved_sets_auto_scroll_on_epub_state() {
    let mut app = test_app(Some(epub_content(&["hello"])));
    let _ = handle_selection_drag_start(&mut app, 0, 0);
    app.state.current_window_size.height = 200.0;
    let _ = handle_mouse_moved(&mut app, 50.0, 9999.0);
    assert!(app.state.epub.markdown_state.auto_scroll_delta.is_some());
    assert!(app.state.markdown.auto_scroll_delta.is_none());
}

#[test]
fn mouse_moved_clamps_pdf_width() {
    let mut app = test_app(None);
    app.state.pdf.sidebar_width = 300.0;
    let _ = handle_sidebar_drag_started(&mut app);
    let _ = handle_mouse_moved(&mut app, 100.0, 100.0);
    let _ = handle_mouse_moved(&mut app, -1000.0, 100.0);
    assert_eq!(app.state.pdf.sidebar_width, 120.0);
    assert!(app.state.pdf.sidebar_resizing);
    assert!(app.state.pdf.sidebar_drag_start_x.is_some());
}

#[test]
fn pdf_mouse_moved_sets_auto_scroll_and_updates_selection() {
    let mut app = test_app(None);
    app.state.pdf.is_selecting = true;
    app.state.current_window_size.height = 200.0;
    let _ = handle_mouse_moved(&mut app, 100.0, 500.0);
    assert!(app.state.pdf.auto_scroll_delta.is_some());
    let delta = app.state.pdf.auto_scroll_delta.unwrap();
    assert!(delta > 0.0);

    let _ = handle_mouse_released(&mut app);
    assert!(!app.state.pdf.is_selecting);
    assert!(app.state.pdf.auto_scroll_delta.is_none());
}

#[test]
fn test_update_current_window_size_respects_remember_window_size() {
    let mut app = test_app(None);
    app.state.window_width = 800.0;
    app.state.window_height = 600.0;
    app.state.window_default_size = iced::Size::new(800.0, 600.0);
    app.state.remember_window_size = false;

    let _ = update_current_window_size(&mut app, 1200.0, 900.0);
    assert_eq!(app.state.window_width, 1200.0);
    assert_eq!(app.state.window_height, 900.0);
    assert_eq!(app.state.window_default_size, iced::Size::new(800.0, 600.0));

    app.state.remember_window_size = true;
    let _ = update_current_window_size(&mut app, 1400.0, 1000.0);
    assert_eq!(app.state.window_width, 1400.0);
    assert_eq!(app.state.window_height, 1000.0);
    assert_eq!(
        app.state.window_default_size,
        iced::Size::new(1400.0, 1000.0)
    );

    // Test clamping to min size
    app.state.window_min_size = iced::Size::new(800.0, 600.0);
    let _ = update_current_window_size(&mut app, 400.0, 300.0);
    assert_eq!(app.state.window_default_size, iced::Size::new(800.0, 600.0));
}

#[test]
fn test_handle_open_link_epub_toc_chapter_jump() {
    let mut app = test_app(None);
    let ch0 = crate::features::epub::types::EpubChapterInfo {
        title: "Table Of Contents".to_string(),
        level: 1,
        anchor: None,
        file_href: "toc.xhtml".to_string(),
        blocks: vec![crate::parsers::markdown::Block::Paragraph(vec![
            crate::parsers::markdown::Inline::Link {
                text: vec![crate::parsers::markdown::Inline::Text(
                    "Chapter 1".to_string(),
                )],
                url: "0_chuong-01-ke-thu-vay-quanh.xhtml".to_string(),
            },
        ])],
    };
    let ch1 = crate::features::epub::types::EpubChapterInfo {
        title: "Chapter 1".to_string(),
        level: 1,
        anchor: None,
        file_href: "0_chuong-01-ke-thu-vay-quanh.xhtml".to_string(),
        blocks: vec![crate::parsers::markdown::Block::Heading {
            level: 1,
            content: vec![crate::parsers::markdown::Inline::Text(
                "Chapter 1 Heading".to_string(),
            )],
        }],
    };

    app.current_content = Some(crate::core::PreviewData::Epub {
        title: "Test Novel".to_string(),
        author: "Author".to_string(),
        chapters: vec![ch0.clone(), ch1.clone()],
        active_chapter: 0,
        images: std::collections::HashMap::new(),
    });
    app.state.epub.chapters = vec![ch0, ch1];
    app.state.epub.active_chapter = 0;
    app.state.epub.reading_mode = crate::core::config::EpubReadingMode::Continuous;
    app.state.epub.chapter_block_offsets = vec![0, 1];
    app.state.epub.chapter_target_offsets = vec![0, 1];
    app.state.epub.markdown_state.block_y_offsets = vec![0.0, 500.0];
    app.state.epub.markdown_state.total_content_height = 2000.0;
    app.state.epub.markdown_state.viewport_height = 600.0;

    let _ = handle_open_link(&mut app, "0_chuong-01-ke-thu-vay-quanh.xhtml".to_string());
    assert_eq!(app.state.epub.active_chapter, 1);
    assert_eq!(app.state.epub.markdown_state.scroll_y, 500.0);
}
