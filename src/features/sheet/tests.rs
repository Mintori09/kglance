use crate::app::KglanceApp;
use crate::core::PreviewData;
use crate::features::common::parser::traits::PreviewParser;
use crate::features::common::parser::types::ParsedContent;
use crate::features::sheet::parser::{
    CsvParser, column_index_to_letter, infer_column_types_and_widths, sniff_delimiter,
};
use crate::features::sheet::types::{ColumnMeta, ColumnType, SheetInfo};
use crate::features::sheet::update::{
    compute_prefix_widths, handle_sheet_tab_clicked, handle_smooth_scroll_tick,
    handle_wheel_scrolled, populate_state, recompute_display_indices,
};
use crate::features::sheet::view::{
    COL_SPACING, ROW_HEIGHT, ROW_STEP, ROWS_LIST_SPACING, compute_csv_column_window,
    compute_csv_virtual_window, estimate_row_height, view_spreadsheet,
};
use crate::ui::theme::AppTheme;
use std::io::Write;
use std::time::Instant;
use tempfile::NamedTempFile;

fn make_uniform_prefix_heights(total_rows: usize, row_step: f32) -> Vec<f32> {
    let mut prefix = Vec::with_capacity(total_rows + 1);
    let mut acc = 0.0;
    prefix.push(acc);
    for _ in 0..total_rows {
        acc += row_step;
        prefix.push(acc);
    }
    prefix
}

#[test]
fn test_column_index_to_letter() {
    assert_eq!(column_index_to_letter(0), "A");
    assert_eq!(column_index_to_letter(1), "B");
    assert_eq!(column_index_to_letter(25), "Z");
    assert_eq!(column_index_to_letter(26), "AA");
    assert_eq!(column_index_to_letter(27), "AB");
    assert_eq!(column_index_to_letter(51), "AZ");
    assert_eq!(column_index_to_letter(52), "BA");
    assert_eq!(column_index_to_letter(701), "ZZ");
    assert_eq!(column_index_to_letter(702), "AAA");
}

#[test]
fn test_compute_csv_virtual_window_empty() {
    let prefix = vec![0.0];
    let win = compute_csv_virtual_window(&prefix, 0.0, 800.0);
    assert_eq!(win.visible_start, 0);
    assert_eq!(win.visible_end, 0);
    assert_eq!(win.top_spacer_height, 0.0);
    assert_eq!(win.bottom_spacer_height, 0.0);
}

#[test]
fn test_compute_csv_virtual_window_small_table() {
    let prefix = make_uniform_prefix_heights(10, ROW_STEP);
    let win = compute_csv_virtual_window(&prefix, 0.0, 600.0);
    assert_eq!(win.visible_start, 0);
    assert_eq!(win.visible_end, 10);
    assert_eq!(win.top_spacer_height, 0.0);
    assert_eq!(win.bottom_spacer_height, 0.0);
}

#[test]
fn test_compute_csv_virtual_window_large_invariance() {
    let total_rows = 5000;
    let prefix = make_uniform_prefix_heights(total_rows, ROW_STEP);
    let expected_total_height = *prefix.last().unwrap();

    for scroll_y in [0.0, 300.0, 1500.0, 15000.0, 140000.0] {
        let win = compute_csv_virtual_window(&prefix, scroll_y, 800.0);
        assert!(win.visible_start <= win.visible_end);
        assert!(win.visible_end <= total_rows);

        let rendered_height = prefix[win.visible_end] - prefix[win.visible_start];
        let sum_height = win.top_spacer_height + rendered_height + win.bottom_spacer_height;

        assert!(
            (sum_height - expected_total_height).abs() < 0.01,
            "Height mismatch at scroll_y {scroll_y}: sum {sum_height} vs expected {expected_total_height}"
        );
    }
}

#[test]
fn test_compute_csv_virtual_window_variable_heights() {
    let mut heights = Vec::new();
    let mut prefix = vec![0.0];
    let mut acc = 0.0;
    for i in 0..1000 {
        let h = if i % 5 == 0 { 120.0 } else { 26.0 };
        heights.push(h);
        acc += h + ROWS_LIST_SPACING;
        prefix.push(acc);
    }
    let total_h = acc;

    for scroll_y in [0.0, 250.0, 1200.0, 8000.0, 25000.0] {
        let win = compute_csv_virtual_window(&prefix, scroll_y, 800.0);
        assert!(win.visible_start <= win.visible_end);
        assert!(win.visible_end <= 1000);

        let rendered_height = prefix[win.visible_end] - prefix[win.visible_start];
        let sum_height = win.top_spacer_height + rendered_height + win.bottom_spacer_height;

        assert!(
            (sum_height - total_h).abs() < 0.01,
            "Variable height mismatch at scroll_y {scroll_y}: sum {sum_height} vs expected {total_h}"
        );
    }
}

#[test]
fn test_estimate_row_height() {
    let columns = vec![
        ColumnMeta {
            name: "Col1".to_string(),
            col_type: ColumnType::Text,
            width: 150.0,
        },
        ColumnMeta {
            name: "Col2".to_string(),
            col_type: ColumnType::Text,
            width: 200.0,
        },
    ];

    let short_row = vec!["Short".to_string(), "123".to_string()];
    let short_h = estimate_row_height(&short_row, &columns);
    assert_eq!(short_h, ROW_HEIGHT);

    let long_text = "This is a very long paragraph inside this cell.".to_string();
    let long_row = vec!["Title".to_string(), long_text];
    let long_h = estimate_row_height(&long_row, &columns);
    assert!(long_h > ROW_HEIGHT);
    assert_eq!(long_h, 41.0);
}

#[test]
fn test_compute_csv_virtual_window_bottom_boundary() {
    let total_rows = 1000;
    let prefix = make_uniform_prefix_heights(total_rows, ROW_STEP);
    let total_h = *prefix.last().unwrap();
    let vh = 600.0;
    let scroll_y = total_h - vh;

    let win = compute_csv_virtual_window(&prefix, scroll_y, vh);
    assert_eq!(win.visible_end, total_rows);
    assert_eq!(win.bottom_spacer_height, 0.0);
    assert!(win.top_spacer_height > 0.0);
}

#[test]
fn test_sniff_delimiter_variants() {
    let csv_sample = b"name,age,city\nAlice,30,Hanoi\nBob,25,Da Nang\n";
    assert_eq!(sniff_delimiter(csv_sample, Some("csv")), b',');

    let semicolon_sample = b"name;age;city\nAlice;30;Hanoi\nBob;25;Da Nang\n";
    assert_eq!(sniff_delimiter(semicolon_sample, None), b';');

    let tsv_sample = b"name\tage\tcity\nAlice\t30\tHanoi\nBob\t25\tDa Nang\n";
    assert_eq!(sniff_delimiter(tsv_sample, Some("tsv")), b'\t');

    let pipe_sample = b"name|age|city\nAlice|30|Hanoi\nBob|25|Da Nang\n";
    assert_eq!(sniff_delimiter(pipe_sample, None), b'|');
}

#[test]
fn test_infer_column_types() {
    let headers = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ];
    let rows = vec![
        vec![
            "Alice".to_string(),
            "25".to_string(),
            "98.5".to_string(),
            "".to_string(),
        ],
        vec![
            "Bob".to_string(),
            "30".to_string(),
            "88.0".to_string(),
            "".to_string(),
        ],
        vec![
            "Charlie".to_string(),
            "45".to_string(),
            "76.25".to_string(),
            "".to_string(),
        ],
    ];

    let cols = infer_column_types_and_widths(&headers, &rows);
    assert_eq!(cols.len(), 4);
    assert_eq!(cols[0].col_type, ColumnType::Text);
    assert_eq!(cols[1].col_type, ColumnType::Integer);
    assert_eq!(cols[2].col_type, ColumnType::Float);
    assert_eq!(cols[3].col_type, ColumnType::Empty);
}

#[test]
fn test_date_column_inference_and_sorting() {
    let headers = vec!["A".to_string(), "B".to_string()];
    let rows = vec![
        vec!["30/11/2025".to_string(), "End of Nov".to_string()],
        vec!["2025-01-12".to_string(), "Jan 12".to_string()],
        vec![
            "06/12/2025 11:07:33".to_string(),
            "Dec 6 Morning".to_string(),
        ],
        vec!["08/12/2025 21:00:27".to_string(), "Dec 8 Night".to_string()],
        vec!["2025-09-13".to_string(), "Sep 13".to_string()],
    ];

    let cols = infer_column_types_and_widths(&headers, &rows);
    assert_eq!(cols[0].col_type, ColumnType::Date);
    assert_eq!(cols[1].col_type, ColumnType::Text);

    let sheet = SheetInfo {
        name: "TestDate".to_string(),
        headers,
        rows,
        columns: cols,
    };

    let sorted_asc = recompute_display_indices(&sheet, "", Some(0), Some(true));
    assert_eq!(sorted_asc, vec![1, 4, 0, 2, 3]);

    let sorted_desc = recompute_display_indices(&sheet, "", Some(0), Some(false));
    assert_eq!(sorted_desc, vec![3, 2, 0, 4, 1]);
}

#[test]
fn test_numeric_sorting_and_filtering() {
    let headers = vec!["A".to_string(), "B".to_string()];
    let rows = vec![
        vec!["Alice".to_string(), "2".to_string()],
        vec!["Bob".to_string(), "100".to_string()],
        vec!["Charlie".to_string(), "20".to_string()],
        vec!["Dave".to_string(), "10".to_string()],
    ];
    let columns = vec![
        ColumnMeta {
            name: "A".to_string(),
            col_type: ColumnType::Text,
            width: 100.0,
        },
        ColumnMeta {
            name: "B".to_string(),
            col_type: ColumnType::Integer,
            width: 80.0,
        },
    ];

    let sheet = SheetInfo {
        name: "Sheet1".to_string(),
        headers,
        rows,
        columns,
    };

    let sorted_asc = recompute_display_indices(&sheet, "", Some(1), Some(true));
    assert_eq!(sorted_asc, vec![0, 3, 2, 1]);

    let sorted_desc = recompute_display_indices(&sheet, "", Some(1), Some(false));
    assert_eq!(sorted_desc, vec![1, 2, 3, 0]);

    let filtered = recompute_display_indices(&sheet, "e", None, None);
    assert_eq!(filtered, vec![0, 2, 3]);
}

#[test]
fn test_csv_parser_rfc4180_quotes_and_multiline() {
    let mut temp = NamedTempFile::new().unwrap();
    let csv_data = "name,description,count\n\
                    \"Alice, Jr.\",\"Likes apples, oranges\",10\n\
                    \"Bob \"\"The Builder\"\"\",\"Works with\nmultiline tools\",5\n";
    temp.write_all(csv_data.as_bytes()).unwrap();

    let parser = CsvParser;
    let res = parser.parse(temp.path()).unwrap();

    if let ParsedContent::Spreadsheet { sheets } = res {
        assert_eq!(sheets.len(), 1);
        let sheet = &sheets[0];
        assert_eq!(
            sheet.headers,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
        assert_eq!(sheet.rows.len(), 3);
        assert_eq!(sheet.rows[0][0], "name");
        assert_eq!(sheet.rows[0][1], "description");
        assert_eq!(sheet.rows[0][2], "count");
        assert_eq!(sheet.rows[1][0], "Alice, Jr.");
        assert_eq!(sheet.rows[1][1], "Likes apples, oranges");
        assert_eq!(sheet.rows[1][2], "10");
        assert_eq!(sheet.rows[2][0], "Bob \"The Builder\"");
        assert_eq!(sheet.rows[2][1], "Works with\nmultiline tools");
        assert_eq!(sheet.rows[2][2], "5");
    } else {
        panic!("Expected ParsedContent::Spreadsheet");
    }
}

#[test]
fn test_bottom_sheet_tab_switching() {
    let sheet1 = SheetInfo {
        name: "So Sánh Giá Các Shop".to_string(),
        headers: vec!["A".to_string(), "B".to_string()],
        rows: vec![vec!["Row1".to_string(), "100".to_string()]],
        columns: vec![
            ColumnMeta {
                name: "A".to_string(),
                col_type: ColumnType::Text,
                width: 100.0,
            },
            ColumnMeta {
                name: "B".to_string(),
                col_type: ColumnType::Integer,
                width: 80.0,
            },
        ],
    };
    let sheet2 = SheetInfo {
        name: "Danh Sách Toàn Bộ Dịch Vụ".to_string(),
        headers: vec!["A".to_string(), "B".to_string(), "C".to_string()],
        rows: vec![vec![
            "Service1".to_string(),
            "Active".to_string(),
            "2026".to_string(),
        ]],
        columns: vec![
            ColumnMeta {
                name: "A".to_string(),
                col_type: ColumnType::Text,
                width: 120.0,
            },
            ColumnMeta {
                name: "B".to_string(),
                col_type: ColumnType::Text,
                width: 100.0,
            },
            ColumnMeta {
                name: "C".to_string(),
                col_type: ColumnType::Integer,
                width: 80.0,
            },
        ],
    };

    let mut app = KglanceApp::default();
    populate_state(&mut app.state, &[sheet1.clone(), sheet2.clone()], 0);
    assert_eq!(app.state.spreadsheet.active_sheet, 0);
    assert_eq!(app.state.spreadsheet.display_indices.len(), 1);

    // Switch to tab 1
    let _ = handle_sheet_tab_clicked(&mut app, 1);
    assert_eq!(app.state.spreadsheet.active_sheet, 1);
    assert_eq!(app.state.spreadsheet.sheets.len(), 2);
    assert_eq!(app.state.spreadsheet.display_indices.len(), 1);

    // Verify view rendering compiles and produces elements
    let _element = view_spreadsheet(&app.state.spreadsheet, AppTheme::Dark);
}

#[test]
fn test_spreadsheet_smooth_scroll_and_touchpad_inertia() {
    let headers = vec!["A".to_string(), "B".to_string()];
    let rows: Vec<Vec<String>> = (0..100)
        .map(|i| vec![format!("Row {i}"), format!("Val {i}")])
        .collect();
    let columns = vec![
        ColumnMeta {
            name: "A".to_string(),
            col_type: ColumnType::Text,
            width: 150.0,
        },
        ColumnMeta {
            name: "B".to_string(),
            col_type: ColumnType::Text,
            width: 150.0,
        },
    ];
    let sheet = SheetInfo {
        name: "Sheet1".to_string(),
        headers,
        rows,
        columns,
    };

    let mut app = KglanceApp::default();
    populate_state(&mut app.state, std::slice::from_ref(&sheet), 0);
    app.current_content = Some(PreviewData::Spreadsheet {
        sheets: vec![sheet],
        active_sheet: 0,
    });

    // 1. Discrete mouse wheel scroll (vertical)
    let _ = handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
    );
    assert!(app.state.spreadsheet.smooth_scroll.is_animating());
    assert!(app.is_active_smooth_scrolling());

    // Advance tick
    let now = Instant::now() + std::time::Duration::from_millis(16);
    let _ = handle_smooth_scroll_tick(&mut app, now);
    assert!(app.state.spreadsheet.scroll_y > 0.0);

    // 2. Touchpad motion and fling inertia (vertical)
    let extent = crate::core::scroll::ViewportExtent {
        content_height: app.state.spreadsheet.total_content_height,
        viewport_height: app.state.spreadsheet.viewport_height,
    };
    app.state.spreadsheet.scroll_y = 500.0;
    app.state.spreadsheet.smooth_scroll.stop(500.0);
    app.state.spreadsheet.scroll_controller.stop(500.0);

    let mut t = Instant::now();
    for i in 1..=6 {
        t += std::time::Duration::from_millis(8);
        app.state.spreadsheet.scroll_controller.handle_input(
            crate::core::scroll::ScrollInput::Motion {
                delta_x: 0.0,
                delta_y: (i * 15) as f32,
                time: t,
            },
            extent,
        );
    }
    assert_eq!(
        app.state.spreadsheet.scroll_controller.state(),
        crate::core::scroll::GestureState::Dragging
    );
    assert!(app.is_active_smooth_scrolling());

    // Trigger fling release
    app.state.spreadsheet.scroll_controller.handle_input(
        crate::core::scroll::ScrollInput::End {
            time: t + std::time::Duration::from_millis(5),
        },
        extent,
    );
    assert_eq!(
        app.state.spreadsheet.scroll_controller.state(),
        crate::core::scroll::GestureState::Flinging
    );
    assert!(app.state.spreadsheet.scroll_controller.is_animating());

    // 3. Shift + Wheel scroll (horizontal)
    app.state.spreadsheet.viewport_width = 200.0;
    app.shift_held = true;
    let _ = handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
    );
    assert!(app.state.spreadsheet.smooth_scroll_x.is_animating());
    assert!(app.is_active_smooth_scrolling());
    app.shift_held = false;

    // 4. Native discrete horizontal wheel scroll
    app.state.spreadsheet.smooth_scroll_x.stop(0.0);
    let _ = handle_wheel_scrolled(
        &mut app,
        iced::mouse::ScrollDelta::Lines { x: -1.0, y: 0.0 },
    );
    assert!(app.state.spreadsheet.smooth_scroll_x.is_animating());

    // 5. Horizontal touchpad gesture fling inertia
    let extent_x = crate::core::scroll::ViewportExtent {
        content_height: app.state.spreadsheet.total_content_width,
        viewport_height: app.state.spreadsheet.viewport_width,
    };
    app.state.spreadsheet.scroll_x = 50.0;
    app.state.spreadsheet.smooth_scroll_x.stop(50.0);
    app.state.spreadsheet.scroll_controller_x.stop(50.0);

    let mut tx = Instant::now();
    for i in 1..=6 {
        tx += std::time::Duration::from_millis(8);
        app.state.spreadsheet.scroll_controller_x.handle_input(
            crate::core::scroll::ScrollInput::Motion {
                delta_x: 0.0,
                delta_y: (i * 20) as f32,
                time: tx,
            },
            extent_x,
        );
    }
    assert_eq!(
        app.state.spreadsheet.scroll_controller_x.state(),
        crate::core::scroll::GestureState::Dragging
    );
    assert!(app.is_active_smooth_scrolling());

    app.state.spreadsheet.scroll_controller_x.handle_input(
        crate::core::scroll::ScrollInput::End {
            time: tx + std::time::Duration::from_millis(5),
        },
        extent_x,
    );
    assert_eq!(
        app.state.spreadsheet.scroll_controller_x.state(),
        crate::core::scroll::GestureState::Flinging
    );
    assert!(app.state.spreadsheet.scroll_controller_x.is_animating());

    // 6. Simultaneous 2D animation tick
    let tick_time = tx + std::time::Duration::from_millis(16);
    let _ = handle_smooth_scroll_tick(&mut app, tick_time);
    assert!(app.state.spreadsheet.scroll_x != 50.0 || app.state.spreadsheet.scroll_y != 0.0);
}

#[test]
fn test_compute_csv_column_window_empty() {
    let prefix = vec![0.0];
    let win = compute_csv_column_window(&prefix, 0.0, 1000.0);
    assert_eq!(win.visible_start, 0);
    assert_eq!(win.visible_end, 0);
    assert_eq!(win.left_spacer_width, 0.0);
    assert_eq!(win.right_spacer_width, 0.0);
}

#[test]
fn test_compute_csv_column_window_invariance() {
    let cols: Vec<ColumnMeta> = (0..50)
        .map(|i| ColumnMeta {
            name: column_index_to_letter(i),
            col_type: ColumnType::Text,
            width: 120.0,
        })
        .collect();
    let prefix = compute_prefix_widths(&cols);
    let total_width = *prefix.last().unwrap();

    for scroll_x in [0.0, 50.0, 300.0, 1500.0, 5000.0] {
        let win = compute_csv_column_window(&prefix, scroll_x, 1000.0);
        assert!(win.visible_start <= win.visible_end);
        assert!(win.visible_end <= 50);

        let rendered_width = prefix[win.visible_end] - prefix[win.visible_start];
        let left_part = if win.visible_start > 0 {
            win.left_spacer_width + COL_SPACING
        } else {
            0.0
        };
        let right_part = if win.visible_end < 50 {
            win.right_spacer_width + COL_SPACING
        } else {
            0.0
        };
        let sum_width = left_part + rendered_width + right_part;
        assert!(
            (sum_width - total_width).abs() < 0.01,
            "Width mismatch at scroll_x {scroll_x}: sum {sum_width} vs expected {total_width}"
        );
    }
}
