use super::*;
use crate::app::test_util::test_app;
use crate::features::pdf::selection::{PdfChar, PdfLine};

fn create_dummy_line(text: &str, y: f32) -> PdfLine {
    let mut chars = Vec::new();
    let mut x = 10.0;
    for ch in text.chars() {
        chars.push(PdfChar {
            ch,
            rect: [x, y, x + 8.0, y + 12.0],
        });
        x += 8.0;
    }
    PdfLine {
        text: text.to_string(),
        rect: [10.0, y, x, y + 12.0],
        chars,
    }
}

#[test]
fn test_search_matching_case_insensitive() {
    let line1 = create_dummy_line("Hello World to everyone", 20.0);
    let line2 = create_dummy_line("Searching in PDF is fast and easy", 40.0);
    let page_text = PdfPageText {
        page_index: 0,
        lines: vec![line1, line2],
    };

    let matches = find_matches_in_page(&page_text, "world");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].page_index, 0);
    assert_eq!(matches[0].line_index, 0);
    assert_eq!(matches[0].char_start, 6);
    assert_eq!(matches[0].char_end, 11);

    let matches_multi = find_matches_in_page(&page_text, "E");
    // "Hello" (1), "everyone" (3), "Searching" (1), "easy" (1) -> total 6 'e's
    assert_eq!(matches_multi.len(), 6);
}

#[test]
fn test_search_state_navigation() {
    let mut state = PdfSearchState::default();
    assert_eq!(state.total_matches(), 0);
    assert_eq!(state.current_match(), None);

    let m1 = PdfSearchMatch {
        page_index: 0,
        line_index: 0,
        char_start: 0,
        char_end: 4,
        rect: [0.0, 0.0, 10.0, 10.0],
    };
    let m2 = PdfSearchMatch {
        page_index: 1,
        line_index: 0,
        char_start: 5,
        char_end: 9,
        rect: [0.0, 0.0, 10.0, 10.0],
    };
    state.matches = vec![m1.clone(), m2.clone()];
    state.query = "test".to_string();
    state.update_info();

    assert_eq!(state.info, "1/2");
    assert_eq!(state.current_match(), Some(&m1));

    // Next match
    assert_eq!(state.next_match(), Some(&m2));
    assert_eq!(state.info, "2/2");

    // Next wraps to beginning
    assert_eq!(state.next_match(), Some(&m1));
    assert_eq!(state.info, "1/2");

    // Prev wraps to end
    assert_eq!(state.prev_match(), Some(&m2));
    assert_eq!(state.info, "2/2");
}

#[test]
fn test_handle_search_workflow() {
    let mut app = test_app(None);
    app.current_content = Some(crate::core::PreviewData::Pdf {
        page_count: 2,
        current_page: 0,
        data: Vec::new(),
        width: 100,
        height: 100,
        outline: Vec::new(),
        page_dimensions: vec![
            crate::features::pdf::types::PageDimensions {
                width_pts: 595.0,
                height_pts: 842.0,
            },
            crate::features::pdf::types::PageDimensions {
                width_pts: 595.0,
                height_pts: 842.0,
            },
        ],
        first_page_text: None,
    });
    app.state.pdf.page_count = 2;
    app.state.pdf.page_y_offsets = vec![0.0, 850.0];
    app.state.pdf.page_ends = vec![842.0, 1692.0];
    app.state.pdf.total_content_height = 1692.0;
    app.state.pdf.viewport_height = 800.0;
    app.state.pdf.page_texts = vec![
        Some(PdfPageText {
            page_index: 0,
            lines: vec![create_dummy_line("First page text here", 50.0)],
        }),
        Some(PdfPageText {
            page_index: 1,
            lines: vec![create_dummy_line(
                "Second page contains keyword target",
                60.0,
            )],
        }),
    ];

    // 1. Open search
    let _ = handle_search_open(&mut app);
    assert!(app.state.pdf.search.is_visible);

    // 2. Query search
    let _ = handle_search_query_changed(&mut app, "keyword".to_string());
    assert_eq!(app.state.pdf.search.total_matches(), 1);
    assert_eq!(app.state.pdf.search.info, "1/1");
    let matched = app.state.pdf.search.current_match().unwrap();
    assert_eq!(matched.page_index, 1);

    // 3. Next / Prev
    let _ = handle_search_next(&mut app);
    assert_eq!(app.state.pdf.search.info, "1/1");
    let _ = handle_search_prev(&mut app);
    assert_eq!(app.state.pdf.search.info, "1/1");

    // 4. Close search
    let _ = handle_search_close(&mut app);
    assert!(!app.state.pdf.search.is_visible);
    assert!(app.state.pdf.search.query.is_empty());
    assert_eq!(app.state.pdf.search.total_matches(), 0);
}
