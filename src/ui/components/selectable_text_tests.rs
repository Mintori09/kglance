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

#[test]
fn test_word_fullwidth_asian_punctuation() {
    let text = "你好，世界！";
    let range = expand_to_word_bounds(text, 0);
    assert_eq!(substr(text, range), "你好");

    let world_offset = text.find("世界").expect("must find 世界");
    let range_world = expand_to_word_bounds(text, world_offset);
    assert_eq!(substr(text, range_world), "世界");
}

#[test]
fn test_ruby_glyph_hit_test_in_cosmic_buffer() {
    let mut font_system = cosmic_text::FontSystem::new();
    let metrics = cosmic_text::Metrics::new(14.0, 14.0 * 1.85);
    let mut buffer = cosmic_text::Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(500.0), Some(100.0));
    let text = "ゴミ箱から臭い匂いがする。";
    buffer.set_text(
        &mut font_system,
        text,
        &cosmic_text::Attrs::new(),
        cosmic_text::Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);

    let bako_start = text.find("箱").unwrap();
    let bako_end = bako_start + "箱".len();

    let mut found_glyphs = 0;
    let mut min_x: Option<f32> = None;
    let mut max_x: Option<f32> = None;

    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            if glyph.end > bako_start && glyph.start < bako_end {
                found_glyphs += 1;
                min_x = Some(min_x.map_or(glyph.x, |m| m.min(glyph.x)));
                max_x = Some(max_x.map_or(glyph.x + glyph.w, |m| m.max(glyph.x + glyph.w)));
            }
        }
    }

    assert!(found_glyphs > 0, "must find glyph for 箱");
    assert!(min_x.is_some());
    assert!(max_x.is_some());
    assert!(max_x.unwrap() > min_x.unwrap());
}

#[test]
fn test_ruby_center_alignment() {
    let md = "## 9. <ruby>詳<rt>くわ</rt></ruby>しい (くわしい) `[3]`";
    let blocks = crate::parsers::markdown::parse_to_blocks(md);
    let content = match &blocks[0] {
        crate::parsers::markdown::Block::Heading { content, .. } => content,
        _ => panic!("expected heading"),
    };
    let counter = std::cell::Cell::new(0);
    let span_ctx = crate::features::markdown::view::components::inline_spans::SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: crate::ui::theme::AppTheme::Dark,
        base_weight: Some(iced::font::Weight::Bold),
        base_style: None,
        base_color: None,
        font_size: 24.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Stacked,
        footnotes: None,
    };
    let (spans, _links, rubies, _tooltips) =
        crate::features::markdown::view::components::inline_spans::inlines_to_spans_with_links(
            content, &span_ctx,
        );
    assert_eq!(rubies.len(), 1);
    assert_eq!(rubies[0].ruby, "くわ");
    assert_eq!(rubies[0].start, 3);
    assert_eq!(rubies[0].end, 6);

    use iced::advanced::text::Paragraph;
    use iced::{Font, Pixels, Size, alignment};

    let text_input = iced::advanced::text::Text {
        content: &spans[..],
        bounds: Size::new(500.0, f32::INFINITY),
        size: Pixels(24.0),
        line_height: iced::advanced::text::LineHeight::Relative(1.5),
        font: Font::default(),
        align_x: alignment::Horizontal::Left.into(),
        align_y: alignment::Vertical::Top,
        shaping: iced::advanced::text::Shaping::Advanced,
        wrapping: iced::advanced::text::Wrapping::Word,
    };
    let p = iced::advanced::graphics::text::Paragraph::with_spans(text_input);
    let buffer = p.buffer();

    let mut k_min_x: Option<f32> = None;
    let mut k_max_x: Option<f32> = None;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            if glyph.end > rubies[0].start && glyph.start < rubies[0].end {
                k_min_x = Some(k_min_x.map_or(glyph.x, |m| m.min(glyph.x)));
                k_max_x = Some(k_max_x.map_or(glyph.x + glyph.w, |m| m.max(glyph.x + glyph.w)));
            }
        }
    }
    assert!(k_min_x.is_some() && k_max_x.is_some());
    let center_x = (k_min_x.unwrap() + k_max_x.unwrap()) * 0.5;
    // Base kanji "詳" starts after "9. " (which has width > 0)
    assert!(center_x > 25.0, "center_x must be placed over 詳, not 9.");
}

#[test]
fn test_wrapped_line_no_phantom_selection_on_unselected_run() {
    let mut font_system = cosmic_text::FontSystem::new();
    let metrics = cosmic_text::Metrics::new(14.0, 14.0 * 1.5);
    let mut buffer = cosmic_text::Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, Some(500.0), Some(100.0));
    let text =
        "細かい お金に 両替してもらえますか。 ➔ Bạn có thể đổi giúp tôi sang tiền lẻ được không?";
    buffer.set_text(
        &mut font_system,
        text,
        &cosmic_text::Attrs::new(),
        cosmic_text::Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);

    let runs: Vec<_> = buffer.layout_runs().collect();
    assert!(runs.len() >= 2, "must wrap into at least 2 visual lines");

    let sel_text = "細かい お金に 両替してもらえ";
    let sel_start = 0;
    let sel_end = sel_text.len();

    let last_run = runs.last().unwrap();
    let mut last_run_min_x: Option<f32> = None;
    for glyph in last_run.glyphs {
        if glyph.end > sel_start && glyph.start < sel_end {
            last_run_min_x = Some(glyph.x);
        }
    }
    assert_eq!(
        last_run_min_x, None,
        "unselected wrapped run must not match selection range on visual line 0"
    );
}

#[test]
fn test_ruby_top_padding_fits_ruby_box() {
    let font_size = 14.0;
    let widget =
        SelectableText::<crate::app::Message>::new(Vec::new(), font_size).rubies(vec![TextRuby {
            start: 0,
            end: 3,
            ruby: "わたし".to_string(),
        }]);
    assert_eq!(
        widget.top_padding(),
        ruby_box_height(font_size) + 2.0,
        "top pad must fit the ruby box plus a gap so first-line ruby never overflows into the block above"
    );
}

#[test]
fn test_ruby_metrics_scale_with_font_size() {
    assert_eq!(ruby_font_size(14.0), 8.0);
    assert_eq!(ruby_box_height(14.0), 8.0 * 1.2);
    assert!(ruby_box_height(20.0) > ruby_box_height(14.0));
}

#[test]
fn test_selectable_text_tooltips_builder() {
    let tooltips = vec![
        TextTooltip {
            start: 0,
            end: 5,
            content: "https://example.com".to_string(),
        },
        TextTooltip {
            start: 6,
            end: 10,
            content: "`code`".to_string(),
        },
    ];
    let widget = SelectableText::<crate::app::Message>::new(Vec::new(), 14.0)
        .tooltips(tooltips.clone())
        .app_theme(AppTheme::Dark);

    assert_eq!(widget.tooltips.len(), 2);
    assert_eq!(widget.tooltips[0].content, "https://example.com");
    assert_eq!(widget.tooltips[1].content, "`code`");
    assert_eq!(widget.app_theme, Some(AppTheme::Dark));
}

#[test]
fn test_selectable_text_drag_active_builder() {
    let range = crate::core::SelectionRange {
        start: crate::core::SelectionPoint {
            block: 0,
            offset: 0,
        },
        end: crate::core::SelectionPoint {
            block: 0,
            offset: 5,
        },
    };
    let widget = SelectableText::<crate::app::Message>::new(Vec::new(), 14.0)
        .block_index(0)
        .drag_active(true)
        .selection_range(Some(range));

    assert!(widget.drag_active);
    assert_eq!(widget.block_index, Some(0));
    assert_eq!(widget.selection_range, Some(range));
}
