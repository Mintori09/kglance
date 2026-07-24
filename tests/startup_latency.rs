/// Integration tests for startup latency.
///
/// Measures the critical path from "file path received" to "content ready to render".
/// Target: full parse pipeline MUST stay under 50ms for common files,
/// leaving 250ms for Iced window creation and frame scheduling within the 300ms budget.
///
/// NOTE: `KglanceState::default()` includes Iced widget initialization
/// (`text_editor::Content::new()`) which requires GPU/font subsystems. That cost
/// is paid once at daemon startup, NOT on each preview request. These tests
/// therefore isolate the parse-only critical path which IS repeated per-request.
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kglance::parsers::ParserRegistry;
use kglance::parsers::markdown::MarkdownParser;
use kglance::parsers::text::TextParser;

// ─── Budget constants ─────────────────────────────────────────────────────────

/// Total time from DBus request to first visible frame.
#[allow(dead_code)]
const DISPLAY_BUDGET: Duration = Duration::from_millis(300);

/// Time budget for parse-only (repeated per preview request).
/// The daemon parses then sends a single event; Iced must handle the rest.
const PARSE_BUDGET: Duration = Duration::from_millis(50);

/// parse_to_blocks() micro-budget for typical markdown.
const BLOCKS_PARSE_BUDGET: Duration = Duration::from_millis(10);

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn build_registry() -> Arc<ParserRegistry> {
    let mut r = ParserRegistry::new();
    r.register(Box::new(MarkdownParser::new()));
    r.register(Box::new(TextParser::new()));
    r.register(Box::new(kglance::parsers::image::ImageParser));
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

// ─── parse_to_blocks latency tests ───────────────────────────────────────────

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

// ─── ParserRegistry::parse latency tests (the per-request critical path) ──────

/// parse() is the entire per-request work in the daemon (after our optimization).
/// KglanceState::default() is paid once at daemon startup and is NOT included here.
#[test]
fn parse_small_md_within_budget() {
    let (_dir, path) = make_md_file(50, 0);
    let registry = build_registry();

    // Warm up registry once (first call may include extension map setup)
    let _ = registry.parse(&path);

    let start = Instant::now();
    let _content = registry.parse(&path).expect("should parse successfully");
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
    let _ = registry.parse(&path); // warm up

    let start = Instant::now();
    let _content = registry.parse(&path).expect("should parse successfully");
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
    // Mermaid blocks must NOT be rendered synchronously.
    let (_dir, path) = make_md_file(200, 3);
    let registry = build_registry();
    let _ = registry.parse(&path); // warm up

    let start = Instant::now();
    let _content = registry.parse(&path).expect("should parse successfully");
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
    use kglance::features::markdown::types::Block;

    let (_dir, path) = make_md_file(100, 2);
    let content = std::fs::read_to_string(&path).unwrap();
    let blocks = kglance::parsers::markdown::parse_to_blocks(&content);

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

#[test]
fn parse_large_md_within_extended_budget() {
    // Large files get a relaxed budget — still within total display window.
    let extended_budget = Duration::from_millis(100);
    let (_dir, path) = make_md_file(3000, 0);
    let registry = build_registry();
    let _ = registry.parse(&path); // warm up

    let start = Instant::now();
    let _content = registry.parse(&path).unwrap();
    let elapsed = start.elapsed();

    assert!(
        elapsed < extended_budget,
        "[FAIL] parse for 3000-line md took {:?}, extended budget {:?}",
        elapsed,
        extended_budget
    );
    println!("[LATENCY] parse 3000-line md: {:?}", elapsed);
}

// ─── Regression: parse must succeed, not panic ───────────────────────────────

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

    // Must not panic
    let blocks = kglance::parsers::markdown::parse_to_blocks(complex);
    assert!(!blocks.is_empty());
}

// ─── Regression: back-to-back requests ───────────────────────────────────────

#[test]
fn consecutive_parse_requests_within_budget() {
    // Simulates rapid file switching: each parse must stay within budget
    let registry = build_registry();
    let files: Vec<_> = (0..5).map(|i| make_md_file(100 + i * 50, 0)).collect();

    for (i, (_dir, path)) in files.iter().enumerate() {
        let start = Instant::now();
        let _content = registry.parse(path).unwrap();
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
