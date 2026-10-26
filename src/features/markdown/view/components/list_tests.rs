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
