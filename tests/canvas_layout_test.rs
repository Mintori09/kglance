use kglance::core::layout_engine::{LayoutConfig, LogicalDocument, TextLayoutEngine, WrapMode};

#[test]
fn test_highlighted_spans_in_layout_engine() {
    let doc = LogicalDocument::from_text("fn main() {\n    let x = 42;\n}");
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::NoWrap,
    };
    let layout = TextLayoutEngine::compute_highlighted(&doc, &config, "rs", true);
    assert_eq!(layout.visual_lines.len(), 3);
    assert!(!layout.visual_lines[0].spans.is_empty());
}
