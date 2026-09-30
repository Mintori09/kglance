use crate::features::csv::view::{ROW_STEP, compute_csv_virtual_window};

#[test]
fn test_compute_csv_virtual_window_empty() {
    let win = compute_csv_virtual_window(0, 0.0, 800.0);
    assert_eq!(win.visible_start, 0);
    assert_eq!(win.visible_end, 0);
    assert_eq!(win.top_spacer_height, 0.0);
    assert_eq!(win.bottom_spacer_height, 0.0);
}

#[test]
fn test_compute_csv_virtual_window_small_table() {
    let win = compute_csv_virtual_window(10, 0.0, 600.0);
    assert_eq!(win.visible_start, 0);
    assert_eq!(win.visible_end, 10);
    assert_eq!(win.top_spacer_height, 0.0);
    assert_eq!(win.bottom_spacer_height, 0.0);
}

#[test]
fn test_compute_csv_virtual_window_large_invariance() {
    let total_rows = 5000;
    let expected_total_height = total_rows as f32 * ROW_STEP;

    for scroll_y in [0.0, 300.0, 1500.0, 15000.0, 140000.0] {
        let win = compute_csv_virtual_window(total_rows, scroll_y, 800.0);
        assert!(win.visible_start <= win.visible_end);
        assert!(win.visible_end <= total_rows);

        let visible_items = win.visible_end - win.visible_start;
        let rendered_height = visible_items as f32 * ROW_STEP;
        let sum_height = win.top_spacer_height + rendered_height + win.bottom_spacer_height;

        assert!(
            (sum_height - expected_total_height).abs() < 0.01,
            "Height mismatch at scroll_y {scroll_y}: sum {sum_height} vs expected {expected_total_height}"
        );
    }
}

#[test]
fn test_compute_csv_virtual_window_bottom_boundary() {
    let total_rows = 1000;
    let total_h = total_rows as f32 * ROW_STEP;
    let vh = 600.0;
    let scroll_y = total_h - vh;

    let win = compute_csv_virtual_window(total_rows, scroll_y, vh);
    assert_eq!(win.visible_end, total_rows);
    assert_eq!(win.bottom_spacer_height, 0.0);
    assert!(win.top_spacer_height > 0.0);
}
