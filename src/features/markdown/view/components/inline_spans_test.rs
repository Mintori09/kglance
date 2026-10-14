use super::*;

#[test]
fn test_inlines_to_spans_caching() {
    let inlines = vec![
        Inline::Text("Hello world".to_string()),
        Inline::Bold(vec![Inline::Text("bold text".to_string())]),
        Inline::Code("code_block()".to_string()),
    ];
    let counter = Cell::new(0);
    let ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Stacked,
        footnotes: None,
    };

    let spans1 = inlines_to_spans(&inlines, &ctx);
    assert_eq!(spans1.len(), 3);
    assert_eq!(spans1[0].text, "Hello world");
    assert_eq!(spans1[1].text, "bold text");
    assert_eq!(spans1[2].text, "code_block()");

    // Second call should hit the cache and return identical content
    let spans2 = inlines_to_spans(&inlines, &ctx);
    assert_eq!(spans2.len(), 3);
    assert_eq!(spans2[0].text, "Hello world");
    assert_eq!(spans2[1].text, "bold text");
    assert_eq!(spans2[2].text, "code_block()");
}

#[test]
fn test_inlines_to_spans_search_query_bypasses_cache() {
    let inlines = vec![Inline::Text("Search keyword test".to_string())];
    let counter = Cell::new(0);
    let ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "keyword",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Stacked,
        footnotes: None,
    };

    let spans = inlines_to_spans(&inlines, &ctx);
    assert!(spans.len() >= 2);
    assert_eq!(counter.get(), 1);
}

#[test]
fn test_inlines_to_spans_ruby_display_modes() {
    let inlines = vec![Inline::Ruby {
        text: "漢字".to_string(),
        ruby: "かんじ".to_string(),
    }];
    let counter = Cell::new(0);

    // Parentheses mode
    let ctx_paren = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Parentheses,
        footnotes: None,
    };
    let spans_paren = inlines_to_spans(&inlines, &ctx_paren);
    assert_eq!(spans_paren.len(), 1);
    assert_eq!(spans_paren[0].text, "漢字(かんじ)");

    // Hidden / Default fallback in spans
    let ctx_hidden = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Hidden,
        footnotes: None,
    };
    let spans_hidden = inlines_to_spans(&inlines, &ctx_hidden);
    assert_eq!(spans_hidden.len(), 1);
    assert_eq!(spans_hidden[0].text, "漢字");
}

fn stacked_ctx<'a>(counter: &'a Cell<usize>) -> SpanCtx<'a> {
    SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Stacked,
        footnotes: None,
    }
}

#[test]
fn test_extract_rubies() {
    let inlines = vec![
        Inline::Ruby {
            text: "私".to_string(),
            ruby: "わたし".to_string(),
        },
        Inline::Text("は".to_string()),
        Inline::Ruby {
            text: "彼".to_string(),
            ruby: "かれ".to_string(),
        },
    ];
    let counter = Cell::new(0);
    let ctx = stacked_ctx(&counter);
    let (spans, _, rubies, _) = inlines_to_spans_with_links(&inlines, &ctx);

    let plain: String = spans.iter().map(|s| s.text.as_ref()).collect();
    assert_eq!(plain, "私は彼");
    assert_eq!(
        rubies,
        vec![
            crate::ui::components::selectable_text::TextRuby {
                start: 0,
                end: "私".len(),
                ruby: "わたし".to_string(),
            },
            crate::ui::components::selectable_text::TextRuby {
                start: "私は".len(),
                end: "私は彼".len(),
                ruby: "かれ".to_string(),
            },
        ]
    );
}

#[test]
fn test_extract_rubies_contiguous_merged() {
    let inlines = vec![
        Inline::Ruby {
            text: "友".to_string(),
            ruby: "ゆう".to_string(),
        },
        Inline::Ruby {
            text: "人".to_string(),
            ruby: "じん".to_string(),
        },
    ];
    let counter = Cell::new(0);
    let ctx = stacked_ctx(&counter);
    let (_, _, rubies, _) = inlines_to_spans_with_links(&inlines, &ctx);
    assert_eq!(rubies.len(), 2);
    assert_eq!(rubies[0].ruby, "ゆう");
    assert_eq!(rubies[1].ruby, "じん");
    assert_eq!(rubies[0].end, rubies[1].start);
}

#[test]
fn test_inlines_to_spans_with_links() {
    let inlines = vec![
        Inline::Link {
            text: vec![Inline::Text("click".to_string())],
            url: "https://example.com".to_string(),
        },
        Inline::Ruby {
            text: "詳".to_string(),
            ruby: "くわ".to_string(),
        },
    ];
    let counter = Cell::new(0);
    let ctx = stacked_ctx(&counter);
    let (spans, links, rubies, tooltips) = inlines_to_spans_with_links(&inlines, &ctx);

    let plain: String = spans.iter().map(|s| s.text.as_ref()).collect();
    assert_eq!(plain, "click詳");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].url, "https://example.com");
    assert_eq!((links[0].start, links[0].end), (0, "click".len()));
    assert_eq!(rubies.len(), 1);
    assert_eq!(rubies[0].ruby, "くわ");
    assert_eq!(
        (rubies[0].start, rubies[0].end),
        ("click".len(), "click詳".len())
    );
    // Link has tooltip
    assert_eq!(tooltips.len(), 1);
    assert_eq!(tooltips[0].content, "https://example.com");
}

#[test]
fn test_extract_tooltips_all_inline_types() {
    let mut footnotes = std::collections::HashMap::new();
    footnotes.insert("1".to_string(), "Footnote definition text".to_string());

    let inlines = vec![
        Inline::Link {
            text: vec![Inline::Text("link".to_string())],
            url: "https://kglance.dev".to_string(),
        },
        Inline::Text(" ".to_string()),
        Inline::Code("cargo test".to_string()),
        Inline::Text(" ".to_string()),
        Inline::InlineMath("x^2".to_string()),
        Inline::Text(" ".to_string()),
        Inline::FootnoteReference("1".to_string()),
        Inline::Text(" ".to_string()),
        Inline::Ruby {
            text: "漢".to_string(),
            ruby: "かん".to_string(),
        },
    ];

    let counter = Cell::new(0);
    let ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Tooltip,
        footnotes: Some(&footnotes),
    };

    let (_, _, _, tooltips) = inlines_to_spans_with_links(&inlines, &ctx);

    // Expect 5 tooltips: link, code, math, footnote, ruby (in tooltip mode)
    assert_eq!(tooltips.len(), 5);
    assert_eq!(tooltips[0].content, "https://kglance.dev");
    assert_eq!(tooltips[1].content, "`cargo test`");
    assert_eq!(tooltips[2].content, "$x^2$");
    assert_eq!(tooltips[3].content, "[^1]: Footnote definition text");
    assert_eq!(tooltips[4].content, "かん");
}

#[test]
fn test_italic_cjk_preserves_normal_font_style() {
    let inlines = vec![Inline::Italic(vec![Inline::Text(
        "(丰 + 刀 + 糸 = chữ 絜 - tay cầm dao)".to_string(),
    )])];

    let counter = Cell::new(0);
    let ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &counter,
        theme: AppTheme::Dark,
        base_weight: None,
        base_color: None,
        font_size: 14.0,
        ruby_display_mode: crate::core::config::RubyDisplayMode::Stacked,
        footnotes: None,
    };

    let spans = inlines_to_spans(&inlines, &ctx);
    // Should split into non-CJK (Italic) and CJK (Normal)
    assert!(spans.len() > 1);

    // Verify all CJK spans have Style::Normal and non-CJK have Style::Italic
    for span in &spans {
        let style = span
            .font
            .map(|f| f.style)
            .unwrap_or(iced::font::Style::Normal);
        if span.text.chars().any(is_cjk) {
            assert_eq!(
                style,
                iced::font::Style::Normal,
                "CJK text '{}' must have Normal style",
                span.text
            );
        } else {
            assert_eq!(
                style,
                iced::font::Style::Italic,
                "Non-CJK text '{}' must have Italic style",
                span.text
            );
        }
    }

    // Verify the concatenated text matches the original
    let full_text: String = spans.iter().map(|s| s.text.as_ref()).collect();
    assert_eq!(full_text, "(丰 + 刀 + 糸 = chữ 絜 - tay cầm dao)");
}
