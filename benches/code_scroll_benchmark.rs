use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use kglance::features::text::anchor::ViewportAnchor;
use kglance::features::text::display_map::{DisplayMap, WrapMode};
use kglance::features::text::document::CodeDocument;
use kglance::features::text::highlight::SyntaxCache;
use kglance::features::text::policy::PerformancePolicy;
use kglance::ui::components::code_viewer::{SelectionRange, TextPosition};
use kglance::ui::theme::AppTheme;
use std::hint::black_box;
use std::time::Instant;

// =========================================================================
// Workload Generators
// =========================================================================

/// 1. Normal Rust source code
fn generate_normal_rust(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 50);
    code.push_str("// Normal Rust Source Code Benchmark\n");
    code.push_str("use std::collections::HashMap;\n\n");
    for i in 0..lines {
        code.push_str(&format!(
            "pub fn process_record_{i}(id: u64, name: &str) -> Option<String> {{\n    let formatted = format!(\"id={{id}}, name={{name}}\");\n    if id > 0 {{\n        Some(formatted)\n    }} else {{\n        None\n    }}\n}}\n"
        ));
    }
    code
}

/// 2. Very long lines (e.g. minified JSON / base64 payloads)
fn generate_long_lines(lines: usize, chars_per_line: usize) -> String {
    let mut code = String::with_capacity(lines * (chars_per_line + 1));
    let chunk = "let long_variable_token_alpha_beta = 1234567890; ";
    let repeat = (chars_per_line / chunk.len()).max(1);
    for i in 0..lines {
        code.push_str(&format!("// Line {i}: "));
        for _ in 0..repeat {
            code.push_str(chunk);
        }
        code.push('\n');
    }
    code
}

/// 3. Many short lines (e.g. config / lists)
fn generate_short_lines(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 12);
    for i in 0..lines {
        code.push_str(&format!("k_{i}: v_{i}\n"));
    }
    code
}

/// 4. Multiline strings & block comments
fn generate_multiline_strings(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 60);
    for i in 0..(lines / 10).max(1) {
        code.push_str(&format!(
            "/*\n * Block comment header section {i}\n * Describing the algorithm and invariants\n */\nconst SQL_{i}: &str = r#\"\n    SELECT u.id, u.username, p.title\n    FROM users u\n    JOIN posts p ON u.id = p.user_id\n    WHERE u.active = 1\n\"#;\n"
        ));
    }
    code
}

/// 5. Unicode-heavy source (CJK, Vietnamese, emoji)
fn generate_unicode_source(lines: usize) -> String {
    let mut code = String::with_capacity(lines * 70);
    for i in 0..lines {
        code.push_str(&format!(
            "/// Dòng thứ {i}: 这是一个测试字符串 🌟 Nhãn tiếng Việt có dấu và ký tự đặc biệt.\npub const MSG_{i}: &str = \"Xin chào thế giới! 🚀\";\n"
        ));
    }
    code
}

// =========================================================================
// 1. File Open / First-Paint Latency
// =========================================================================
fn bench_01_first_paint_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("01_FirstPaint_Latency");

    for &lines in &[10_000, 100_000] {
        let code = generate_normal_rust(lines);
        group.bench_with_input(
            BenchmarkId::new("NormalRust", lines),
            &code,
            |b, content| {
                b.iter(|| {
                    let doc = CodeDocument::new(black_box(content.as_str()));
                    let total_lines = doc.total_lines();
                    let policy = PerformancePolicy::default();
                    let interval = policy.recommended_checkpoint_interval(total_lines);
                    let mut syntax = SyntaxCache::new("rs", interval);
                    let display_map = DisplayMap::new(&doc, 14.0, WrapMode::None, 800.0);

                    let prefetch_count = total_lines.min(60);
                    let mut total_spans = 0;
                    for idx in 0..prefetch_count {
                        total_spans += syntax.get_or_tokenize_line(idx, &doc, AppTheme::Dark).len();
                    }

                    black_box((doc.total_lines(), display_map.gutter_width, total_spans))
                });
            },
        );
    }

    let unicode_code = generate_unicode_source(10_000);
    group.bench_with_input(
        BenchmarkId::new("UnicodeSource", 10_000),
        &unicode_code,
        |b, content| {
            b.iter(|| {
                let doc = CodeDocument::new(black_box(content.as_str()));
                let total_lines = doc.total_lines();
                let policy = PerformancePolicy::default();
                let interval = policy.recommended_checkpoint_interval(total_lines);
                let mut syntax = SyntaxCache::new("rs", interval);
                let display_map = DisplayMap::new(&doc, 14.0, WrapMode::None, 800.0);

                let prefetch_count = total_lines.min(60);
                let mut total_spans = 0;
                for idx in 0..prefetch_count {
                    total_spans += syntax.get_or_tokenize_line(idx, &doc, AppTheme::Dark).len();
                }

                black_box((doc.total_lines(), display_map.gutter_width, total_spans))
            });
        },
    );

    group.finish();
}

// =========================================================================
// 2. Line Indexing Throughput (MB/s)
// =========================================================================
fn bench_02_line_indexing_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("02_LineIndexing_Throughput");

    for &lines in &[10_000, 100_000] {
        let code = generate_normal_rust(lines);
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("MemchrIndexing_Normal", lines),
            &code,
            |b, content| {
                b.iter(|| {
                    let doc = CodeDocument::new(black_box(content.as_str()));
                    black_box(doc.total_lines())
                });
            },
        );
    }

    let short_code = generate_short_lines(50_000);
    group.throughput(Throughput::Bytes(short_code.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("ShortLines", 50_000),
        &short_code,
        |b, content| {
            b.iter(|| {
                let doc = CodeDocument::new(black_box(content.as_str()));
                black_box(doc.total_lines())
            });
        },
    );

    let multi_code = generate_multiline_strings(20_000);
    group.throughput(Throughput::Bytes(multi_code.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("MultilineStrings", 20_000),
        &multi_code,
        |b, content| {
            b.iter(|| {
                let doc = CodeDocument::new(black_box(content.as_str()));
                black_box(doc.total_lines())
            });
        },
    );

    let long_code = generate_long_lines(10_000, 500);
    group.throughput(Throughput::Bytes(long_code.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("LongLines_10k", 500),
        &long_code,
        |b, content| {
            b.iter(|| {
                let doc = CodeDocument::new(black_box(content.as_str()));
                black_box(doc.total_lines())
            });
        },
    );

    group.finish();
}

// =========================================================================
// 3. No-Wrap Viewport Calculation (O(1) Range)
// =========================================================================
fn bench_03_no_wrap_viewport_calc(c: &mut Criterion) {
    let mut group = c.benchmark_group("03_NoWrap_ViewportCalc");
    let code = generate_normal_rust(100_000);
    let doc = CodeDocument::new(code);
    let map = DisplayMap::new(&doc, 14.0, WrapMode::None, 800.0);

    group.bench_function("VisibleRange_ScrollMiddle", |b| {
        let mut y = 50_000.0 * 18.9;
        b.iter(|| {
            y = (y + 120.0) % (100_000.0 * 18.9);
            let range = map.compute_visible_range(black_box(y), 800.0, 5);
            black_box(range);
        });
    });

    group.finish();
}

// =========================================================================
// 4. Word-Wrap Layout Calculation
// =========================================================================
fn bench_04_word_wrap_layout_calc(c: &mut Criterion) {
    let mut group = c.benchmark_group("04_WordWrap_LayoutCalc");
    let code = generate_normal_rust(50_000);
    let doc = CodeDocument::new(code);

    let map_none = DisplayMap::new(&doc, 14.0, WrapMode::None, 800.0);
    let map_word = DisplayMap::new(&doc, 14.0, WrapMode::Word, 800.0);

    group.bench_function("WrapNone_ComputeRange", |b| {
        b.iter(|| {
            let range = map_none.compute_visible_range(black_box(10_000.0), 800.0, 4);
            black_box(range);
        });
    });

    group.bench_function("WrapWord_ComputeRange", |b| {
        b.iter(|| {
            let range = map_word.compute_visible_range(black_box(10_000.0), 800.0, 4);
            black_box(range);
        });
    });

    group.finish();
}

// =========================================================================
// 5. Random Line Jump Latency (Checkpoint Seek)
// =========================================================================
fn bench_05_random_line_jump_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("05_RandomLineJump_Latency");
    let code = generate_normal_rust(100_000);
    let doc = CodeDocument::new(code);

    for &target_line in &[10, 10_000, 50_000, 95_000] {
        group.bench_with_input(
            BenchmarkId::new("JumpToLine", target_line),
            &target_line,
            |b, &line| {
                let mut cache = SyntaxCache::new("rs", 128);
                b.iter(|| {
                    let spans = cache.get_or_tokenize_line(black_box(line), &doc, AppTheme::Dark);
                    black_box(spans.len());
                });
            },
        );
    }

    group.finish();
}

// =========================================================================
// 6. Continuous Scroll Frame Time Simulation (p50/p95/p99)
// =========================================================================
fn bench_06_continuous_scroll_frame_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("06_ContinuousScroll_FrameTime");
    let code = generate_normal_rust(20_000);
    let doc = CodeDocument::new(code);
    let map = DisplayMap::new(&doc, 14.0, WrapMode::None, 800.0);

    group.bench_function("Scroll500Steps_ViewportUpdate", |b| {
        let mut cache = SyntaxCache::new("rs", 128);
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let mut scroll_y = 0.0;
                for _ in 0..500 {
                    scroll_y += 20.0;
                    let (start_line, end_line) = map.compute_visible_range(scroll_y, 800.0, 2);
                    for line_idx in start_line..=end_line {
                        let _ = cache.get_or_tokenize_line(line_idx, &doc, AppTheme::Dark);
                    }
                }
            }
            start.elapsed()
        });
    });

    group.finish();
}

// =========================================================================
// 7. Syntect Cold Highlight Latency
// =========================================================================
fn bench_07_syntect_cold_highlight_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("07_Syntect_ColdHighlight");
    let code = generate_normal_rust(1_000);
    let doc = CodeDocument::new(code);

    group.bench_function("ColdTokenize_50LinesUnseen", |b| {
        b.iter(|| {
            let mut cache = SyntaxCache::new("rs", 64);
            let mut total_spans = 0;
            for line in 500..550 {
                let spans = cache.get_or_tokenize_line(line, &doc, AppTheme::Dark);
                total_spans += spans.len();
            }
            black_box(total_spans);
        });
    });

    group.finish();
}

// =========================================================================
// 8. Syntect Warm Highlight Latency (100% Cache Hit)
// =========================================================================
fn bench_08_syntect_warm_highlight_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("08_Syntect_WarmHighlight");
    let code = generate_normal_rust(1_000);
    let doc = CodeDocument::new(code);
    let mut cache = SyntaxCache::new("rs", 64);

    // Warm up cache for lines 100..160
    for line in 100..160 {
        let _ = cache.get_or_tokenize_line(line, &doc, AppTheme::Dark);
    }

    group.bench_function("WarmTokenize_60LinesHit", |b| {
        b.iter(|| {
            let mut total_spans = 0;
            for line in 100..160 {
                let spans = cache.get_or_tokenize_line(line, &doc, AppTheme::Dark);
                total_spans += spans.len();
            }
            black_box(total_spans);
        });
    });

    group.finish();
}

// =========================================================================
// 9. Memory + Allocation Footprint
// =========================================================================
fn bench_09_memory_allocation_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("09_Memory_Footprint");

    for &lines in &[10_000, 100_000] {
        let code = generate_normal_rust(lines);
        group.bench_with_input(
            BenchmarkId::new("DocumentMemoryBytes", lines),
            &code,
            |b, content| {
                b.iter(|| {
                    let doc = CodeDocument::new(content.as_str());
                    let bytes =
                        doc.as_str().len() + (doc.total_lines() * std::mem::size_of::<usize>());
                    black_box(bytes);
                });
            },
        );
    }

    group.finish();
}

// =========================================================================
// 10. Gutter + Code Layout / Hit-Test
// =========================================================================
fn bench_10_gutter_layout_hit_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("10_Gutter_HitTest");

    group.bench_function("CalculateGutterWidth", |b| {
        b.iter(|| {
            let w1 = DisplayMap::calculate_gutter_width(black_box(3), 14.0);
            let w2 = DisplayMap::calculate_gutter_width(black_box(6), 14.0);
            black_box((w1, w2));
        });
    });

    group.bench_function("SelectionRangeNormalization", |b| {
        let sel = SelectionRange::new(TextPosition::new(150, 20), TextPosition::new(10, 5));
        b.iter(|| {
            let (start, end) = black_box(sel).normalized();
            black_box((start, end));
        });
    });

    group.bench_function("AnchorRestoreScrollY", |b| {
        let anchor = ViewportAnchor::new(5000, 8.5);
        b.iter(|| {
            let y = black_box(anchor).restore_scroll_y(18.9);
            black_box(y);
        });
    });

    group.finish();
}

// =========================================================================
// Criterion Group Registration
// =========================================================================
criterion_group!(
    benches,
    bench_01_first_paint_latency,
    bench_02_line_indexing_throughput,
    bench_03_no_wrap_viewport_calc,
    bench_04_word_wrap_layout_calc,
    bench_05_random_line_jump_latency,
    bench_06_continuous_scroll_frame_time,
    bench_07_syntect_cold_highlight_latency,
    bench_08_syntect_warm_highlight_latency,
    bench_09_memory_allocation_footprint,
    bench_10_gutter_layout_hit_test,
);
criterion_main!(benches);
