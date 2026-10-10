use super::*;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_parse_markdown_syntax_documentation_desktop_file() {
    let desktop_path = Path::new("/home/mintori/Desktop/markdown-syntax-documentation.md");
    if !desktop_path.exists() {
        return;
    }

    let parser = MarkdownParser::new();
    let result = parser.parse(desktop_path);
    assert!(
        result.is_ok(),
        "Failed to parse markdown syntax documentation: {:?}",
        result.err()
    );

    let (content, images, blocks) = match result.unwrap() {
        ParsedContent::Markdown {
            content,
            images,
            blocks,
        } => (content, images, blocks),
        _ => panic!("Expected Markdown ParsedContent"),
    };

    assert!(!content.is_empty());
    assert!(!blocks.is_empty());

    // Check TOC extraction
    let toc = extract_toc(&blocks, 14.0, &HashMap::new(), 800.0);
    assert!(
        toc.len() >= 15,
        "Expected at least 15 TOC items from documentation headings, got {}",
        toc.len()
    );

    // Verify first heading is the Level 1 setext heading
    assert_eq!(toc[0].level, 1);
    assert!(toc[0].text.contains("Markdown: Syntax"));

    // Verify HTML headings are present in TOC
    let toc_texts: Vec<String> = toc.iter().map(|t| t.text.clone()).collect();
    assert!(toc_texts.iter().any(|t| t.contains("Overview")));
    assert!(toc_texts.iter().any(|t| t.contains("Philosophy")));
    assert!(toc_texts.iter().any(|t| t.contains("Inline HTML")));
    assert!(toc_texts.iter().any(|t| t.contains("Block Elements")));
    assert!(toc_texts.iter().any(|t| t.contains("Headers")));
    assert!(toc_texts.iter().any(|t| t.contains("Span Elements")));
    assert!(toc_texts.iter().any(|t| t.contains("Links")));
    assert!(toc_texts.iter().any(|t| t.contains("Emphasis")));
    assert!(toc_texts.iter().any(|t| t.contains("Images")));
    assert!(toc_texts.iter().any(|t| t.contains("Miscellaneous")));

    println!(
        "Successfully parsed {} blocks, {} TOC items, {} images",
        blocks.len(),
        toc.len(),
        images.len()
    );
}

#[test]
fn test_html_heading_parsing_and_toc() {
    let md = r#"
# Main Title

<h2 id="section-1">First HTML Section</h2>

Some introductory text.

<h3 id="sub-1">Sub Section A</h3>

More details.
"#;
    let blocks = parse_to_blocks(md);
    let toc = extract_toc(&blocks, 14.0, &HashMap::new(), 800.0);

    assert_eq!(toc.len(), 3);
    assert_eq!(toc[0].level, 1);
    assert_eq!(toc[0].text, "Main Title");

    assert_eq!(toc[1].level, 2);
    assert_eq!(toc[1].text, "First HTML Section");

    assert_eq!(toc[2].level, 3);
    assert_eq!(toc[2].text, "Sub Section A");
}

#[test]
fn test_html_list_parsing() {
    let md = r#"
<ul id="ProjectSubmenu">
    <li><a href="/projects/markdown/" title="Markdown Project Page">Main</a></li>
    <li><a href="/projects/markdown/basics" title="Markdown Basics">Basics</a></li>
    <li><a class="selected" title="Markdown Syntax Documentation">Syntax</a></li>
</ul>
"#;
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::List { ordered, items, .. } => {
            assert!(!ordered);
            assert_eq!(items.len(), 3);

            let first_item_text = flatten_inlines(&items[0].content);
            assert!(first_item_text.contains("Main"));
            assert!(first_item_text.contains("/projects/markdown/"));

            let second_item_text = flatten_inlines(&items[1].content);
            assert!(second_item_text.contains("Basics"));
        }
        other => panic!("Expected Block::List, got {other:?}"),
    }
}

#[test]
fn test_inline_html_formatting() {
    let md = "Here is <b>bold text</b>, <i>italic text</i>, <code>code text</code>, and <a href=\"https://example.com\">link text</a>.";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert!(inlines.iter().any(|i| matches!(i, Inline::Bold(_))));
            assert!(inlines.iter().any(|i| matches!(i, Inline::Italic(_))));
            assert!(inlines.iter().any(|i| matches!(i, Inline::Code(_))));
            assert!(inlines.iter().any(|i| matches!(i, Inline::Link { .. })));

            let visual = flatten_inlines_visual(inlines);
            assert_eq!(
                visual,
                "Here is bold text, italic text, code text, and link text."
            );
        }
        other => panic!("Expected Block::Paragraph, got {other:?}"),
    }
}

#[test]
fn test_html_entity_decoding() {
    let md = "Copyright &copy; 2024 &mdash; AT&T &amp; 4 &lt; 5 &gt; 3 &#8212; special.";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::Paragraph(inlines) => {
            let text = flatten_inlines_visual(inlines);
            assert!(text.contains('©'));
            assert!(text.contains('—'));
            assert!(text.contains('&'));
            assert!(text.contains('<'));
            assert!(text.contains('>'));
        }
        other => panic!("Expected Block::Paragraph, got {other:?}"),
    }
}

#[test]
fn test_html_table_parsing() {
    let md = r#"
<table>
    <tr>
        <th>Header 1</th>
        <th>Header 2</th>
    </tr>
    <tr>
        <td>Value 1</td>
        <td>Value 2</td>
    </tr>
</table>
"#;
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);

    match &blocks[0] {
        Block::Table(tbl) => {
            assert_eq!(tbl.headers.len(), 2);
            assert_eq!(flatten_inlines(&tbl.headers[0].content), "Header 1");
            assert_eq!(flatten_inlines(&tbl.headers[1].content), "Header 2");
            assert_eq!(tbl.rows.len(), 1);
            assert_eq!(flatten_inlines(&tbl.rows[0][0].content), "Value 1");
            assert_eq!(flatten_inlines(&tbl.rows[0][1].content), "Value 2");
        }
        other => panic!("Expected Block::Table, got {other:?}"),
    }
}

#[test]
fn test_html_comment_ignored() {
    let md = "<!-- This is a comment -->\n\nActual paragraph.";
    let blocks = parse_to_blocks(md);
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        Block::Paragraph(inlines) => {
            assert_eq!(flatten_inlines(inlines), "Actual paragraph.");
        }
        other => panic!("Expected Block::Paragraph, got {other:?}"),
    }
}
