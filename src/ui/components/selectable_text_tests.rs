use super::*;

fn substr(text: &str, range: (usize, usize)) -> &str {
    &text[range.0..range.1]
}

#[test]
fn test_word_mid_cursor() {
    let text = "Hello World Rust";
    let range = expand_to_word_bounds(text, 7);
    assert_eq!(
        substr(text, range),
        "World",
        "cursor giữa từ phải chọn đúng từ"
    );
}

#[test]
fn test_word_at_start_of_word() {
    let text = "Hello World Rust";
    let range = expand_to_word_bounds(text, 6);
    assert_eq!(substr(text, range), "World");
}

#[test]
fn test_word_at_end_of_word() {
    let text = "Hello World Rust";
    let range = expand_to_word_bounds(text, 10);
    assert_eq!(substr(text, range), "World");
}

#[test]
fn test_word_on_whitespace_returns_empty_or_space() {
    let text = "Hello World";
    let range = expand_to_word_bounds(text, 5);

    let selected = substr(text, range);
    assert!(
        selected.trim().is_empty() || selected == " ",
        "cursor trên whitespace không được chọn vào từ bên cạnh, got: {selected:?}"
    );
}

#[test]
fn test_word_punctuation_not_included() {
    let text = "Hello, world.";
    let range_comma = expand_to_word_bounds(text, 5);
    let selected = substr(text, range_comma);

    assert!(
        !selected.contains(','),
        "dấu phẩy không được nằm trong từ được chọn, got: {selected:?}"
    );

    let range_word = expand_to_word_bounds(text, 1);
    assert_eq!(substr(text, range_word), "Hello");
}

#[test]
fn test_word_empty_string() {
    assert_eq!(expand_to_word_bounds("", 0), (0, 0));
}

#[test]
fn test_word_out_of_range_does_not_panic() {
    let text = "Hi";
    let range = expand_to_word_bounds(text, 9999);

    assert!(range.0 <= text.len(), "start vượt biên");
    assert!(range.1 <= text.len(), "end vượt biên");
    assert!(range.0 <= range.1, "start phải <= end");
}

#[test]
fn test_word_utf8_char_boundary() {
    let text = "Xin chào Kglance";

    let range = expand_to_word_bounds(text, 4);

    assert!(
        std::panic::catch_unwind(|| {
            let _ = &text[range.0..range.1];
        })
        .is_ok(),
        "slice [start..end] phải hợp lệ trên char boundary"
    );
}

#[test]
fn test_extract_empty_spans() {
    let spans: Vec<Span<(), Font>> = vec![];
    let widget = SelectableText::<(), iced::Theme, iced::Renderer>::new(spans, 14.0);
    assert_eq!(widget.extract_plain_text(), "");
}

#[test]
fn test_extract_multiple_spans_concat() {
    let spans = vec![Span::new("Rust "), Span::new("is "), Span::new("great")];
    let widget = SelectableText::<(), iced::Theme, iced::Renderer>::new(spans, 14.0);
    let result = widget.extract_plain_text();
    assert_eq!(result, "Rust is great");
    assert_eq!(result.len(), 13, "độ dài phải khớp chính xác");
}

#[test]
fn test_cross_block_effective_selection_calculation() {
    use crate::core::{SelectionPoint, SelectionRange};

    let range = SelectionRange {
        start: SelectionPoint {
            block: 0,
            offset: 5,
        },
        end: SelectionPoint {
            block: 2,
            offset: 4,
        },
    };

    let block0_len = 9;
    let block1_len = 15;
    let block2_len = 14;

    let sel_b0 = (range.start.offset.min(block0_len), block0_len);
    assert_eq!(sel_b0, (5, 9));

    let sel_b1 = (0, block1_len);
    assert_eq!(sel_b1, (0, 15));

    let sel_b2 = (0, range.end.offset.min(block2_len));
    assert_eq!(sel_b2, (0, 4));

    let is_b3_in_range = 3 >= range.start.block && 3 <= range.end.block;
    assert!(!is_b3_in_range);
}

#[cfg(feature = "profile-telemetry")]
#[test]
fn test_telemetry_counters() {
    use std::time::Duration;

    telemetry::reset();
    assert_eq!(telemetry::stats(), (0, 0, 0));

    let mat_id = telemetry::record_materialize();
    assert_eq!(mat_id, 1);

    telemetry::record_layout_miss(Duration::from_micros(150), 3, 42, mat_id);
    telemetry::record_layout_hit(Duration::from_micros(10), 3, 42);

    assert_eq!(telemetry::stats(), (1, 1, 1));

    telemetry::reset();
    assert_eq!(telemetry::stats(), (0, 0, 0));
}
