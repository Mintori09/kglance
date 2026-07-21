/// Benchmarks for daemon startup latency.
///
/// Measures the critical path from "file path received" to "content ready to render".
/// Target: full parse + populate_state pipeline MUST stay under 50ms for common files,
/// leaving 250ms headroom for Iced window scheduling within the 300ms display budget.
use criterion::{Criterion, criterion_group, criterion_main};
use std::io::Write;
use std::sync::Arc;

use kglance::core::FilePreviewer;
use kglance::parsers::ParserRegistry;
use kglance::parsers::markdown::{MarkdownParser, parse_to_blocks};
use kglance::parsers::text::TextParser;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn build_registry() -> Arc<ParserRegistry> {
    let mut r = ParserRegistry::new();
    r.register(Box::new(MarkdownParser::new()));
    r.register(Box::new(TextParser::new()));
    r.register(Box::new(kglance::parsers::image::ImageParser));
    Arc::new(r)
}

fn make_md(lines: usize, mermaid_blocks: usize) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bench.md");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "# Benchmark Document\n").unwrap();
    let mermaid_every = if mermaid_blocks > 0 {
        (lines / mermaid_blocks).max(1)
    } else {
        usize::MAX
    };
    for i in 0..lines {
        writeln!(
            f,
            "Line {i}: Lorem ipsum dolor sit amet, consectetur adipiscing elit."
        )
        .unwrap();
        if mermaid_blocks > 0 && i % mermaid_every == 0 {
            writeln!(
                f,
                "\n```mermaid\ngraph TD\n  A[Start {i}] --> B[Process]\n  B --> C[End]\n```\n"
            )
            .unwrap();
        }
    }
    (dir, path)
}

fn make_text(lines: usize) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bench.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    for i in 0..lines {
        writeln!(f, "Line {i}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed ut perspiciatis.")
            .unwrap();
    }
    (dir, path)
}

// ─── parse_to_blocks micro-benchmark ─────────────────────────────────────────

fn bench_parse_to_blocks(c: &mut Criterion) {
    let small = "# Title\n\nParagraph text.\n\n```rust\nfn main() {}\n```\n";
    let medium = {
        let mut s = String::with_capacity(20_000);
        for i in 0..200 {
            s.push_str(&format!(
                "## Section {i}\n\nParagraph {i} with some text.\n\n"
            ));
            if i % 20 == 0 {
                s.push_str("```mermaid\ngraph TD\nA-->B\n```\n\n");
            }
        }
        s
    };
    let large = {
        let mut s = String::with_capacity(200_000);
        for i in 0..3000 {
            s.push_str(&format!("## Section {i}\n\nContent line {i}.\n\n"));
        }
        s
    };

    let mut group = c.benchmark_group("parse_to_blocks");
    group.sample_size(200);

    group.bench_function("small_50lines", |b| b.iter(|| parse_to_blocks(small)));
    group.bench_function("medium_200sections_10mermaid", |b| {
        b.iter(|| parse_to_blocks(&medium))
    });
    group.bench_function("large_3000sections", |b| b.iter(|| parse_to_blocks(&large)));

    group.finish();
}

// ─── Full FilePreviewer::parse benchmarks ────────────────────────────────────

fn bench_parse_md(c: &mut Criterion) {
    let registry = build_registry();
    let (_d1, p_small) = make_md(50, 0);
    let (_d2, p_medium) = make_md(500, 0);
    let (_d3, p_large) = make_md(3000, 0);
    let (_d4, p_mermaid) = make_md(200, 3);

    let mut group = c.benchmark_group("parse_md");
    group.sample_size(50);

    group.bench_function("50lines", |b| {
        b.iter(|| FilePreviewer::parse(&*registry, &p_small).unwrap())
    });
    group.bench_function("500lines", |b| {
        b.iter(|| FilePreviewer::parse(&*registry, &p_medium).unwrap())
    });
    group.bench_function("3000lines", |b| {
        b.iter(|| FilePreviewer::parse(&*registry, &p_large).unwrap())
    });
    group.bench_function("200lines_3mermaid_no_render", |b| {
        b.iter(|| FilePreviewer::parse(&*registry, &p_mermaid).unwrap())
    });

    group.finish();
}

fn bench_parse_text(c: &mut Criterion) {
    let registry = build_registry();
    let (_d1, p_small) = make_text(100);
    let (_d2, p_large) = make_text(5000);

    let mut group = c.benchmark_group("parse_text");
    group.sample_size(50);

    group.bench_function("100lines", |b| {
        b.iter(|| FilePreviewer::parse(&*registry, &p_small).unwrap())
    });
    group.bench_function("5000lines", |b| {
        b.iter(|| FilePreviewer::parse(&*registry, &p_large).unwrap())
    });

    group.finish();
}

// ─── State preparation benchmark ─────────────────────────────────────────────

fn bench_populate_state(c: &mut Criterion) {
    use kglance::core::KglanceState;

    let registry = build_registry();
    let (_dir, path) = make_md(500, 2);
    let content = FilePreviewer::parse(&*registry, &path).unwrap();

    let mut group = c.benchmark_group("populate_state");
    group.sample_size(200);

    group.bench_function("md_500lines_2mermaid", |b| {
        b.iter(|| {
            let mut state = KglanceState::default();
            content.populate_state(&mut state);
        });
    });

    group.finish();
}

// ─── End-to-end pipeline: the true latency budget ────────────────────────────

/// Full critical path: parse + populate_state.
/// This is the wall-clock time the daemon adds before the window opens.
/// Budget constraint: MUST stay under 50ms for the window to appear within 300ms.
fn bench_e2e_pipeline(c: &mut Criterion) {
    use kglance::core::KglanceState;

    let registry = build_registry();

    let mut group = c.benchmark_group("e2e_pipeline");
    group.sample_size(30);

    let scenarios: &[(&str, usize, usize)] = &[
        ("md_50lines", 50, 0),
        ("md_300lines_2mermaid", 300, 2),
        ("md_1000lines_5mermaid", 1000, 5),
    ];

    for &(name, lines, mermaid) in scenarios {
        let (_dir, path) = make_md(lines, mermaid);
        group.bench_function(name, |b| {
            b.iter(|| {
                let content = FilePreviewer::parse(&*registry, &path).unwrap();
                let mut state = KglanceState::default();
                content.populate_state(&mut state);
            });
        });
    }

    group.finish();
}

criterion_group!(
    startup_benches,
    bench_parse_to_blocks,
    bench_parse_md,
    bench_parse_text,
    bench_populate_state,
    bench_e2e_pipeline,
);
criterion_main!(startup_benches);
