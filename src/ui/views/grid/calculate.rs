use crate::ui::views::grid::constants::ELLIPSIS;

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
