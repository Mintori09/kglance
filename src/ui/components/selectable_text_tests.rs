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
        "cursor within word must select the correct word"
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
        "cursor on whitespace must not select adjacent word, got: {selected:?}"
    );
}

#[test]
fn test_word_punctuation_not_included() {
    let text = "Hello, world.";
    let range_comma = expand_to_word_bounds(text, 5);
    let selected = substr(text, range_comma);

    assert!(
        !selected.contains(','),
        "comma must not be included in selected word, got: {selected:?}"
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

    assert!(range.0 <= text.len(), "start out of bounds");
    assert!(range.1 <= text.len(), "end out of bounds");
    assert!(range.0 <= range.1, "start must be <= end");
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
        "slice [start..end] must be valid on char boundary"
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
    assert_eq!(result.len(), 13, "length must match exactly");
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

#[test]
fn test_multiline_selection_per_line_slices() {
    let line0 = "use iced::font::Weight;\n";
    let line1 = "use serde::{Deserialize, Serialize};\n";
    let line2 = "pub enum FontWeightConfig {\n";
    let _line3 = "    Light,\n";

    let line0_len = line0.trim_end_matches(['\r', '\n']).len();
    let line1_len = line1.trim_end_matches(['\r', '\n']).len();
    let line2_len = line2.trim_end_matches(['\r', '\n']).len();

    let line_starts = [
        0,
        line0.len(),
        line0.len() + line1.len(),
        line0.len() + line1.len() + line2.len(),
    ];

    // Select from column 4 in line 0 ("iced...") down to column 8 in line 2 ("pub enum")
    let start: usize = 4;
    let end: usize = line_starts[2] + 8;

    // Line 0 slice:
    let sel_l0_start = start.saturating_sub(line_starts[0]).min(line0_len);
    let sel_l0_end = end.saturating_sub(line_starts[0]).min(line0_len);
    assert_eq!((sel_l0_start, sel_l0_end), (4, line0_len));

    // Line 1 slice (intermediate line): should be fully selected
    let sel_l1_start = start.saturating_sub(line_starts[1]).min(line1_len);
    let sel_l1_end = end.saturating_sub(line_starts[1]).min(line1_len);
    assert_eq!((sel_l1_start, sel_l1_end), (0, line1_len));

    // Line 2 slice (end line): should be 0..8
    let sel_l2_start = start.saturating_sub(line_starts[2]).min(line2_len);
    let sel_l2_end = end.saturating_sub(line_starts[2]).min(line2_len);
    assert_eq!((sel_l2_start, sel_l2_end), (0, 8));

    // Line 3 slice (after selection): line_start >= end, so not selected
    assert!(line_starts[3] >= end);
}

#[test]
fn test_word_vietnamese_and_unicode_quotes() {
    let text = "“Xin chào” — Kglance!";
    let chao_offset = text.find("chào").expect("must find chào");
    let range_chao = expand_to_word_bounds(text, chao_offset + 1);
    assert_eq!(substr(text, range_chao), "chào");

    let xin_offset = text.find("Xin").expect("must find Xin");
    let range_xin = expand_to_word_bounds(text, xin_offset);
    assert_eq!(substr(text, range_xin), "Xin");

    let kglance_offset = text.find("Kglance").expect("must find Kglance");
    let range_kglance = expand_to_word_bounds(text, kglance_offset + 2);
    assert_eq!(substr(text, range_kglance), "Kglance");
}
