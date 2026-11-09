use super::*;
use crate::app::test_util::test_app;

#[test]
fn active_pdf_state_selects_shared_pdf_state() {
    let mut app = test_app(None);
    active_pdf_state_mut(&mut app).sidebar_width = 321.0;
    assert_eq!(app.state.pdf.sidebar_width, 321.0);
}

#[test]
fn evict_distant_pages_works() {
    let mut pdf_state = crate::core::PdfState {
        pages: crate::core::types::PageCache::new(30),
        page_count: 30,
        ..Default::default()
    };

    for i in 0..20 {
        pdf_state.pages.insert(
            i,
            crate::core::PageCacheEntry {
                width: 10,
                height: 10,
                handle: iced::widget::image::Handle::from_rgba(1, 1, vec![0; 4]),
            },
            15,
        );
    }

    assert!(pdf_state.pages.count() <= MAX_CACHED_PAGES);
    // Near pages around 15 are retained
    assert!(pdf_state.pages.get(15).is_some());
    assert!(pdf_state.pages.get(14).is_some());
    assert!(pdf_state.pages.get(16).is_some());
}

#[test]
fn test_disk_cache_instant_promotion() {
    let mut state = crate::core::PdfState {
        page_count: 50,
        pages: crate::core::types::PageCache::new(50),
        ..Default::default()
    };
    let cache = std::sync::Arc::new(crate::features::pdf::cache::PdfDiskCache::new(77777).unwrap());
    let _ = cache.save_page_with_meta(10, b"\x89PNG\r\n\x1a\nfake", 800, 1000);
    state.disk_cache = Some(cache);

    assert!(state.pages.get(10).is_none());
    assert!(promote_page_from_disk_if_cached(&mut state, 10));
    assert!(state.pages.get(10).is_some());
}

#[test]
fn test_pdf_auto_scroll_tick() {
    let mut app = test_app(None);
    app.state.pdf.page_count = 2;
    app.state.pdf.page_dimensions = vec![
        crate::features::pdf::types::PageDimensions {
            width_pts: 595.0,
            height_pts: 842.0,
        },
        crate::features::pdf::types::PageDimensions {
            width_pts: 595.0,
            height_pts: 842.0,
        },
    ];
    app.state.pdf.page_y_offsets = vec![0.0, 850.0];
    app.state.pdf.page_ends = vec![842.0, 1692.0];
    app.state.pdf.total_content_height = 1692.0;
    app.state.pdf.viewport_height = 800.0;
    app.state.pdf.scroll_y = 100.0;
    app.state.pdf.auto_scroll_delta = Some(20.0);
    app.state.pdf.is_selecting = true;
    app.state.pdf.selection_drag_start = Some(crate::features::pdf::PdfPosition {
        page: 0,
        line: 0,
        char_idx: 0,
    });

    let _ = handle_auto_scroll_tick(&mut app);
    assert_eq!(app.state.pdf.scroll_y, 120.0);
}

#[test]
fn test_handle_goto_page_workflow() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Pdf {
        page_count: 5,
        current_page: 0,
        data: Vec::new(),
        width: 100,
        height: 100,
        outline: Vec::new(),
        page_dimensions: Vec::new(),
        first_page_text: None,
    });
    app.state.pdf.page_count = 5;
    app.state.pdf.page_y_offsets = vec![0.0, 500.0, 1000.0, 1500.0, 2000.0];
    app.state.pdf.page_ends = vec![500.0, 1000.0, 1500.0, 2000.0, 2500.0];
    app.state.pdf.total_content_height = 2500.0;
    app.state.pdf.viewport_height = 600.0;
    app.state.pdf.current_page = 0;
    app.state
        .pdf
        .visible_page
        .store(0, std::sync::atomic::Ordering::Relaxed);

    // 1. Open goto page
    let _ = handle_goto_page_open(&mut app);
    assert!(app.state.pdf.goto_page.is_visible);
    assert_eq!(app.state.pdf.goto_page.input_value, "1");

    // 2. Change input to "4"
    let _ = handle_goto_page_input_changed(&mut app, "4".to_string());
    assert_eq!(app.state.pdf.goto_page.input_value, "4");

    // 3. Submit valid page
    let _ = handle_goto_page_submit(&mut app);
    assert!(!app.state.pdf.goto_page.is_visible);
    assert_eq!(app.state.pdf.current_page, 3);
    assert_eq!(
        app.state
            .pdf
            .visible_page
            .load(std::sync::atomic::Ordering::Relaxed),
        3
    );
    assert_eq!(app.state.pdf.scroll_y, 1500.0);
    assert_eq!(app.state.pdf.scroll_controller.position_y(), 1500.0);

    // 4. Invalid input submits error
    let _ = handle_goto_page_open(&mut app);
    let _ = handle_goto_page_input_changed(&mut app, "99".to_string());
    let _ = handle_goto_page_submit(&mut app);
    assert!(app.state.pdf.goto_page.is_visible);
    assert!(app.state.pdf.goto_page.has_error);

    // 5. Close dialog
    let _ = handle_goto_page_close(&mut app);
    assert!(!app.state.pdf.goto_page.is_visible);
}

#[test]
fn test_handle_toc_item_clicked_updates_page_and_scroll() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Pdf {
        page_count: 5,
        current_page: 0,
        data: Vec::new(),
        width: 100,
        height: 100,
        outline: vec![
            crate::features::pdf::parser::PdfTocEntry {
                title: "Chapter 1".to_string(),
                page: 0,
                level: 1,
            },
            crate::features::pdf::parser::PdfTocEntry {
                title: "Chapter 2".to_string(),
                page: 2,
                level: 1,
            },
        ],
        page_dimensions: Vec::new(),
        first_page_text: None,
    });
    app.state.pdf.page_count = 5;
    app.state.pdf.page_y_offsets = vec![0.0, 500.0, 1000.0, 1500.0, 2000.0];
    app.state.pdf.page_ends = vec![500.0, 1000.0, 1500.0, 2000.0, 2500.0];
    app.state.pdf.total_content_height = 2500.0;
    app.state.pdf.viewport_height = 600.0;
    app.state.pdf.current_page = 0;
    app.state
        .pdf
        .visible_page
        .store(0, std::sync::atomic::Ordering::Relaxed);
    app.state.pdf.outline_state = crate::features::pdf::outline::PdfOutlineState::new(vec![
        crate::features::pdf::outline::PdfOutlineNode {
            title: "Chapter 1".to_string(),
            page: 0,
            depth: 0,
            is_expanded: true,
            children: None,
        },
        crate::features::pdf::outline::PdfOutlineNode {
            title: "Chapter 2".to_string(),
            page: 2,
            depth: 0,
            is_expanded: true,
            children: None,
        },
    ]);

    // Click TOC item for page 2
    let _ = handle_toc_item_clicked(&mut app, 2);

    assert_eq!(app.state.pdf.current_page, 2);
    assert_eq!(
        app.state
            .pdf
            .visible_page
            .load(std::sync::atomic::Ordering::Relaxed),
        2
    );
    assert_eq!(app.state.pdf.scroll_y, 1000.0);
    assert_eq!(app.state.pdf.scroll_controller.position_y(), 1000.0);

    // Toggle outline node expansion
    let _ = handle_toggle_outline_node(&mut app, 0, "Chapter 1".to_string());
}

#[test]
fn test_pdf_record_and_restore_read_position() {
    let mut app = test_app(None);
    let path = "/test/doc.pdf";
    app.state.file_name = path.to_string();
    app.current_content = Some(crate::core::PreviewData::Pdf {
        page_count: 5,
        current_page: 0,
        data: Vec::new(),
        width: 100,
        height: 100,
        outline: Vec::new(),
        page_dimensions: Vec::new(),
        first_page_text: None,
    });
    app.state.pdf.page_count = 5;
    app.state.pdf.page_y_offsets = vec![0.0, 500.0, 1000.0, 1500.0, 2000.0];
    app.state.pdf.page_ends = vec![500.0, 1000.0, 1500.0, 2000.0, 2500.0];
    app.state.pdf.total_content_height = 2500.0;
    app.state.pdf.viewport_height = 600.0;
    app.state.pdf.scroll_y = 1500.0;
    app.state
        .pdf
        .visible_page
        .store(3, std::sync::atomic::Ordering::Relaxed);
    app.state.pdf.current_page = 3;

    // 1. Record position
    app.record_read_position();
    assert!(app.state.read_positions_dirty);
    let saved = app.state.read_positions.get(path);
    assert_eq!(
        saved,
        Some(crate::core::ReadPosition {
            scroll_y: 1500.0,
            chapter: 3,
            reading_mode: None,
        })
    );

    // 2. Reset PDF state to simulate reopening
    app.state.pdf.scroll_y = 0.0;
    app.state
        .pdf
        .visible_page
        .store(0, std::sync::atomic::Ordering::Relaxed);
    app.state.pdf.current_page = 0;

    // 3. Restore position
    let _ = app.restore_read_position_for(path);
    assert_eq!(app.state.pdf.scroll_y, 1500.0);
    assert_eq!(
        app.state
            .pdf
            .visible_page
            .load(std::sync::atomic::Ordering::Relaxed),
        3
    );
    assert_eq!(app.state.pdf.current_page, 3);
}
