use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kglance::core::types::KglanceState;
use kglance::features::markdown::parser::{extract_toc, parse_to_blocks};
use kglance::features::markdown::state::{compute_block_layouts, populate_state};
use kglance::features::markdown::view_markdown;
use kglance::ui::theme::AppTheme;
use std::collections::HashMap;

fn generate_markdown_content(sections: usize) -> String {
    let mut s = String::with_capacity(sections * 250);
    s.push_str("# Performance Benchmark Document\n\n");
    for i in 0..sections {
        s.push_str(&format!("## Section {i}: Detailed Report\n\n"));
        s.push_str(&format!(
            "This is paragraph **{i}** discussing performance optimization with `inline code` and *emphasis*.\n\n"
        ));
        s.push_str(&format!(
            "- Key takeaway {i}-A with extra notes\n- Key takeaway {i}-B with metadata\n- Key takeaway {i}-C with details\n\n"
        ));
        if i % 5 == 0 {
            s.push_str(&format!(
                "```rust\nfn process_data_{i}(input: &str) -> usize {{\n    input.lines().count()\n}}\n```\n\n"
            ));
        }
        if i % 10 == 0 {
            s.push_str("| Metric | Baseline | Optimized |\n|---|---|---|\n| Frame Time | 16ms | 2ms |\n| Memory | 50MB | 12MB |\n\n");
        }
    }
    s
}

fn bench_markdown_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown/parse_to_blocks");
    for sections in [20, 200, 1000] {
        let md = generate_markdown_content(sections);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{sections}_sections")),
            &md,
            |b, input| b.iter(|| parse_to_blocks(input)),
        );
    }
    group.finish();
}

fn bench_markdown_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown/layout_offsets");
    let image_sizes = HashMap::new();
    let font_size = 14.0;
    let content_width = 800.0;

    for sections in [50, 500, 2000] {
        let md = generate_markdown_content(sections);
        let blocks = parse_to_blocks(&md);

        group.bench_with_input(
            BenchmarkId::new("compute_layouts", format!("{}_blocks", blocks.len())),
            &blocks,
            |b, blks| {
                b.iter(|| compute_block_layouts(blks, font_size, &image_sizes, content_width))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("extract_toc", format!("{}_blocks", blocks.len())),
            &blocks,
            |b, blks| b.iter(|| extract_toc(blks, font_size, &image_sizes, content_width)),
        );
    }
    group.finish();
}

fn bench_markdown_populate_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown/populate_state");

    for sections in [50, 500, 2000] {
        let md = generate_markdown_content(sections);
        let blocks = parse_to_blocks(&md);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_blocks", blocks.len())),
            &blocks,
            |b, blks| {
                b.iter(|| {
                    let mut state = KglanceState::default();
                    populate_state(&mut state, blks);
                    state
                })
            },
        );
    }
    group.finish();
}

fn bench_markdown_view_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown/view_tree_generation");

    for sections in [50, 500, 2000] {
        let md = generate_markdown_content(sections);
        let blocks = parse_to_blocks(&md);

        let mut state = KglanceState {
            window_width: 1200.0,
            window_height: 800.0,
            ..Default::default()
        };
        populate_state(&mut state, &blocks);

        // 1. Virtualized view generation (current production path)
        group.bench_with_input(
            BenchmarkId::new("virtualized", format!("{}_blocks", blocks.len())),
            &(&blocks, &state.markdown),
            |b, (blks, md_state)| {
                b.iter(|| {
                    let _element = view_markdown(
                        blks,
                        md_state,
                        14.0,
                        AppTheme::Dark,
                        None,
                        None,
                        Some(800.0),
                    );
                })
            },
        );

        // 2. Non-virtualized view generation (simulating naive full tree by wiping block_y_offsets)
        let mut naive_state = state.markdown.clone();
        naive_state.block_y_offsets.clear(); // forces fallback to all blocks rendered

        group.bench_with_input(
            BenchmarkId::new("naive_all_blocks", format!("{}_blocks", blocks.len())),
            &(&blocks, &naive_state),
            |b, (blks, md_state)| {
                b.iter(|| {
                    let _element = view_markdown(
                        blks,
                        md_state,
                        14.0,
                        AppTheme::Dark,
                        None,
                        None,
                        Some(800.0),
                    );
                })
            },
        );
    }
    group.finish();
}

fn bench_markdown_scroll_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown/scroll_simulation");

    let md = generate_markdown_content(1000);
    let blocks = parse_to_blocks(&md);

    let mut state = KglanceState {
        window_width: 1200.0,
        window_height: 800.0,
        ..Default::default()
    };
    populate_state(&mut state, &blocks);

    let total_h = state.markdown.total_content_height;
    let step = (total_h / 60.0).max(10.0);

    // Benchmarking 60 frames of scroll updates
    group.bench_function("virtualized_60_frames_scroll", |b| {
        b.iter(|| {
            let mut md_state = state.markdown.clone();
            for i in 0..60 {
                md_state.scroll_y = (i as f32 * step).min(total_h);
                let _element = view_markdown(
                    &blocks,
                    &md_state,
                    14.0,
                    AppTheme::Dark,
                    None,
                    None,
                    Some(800.0),
                );
            }
        })
    });

    let mut naive_state = state.markdown.clone();
    naive_state.block_y_offsets.clear();

    group.bench_function("naive_60_frames_scroll", |b| {
        b.iter(|| {
            let mut md_state = naive_state.clone();
            for i in 0..60 {
                md_state.scroll_y = (i as f32 * step).min(total_h);
                let _element = view_markdown(
                    &blocks,
                    &md_state,
                    14.0,
                    AppTheme::Dark,
                    None,
                    None,
                    Some(800.0),
                );
            }
        })
    });

    group.finish();
}

fn generate_heavy_glyphs_markdown(sections: usize) -> String {
    let mut s = String::with_capacity(sections * 450);
    s.push_str("# 🚀 Multilingual & Complex Glyphs Benchmark (多言語・이모지・Toán học)\n\n");
    for i in 0..sections {
        s.push_str(&format!(
            "## Mục {i}: 🌏 CJK, Diacritics & Emojis (こんにちは・你好・안녕하세요) 👨‍👩‍👧‍👦 🏳️‍🌈\n\n"
        ));
        s.push_str(&format!(
            "Đoạn văn **{i}**: Kiểm tra khả năng xử lý *font shaping* và đo lường layout với dấu tiếng Việt (nghiêng, đậm, gạch chân). \
            Tiếp theo là tiếng Nhật: 「ソフトウェア開発のパフォーマンス評価」、tiếng Trung: “高性能文件预览系统”、\
            và tiếng Hàn: 『고성능 파일 미리보기 시스템』. Kết hợp emoji: 🦀 ⚡ 📦 ✨ 🔍 🔥 🎯 💻 📊.\n\n"
        ));
        s.push_str(&format!(
            "- 🔹 Điểm đo {i}-A: Arabic RTL مرحبا بالعالم & Hebrew שלום עולם\n\
             - 🔹 Điểm đo {i}-B: Greek & Math $\\sum_{{k=1}}^n \\frac{{\\alpha_k \\cdot \\beta_k}}{{\\sqrt{{\\gamma_k^2 + \\omega_k^2}}}}$\n\
             - 🔹 Điểm đo {i}-C: Complex Emojis: 👩🏽‍💻 👨🏼‍🔬 🧙‍♂️ 🧟‍♀️ 🏇🏿 🏄‍♂️\n\n"
        ));
        if i % 4 == 0 {
            s.push_str(&format!(
                "```rust\n// 🦀 Hàm xử lý chuỗi UTF-8 phức tạp với Unicode Graphemes\nfn kiểm_tra_ký_tự_{i}(chuỗi_vào: &str) -> (usize, &str) {{\n    let nhãn = \"🚀 測試_테스트_DữLiệu_{i}\";\n    (chuỗi_vào.chars().count(), nhãn)\n}}\n```\n\n"
            ));
        }
        if i % 6 == 0 {
            s.push_str(&format!(
                "$$\n\\int_{{-\\infty}}^{{+\\infty}} e^{{-x^2}} dx = \\sqrt{{\\pi}} \\quad \\Longleftrightarrow \\quad \\oint_C \\mathbf{{B}} \\cdot d\\boldsymbol{{\\ell}} = \\mu_0 I_{{\\text{{enc}}}} + \\mu_0 \\varepsilon_0 \\frac{{d\\Phi_E}}{{dt}} \\quad (Mục \\ {i})\n$$\n\n"
            ));
        }
        if i % 8 == 0 {
            s.push_str("| Ngôn ngữ / Glyphs | Ký tự mẫu | Trạng thái |\n|---|---|---|\n| CJK & Tiếng Việt | 漢字・한글・Tiếng Việt có dấu | ✅ Tối ưu |\n| Emojis ZWJ | 👨‍👩‍👧‍👦 👩🏾‍🚀 🦸🏼 | ✅ Hỗ trợ |\n| Math & Greek | $\\forall x \\in \\mathbb{R}, \\exists \\varepsilon > 0$ | ✅ Render |\n\n");
        }
    }
    s
}

fn bench_heavy_glyphs_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("glyphs/parse_to_blocks");
    for sections in [20, 200, 1000] {
        let md = generate_heavy_glyphs_markdown(sections);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{sections}_sections")),
            &md,
            |b, input| b.iter(|| parse_to_blocks(input)),
        );
    }
    group.finish();
}

fn bench_heavy_glyphs_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("glyphs/layout_offsets");
    let image_sizes = HashMap::new();
    let font_size = 14.0;
    let content_width = 800.0;

    for sections in [50, 500, 2000] {
        let md = generate_heavy_glyphs_markdown(sections);
        let blocks = parse_to_blocks(&md);

        group.bench_with_input(
            BenchmarkId::new("compute_layouts", format!("{}_blocks", blocks.len())),
            &blocks,
            |b, blks| {
                b.iter(|| compute_block_layouts(blks, font_size, &image_sizes, content_width))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("extract_toc", format!("{}_blocks", blocks.len())),
            &blocks,
            |b, blks| b.iter(|| extract_toc(blks, font_size, &image_sizes, content_width)),
        );
    }
    group.finish();
}

fn bench_heavy_glyphs_view_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("glyphs/view_tree_generation");

    for sections in [50, 500, 2000] {
        let md = generate_heavy_glyphs_markdown(sections);
        let blocks = parse_to_blocks(&md);

        let mut state = KglanceState {
            window_width: 1200.0,
            window_height: 800.0,
            ..Default::default()
        };
        populate_state(&mut state, &blocks);

        // 1. Virtualized view generation
        group.bench_with_input(
            BenchmarkId::new("virtualized", format!("{}_blocks", blocks.len())),
            &(&blocks, &state.markdown),
            |b, (blks, md_state)| {
                b.iter(|| {
                    let _element = view_markdown(
                        blks,
                        md_state,
                        14.0,
                        AppTheme::Dark,
                        None,
                        None,
                        Some(800.0),
                    );
                })
            },
        );

        // 2. Non-virtualized view generation
        let mut naive_state = state.markdown.clone();
        naive_state.block_y_offsets.clear();

        group.bench_with_input(
            BenchmarkId::new("naive_all_blocks", format!("{}_blocks", blocks.len())),
            &(&blocks, &naive_state),
            |b, (blks, md_state)| {
                b.iter(|| {
                    let _element = view_markdown(
                        blks,
                        md_state,
                        14.0,
                        AppTheme::Dark,
                        None,
                        None,
                        Some(800.0),
                    );
                })
            },
        );
    }
    group.finish();
}

fn bench_heavy_glyphs_scroll_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("glyphs/scroll_simulation");

    let md = generate_heavy_glyphs_markdown(1000);
    let blocks = parse_to_blocks(&md);

    let mut state = KglanceState {
        window_width: 1200.0,
        window_height: 800.0,
        ..Default::default()
    };
    populate_state(&mut state, &blocks);

    let total_h = state.markdown.total_content_height;
    let step = (total_h / 60.0).max(10.0);

    group.bench_function("virtualized_60_frames_scroll", |b| {
        b.iter(|| {
            let mut md_state = state.markdown.clone();
            for i in 0..60 {
                md_state.scroll_y = (i as f32 * step).min(total_h);
                let _element = view_markdown(
                    &blocks,
                    &md_state,
                    14.0,
                    AppTheme::Dark,
                    None,
                    None,
                    Some(800.0),
                );
            }
        })
    });

    let mut naive_state = state.markdown.clone();
    naive_state.block_y_offsets.clear();

    group.bench_function("naive_60_frames_scroll", |b| {
        b.iter(|| {
            let mut md_state = naive_state.clone();
            for i in 0..60 {
                md_state.scroll_y = (i as f32 * step).min(total_h);
                let _element = view_markdown(
                    &blocks,
                    &md_state,
                    14.0,
                    AppTheme::Dark,
                    None,
                    None,
                    Some(800.0),
                );
            }
        })
    });

    group.finish();
}

criterion_group!(
    markdown_benches,
    bench_markdown_parse,
    bench_markdown_layout,
    bench_markdown_populate_state,
    bench_markdown_view_generation,
    bench_markdown_scroll_simulation,
    bench_heavy_glyphs_parse,
    bench_heavy_glyphs_layout,
    bench_heavy_glyphs_view_generation,
    bench_heavy_glyphs_scroll_simulation,
);
criterion_main!(markdown_benches);
