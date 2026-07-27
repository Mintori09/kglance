use kglance::core::layout_engine::{LayoutConfig, LogicalDocument, TextLayoutEngine, WrapMode};

#[test]
fn test_japanese_and_ipa_character_parsing_and_wrapping() {
    let text = "Line 1: /pɑːrtɪʃənɪŋ/\nLine 2: 映画の後、 me::meaning_jp 映画の後、 we decided to head back home.\nLine 3: /kɔːrt/";
    let doc = LogicalDocument::from_text(text);
    let config = LayoutConfig {
        viewport_width: 800.0,
        font_size: 14.0,
        tab_width: 4,
        wrap_mode: WrapMode::CharWrap(30),
    };

    let visual_doc = TextLayoutEngine::compute_highlighted(&doc, &config, "json", true);

    // Verify wrapping for long Japanese line (Line 2)
    assert!(visual_doc.visual_lines.len() > 3);

    // Check IPA characters preservation in visual lines
    let ipa_line = &visual_doc.visual_lines[0];
    assert!(ipa_line.text.contains("/pɑːrtɪʃənɪŋ/"));

    // Check Japanese characters preservation in visual lines
    let jp_line = &visual_doc.visual_lines[1];
    assert!(jp_line.text.contains("映画の後"));
}
