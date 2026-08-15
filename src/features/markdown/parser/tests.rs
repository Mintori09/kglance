use super::*;
use std::io::Write;

fn text(s: &str) -> Inline {
    Inline::Text(s.to_string())
}

fn code(s: &str) -> Inline {
    Inline::Code(s.to_string())
}

#[test]
fn parses_basic_markdown() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    write!(tmp, "# Hello\n\nThis is **bold** and `code`").unwrap();
    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();
    match result {
        ParsedContent::Markdown {
            content, images, ..
        } => {
            assert!(content.contains("Hello"));
            assert!(images.is_empty());
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn test_inline_math_in_spans() {
    let inlines = vec![
        text("Result: "),
        Inline::InlineMath("\\Rightarrow".to_string()),
        text(" done"),
    ];
    let flat = flatten_inlines_visual(&inlines);
    assert_eq!(flat, "Result: ⇒ done");
}

#[test]
fn parses_inline_bold_and_code_in_paragraph() {
    let blocks = parse_to_blocks("This is **bold** and `code`");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(inlines.len(), 4);
            assert!(matches!(inlines[0], Inline::Text(_)));
            assert!(matches!(inlines[1], Inline::Bold(_)));
            assert!(matches!(inlines[2], Inline::Text(_)));
            assert!(matches!(inlines[3], Inline::Code(_)));
        }
        _ => panic!("expected Paragraph"),
    }
}

#[test]
fn parses_inline_italic_and_link() {
    let blocks = parse_to_blocks("*italic* and [link](https://example.com)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(matches!(inlines[0], Inline::Italic(_)));
            match &inlines[2] {
                Inline::Link { text, url } => {
                    assert_eq!(url, "https://example.com");
                    assert_eq!(flatten_inlines(text), "link");
                }
                _ => panic!("expected Link"),
            }
        }
        _ => panic!("expected Paragraph"),
    }
}

#[test]
fn parses_strikethrough() {
    let blocks = parse_to_blocks("~~struck~~");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(matches!(inlines[0], Inline::Strikethrough(_)));
            assert_eq!(flatten_inlines_plain(&inlines[0..1]), "struck");
            assert_eq!(flatten_inlines(&inlines[0..1]), "~~struck~~");
        }
        _ => panic!("expected Paragraph"),
    }
}

#[test]
fn flatten_inlines_visual_matches_rendered_text() {
    let blocks = parse_to_blocks("**đậm** chữ `code` ![ảnh](u.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(flatten_inlines_visual(inlines), "đậm chữ code [ảnh]");
        }
        _ => panic!("expected Paragraph"),
    }
}

#[test]
fn parses_image_block_from_markdown() {
    let blocks = parse_to_blocks("![alt text](path/to/image.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Image { alt, path } => {
            assert_eq!(alt, "alt text");
            assert_eq!(path, "path/to/image.png");
        }
        _ => panic!("expected Block::Image, got {:?}", blocks[0]),
    }
}

#[test]
fn parses_image_with_remote_url() {
    let blocks = parse_to_blocks("![remote](https://example.com/image.png)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Image { alt, path } => {
            assert_eq!(alt, "remote");
            assert_eq!(path, "https://example.com/image.png");
        }
        _ => panic!("expected Block::Image, got {:?}", blocks[0]),
    }
}

#[test]
fn parses_code_block() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    write!(tmp, "```rust\nfn main() {{}}\n```").unwrap();
    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();
    match result {
        ParsedContent::Markdown { content, .. } => {
            assert!(content.contains("rust"));
            assert!(content.contains("fn main"));
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn parses_indented_code_block() {
    let md = "- **Bản tin:**\n\n  ```json\n  {\n    \"device_id\": \"string\"\n  }\n  ```";
    let blocks = parse_to_blocks(md);
    assert!(!blocks.is_empty());

    let direct_code_md = "```json\n{\n  \"device_id\": \"string\"\n}\n```";
    let direct_blocks = parse_to_blocks(direct_code_md);
    assert_eq!(direct_blocks.len(), 1);
    match &direct_blocks[0] {
        Block::CodeBlock { lang, code, .. } => {
            assert_eq!(lang.as_deref(), Some("json"));
            assert!(code.contains("\"device_id\": \"string\""));
        }
        _ => panic!("expected CodeBlock"),
    }
}

#[test]
fn extracts_images() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    write!(tmp, "![alt](image.png)").unwrap();
    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();
    match result {
        ParsedContent::Markdown { images, .. } => {
            assert_eq!(images.len(), 1);
            assert_eq!(images[0].alt_text, "alt");
            assert!(images[0].path.ends_with("image.png"));
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn supports_extension() {
    let parser = MarkdownParser::new();
    assert!(parser.is_supported(Path::new("test.md")));
    assert!(parser.is_supported(Path::new("test.markdown")));
    assert!(!parser.is_supported(Path::new("test.txt")));
}

#[test]
fn returns_error_for_nonexistent_file() {
    let parser = MarkdownParser::new();
    let result = parser.parse(Path::new("/nonexistent/file.md"));
    assert!(result.is_err());
}

#[test]
fn parses_large_markdown_file_over_3000_lines() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();

    let mut large_content = String::new();
    for i in 1..=3500 {
        if i % 100 == 0 {
            large_content.push_str(&format!("## Heading Level 2 at line {i}\n\n"));
        } else {
            large_content.push_str(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor.\n",
            );
        }
    }

    write!(tmp, "{large_content}").unwrap();

    let parser = MarkdownParser::new();
    let start_time = std::time::Instant::now();
    let result = parser.parse(tmp.path()).unwrap();
    let duration = start_time.elapsed();

    match result {
        ParsedContent::Markdown {
            content, images, ..
        } => {
            assert!(content.contains("Heading Level 2 at line 3500"));
            assert_eq!(content.lines().count(), large_content.lines().count());
            assert!(images.is_empty());
            assert!(
                duration.as_millis() < 200,
                "Parsing took too long: {:?}",
                duration
            );
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn extracts_multiple_images_from_large_file() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    let parent_dir = tmp.path().parent().unwrap().to_path_buf();

    let mut large_content = String::new();
    large_content.push_str("![First Image](assets/img_first.png)\n");

    for i in 2..3100 {
        if i == 1500 {
            large_content.push_str("\n![Middle Image](assets/img_middle.jpg)\n\n");
        } else {
            large_content.push_str(
                "Testing parser stability with a massive volume of plain text structures.\n",
            );
        }
    }
    large_content.push_str("\n![Last Image](/absolute/path/img_last.svg)\n");

    write!(tmp, "{large_content}").unwrap();

    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();

    match result {
        ParsedContent::Markdown { images, .. } => {
            assert_eq!(images.len(), 3);
            assert_eq!(images[0].alt_text, "First Image");
            let expected_first_path = parent_dir
                .join("assets/img_first.png")
                .to_string_lossy()
                .to_string();
            assert_eq!(images[0].path, expected_first_path);

            assert_eq!(images[1].alt_text, "Middle Image");
            let expected_mid_path = parent_dir
                .join("assets/img_middle.jpg")
                .to_string_lossy()
                .to_string();
            assert_eq!(images[1].path, expected_mid_path);

            assert_eq!(images[2].alt_text, "Last Image");
            assert_eq!(images[2].path, "/absolute/path/img_last.svg");
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn handles_images_inside_code_blocks_in_large_file() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    let mut large_content = String::new();

    large_content.push_str("![Valid Image](valid.png)\n");
    for i in 2..3200 {
        if i == 1600 {
            large_content.push_str(
                "\n```markdown\nThis is a code block sample: ![Fake Image](fake.png)\n```\n\n",
            );
        } else {
            large_content.push_str("Standard text row filler.\n");
        }
    }

    write!(tmp, "{large_content}").unwrap();

    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();

    match result {
        ParsedContent::Markdown { images, .. } => {
            assert_eq!(images.len(), 1);
            assert_eq!(images[0].alt_text, "Valid Image");
            assert!(images[0].path.ends_with("valid.png"));
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn test_parse_heading_paragraph_to_blocks() {
    let md = "# Hello\n\nSome text";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 2);
    assert!(matches!(blocks[0], Block::Heading { level: 1, .. }));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
}

#[test]
fn test_parse_code_block_to_blocks() {
    let md = "```rust\nfn main() {}\n```";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::CodeBlock { lang, code, .. } => {
            assert_eq!(lang.as_deref(), Some("rust"));
            assert!(code.contains("fn main"));
        }
        _ => panic!("expected CodeBlock block"),
    }
}

#[test]
fn test_parse_table() {
    let md = "| H1 | H2 |\n|---|---|\n| C1 | C2 |";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Table(tbl) => {
            assert_eq!(tbl.headers.len(), 2);
            assert_eq!(flatten_inlines(&tbl.headers[0].content), "H1");
            assert_eq!(flatten_inlines(&tbl.headers[1].content), "H2");
            assert_eq!(tbl.rows.len(), 1);
            assert_eq!(flatten_inlines(&tbl.rows[0][0].content), "C1");
            assert_eq!(flatten_inlines(&tbl.rows[0][1].content), "C2");
        }
        _ => panic!("expected Table block"),
    }
}

#[test]
fn test_parse_table_headers_only() {
    let md = "| A | B |\n|---|---|";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Table(tbl) => {
            assert_eq!(tbl.headers.len(), 2);
            assert!(tbl.rows.is_empty());
        }
        _ => panic!("expected Table block"),
    }
}

#[test]
fn test_parse_unordered_list() {
    let md = "- Item A\n- Item B\n    - Nested";
    let blocks = parse_to_blocks(md);
    assert!(!blocks.is_empty());
    match &blocks[0] {
        Block::List { ordered, items, .. } => {
            assert!(!ordered);
            assert_eq!(items.len(), 2);
            assert!(
                items[0]
                    .content
                    .iter()
                    .any(|i| matches!(i, Inline::Text(t) if t.contains("Item A")))
            );
        }
        _ => panic!("expected List block"),
    }
}

#[test]
fn test_parse_ordered_list() {
    let md = "1. First\n2. Second";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::List { ordered, items, .. } => {
            assert!(ordered);
            assert_eq!(items.len(), 2);
        }
        _ => panic!("expected List block"),
    }
}

#[test]
fn test_parse_blockquote() {
    let md = "> hello\n> world";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Quote(inner) => {
            assert_eq!(inner.len(), 1);
            match &inner[0] {
                Block::Paragraph(inlines) => {
                    assert_eq!(flatten_inlines(inlines), "hello world");
                }
                _ => panic!("expected Paragraph in quote"),
            }
        }
        _ => panic!("expected Quote block"),
    }
}

#[test]
fn test_parse_horizontal_rule() {
    let md = "Before\n\n---\n\nAfter";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 3);
    assert!(matches!(blocks[0], Block::Paragraph(_)));
    assert!(matches!(blocks[1], Block::HorizontalRule));
    assert!(matches!(blocks[2], Block::Paragraph(_)));
}

#[test]
fn test_link_contains_url() {
    let md = "[Click](https://example.com)";
    let blocks = parse_to_blocks(md);
    let flat = flatten_inlines(match &blocks[0] {
        Block::Paragraph(inlines) => inlines,
        _ => panic!("expected Paragraph"),
    });
    assert!(flat.contains("example.com"));
    assert!(flat.contains("Click"));
}

#[test]
fn test_flatten_inlines_preserves_code() {
    let inlines = vec![text("use "), code("std::fs"), text(";")];
    let flat = flatten_inlines(&inlines);
    assert_eq!(flat, "use `std::fs`;");
}

// ── Mermaid tests ─────────────────────────────────────────────────────────

#[test]
fn mermaid_block_rendered_none_after_parse_to_blocks() {
    let md = "```mermaid\ngraph TD\nA-->B\n```";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Mermaid { lines, rendered } => {
            assert!(rendered.is_none(), "Mermaid block must start unrendered");
            assert_eq!(lines.join("\n"), "graph TD\nA-->B");
        }
        _ => panic!("expected Mermaid block"),
    }
}

#[test]
fn parse_returns_mermaid_blocks_unrendered() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    write!(
        tmp,
        "# Title\n\n```mermaid\ngraph LR\nA-->B\n```\n\nSome text."
    )
    .unwrap();
    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();
    match result {
        ParsedContent::Markdown { blocks, .. } => {
            assert_eq!(blocks.len(), 3, "heading + mermaid + paragraph");
            match &blocks[1] {
                Block::Mermaid { lines, rendered } => {
                    assert!(
                        rendered.is_none(),
                        "parse() must NOT render mermaid blocks synchronously"
                    );
                    assert!(lines.join(" ").contains("graph LR"));
                }
                other => panic!("expected Mermaid at index 1, got {other:?}"),
            }
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn multiple_mermaid_blocks_all_unrendered() {
    let md =
        "# M1\n\n```mermaid\ngraph TD\nA\n```\n\nText\n\n```mermaid\nsequenceDiagram\nA->>B\n```";
    let blocks = parse_to_blocks(md);
    let mermaid_count = blocks
        .iter()
        .filter(|b| matches!(b, Block::Mermaid { rendered, .. } if rendered.is_none()))
        .count();
    assert_eq!(mermaid_count, 2, "both mermaid blocks must be unrendered");
}

#[test]
fn non_mermaid_content_intact_after_parse() {
    let mut tmp = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
    write!(
        tmp,
        "# Heading\n\nA paragraph.\n\n```rust\nfn main() {{}}\n```\n\n```mermaid\ngraph TD\nA-->B\n```"
    )
    .unwrap();
    let parser = MarkdownParser::new();
    let result = parser.parse(tmp.path()).unwrap();
    match result {
        ParsedContent::Markdown { blocks, .. } => {
            assert_eq!(blocks.len(), 4);
            assert!(matches!(&blocks[0], Block::Heading { level: 1, .. }));
            assert!(matches!(&blocks[1], Block::Paragraph(_)));
            assert!(matches!(&blocks[2], Block::CodeBlock { .. }));
            assert!(matches!(
                 &blocks[3],
                 Block::Mermaid { rendered, .. } if rendered.is_none()
            ));
        }
        _ => panic!("expected Markdown variant"),
    }
}

#[test]
fn render_mermaid_to_png_returns_valid_png() {
    let result = render_mermaid_to_png("sequenceDiagram\nAlice->>Bob: Hello", None, false);
    assert!(result.is_some(), "should render successfully in-process");

    let bg = resvg::tiny_skia::Color::from_rgba8(30, 30, 30, 255);
    let result_bg = render_mermaid_to_png("graph TD\nA-->B", Some(bg), false);
    assert!(
        result_bg.is_some(),
        "should render successfully with background color"
    );
}

#[test]
fn mermaid_block_update_after_async_render() {
    let mut blocks = [
        Block::Mermaid {
            lines: vec!["graph TD".into(), "A-->B".into()],
            rendered: None,
        },
        Block::Mermaid {
            lines: vec!["sequenceDiagram".into(), "A->>B".into()],
            rendered: None,
        },
    ]
    .to_vec();

    let png_bytes = Some(vec![1, 2, 3]);
    if let Block::Mermaid {
        ref mut rendered, ..
    } = blocks[1]
    {
        *rendered = png_bytes;
    }

    assert!(matches!(&blocks[0], Block::Mermaid { rendered, .. } if rendered.is_none()));
    assert!(
        matches!(&blocks[1], Block::Mermaid { rendered, .. } if rendered.as_deref() == Some(&[1u8, 2, 3]))
    );
}

#[test]
fn markdown_state_caches_mermaid_handles_correctly() {
    use crate::core::MarkdownState;
    use iced::widget::image::Handle;

    let blocks = [
        Block::Heading {
            level: 1,
            content: vec![text("Title")],
        },
        Block::Mermaid {
            lines: vec!["graph TD".into(), "A-->B".into()],
            rendered: None,
        },
        Block::Paragraph(vec![text("Some text.")]),
        Block::Mermaid {
            lines: vec!["sequenceDiagram".into(), "A->>B".into()],
            rendered: None,
        },
        Block::Mermaid {
            lines: vec!["graph LR".into(), "C-->D".into()],
            rendered: None,
        },
    ];

    let mut state = MarkdownState::default();
    assert!(
        state.cached_mermaid_handles.is_empty(),
        "fresh MarkdownState must start with an empty HashMap"
    );

    for (i, block) in blocks.iter().enumerate() {
        if let Block::Mermaid {
            rendered: Some(_), ..
        } = block
        {
            state
                .cached_mermaid_handles
                .insert(i, Handle::from_rgba(1, 1, vec![0, 0, 0, 255]));
        }
    }
    assert!(
        state.cached_mermaid_handles.is_empty(),
        "cache must be empty when no Mermaid block has rendered output yet"
    );

    state
        .cached_mermaid_handles
        .insert(1, Handle::from_rgba(2, 2, vec![128; 16]));

    assert_eq!(state.cached_mermaid_handles.len(), 1);
    assert!(
        state.cached_mermaid_handles.contains_key(&1),
        "handle must be stored under block index 1"
    );
    assert!(
        !state.cached_mermaid_handles.contains_key(&0),
        "non-Mermaid block (heading) must NOT have a cache entry"
    );
    assert!(
        !state.cached_mermaid_handles.contains_key(&3),
        "unrendered Mermaid block must NOT have a cache entry yet"
    );

    state
        .cached_mermaid_handles
        .insert(3, Handle::from_rgba(2, 2, vec![255; 16]));

    assert_eq!(
        state.cached_mermaid_handles.len(),
        2,
        "two handles cached after two Mermaid blocks rendered"
    );
    assert!(state.cached_mermaid_handles.contains_key(&3));

    state
        .cached_mermaid_handles
        .insert(1, Handle::from_rgba(4, 4, vec![64; 64]));

    assert_eq!(
        state.cached_mermaid_handles.len(),
        2,
        "replacing an existing handle must NOT increase HashMap size"
    );

    assert!(
        state.cached_mermaid_handles.contains_key(&1),
        "render_mermaid must find handle for block[1]"
    );
    assert!(
        state.cached_mermaid_handles.contains_key(&3),
        "render_mermaid must find handle for block[3]"
    );
    assert!(
        !state.cached_mermaid_handles.contains_key(&0),
        "render_mermaid must NOT find handle for heading block"
    );
    assert!(
        !state.cached_mermaid_handles.contains_key(&2),
        "render_mermaid must NOT find handle for paragraph block"
    );
    assert!(
        !state.cached_mermaid_handles.contains_key(&4),
        "render_mermaid must NOT find handle for unrendered Mermaid block"
    );
}

#[test]
fn markdown_state_ignores_non_mermaid_blocks() {
    use crate::core::MarkdownState;
    use iced::widget::image::Handle;

    let blocks = [
        Block::Paragraph(vec![text("Hello")]),
        Block::CodeBlock {
            lang: Some("rust".into()),
            title: None,
            code: "fn main() {}".into(),
        },
        Block::Table(TableBlock {
            headers: vec![TableCell {
                content: vec![text("A")],
            }],
            rows: vec![vec![TableCell {
                content: vec![text("1")],
            }]],
        }),
    ];

    let mut state = MarkdownState::default();
    for (i, block) in blocks.iter().enumerate() {
        if let Block::Mermaid {
            rendered: Some(_), ..
        } = block
        {
            state
                .cached_mermaid_handles
                .insert(i, Handle::from_rgba(1, 1, vec![0, 0, 0, 255]));
        }
    }
    assert!(
        state.cached_mermaid_handles.is_empty(),
        "cache must be empty when markdown has zero Mermaid blocks"
    );
}

// ── Math tests ────────────────────────────────────────────────────────────

#[test]
fn parses_inline_math() {
    let blocks = parse_to_blocks("Khoảng cách $D$ từ vị trí ban đầu");
    assert_eq!(blocks.len(), 1);
    if let Block::Paragraph(inlines) = &blocks[0] {
        let math = inlines.iter().find(|i| matches!(i, Inline::InlineMath(_)));
        assert!(math.is_some(), "should find InlineMath");
        if let Some(Inline::InlineMath(latex)) = math {
            assert_eq!(latex, "D");
        }
    } else {
        panic!("expected Paragraph");
    }
}

#[test]
fn parses_display_math() {
    let src = r"Before

$$
d = 2R \cdot \arcsin\left(\sqrt{\sin^2\left(\frac{\Delta \phi}{2}\right)}\right)
$$

After";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 3, "three blocks");
    assert!(
        matches!(&blocks[1], Block::Math(_)),
        "should find Block::Math as second block"
    );
}

#[test]
fn parses_inline_math_with_text_and_le() {
    let src = "$10\\text{m} \\le D \\le 50\\text{m}$";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);
    if let Block::Paragraph(inlines) = &blocks[0] {
        let math = inlines.iter().find(|i| matches!(i, Inline::InlineMath(_)));
        assert!(math.is_some(), "should find InlineMath");
    } else {
        panic!("expected Paragraph");
    }
}

#[test]
fn parses_greek_latex_in_list_items() {
    let src = "\
- $\\phi_1, \\phi_2$ là vĩ độ (latitude) của 2 điểm (tính bằng radian).
- $\\Delta \\phi = \\phi_2 - \\phi_1$.
- $\\Delta \\lambda = longitude_2 - longitude_1$ (chênh lệch kinh độ tính bằng radian).
- $R$ là bán kính Trái Đất (lấy xấp xỉ $6.371\\text{ km}$).";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1, "one list block");
    if let Block::List { items, .. } = &blocks[0] {
        assert_eq!(items.len(), 4, "four list items");
        for (i, item) in items.iter().enumerate() {
            let math_count = item
                .content
                .iter()
                .filter(|inline| matches!(inline, Inline::InlineMath(_)))
                .count();
            assert!(
                math_count >= 1,
                "item {i} should have at least one InlineMath, found {math_count}"
            );
        }
        let last_math_count = items[3]
            .content
            .iter()
            .filter(|inline| matches!(inline, Inline::InlineMath(_)))
            .count();
        assert_eq!(
            last_math_count, 2,
            "last item should have two InlineMath ($R$ and $6.371\\text{{ km}}$)"
        );
    } else {
        panic!("expected List block, got {:?}", blocks[0]);
    }
}

#[test]
fn test_standalone_taylor_series_display_math() {
    let src = "$$f(x) = \\sum_{n=0}^{\\infty} \\frac{f^{(n)}(x_0)}{n!} (x - x_0)^n = f(x_0) + f'(x_0)(x - x_0) + \\frac{f''(x_0)}{2!}(x - x_0)^2 + \\cdots + R_n(x)$$";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1, "should parse into 1 block");
    if let Block::Math(s) = &blocks[0] {
        let rendered =
            crate::features::markdown::view::components::inline_spans::render_latex_to_text(s);
        assert!(rendered.contains("∑"));
        assert!(rendered.contains("∞"));
        assert!(
            !rendered.contains("∈fty"),
            "Must not corrupt \\infty into ∈fty"
        );
        assert!(rendered.contains("⋯"));
        assert!(!rendered.contains("·s"), "Must not corrupt \\cdots into ·s");
        assert!(rendered.contains("(f⁽ⁿ⁾(x₀))/(n!)"));
    } else {
        panic!("expected Block::Math");
    }
}

#[test]
fn test_gauss_theorem_and_greek_math() {
    let src = "$$\\oiint_{S} F \\cdot n \\, dS = \\iiint_{V} (\\nabla \\cdot F) \\, dV$$";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);
    if let Block::Math(s) = &blocks[0] {
        let rendered =
            crate::features::markdown::view::components::inline_spans::render_latex_to_text(s);
        assert!(rendered.contains("∯"));
        assert!(rendered.contains("∭"));
        assert!(rendered.contains("∇"));
        assert!(rendered.contains("·"));
    } else {
        panic!("expected Block::Math");
    }

    let xi_src = "$\\xi \\in (x_0, x)$";
    let blocks = parse_to_blocks(xi_src);
    if let Block::Paragraph(inlines) = &blocks[0] {
        let rendered =
            crate::features::markdown::view::components::inline_spans::render_latex_to_text(
                if let Inline::InlineMath(s) = &inlines[0] {
                    s
                } else {
                    ""
                },
            );
        assert_eq!(rendered, "ξ ∈ (x₀, x)");
    }
}

#[test]
fn test_matrix_math_in_list_item() {
    let src = "* Item:\n$$\\Sigma = \\begin{bmatrix} 1 & 0 \\\\ 0 & 1 \\end{bmatrix}$$";
    let blocks = parse_to_blocks(src);
    assert_eq!(blocks.len(), 1);
    if let Block::List { items, .. } = &blocks[0] {
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].sub_blocks.len(), 1);
        assert!(matches!(&items[0].sub_blocks[0], Block::Math(_)));
    } else {
        panic!("expected Block::List");
    }
}

#[test]
fn test_adjacent_text_and_display_math() {
    let src = "Hàm Log-Likelihood và công thức tối ưu MLE:\n$$\\hat{\\theta}_{\\text{MLE}} = \\arg\\max_{\\theta} \\ell(\\theta; \\mathcal{D})$$";
    let blocks = parse_to_blocks(src);
    assert_eq!(
        blocks.len(),
        2,
        "should split into Paragraph and Math block"
    );
    assert!(matches!(&blocks[0], Block::Paragraph(_)));
    assert!(matches!(&blocks[1], Block::Math(_)));
}

#[test]
fn test_cong_thuc_toan_va_khoa_hoc_latex_file() {
    let path = "/home/mintori/Downloads/cong_thuc_toan_va_khoa_hoc_latex.md";
    if let Ok(content) = std::fs::read_to_string(path) {
        let blocks = parse_to_blocks(&content);
        assert!(!blocks.is_empty(), "Should parse blocks from document");

        // Verify that math inlines and display blocks are extracted properly
        let mut total_math = 0;
        for block in &blocks {
            match block {
                Block::Math(latex) => {
                    total_math += 1;
                    let text = crate::features::markdown::view::components::inline_spans::render_latex_to_text(latex);
                    assert!(!text.contains("∈fty"), "Must not contain corrupted ∈fty");
                    assert!(!text.contains("·s"), "Must not contain corrupted ·s");
                }
                Block::Paragraph(inlines) => {
                    for inline in inlines {
                        if let Inline::InlineMath(latex) | Inline::DisplayMath(latex) = inline {
                            total_math += 1;
                            let text = crate::features::markdown::view::components::inline_spans::render_latex_to_text(latex);
                            assert!(!text.contains("∈fty"), "Must not contain corrupted ∈fty");
                            assert!(!text.contains("·s"), "Must not contain corrupted ·s");
                        }
                    }
                }
                _ => {}
            }
        }
        assert!(
            total_math > 10,
            "Document should contain multiple math equations"
        );
    }
}
