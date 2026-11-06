use super::*;
use crate::core::config::RubyDisplayMode;
use crate::parsers::markdown::{Block, Inline};

#[test]
fn test_has_stacked_ruby_stacked_mode() {
    let inlines = vec![
        Inline::Text("Normal text ".to_string()),
        Inline::Ruby {
            text: "共感覚".to_string(),
            ruby: "きょうかんかく".to_string(),
        },
    ];
    assert!(has_stacked_ruby(&inlines, RubyDisplayMode::Stacked));
    assert!(!has_stacked_ruby(&inlines, RubyDisplayMode::Tooltip));
    assert!(!has_stacked_ruby(&inlines, RubyDisplayMode::Parentheses));
    assert!(!has_stacked_ruby(&inlines, RubyDisplayMode::Hidden));
}

#[test]
fn test_has_stacked_ruby_nested() {
    let inlines = vec![Inline::Bold(vec![Inline::Ruby {
        text: "漢字".to_string(),
        ruby: "かんじ".to_string(),
    }])];
    assert!(has_stacked_ruby(&inlines, RubyDisplayMode::Stacked));
}

#[test]
fn test_has_stacked_ruby_negative() {
    let inlines = vec![
        Inline::Text("Just normal text".to_string()),
        Inline::Bold(vec![Inline::Text("bold".to_string())]),
    ];
    assert!(!has_stacked_ruby(&inlines, RubyDisplayMode::Stacked));
}

#[test]
fn test_block_starts_with_stacked_ruby() {
    let block_with_ruby = Block::Paragraph(vec![Inline::Ruby {
        text: "漢字".to_string(),
        ruby: "かんじ".to_string(),
    }]);
    assert!(block_starts_with_stacked_ruby(
        &block_with_ruby,
        RubyDisplayMode::Stacked
    ));
    assert!(!block_starts_with_stacked_ruby(
        &block_with_ruby,
        RubyDisplayMode::Hidden
    ));

    let block_without_ruby = Block::Paragraph(vec![Inline::Text("No ruby here".to_string())]);
    assert!(!block_starts_with_stacked_ruby(
        &block_without_ruby,
        RubyDisplayMode::Stacked
    ));

    let heading_with_ruby = Block::Heading {
        level: 3,
        content: vec![Inline::Ruby {
            text: "言葉".to_string(),
            ruby: "ことば".to_string(),
        }],
    };
    assert!(block_starts_with_stacked_ruby(
        &heading_with_ruby,
        RubyDisplayMode::Stacked
    ));
}

fn mock_render_context<'a>(
    next_idx: &'a std::cell::Cell<usize>,
    counter: &'a std::cell::Cell<usize>,
    is_epub: bool,
    epub_override_book_font: bool,
    font_size: f32,
) -> RenderContext<'a> {
    RenderContext {
        next_block_index: next_idx,
        selection_range: None,
        drag_active: false,
        search_query: "",
        active_match: 0,
        counter,
        theme: crate::ui::theme::AppTheme::default(),
        font_size,
        font_family: None,
        font_family_mono: None,
        ruby_display_mode: RubyDisplayMode::Hidden,
        footnotes: None,
        result_popup: false,
        show_heading_divider: true,
        is_epub,
        epub_override_book_font,
        content_width: 800.0,
    }
}

#[test]
fn test_resolve_body_font_size_standard_markdown() {
    let next_idx = std::cell::Cell::new(0);
    let counter = std::cell::Cell::new(0);
    let ctx = mock_render_context(&next_idx, &counter, false, false, 16.0);

    assert_eq!(resolve_body_font_size(None, &ctx), 16.0);
    assert_eq!(resolve_body_font_size(Some(0.85), &ctx), 14.0);
    assert_eq!(resolve_body_font_size(Some(1.2), &ctx), 19.0);
}

#[test]
fn test_resolve_body_font_size_epub_normalization() {
    let next_idx = std::cell::Cell::new(0);
    let counter = std::cell::Cell::new(0);
    let ctx = mock_render_context(&next_idx, &counter, true, false, 16.0);

    // Minor print-oriented font-size scales (0.80..=1.15) MUST normalize to user base font size
    assert_eq!(resolve_body_font_size(None, &ctx), 16.0);
    assert_eq!(resolve_body_font_size(Some(0.875), &ctx), 16.0); // 10.5pt
    assert_eq!(resolve_body_font_size(Some(0.833), &ctx), 16.0); // 10.0pt
    assert_eq!(resolve_body_font_size(Some(0.916), &ctx), 16.0); // 11.0pt
    assert_eq!(resolve_body_font_size(Some(1.10), &ctx), 16.0); // 1.1em

    // Structural font scale changes outside body band MUST be preserved
    assert_eq!(resolve_body_font_size(Some(0.65), &ctx), 10.0); // Footnotes/subscripts
    assert_eq!(resolve_body_font_size(Some(1.30), &ctx), 21.0); // Large display callouts
}

#[test]
fn test_resolve_body_font_size_epub_override_book_font() {
    let next_idx = std::cell::Cell::new(0);
    let counter = std::cell::Cell::new(0);
    let ctx = mock_render_context(&next_idx, &counter, true, true, 16.0);

    // When reader overrides book font, all body font scales normalize to base font size
    assert_eq!(resolve_body_font_size(Some(0.875), &ctx), 16.0);
    assert_eq!(resolve_body_font_size(Some(0.65), &ctx), 16.0);
    assert_eq!(resolve_body_font_size(Some(1.30), &ctx), 16.0);
}

#[test]
fn test_render_list_with_font_scale_synchronization() {
    let next_idx = std::cell::Cell::new(0);
    let counter = std::cell::Cell::new(0);
    let ctx = mock_render_context(&next_idx, &counter, true, false, 16.0);
    let state = crate::core::MarkdownState::default();

    let items = vec![ListItem {
        content: vec![Inline::Text(
            "Option 3: If a class has one constructor".to_string(),
        )],
        sub_blocks: Vec::new(),
        is_task: None,
    }];

    let style = crate::features::epub::parser::css::BlockStyle {
        font_scale: Some(0.875),
        ..Default::default()
    };

    // Rendering list with 0.875 font_scale in EPUB mode must succeed without panic
    let _element = render_list(false, 1, &items, Some(&style), &state, &ctx);
    assert!(next_idx.get() > 0, "Block index should be allocated");
}
