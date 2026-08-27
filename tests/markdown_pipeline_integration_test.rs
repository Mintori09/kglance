use kglance::core::preview::{FilePreviewer, PreviewData};
use kglance::core::types::KglanceState;
use kglance::engine::build_registry;
use kglance::features::markdown::parser::{Block, parse_markdown};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_markdown_parser_and_state_pipeline() {
    let dir = tempdir().expect("Failed to create tempdir");
    let md_path = dir.path().join("document.md");

    let mut file = File::create(&md_path).expect("Failed to create md file");
    let markdown_content = r#"# Main Title

This is a paragraph with **bold text**, *italic text*, and `inline code`.

Here is a math formula: $E = mc^2$ and block math:
$$
\int_{0}^{\infty} e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$

## Code Section

```rust
pub fn compute_sum(a: i32, b: i32) -> i32 {
    a + b
}
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

## Table Section

| Feature | Supported | Latency |
| :--- | :---: | ---: |
| Markdown | Yes | <5ms |
| EPUB | Yes | <10ms |
"#;
    file.write_all(markdown_content.as_bytes()).unwrap();

    let registry = build_registry();
    let preview_data = FilePreviewer::parse(&registry, &md_path)
        .expect("Markdown file must parse successfully via FilePreviewer");

    if let PreviewData::Markdown { blocks, raw_text } = &preview_data {
        assert_eq!(raw_text, markdown_content);
        assert!(!blocks.is_empty(), "Parsed blocks should not be empty");

        let mut state = KglanceState::default();
        preview_data.populate_state(&mut state);

        assert!(
            !state.markdown.toc.is_empty(),
            "Should extract headings into TOC"
        );
        assert_eq!(state.markdown.toc[0].text, "Main Title");
        assert_eq!(state.markdown.toc[0].level, 1);
        assert_eq!(state.markdown.toc[1].text, "Code Section");
        assert_eq!(state.markdown.toc[1].level, 2);
    } else {
        panic!("Expected PreviewData::Markdown");
    }
}

#[test]
fn test_markdown_direct_block_parsing_and_math() {
    let snippet = "# Header\n\nParagraph with math $a^2 + b^2 = c^2$\n\n```rust\nlet x = 1;\n```";
    let (blocks, _) = parse_markdown(snippet, std::path::Path::new("."));

    assert_eq!(blocks.len(), 3);
    assert!(matches!(blocks[0], Block::Heading { level: 1, .. }));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
    assert!(matches!(blocks[2], Block::CodeBlock { .. }));
}
