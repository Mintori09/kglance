pub fn find_visible_page(
    page_y_offsets: &[f32],
    scroll_y: f32,
    view_height: f32,
    focus_fraction: f32,
) -> usize {
    if page_y_offsets.is_empty() {
        return 0;
    }

    let focus_y = scroll_y + view_height * focus_fraction;

    let idx = page_y_offsets.partition_point(|&offset| offset <= focus_y);
    idx.saturating_sub(1).min(page_y_offsets.len() - 1)
}

pub fn page_scroll_offset(page_y_offsets: &[f32], page_index: usize) -> f32 {
    page_y_offsets.get(page_index).copied().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_offsets() -> Vec<f32> {
        // 5 pages: height ~1130 each, spacing 10
        // offsets: [0, 1140, 2280, 3420, 4560]
        vec![0.0, 1140.0, 2280.0, 3420.0, 4560.0]
    }

    #[test]
    fn finds_first_page_at_top() {
        let offsets = sample_offsets();
        assert_eq!(find_visible_page(&offsets, 0.0, 800.0, 0.3), 0);
    }

    #[test]
    fn finds_correct_page_mid_scroll() {
        let offsets = sample_offsets();
        // scroll_y = 2500, view_h = 800, focus = 0.3 -> focus_y = 2740
        // Page 2 starts at 2280, page 3 at 3420 -> visible = page 2
        assert_eq!(find_visible_page(&offsets, 2500.0, 800.0, 0.3), 2);
    }

    #[test]
    fn finds_last_page_at_bottom() {
        let offsets = sample_offsets();
        assert_eq!(find_visible_page(&offsets, 9999.0, 800.0, 0.3), 4);
    }

    #[test]
    fn empty_offsets_returns_zero() {
        assert_eq!(find_visible_page(&[], 100.0, 500.0, 0.3), 0);
    }

    #[test]
    fn page_scroll_offset_returns_correct_values() {
        let offsets = sample_offsets();
        assert_eq!(page_scroll_offset(&offsets, 0), 0.0);
        assert_eq!(page_scroll_offset(&offsets, 2), 2280.0);
        assert_eq!(page_scroll_offset(&offsets, 99), 0.0); // out of bounds
    }
}
