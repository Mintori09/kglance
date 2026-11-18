use crate::features::grid::constants::ELLIPSIS;

pub fn calculate_column_count(container_width: f32, item_width: f32, gap: f32) -> usize {
    ((container_width - gap) / (item_width + gap))
        .floor()
        .max(1.0) as usize
}

pub fn calculate_horizontal_padding(
    container_width: f32,
    columns_count: usize,
    item_width: f32,
    gap: f32,
) -> f32 {
    let columns_f32 = columns_count as f32;
    let total_used_width = columns_f32 * item_width + (columns_f32 - 1.0) * gap;
    ((container_width - total_used_width) / 2.0).max(0.0)
}

pub fn calculate_visible_row_range(
    total_rows: usize,
    row_step: f32,
    scroll_y: f32,
    viewport_height: f32,
    overscan_rows: usize,
) -> (usize, usize) {
    if total_rows == 0 || row_step <= 0.0 {
        return (0, 0);
    }

    if viewport_height <= 0.0 {
        return (0, total_rows.min(12));
    }

    let first = (scroll_y.max(0.0) / row_step).floor() as usize;
    let visible_count = (viewport_height / row_step).ceil().max(1.0) as usize + 1;
    let end = (first + visible_count + overscan_rows).min(total_rows);
    let start = first.saturating_sub(overscan_rows).min(end);
    (start, end)
}

pub fn truncate_middle(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }

    let segment_length = max_chars.saturating_sub(1) / 2;
    let start_segment: String = text.chars().take(segment_length).collect();
    let end_segment: String = text
        .chars()
        .rev()
        .take(segment_length)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    format!("{}{}{}", start_segment, ELLIPSIS, end_segment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_visible_row_range_clamps_inverted_range() {
        let (start, end) = calculate_visible_row_range(1, 200.0, 5000.0, 600.0, 2);
        assert!(start <= end);
        assert_eq!((start, end), (1, 1));
    }

    #[test]
    fn test_calculate_visible_row_range_normal() {
        let (start, end) = calculate_visible_row_range(20, 100.0, 300.0, 400.0, 2);
        assert_eq!((start, end), (1, 10));
    }
}
