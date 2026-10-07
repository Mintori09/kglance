use super::*;

fn mock_page_text() -> PdfPageText {
    PdfPageText {
        page_index: 0,
        lines: vec![
            PdfLine {
                text: "Hello World".to_string(),
                rect: [10.0, 10.0, 110.0, 30.0],
                chars: vec![
                    PdfChar {
                        ch: 'H',
                        rect: [10.0, 10.0, 20.0, 30.0],
                    },
                    PdfChar {
                        ch: 'e',
                        rect: [20.0, 10.0, 30.0, 30.0],
                    },
                    PdfChar {
                        ch: 'l',
                        rect: [30.0, 10.0, 40.0, 30.0],
                    },
                    PdfChar {
                        ch: 'l',
                        rect: [40.0, 10.0, 50.0, 30.0],
                    },
                    PdfChar {
                        ch: 'o',
                        rect: [50.0, 10.0, 60.0, 30.0],
                    },
                    PdfChar {
                        ch: ' ',
                        rect: [60.0, 10.0, 70.0, 30.0],
                    },
                    PdfChar {
                        ch: 'W',
                        rect: [70.0, 10.0, 80.0, 30.0],
                    },
                    PdfChar {
                        ch: 'o',
                        rect: [80.0, 10.0, 90.0, 30.0],
                    },
                    PdfChar {
                        ch: 'r',
                        rect: [90.0, 10.0, 100.0, 30.0],
                    },
                    PdfChar {
                        ch: 'l',
                        rect: [100.0, 10.0, 105.0, 30.0],
                    },
                    PdfChar {
                        ch: 'd',
                        rect: [105.0, 10.0, 110.0, 30.0],
                    },
                ],
            },
            PdfLine {
                text: "Second Line".to_string(),
                rect: [10.0, 35.0, 120.0, 55.0],
                chars: vec![
                    PdfChar {
                        ch: 'S',
                        rect: [10.0, 35.0, 20.0, 55.0],
                    },
                    PdfChar {
                        ch: 'e',
                        rect: [20.0, 35.0, 30.0, 55.0],
                    },
                    PdfChar {
                        ch: 'c',
                        rect: [30.0, 35.0, 40.0, 55.0],
                    },
                    PdfChar {
                        ch: 'o',
                        rect: [40.0, 35.0, 50.0, 55.0],
                    },
                    PdfChar {
                        ch: 'n',
                        rect: [50.0, 35.0, 60.0, 55.0],
                    },
                    PdfChar {
                        ch: 'd',
                        rect: [60.0, 35.0, 70.0, 55.0],
                    },
                    PdfChar {
                        ch: ' ',
                        rect: [70.0, 35.0, 80.0, 55.0],
                    },
                    PdfChar {
                        ch: 'L',
                        rect: [80.0, 35.0, 90.0, 55.0],
                    },
                    PdfChar {
                        ch: 'i',
                        rect: [90.0, 35.0, 95.0, 55.0],
                    },
                    PdfChar {
                        ch: 'n',
                        rect: [95.0, 35.0, 105.0, 55.0],
                    },
                    PdfChar {
                        ch: 'e',
                        rect: [105.0, 35.0, 115.0, 55.0],
                    },
                ],
            },
        ],
    }
}

#[test]
fn test_hit_test_on_mock_page() {
    let page = mock_page_text();

    let hit_before_h = page.hit_test(12.0, 20.0);
    assert_eq!(hit_before_h, Some((0, 0)));

    let hit_after_h = page.hit_test(18.0, 20.0);
    assert_eq!(hit_after_h, Some((0, 1)));

    let hit_before_w = page.hit_test(72.0, 20.0);
    assert_eq!(hit_before_w, Some((0, 6)));

    let hit_before_e = page.hit_test(22.0, 45.0);
    assert_eq!(hit_before_e, Some((1, 1)));
}

#[test]
fn test_is_point_over_text() {
    let page = mock_page_text();

    assert!(page.is_point_over_text(50.0, 20.0));
    assert!(page.is_point_over_text(50.0, 45.0));
    assert!(!page.is_point_over_text(200.0, 200.0));
}

#[test]
fn test_find_word_range() {
    let page = mock_page_text();

    let (start, end) = page.find_word_range(0, 1);
    assert_eq!((start, end), (0, 5));

    let (w_start, w_end) = page.find_word_range(0, 7);
    assert_eq!((w_start, w_end), (6, 11));
}

#[test]
fn test_selection_char_range_and_extraction() {
    let page = mock_page_text();
    let page_texts = vec![Some(page)];

    let selection = PdfSelection::new(
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 0,
        },
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 5,
        },
    );

    let extracted = extract_selected_text(&page_texts, &selection);
    assert_eq!(extracted, Some("Hello".to_string()));

    let multiline_sel = PdfSelection::new(
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 6,
        },
        PdfPosition {
            page: 0,
            line: 1,
            char_idx: 6,
        },
    );

    let multiline_text = extract_selected_text(&page_texts, &multiline_sel);
    assert_eq!(multiline_text, Some("World\nSecond".to_string()));
}

#[test]
fn test_compute_selection_rects() {
    let page = mock_page_text();
    let selection = PdfSelection::new(
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 0,
        },
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 5,
        },
    );

    let rects = compute_selection_rects(&page, 0, &selection, 2.0, 100.0, 50.0);
    assert_eq!(rects.len(), 1);
    let r = rects[0];
    assert_eq!(r.x, 100.0 + 10.0 * 2.0);
    assert_eq!(r.y, 50.0 + 10.0 * 2.0);
    assert_eq!(r.width, (60.0 - 10.0) * 2.0);
    assert_eq!(r.height, (30.0 - 10.0) * 2.0);
}

#[test]
fn test_drag_selection_workflow() {
    let mut state = crate::core::PdfState {
        page_count: 1,
        page_texts: vec![Some(mock_page_text())],
        ..Default::default()
    };

    // 1. Drag start at line 0, char 0
    let start_pos = PdfPosition {
        page: 0,
        line: 0,
        char_idx: 0,
    };
    handle_selection_drag_start(&mut state, start_pos);
    assert!(state.is_selecting);
    assert_eq!(state.selection_drag_start, Some(start_pos));
    assert_eq!(
        state.selection,
        Some(PdfSelection::new(start_pos, start_pos))
    );

    // 2. Drag update to line 0, char 5 ("Hello")
    let move_pos = PdfPosition {
        page: 0,
        line: 0,
        char_idx: 5,
    };
    handle_selection_drag_update(&mut state, move_pos);
    assert_eq!(
        state.selection,
        Some(PdfSelection::new(start_pos, move_pos))
    );
    assert_eq!(state.selected_text, Some("Hello".to_string()));

    // Verify rect calculation during drag
    let page_text = state.page_texts[0].as_ref().unwrap();
    let rects = compute_selection_rects(page_text, 0, &state.selection.unwrap(), 1.0, 0.0, 0.0);
    assert_eq!(rects.len(), 1);
    assert!(rects[0].width > 0.0);
    assert!(rects[0].height > 0.0);

    // 3. Drag end
    handle_selection_drag_end(&mut state);
    assert!(!state.is_selecting);
    assert_eq!(state.selected_text, Some("Hello".to_string()));

    // 4. Selection clear
    handle_selection_clear(&mut state);
    assert!(state.selection.is_none());
    assert!(state.selected_text.is_none());
}

#[test]
fn test_2d_hit_test_scattered_blocks() {
    let page = PdfPageText {
        page_index: 0,
        lines: vec![
            PdfLine {
                text: "Title in Middle".to_string(),
                rect: [150.0, 100.0, 350.0, 130.0],
                chars: vec![
                    PdfChar {
                        ch: 'T',
                        rect: [150.0, 100.0, 170.0, 130.0],
                    },
                    PdfChar {
                        ch: 'i',
                        rect: [170.0, 100.0, 180.0, 130.0],
                    },
                ],
            },
            PdfLine {
                text: "Header at Top".to_string(),
                rect: [100.0, 20.0, 300.0, 40.0],
                chars: vec![
                    PdfChar {
                        ch: 'H',
                        rect: [100.0, 20.0, 120.0, 40.0],
                    },
                    PdfChar {
                        ch: 'e',
                        rect: [120.0, 20.0, 130.0, 40.0],
                    },
                ],
            },
        ],
    };

    // Click on Header at top (20..40)
    let hit_header = page.hit_test(105.0, 30.0);
    assert_eq!(hit_header, Some((1, 0)));

    // Click on Title in middle (100..130)
    let hit_title = page.hit_test(155.0, 110.0);
    assert_eq!(hit_title, Some((0, 0)));
}

#[test]
fn test_select_all_workflow() {
    let mut state = crate::core::PdfState {
        page_count: 1,
        page_texts: vec![Some(mock_page_text())],
        ..Default::default()
    };

    handle_select_all(&mut state);
    assert!(state.selection.is_some());
    let selected_str = state.selected_text.unwrap();
    assert!(selected_str.contains("Hello World"));
    assert!(selected_str.contains("Second Line"));
    assert!(state.selected_html.is_some());
    let html_str = state.selected_html.unwrap();
    assert!(html_str.contains("<meta http-equiv=\"content-type\""));
    assert!(html_str.contains("Hello World"));
}

#[test]
fn test_rich_text_table_clustering() {
    let page = PdfPageText {
        page_index: 0,
        lines: vec![
            PdfLine {
                text: "GVHD:".to_string(),
                rect: [100.0, 200.0, 160.0, 215.0],
                chars: vec![
                    PdfChar {
                        ch: 'G',
                        rect: [100.0, 200.0, 110.0, 215.0],
                    },
                    PdfChar {
                        ch: 'V',
                        rect: [110.0, 200.0, 120.0, 215.0],
                    },
                    PdfChar {
                        ch: 'H',
                        rect: [120.0, 200.0, 130.0, 215.0],
                    },
                    PdfChar {
                        ch: 'D',
                        rect: [130.0, 200.0, 140.0, 215.0],
                    },
                    PdfChar {
                        ch: ':',
                        rect: [140.0, 200.0, 150.0, 215.0],
                    },
                ],
            },
            PdfLine {
                text: "TS. Nguyen Van A".to_string(),
                rect: [250.0, 201.0, 420.0, 216.0],
                chars: vec![
                    PdfChar {
                        ch: 'T',
                        rect: [250.0, 201.0, 260.0, 216.0],
                    },
                    PdfChar {
                        ch: 'S',
                        rect: [260.0, 201.0, 270.0, 216.0],
                    },
                    PdfChar {
                        ch: '.',
                        rect: [270.0, 201.0, 275.0, 216.0],
                    },
                ],
            },
            PdfLine {
                text: "SVTH:".to_string(),
                rect: [100.0, 230.0, 160.0, 245.0],
                chars: vec![
                    PdfChar {
                        ch: 'S',
                        rect: [100.0, 230.0, 110.0, 245.0],
                    },
                    PdfChar {
                        ch: 'V',
                        rect: [110.0, 230.0, 120.0, 245.0],
                    },
                    PdfChar {
                        ch: 'T',
                        rect: [120.0, 230.0, 130.0, 245.0],
                    },
                    PdfChar {
                        ch: 'H',
                        rect: [130.0, 230.0, 140.0, 245.0],
                    },
                    PdfChar {
                        ch: ':',
                        rect: [140.0, 230.0, 150.0, 245.0],
                    },
                ],
            },
            PdfLine {
                text: "Tran Van B".to_string(),
                rect: [250.0, 231.0, 400.0, 246.0],
                chars: vec![
                    PdfChar {
                        ch: 'T',
                        rect: [250.0, 231.0, 260.0, 246.0],
                    },
                    PdfChar {
                        ch: 'r',
                        rect: [260.0, 231.0, 270.0, 246.0],
                    },
                    PdfChar {
                        ch: 'a',
                        rect: [270.0, 231.0, 280.0, 246.0],
                    },
                    PdfChar {
                        ch: 'n',
                        rect: [280.0, 231.0, 290.0, 246.0],
                    },
                ],
            },
        ],
    };

    let sel = PdfSelection::new(
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 0,
        },
        PdfPosition {
            page: 0,
            line: 3,
            char_idx: 4,
        },
    );

    let (plain, html) = extract_selected_content(&[Some(page)], &sel);

    assert!(plain.is_some());
    let plain_str = plain.unwrap();
    // Plain text has tab separating the columns
    assert!(plain_str.contains("GVHD:\tTS."));
    assert!(plain_str.contains("SVTH:\tTran"));

    assert!(html.is_some());
    let html_str = html.unwrap();
    // HTML has table with rows and cells
    assert!(html_str.contains("<table"));
    assert!(html_str.contains("<tbody>"));
    assert!(html_str.contains("<tr><td"));
    assert!(html_str.contains("GVHD:</td>"));
    assert!(html_str.contains("TS.</td>"));
}

#[test]
fn test_rich_text_heading_detection_and_escaping() {
    let page = PdfPageText {
        page_index: 0,
        lines: vec![PdfLine {
            text: "PROJECT REPORT & SUMMARY <2026>".to_string(),
            rect: [100.0, 50.0, 450.0, 75.0], // height = 25.0 (>= 16.0 heading)
            chars: vec![
                PdfChar {
                    ch: 'P',
                    rect: [100.0, 50.0, 120.0, 75.0],
                },
                PdfChar {
                    ch: 'R',
                    rect: [120.0, 50.0, 140.0, 75.0],
                },
                PdfChar {
                    ch: 'O',
                    rect: [140.0, 50.0, 160.0, 75.0],
                },
                PdfChar {
                    ch: 'J',
                    rect: [160.0, 50.0, 180.0, 75.0],
                },
                PdfChar {
                    ch: 'E',
                    rect: [180.0, 50.0, 200.0, 75.0],
                },
                PdfChar {
                    ch: 'C',
                    rect: [200.0, 50.0, 220.0, 75.0],
                },
                PdfChar {
                    ch: 'T',
                    rect: [220.0, 50.0, 240.0, 75.0],
                },
                PdfChar {
                    ch: ' ',
                    rect: [240.0, 50.0, 250.0, 75.0],
                },
                PdfChar {
                    ch: '&',
                    rect: [250.0, 50.0, 270.0, 75.0],
                },
                PdfChar {
                    ch: ' ',
                    rect: [270.0, 50.0, 280.0, 75.0],
                },
                PdfChar {
                    ch: '<',
                    rect: [280.0, 50.0, 295.0, 75.0],
                },
                PdfChar {
                    ch: '>',
                    rect: [295.0, 50.0, 310.0, 75.0],
                },
            ],
        }],
    };

    let sel = PdfSelection::new(
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 0,
        },
        PdfPosition {
            page: 0,
            line: 0,
            char_idx: 12,
        },
    );

    let (plain, html) = extract_selected_content(&[Some(page)], &sel);

    assert_eq!(plain, Some("PROJECT & <>".to_string()));
    assert!(html.is_some());
    let html_str = html.unwrap();
    // HTML escapes '&' and '<', '>' and applies bold heading style
    assert!(html_str.contains("font-weight: bold"));
    assert!(html_str.contains("&amp;"));
    assert!(html_str.contains("&lt;&gt;"));
}

#[test]
fn test_multipage_drag_selection_workflow() {
    let page0 = mock_page_text();
    let page1 = PdfPageText {
        page_index: 1,
        lines: vec![PdfLine {
            text: "Page Two Line One".to_string(),
            rect: [10.0, 10.0, 150.0, 30.0],
            chars: vec![
                PdfChar {
                    ch: 'P',
                    rect: [10.0, 10.0, 20.0, 30.0],
                },
                PdfChar {
                    ch: 'a',
                    rect: [20.0, 10.0, 30.0, 30.0],
                },
                PdfChar {
                    ch: 'g',
                    rect: [30.0, 10.0, 40.0, 30.0],
                },
                PdfChar {
                    ch: 'e',
                    rect: [40.0, 10.0, 50.0, 30.0],
                },
            ],
        }],
    };

    let mut state = crate::core::PdfState {
        page_count: 2,
        page_texts: vec![Some(page0.clone()), Some(page1.clone())],
        ..Default::default()
    };

    // 1. Drag starts on page 0 at line 0, char 6 ("World")
    let start_pos = PdfPosition {
        page: 0,
        line: 0,
        char_idx: 6,
    };
    handle_selection_drag_start(&mut state, start_pos);
    assert!(state.is_selecting);

    // 2. Drag moves across to page 1 at line 0, char 4 ("Page")
    let move_pos = PdfPosition {
        page: 1,
        line: 0,
        char_idx: 4,
    };
    handle_selection_drag_update(&mut state, move_pos);

    let sel = state.selection.unwrap();
    assert_eq!(sel.start, start_pos);
    assert_eq!(sel.end, move_pos);

    // Page 0 rects should cover line 0 from char 6, plus all of line 1
    let rects_page0 = compute_selection_rects(&page0, 0, &sel, 1.0, 0.0, 0.0);
    assert_eq!(rects_page0.len(), 2);

    // Page 1 rects should cover line 0 up to char 4
    let rects_page1 = compute_selection_rects(&page1, 1, &sel, 1.0, 0.0, 0.0);
    assert_eq!(rects_page1.len(), 1);

    // 3. End drag
    handle_selection_drag_end(&mut state);
    assert!(!state.is_selecting);

    let text = state.selected_text.unwrap();
    assert!(text.contains("World"));
    assert!(text.contains("Second Line"));
    assert!(text.contains("Page"));
}

#[test]
fn test_sort_lines_reading_order_scattered_presentation_blocks() {
    let mut lines = vec![
        PdfLine {
            text: "Bullet 4 (Bottom)".to_string(),
            rect: [100.0, 280.0, 500.0, 310.0],
            chars: vec![],
        },
        PdfLine {
            text: "Slide Header (Top)".to_string(),
            rect: [100.0, 20.0, 500.0, 50.0],
            chars: vec![],
        },
        PdfLine {
            text: "Bullet 2".to_string(),
            rect: [100.0, 180.0, 500.0, 210.0],
            chars: vec![],
        },
        PdfLine {
            text: "Section a".to_string(),
            rect: [100.0, 80.0, 500.0, 110.0],
            chars: vec![],
        },
        PdfLine {
            text: "Bullet 3".to_string(),
            rect: [100.0, 230.0, 500.0, 260.0],
            chars: vec![],
        },
        PdfLine {
            text: "Bullet 1".to_string(),
            rect: [100.0, 130.0, 500.0, 160.0],
            chars: vec![],
        },
    ];

    crate::features::pdf::parser::sort_lines_reading_order(&mut lines);

    assert_eq!(lines[0].text, "Slide Header (Top)");
    assert_eq!(lines[1].text, "Section a");
    assert_eq!(lines[2].text, "Bullet 1");
    assert_eq!(lines[3].text, "Bullet 2");
    assert_eq!(lines[4].text, "Bullet 3");
    assert_eq!(lines[5].text, "Bullet 4 (Bottom)");
}
