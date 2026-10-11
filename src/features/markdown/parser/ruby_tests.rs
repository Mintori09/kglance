use super::*;

#[test]
fn test_parse_single_ruby_tag() {
    let md = "<ruby>漢字<rt>かんじ</rt></ruby>";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(
                inlines,
                &[Inline::Ruby {
                    text: "漢字".to_string(),
                    ruby: "かんじ".to_string(),
                }]
            );
        }
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_parse_multiple_ruby_tags() {
    let md = "<ruby>不<rt>ふ</rt></ruby><ruby>自<rt>じ</rt></ruby><ruby>由<rt>ゆう</rt></ruby>な";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(
                inlines,
                &[
                    Inline::Ruby {
                        text: "不".to_string(),
                        ruby: "ふ".to_string(),
                    },
                    Inline::Ruby {
                        text: "自".to_string(),
                        ruby: "じ".to_string(),
                    },
                    Inline::Ruby {
                        text: "由".to_string(),
                        ruby: "ゆう".to_string(),
                    },
                    Inline::Text("な".to_string()),
                ]
            );
        }
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_parse_tabular_ruby_tag() {
    let md = "<ruby>不<rt>ふ</rt>自<rt>じ</rt>由<rt>ゆう</rt></ruby>な";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(
                inlines,
                &[
                    Inline::Ruby {
                        text: "不".to_string(),
                        ruby: "ふ".to_string(),
                    },
                    Inline::Ruby {
                        text: "自".to_string(),
                        ruby: "じ".to_string(),
                    },
                    Inline::Ruby {
                        text: "由".to_string(),
                        ruby: "ゆう".to_string(),
                    },
                    Inline::Text("な".to_string()),
                ]
            );
        }
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_parse_ruby_with_rb_and_rp() {
    let md = "<ruby><rb>漢字</rb><rp>(</rp><rt>かんじ</rt><rp>)</rp></ruby>";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(
                inlines,
                &[Inline::Ruby {
                    text: "漢字".to_string(),
                    ruby: "かんじ".to_string(),
                }]
            );
        }
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_ruby_flatten_modes() {
    let inlines = vec![
        Inline::Ruby {
            text: "漢字".to_string(),
            ruby: "かんじ".to_string(),
        },
        Inline::Text("テスト".to_string()),
    ];

    assert_eq!(
        flatten_inlines(&inlines),
        "<ruby>漢字<rt>かんじ</rt></ruby>テスト"
    );
    assert_eq!(flatten_inlines_visual(&inlines), "漢字テスト");
    assert_eq!(flatten_inlines_plain(&inlines), "漢字テスト");
    assert_eq!(flatten_inlines_toc(&inlines), "漢字テスト");
}

#[test]
fn test_parse_ruby_without_rt() {
    let md = "<ruby>漢字</ruby>";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(inlines, &[Inline::Text("漢字".to_string())]);
        }
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_parse_real_ruby_sample() {
    let md = "## 73. <ruby>変<rt>へん</rt></ruby>な (へんな) `[1]`\n\n> <ruby>変<rt>へん</rt></ruby>な メールが <ruby>届<rt>とど</rt></ruby>いたのですぐに <ruby>消<rt>け</rt></ruby>した。\n";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 2);
    match &blocks[0] {
        Block::Heading { level, content } => {
            assert_eq!(*level, 2);
            assert_eq!(
                content,
                &[
                    Inline::Text("73. ".to_string()),
                    Inline::Ruby {
                        text: "変".to_string(),
                        ruby: "へん".to_string(),
                    },
                    Inline::Text("な (へんな) ".to_string()),
                    Inline::Code("[1]".to_string()),
                ]
            );
        }
        _ => panic!("expected heading"),
    }
}

#[test]
fn test_parse_inter_ruby_text() {
    let raw = "<ruby>A<rt>a</rt></ruby> and <ruby>B<rt>b</rt></ruby>";
    let inlines = super::html::parse_ruby_string(raw);
    assert_eq!(
        inlines,
        &[
            Inline::Ruby {
                text: "A".to_string(),
                ruby: "a".to_string(),
            },
            Inline::Text(" and ".to_string()),
            Inline::Ruby {
                text: "B".to_string(),
                ruby: "b".to_string(),
            },
        ]
    );
}

#[test]
fn test_parse_ruby_with_formatting() {
    let md = "<ruby>**漢**<rt>かん</rt></ruby>";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(
                inlines,
                &[Inline::Ruby {
                    text: "漢".to_string(),
                    ruby: "かん".to_string(),
                }]
            );
        }
        _ => panic!("expected paragraph block"),
    }
}

#[test]
fn test_parse_ruby_with_whitespace_closing_tags() {
    let md = "<ruby>漢<rt>かん</rt ></ruby >";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(
                inlines,
                &[Inline::Ruby {
                    text: "漢".to_string(),
                    ruby: "かん".to_string(),
                }]
            );
        }
        _ => panic!("expected paragraph block"),
    }
}
