use kglance::core::layout_engine::{LayoutConfig, LogicalDocument, TextLayoutEngine, WrapMode};

// Scenario 1: Soft-Wrap Line Number Alignment Integrity
// Ensures line numbers appear ONLY on the first visual line of wrapped logical lines,
// while continuation lines have line_number = None.
#[test]
fn test_scenario_1_soft_wrap_line_number_alignment() {
    let text = "Line 1\nThis is a very long line designed to wrap into multiple visual lines when configured with a small character threshold limit.\nLine 3";
    let doc = LogicalDocument::from_text(text);
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::CharWrap(25),
    };

    let visual_doc = TextLayoutEngine::compute(&doc, &config);

    // Assert overall visual lines count > 3 due to wrapping
    assert!(visual_doc.visual_lines.len() > 3);

    // First visual line maps to Logical Line 1
    assert_eq!(visual_doc.visual_lines[0].logical_line_index, 1);
    assert_eq!(visual_doc.visual_lines[0].line_number, Some(1));

    // Middle wrapped lines map to Logical Line 2
    assert_eq!(visual_doc.visual_lines[1].logical_line_index, 2);
    assert_eq!(visual_doc.visual_lines[1].line_number, Some(2)); // First wrapped chunk has line number

    assert_eq!(visual_doc.visual_lines[2].logical_line_index, 2);
    assert_eq!(visual_doc.visual_lines[2].line_number, None); // Continuation chunk has NO line number

    assert_eq!(visual_doc.visual_lines[3].logical_line_index, 2);
    assert_eq!(visual_doc.visual_lines[3].line_number, None);

    // Last visual line maps to Logical Line 3
    let last = visual_doc.visual_lines.last().unwrap();
    assert_eq!(last.logical_line_index, 3);
    assert_eq!(last.line_number, Some(3));
}

// Scenario 2: Multi-language Syntect Syntax Highlighting Tokenization
// Ensures language syntax tokens (keywords, literals, types) produce separate HighlightedSpans with distinct colors.
#[test]
fn test_scenario_2_syntect_syntax_highlighting_tokens() {
    let code = "fn main() {\n    let x: i32 = 42;\n}";
    let doc = LogicalDocument::from_text(code);
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::NoWrap,
    };

    let visual_doc = TextLayoutEngine::compute_highlighted(&doc, &config, "rs", true);

    assert_eq!(visual_doc.visual_lines.len(), 3);
    let line2_spans = &visual_doc.visual_lines[1].spans;

    // Rust line `let x: i32 = 42;` should contain multiple tokens/spans
    assert!(line2_spans.len() > 1);

    // Verify non-empty text strings across spans
    let reconstructed: String = line2_spans.iter().map(|s| s.text.as_str()).collect();
    assert_eq!(reconstructed, "    let x: i32 = 42;");
}

// Scenario 3: Theme Adaptation (Dark vs Light mode)
// Ensures dark theme and light theme produce distinct default text & token background colors.
#[test]
fn test_scenario_3_theme_adaptation_dark_light() {
    let doc = LogicalDocument::from_text("const MAX: u32 = 100;");
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::NoWrap,
    };

    let dark_doc = TextLayoutEngine::compute_highlighted(&doc, &config, "rs", true);
    let light_doc = TextLayoutEngine::compute_highlighted(&doc, &config, "rs", false);

    assert!(!dark_doc.visual_lines[0].spans.is_empty());
    assert!(!light_doc.visual_lines[0].spans.is_empty());

    // Color outputs between dark mode and light mode must differ
    let dark_color = dark_doc.visual_lines[0].spans[0].color;
    let light_color = light_doc.visual_lines[0].spans[0].color;
    assert_ne!(dark_color, light_color);
}

// Scenario 4: JSON Raw Formatting and Structured Tokenization
// Tests parsing JSON strings to verify key, string, number and bracket tokens render correctly.
#[test]
fn test_scenario_4_json_raw_formatting_and_spans() {
    let json_text = "{\n  \"key\": \"value\",\n  \"number\": 12345\n}";
    let doc = LogicalDocument::from_text(json_text);
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 2,
        wrap_mode: WrapMode::NoWrap,
    };

    let visual_doc = TextLayoutEngine::compute_highlighted(&doc, &config, "json", true);

    assert_eq!(visual_doc.visual_lines.len(), 4);
    assert_eq!(visual_doc.visual_lines[0].line_number, Some(1));
    assert_eq!(visual_doc.visual_lines[3].line_number, Some(4));

    // Full document reconstruction check
    let full_text: String = visual_doc
        .visual_lines
        .iter()
        .map(|v| v.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(full_text, json_text);
}

// Scenario 5: Edge Cases (Empty files, single line files, unicode strings)
// Tests stability against zero-length strings, single lines, and CJK/Unicode multi-byte characters.
#[test]
fn test_scenario_5_edge_cases_empty_and_unicode() {
    // 5a. Empty Document
    let empty_doc = LogicalDocument::from_text("");
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::NoWrap,
    };
    let empty_visual = TextLayoutEngine::compute(&empty_doc, &config);
    assert_eq!(empty_visual.visual_lines.len(), 0);

    // 5b. Multi-byte CJK / Vietnamese Unicode Text
    let unicode_text = "Dòng 1: Tiếng Việt có dấu và chữ Nhật 映画の後\nDòng 2: Short";
    let unicode_doc = LogicalDocument::from_text(unicode_text);
    let unicode_config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::CharWrap(15),
    };
    let unicode_visual = TextLayoutEngine::compute(&unicode_doc, &unicode_config);

    assert!(unicode_visual.visual_lines.len() >= 2);
    assert_eq!(unicode_visual.visual_lines[0].line_number, Some(1));
    assert!(unicode_visual.visual_lines[0].text.contains("Dòng 1"));
}
