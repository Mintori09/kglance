use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kglance::core::FilePreviewer;
use kglance::parsers::common::parser::traits::ParserRegistry;
use kglance::parsers::image::parser::ImageParser;
use kglance::parsers::markdown::MarkdownParser;
use kglance::parsers::text::parser::TextParser;

#[allow(dead_code)]
const DISPLAY_BUDGET: Duration = Duration::from_millis(300);
const PARSE_BUDGET: Duration = Duration::from_millis(50);
const BLOCKS_PARSE_BUDGET: Duration = Duration::from_millis(10);

fn build_registry() -> Arc<ParserRegistry> {
    let mut r = ParserRegistry::new();
    r.register(Box::new(MarkdownParser::new()));
    r.register(Box::new(TextParser::new()));
    r.register(Box::new(ImageParser));
    Arc::new(r)
}

fn make_md_file(lines: usize, mermaid_blocks: usize) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.md");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "# Test Document\n").unwrap();
    let mermaid_every = lines
        .checked_div(mermaid_blocks)
        .map(|v| v.max(1))
        .unwrap_or(usize::MAX);
    for i in 0..lines {
        writeln!(f, "Line {i}: Lorem ipsum dolor sit amet consectetur.").unwrap();
        if mermaid_blocks > 0 && i % mermaid_every == 0 {
            writeln!(f, "\n```mermaid\ngraph TD\n  A[Node {i}] --> B[End]\n```\n").unwrap();
        }
    }
    (dir, path)
}

#[test]
fn parse_to_blocks_small_md_within_budget() {
    let content =
        "# Title\n\nSome paragraph text.\n\n- Item 1\n- Item 2\n\n```rust\nfn main() {}\n```\n";
    let start = Instant::now();
    let blocks = kglance::parsers::markdown::parse_to_blocks(content);
    let elapsed = start.elapsed();

    assert!(!blocks.is_empty(), "should produce at least one block");
    assert!(
        elapsed < BLOCKS_PARSE_BUDGET,
        "parse_to_blocks small doc took {:?}, budget is {:?}",
        elapsed,
        BLOCKS_PARSE_BUDGET
    );
    println!("[LATENCY] parse_to_blocks small: {:?}", elapsed);
}

#[test]
fn parse_to_blocks_medium_md_within_budget() {
    let mut content = String::with_capacity(50_000);
    for i in 0..500 {
        content.push_str(&format!("## Section {i}\n\nContent for section {i}.\n\n"));
    }
    let start = Instant::now();
    let blocks = kglance::parsers::markdown::parse_to_blocks(&content);
    let elapsed = start.elapsed();

    assert!(!blocks.is_empty());
    assert!(
        elapsed < BLOCKS_PARSE_BUDGET,
        "parse_to_blocks 500-section doc took {:?}, budget is {:?}",
        elapsed,
        BLOCKS_PARSE_BUDGET
    );
    println!(
        "[LATENCY] parse_to_blocks medium (500 sections): {:?}",
        elapsed
    );
}

#[test]
fn parse_small_md_within_budget() {
    let (_dir, path) = make_md_file(50, 0);
    let registry = build_registry();

    // Warm up registry once (first call may include extension map setup)
    let _ = FilePreviewer::parse(&*registry, &path);

    let start = Instant::now();
    let _content = FilePreviewer::parse(&*registry, &path).expect("should parse successfully");
    let elapsed = start.elapsed();

    assert!(
        elapsed < PARSE_BUDGET,
        "[FAIL] parse for 50-line md took {:?}, budget {:?}",
        elapsed,
        PARSE_BUDGET
    );
    println!("[LATENCY] parse 50-line md: {:?}", elapsed);
}

#[test]
fn parse_medium_md_within_budget() {
    let (_dir, path) = make_md_file(500, 0);
    let registry = build_registry();
    let _ = FilePreviewer::parse(&*registry, &path); // warm up

    let start = Instant::now();
    let _content = FilePreviewer::parse(&*registry, &path).expect("should parse successfully");
    let elapsed = start.elapsed();

    assert!(
        elapsed < PARSE_BUDGET,
        "[FAIL] parse for 500-line md took {:?}, budget {:?}",
        elapsed,
        PARSE_BUDGET
    );
    println!("[LATENCY] parse 500-line md: {:?}", elapsed);
}

#[test]
fn parse_md_with_mermaid_blocks_no_render_within_budget() {
    let (_dir, path) = make_md_file(200, 3);
    let registry = build_registry();
    let _ = FilePreviewer::parse(&*registry, &path); // warm up

    let start = Instant::now();
    let _content = FilePreviewer::parse(&*registry, &path).expect("should parse successfully");
    let elapsed = start.elapsed();

    assert!(
        elapsed < PARSE_BUDGET,
        "[FAIL] parse for 200-line md with 3 mermaid blocks took {:?}, budget {:?}",
        elapsed,
        PARSE_BUDGET
    );
    println!(
        "[LATENCY] parse 200-line md +3 mermaid (no render): {:?}",
        elapsed
    );
}

#[test]
fn mermaid_blocks_are_not_rendered_synchronously() {
    use kglance::core::preview::PreviewData;
    use kglance::parsers::markdown::Block;

    let (_dir, path) = make_md_file(100, 2);
    let registry = build_registry();

    let content = FilePreviewer::parse(&*registry, &path).unwrap();

    if let PreviewData::Markdown { blocks, .. } = content {
        let mermaid_count = blocks
            .iter()
            .filter(|b| matches!(b, Block::Mermaid { .. }))
            .count();
        let rendered_count = blocks
            .iter()
            .filter(|b| {
                matches!(
                    b,
                    Block::Mermaid {
                        rendered: Some(_),
                        ..
                    }
                )
            })
            .count();

        assert!(mermaid_count > 0, "should have parsed mermaid blocks");
        assert_eq!(
            rendered_count, 0,
            "mermaid blocks must NOT be rendered synchronously during parse \
             (would block the UI thread)"
        );
        println!(
            "[CHECK] {} mermaid blocks parsed, {} rendered sync (should be 0)",
            mermaid_count, rendered_count
        );
    }
}

#[test]
fn parse_large_md_within_extended_budget() {
    // Large files get a relaxed budget — still within total display window.
    let extended_budget = Duration::from_millis(100);
    let (_dir, path) = make_md_file(3000, 0);
    let registry = build_registry();
    let _ = FilePreviewer::parse(&*registry, &path); // warm up

    let start = Instant::now();
    let _content = FilePreviewer::parse(&*registry, &path).unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed < extended_budget,
        "[FAIL] parse for 3000-line md took {:?}, extended budget {:?}",
        elapsed,
        extended_budget
    );
    println!("[LATENCY] parse 3000-line md: {:?}", elapsed);
}

#[test]
fn parse_does_not_panic_on_complex_markdown() {
    let complex = r#"
# Title with **bold** and `code`

A paragraph with [link](http://example.com) and ![img](./img.png).

| Col A | Col B |
|-------|-------|
| Cell1 | Cell2 |

```mermaid
graph TD
  A[Start] --> B{Decision}
  B -- Yes --> C[End]
  B -- No --> D[Loop]
  D --> A
```

```rust
fn main() {
    println!("hello");
}
```

> Blockquote text

1. Item one
2. Item two
   - Nested
"#;

    let blocks = kglance::parsers::markdown::parse_to_blocks(complex);
    assert!(!blocks.is_empty());
}

#[test]
fn consecutive_parse_requests_within_budget() {
    let registry = build_registry();
    let files: Vec<_> = (0..5).map(|i| make_md_file(100 + i * 50, 0)).collect();

    for (i, (_dir, path)) in files.iter().enumerate() {
        let start = Instant::now();
        let _content = FilePreviewer::parse(&*registry, path).unwrap();
        let elapsed = start.elapsed();
        assert!(
            elapsed < PARSE_BUDGET,
            "[FAIL] consecutive parse #{i} took {:?}, budget {:?}",
            elapsed,
            PARSE_BUDGET
        );
        println!("[LATENCY] consecutive parse #{i}: {:?}", elapsed);
    }
}
