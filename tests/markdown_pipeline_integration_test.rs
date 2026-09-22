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

#[test]
fn test_virtual_scroll_height_invariance() {
    let padding = 24.0f32;
    let block_count = 500;
    let block_h = 50.0f32;

    let mut offsets = Vec::with_capacity(block_count);
    let mut y = padding;
    for _ in 0..block_count {
        offsets.push(y);
        y += block_h;
    }
    let total_content_height = y + padding;

    let vh: f32 = 800.0;
    const CHUNK_SIZE: usize = 32;
    const OVERSCAN_CHUNKS: usize = 1;

    for scroll_y in [0.0f32, 500.0, 1500.0, 5000.0, 12000.0, 20000.0, 24000.0] {
        let overscan_px = (vh * 1.5f32).clamp(vh, (vh * 5.0f32).max(3000.0f32));
        let view_top = (scroll_y - overscan_px).max(0.0f32);
        let view_bottom = scroll_y + vh + overscan_px;

        let raw_first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
        let raw_last = offsets
            .partition_point(|&y| y <= view_bottom)
            .min(block_count);

        let remaining_blocks = block_count.saturating_sub(raw_last);
        let dist_to_bottom = total_content_height - (scroll_y + vh);
        let is_near_bottom =
            remaining_blocks <= CHUNK_SIZE * 2 || dist_to_bottom <= overscan_px * 1.5;

        let first_visible =
            raw_first.saturating_sub(OVERSCAN_CHUNKS * CHUNK_SIZE) / CHUNK_SIZE * CHUNK_SIZE;
        let last_visible = if is_near_bottom {
            block_count
        } else {
            (raw_last + OVERSCAN_CHUNKS * CHUNK_SIZE)
                .min(block_count)
                .div_ceil(CHUNK_SIZE)
                * CHUNK_SIZE
        };
        let last_visible = last_visible.min(block_count);

        let top_height = if first_visible > 0 {
            offsets[first_visible] - offsets[0]
        } else {
            0.0
        };
        let bottom_height = if last_visible < block_count {
            ((total_content_height - padding) - offsets[last_visible]).max(0.0)
        } else {
            0.0
        };

        let rendered_height = (last_visible - first_visible) as f32 * block_h;
        let total_inner_column = padding + top_height + rendered_height + bottom_height + padding;

        assert_eq!(
            total_inner_column, total_content_height,
            "Total virtual column height must equal total_content_height at scroll_y={scroll_y}"
        );
    }
}
