use kglance::core::layout_engine::{LayoutConfig, LogicalDocument, TextLayoutEngine, WrapMode};

#[test]
fn test_code_canvas_geometry_bounds_calculation() {
    let content = "fn main() {\n    println!(\"Hello World\");\n}";
    let doc = LogicalDocument::from_text(content);
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::NoWrap,
    };
    let visual_doc = TextLayoutEngine::compute_highlighted(&doc, &config, "rs", true);

    let line_height = 14.0 * 1.4;
    let expected_height = visual_doc.visual_lines.len() as f32 * line_height;

    assert_eq!(visual_doc.visual_lines.len(), 3);
    assert!(expected_height > 0.0);
}
