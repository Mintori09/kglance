use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use iced::widget::text::Span;
use iced::widget::{column, container};
use iced::{Element, Font, Length, Padding};
use kglance::app::Message;
use kglance::core::types::MarkdownState;
use kglance::features::markdown::parser::{Block, BlockLayout, Inline};
use kglance::features::markdown::state::{apply_measured_block_heights, recompute_markdown_layout};
use kglance::ui::components::selectable_text::SelectableText;
use std::sync::Arc;

fn create_complex_inlines_sample() -> Vec<Inline> {
    vec![
        Inline::Text("Đây là đoạn văn bản kiểm thử ".to_string()),
        Inline::Bold(vec![Inline::Text("in đậm chữ Việt ".to_string())]),
        Inline::Italic(vec![Inline::Text("và in nghiêng tiếng Nhật: ".to_string())]),
        Inline::Text("「ソフトウェア」 ".to_string()),
        Inline::Code("fn process_data() -> Result<(), Error>".to_string()),
        Inline::Text(" cùng với công thức toán ".to_string()),
        Inline::InlineMath("\\sum_{k=1}^n \\frac{\\alpha_k}{\\sqrt{\\beta_k}}".to_string()),
        Inline::Text(" và Emojis: 🚀 🦀 👨‍👩‍👧‍👦 ⚡".to_string()),
    ]
}

// ---------------------------------------------------------------------------
// 1. Solution 1: Retained Spans vs On-the-Fly Spans Creation
// ---------------------------------------------------------------------------
fn bench_solution_retained_spans(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/retained_spans");

    let sample_inlines = create_complex_inlines_sample();
    let num_blocks = 30; // ~1 visible window of blocks
    let inlines_list: Vec<Vec<Inline>> = (0..num_blocks).map(|_| sample_inlines.clone()).collect();

    // A. On-the-fly: Parsing inlines and constructing Vec<Span> every frame
    group.bench_function("on_the_fly_spans_30_blocks", |b| {
        b.iter(|| {
            let mut all_spans: Vec<Vec<Span<'static, (), Font>>> = Vec::with_capacity(num_blocks);
            for inlines in &inlines_list {
                let mut spans = Vec::new();
                for inline in inlines {
                    match inline {
                        Inline::Text(t) => spans.push(Span::new(t.clone())),
                        Inline::Bold(children) => {
                            for c in children {
                                if let Inline::Text(t) = c {
                                    spans.push(Span::new(t.clone()).font(Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Font::DEFAULT
                                    }));
                                }
                            }
                        }
                        Inline::Italic(children) => {
                            for c in children {
                                if let Inline::Text(t) = c {
                                    spans.push(Span::new(t.clone()).font(Font {
                                        style: iced::font::Style::Italic,
                                        ..Font::DEFAULT
                                    }));
                                }
                            }
                        }
                        Inline::Code(code) => {
                            spans.push(Span::new(code.clone()).font(Font::MONOSPACE));
                        }
                        Inline::InlineMath(latex) => {
                            spans.push(Span::new(latex.clone()));
                        }
                        _ => {}
                    }
                }
                all_spans.push(spans);
            }
            all_spans
        })
    });

    // B. Retained / Cached: Pre-built Arc<Vec<Span>> / cached references
    let pre_built_spans: Vec<Arc<Vec<Span<'static, (), Font>>>> = (0..num_blocks)
        .map(|_| {
            let spans = vec![
                Span::new("Đây là đoạn văn bản kiểm thử "),
                Span::new("in đậm chữ Việt ").font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                Span::new("và in nghiêng tiếng Nhật: ").font(Font {
                    style: iced::font::Style::Italic,
                    ..Font::DEFAULT
                }),
                Span::new("「ソフトウェア」 "),
                Span::new("fn process_data() -> Result<(), Error>").font(Font::MONOSPACE),
                Span::new(" cùng với công thức toán ∑ (αk)/√(βk)"),
                Span::new(" và Emojis: 🚀 🦀 👨‍👩‍👧‍👦 ⚡"),
            ];
            Arc::new(spans)
        })
        .collect();

    group.bench_function("retained_arc_spans_30_blocks", |b| {
        b.iter(|| {
            let cloned: Vec<Arc<Vec<Span<'static, (), Font>>>> =
                pre_built_spans.iter().map(Arc::clone).collect();
            cloned
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 2. Solution 2: Chunk Coalescing (Separate widgets vs Coalesced Single widget)
// ---------------------------------------------------------------------------
fn bench_solution_chunk_coalescing(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/chunk_coalescing");

    let num_items = 64; // 64 short list items or paragraphs
    let items_text: Vec<String> = (0..num_items)
        .map(|i| format!("- Mục số {i}: Dữ liệu ghi chú ngắn cho danh sách"))
        .collect();

    // A. Naive: 64 separate Containers + SelectableText widgets in a Column
    group.bench_function("separate_64_individual_widgets", |b| {
        b.iter(|| {
            let mut elements: Vec<Element<'_, Message>> = Vec::with_capacity(num_items);
            for (idx, text) in items_text.iter().enumerate() {
                let widget: SelectableText<'_, Message> =
                    SelectableText::new(vec![Span::new(text.as_str())], 14.0)
                        .width(Length::Fill)
                        .block_index(idx);
                let cont = container(widget)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 4.0,
                        left: 0.0,
                    })
                    .width(Length::Fill);
                elements.push(cont.into());
            }
            let col: Element<'_, Message> = column(elements).width(Length::Fill).into();
            col
        })
    });

    // B. Coalesced: 1 single composite SelectableText widget holding all 64 items as spans with '\n'
    group.bench_function("coalesced_1_composite_widget", |b| {
        b.iter(|| {
            let mut spans = Vec::with_capacity(num_items * 2);
            for text in &items_text {
                spans.push(Span::new(text.as_str()));
                spans.push(Span::new("\n"));
            }
            let widget: SelectableText<'_, Message> = SelectableText::new(spans, 14.0)
                .width(Length::Fill)
                .block_index(0);
            let cont: Element<'_, Message> = container(widget)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 4.0,
                    left: 0.0,
                })
                .width(Length::Fill)
                .into();
            cont
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3. Solution 3: Dynamic Physics-Aware Overscan vs Static Overscan
// ---------------------------------------------------------------------------
fn bench_solution_dynamic_overscan(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/overscan_slicing");

    // 10,000 blocks document
    let num_blocks = 10_000;
    let offsets: Vec<f32> = (0..num_blocks).map(|i| i as f32 * 45.0).collect();
    let total_content_height: f32 = num_blocks as f32 * 45.0;
    let vh: f32 = 800.0;
    let scroll_y: f32 = 120_000.0; // Middle of document

    // Static overscan (fixed 800px)
    group.bench_function("static_overscan_slicing", |b| {
        b.iter(|| {
            let overscan_px: f32 = 800.0;
            let view_top = (scroll_y - overscan_px).max(0.0);
            let view_bottom = scroll_y + vh + overscan_px;

            let first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
            let last = offsets
                .partition_point(|&y| y <= view_bottom)
                .min(num_blocks);

            (first, last)
        })
    });

    // Dynamic overscan across 3 velocity states (Stationary, Moderate, High-speed flick)
    for (name, velocity) in [
        ("dynamic_stationary_v0", 0.0f32),
        ("dynamic_moderate_v1500", 1500.0f32),
        ("dynamic_fast_flick_v6000", 6000.0f32),
    ] {
        group.bench_with_input(
            BenchmarkId::new("dynamic_overscan", name),
            &velocity,
            |b, &v: &f32| {
                b.iter(|| {
                    let overscan_px =
                        (vh * 1.2 + v.abs() * 0.12).clamp(vh * 1.0, (vh * 4.0).max(2400.0));
                    let view_top = (scroll_y - overscan_px).max(0.0);
                    let view_bottom = scroll_y + vh + overscan_px;

                    const CHUNK_SIZE: usize = 8;
                    const OVERSCAN_CHUNKS: usize = 2;

                    let raw_first = offsets.partition_point(|&y| y < view_top).saturating_sub(1);
                    let raw_last = offsets
                        .partition_point(|&y| y <= view_bottom)
                        .min(num_blocks);

                    let remaining_blocks = num_blocks.saturating_sub(raw_last);
                    let dist_to_bottom = total_content_height - (scroll_y + vh);
                    let is_near_bottom = raw_last + OVERSCAN_CHUNKS * CHUNK_SIZE >= num_blocks
                        || remaining_blocks <= CHUNK_SIZE * 2
                        || dist_to_bottom <= overscan_px * 1.5;

                    let first_visible = raw_first.saturating_sub(OVERSCAN_CHUNKS * CHUNK_SIZE)
                        / CHUNK_SIZE
                        * CHUNK_SIZE;

                    let last_visible = if is_near_bottom {
                        num_blocks
                    } else {
                        (raw_last + OVERSCAN_CHUNKS * CHUNK_SIZE)
                            .min(num_blocks)
                            .div_ceil(CHUNK_SIZE)
                            * CHUNK_SIZE
                    };

                    (first_visible, last_visible.min(num_blocks))
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// 4. Solution 4: Incremental Measured Heights Update vs Full Recompute
// ---------------------------------------------------------------------------
fn bench_solution_incremental_measured_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/layout_incremental_update");

    let num_blocks = 5_000;
    let initial_layouts: Vec<BlockLayout> =
        (0..num_blocks).map(|_| BlockLayout::new(50.0)).collect();
    let initial_offsets: Vec<f32> = (0..num_blocks).map(|i| i as f32 * 50.0).collect();

    let state = MarkdownState {
        block_layouts: initial_layouts,
        block_y_offsets: initial_offsets,
        total_content_height: num_blocks as f32 * 50.0,
        scroll_y: 50_000.0,
        viewport_height: 800.0,
        ..Default::default()
    };

    let measurements = vec![
        (100, 75.0),
        (101, 85.0),
        (102, 60.0),
        (103, 110.0),
        (104, 90.0),
    ];

    // A. Incremental batched measurement update with scroll anchoring
    group.bench_function("incremental_apply_measurements_5_blocks", |b| {
        b.iter(|| {
            let mut s = state.clone();
            apply_measured_block_heights(&mut s, &measurements, 14.0);
            s
        })
    });

    // B. Recomputing all 5000 block layouts from scratch
    let dummy_blocks: Vec<Block> = (0..num_blocks)
        .map(|i| {
            Block::Paragraph(vec![Inline::Text(format!(
                "Paragraph line {i} sample text"
            ))])
        })
        .collect();

    group.bench_function("full_recompute_all_5000_blocks", |b| {
        b.iter(|| {
            let mut s = state.clone();
            recompute_markdown_layout(&mut s, &dummy_blocks, 14.0, 800.0);
            s
        })
    });

    group.finish();
}

fn create_heavy_multilingual_inlines_sample() -> Vec<Inline> {
    vec![
        Inline::Text("Đoạn văn kiểm thử đa ngữ và font shaping nâng cao: ".to_string()),
        Inline::Bold(vec![Inline::Text("Tiếng Việt có dấu (nghiêng, đậm, gạch chân) ".to_string())]),
        Inline::Italic(vec![Inline::Text("「ソフトウェア開発のパフォーマンス評価」 ".to_string())]),
        Inline::Text("tiếng Trung: “高性能文件预览系统”、tiếng Hàn: 『고성능 파일 미리보기 시스템』. ".to_string()),
        Inline::Code("fn xử_lý_unicode_utf8_graphemes<T: Debug>(dữ_liệu: &str) -> (usize, &str)".to_string()),
        Inline::Text(" Công thức: ".to_string()),
        Inline::InlineMath("\\int_{-\\infty}^{+\\infty} e^{-x^2} dx = \\sqrt{\\pi} \\quad \\Longleftrightarrow \\quad \\sum_{k=1}^n \\frac{\\alpha_k \\cdot \\beta_k}{\\sqrt{\\gamma_k^2 + \\omega_k^2}}".to_string()),
        Inline::Text(" RTL: مرحبا بالعالم & שלום עולם. ".to_string()),
        Inline::Text(" Emojis phức tạp: 👨‍👩‍👧‍👦 👩🏽‍💻 🏳️‍🌈 🦸🏼 🚀 🦀 ⚡ 📦 ✨ 🔍 🔥 🎯 💻 📊".to_string()),
    ]
}

fn bench_solution_retained_spans_heavy_glyphs(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/retained_spans_heavy_glyphs");

    let sample_inlines = create_heavy_multilingual_inlines_sample();
    let num_blocks = 30;
    let inlines_list: Vec<Vec<Inline>> = (0..num_blocks).map(|_| sample_inlines.clone()).collect();

    group.bench_function("on_the_fly_heavy_glyphs_30_blocks", |b| {
        b.iter(|| {
            let mut all_spans: Vec<Vec<Span<'static, (), Font>>> = Vec::with_capacity(num_blocks);
            for inlines in &inlines_list {
                let mut spans = Vec::new();
                for inline in inlines {
                    match inline {
                        Inline::Text(t) => spans.push(Span::new(t.clone())),
                        Inline::Bold(children) => {
                            for c in children {
                                if let Inline::Text(t) = c {
                                    spans.push(Span::new(t.clone()).font(Font {
                                        weight: iced::font::Weight::Bold,
                                        ..Font::DEFAULT
                                    }));
                                }
                            }
                        }
                        Inline::Italic(children) => {
                            for c in children {
                                if let Inline::Text(t) = c {
                                    spans.push(Span::new(t.clone()).font(Font {
                                        style: iced::font::Style::Italic,
                                        ..Font::DEFAULT
                                    }));
                                }
                            }
                        }
                        Inline::Code(code) => {
                            spans.push(Span::new(code.clone()).font(Font::MONOSPACE));
                        }
                        Inline::InlineMath(latex) => {
                            spans.push(Span::new(latex.clone()));
                        }
                        _ => {}
                    }
                }
                all_spans.push(spans);
            }
            all_spans
        })
    });

    let pre_built_spans: Vec<Arc<Vec<Span<'static, (), Font>>>> = (0..num_blocks)
        .map(|_| {
            let spans = vec![
                Span::new("Đoạn văn kiểm thử đa ngữ và font shaping nâng cao: "),
                Span::new("Tiếng Việt có dấu (nghiêng, đậm, gạch chân) ").font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::DEFAULT
                }),
                Span::new("「ソフトウェア開発のパフォーマンス評価」 ").font(Font {
                    style: iced::font::Style::Italic,
                    ..Font::DEFAULT
                }),
                Span::new("tiếng Trung: “高性能文件预览系统”、tiếng Hàn: 『고성능 파일 미리보기 시스템』. "),
                Span::new("fn xử_lý_unicode_utf8_graphemes<T: Debug>(dữ_liệu: &str) -> (usize, &str)").font(Font::MONOSPACE),
                Span::new(" Công thức: ∫ e^-x^2 dx = √π ⇔ ∑ (αk·βk)/√(γk^2+ωk^2)"),
                Span::new(" RTL: مرحبا بالعالم & שלום עולם. "),
                Span::new(" Emojis phức tạp: 👨‍👩‍👧‍👦 👩🏽‍💻 🏳️‍🌈 🦸🏼 🚀 🦀 ⚡ 📦 ✨ 🔍 🔥 🎯 💻 📊"),
            ];
            Arc::new(spans)
        })
        .collect();

    group.bench_function("retained_arc_heavy_glyphs_30_blocks", |b| {
        b.iter(|| {
            let cloned: Vec<Arc<Vec<Span<'static, (), Font>>>> =
                pre_built_spans.iter().map(Arc::clone).collect();
            cloned
        })
    });

    group.finish();
}

fn bench_solution_chunk_coalescing_heavy_glyphs(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions/chunk_coalescing_heavy_glyphs");

    let num_items = 64;
    let items_text: Vec<String> = (0..num_items)
        .map(|i| {
            format!(
                "- Mục {i}: 🌏 日本語・漢字・한글・Tiếng Việt 🚀 👨‍👩‍👧‍👦 Formula: $\\sum_{{k=1}}^{i} \\alpha_k$"
            )
        })
        .collect();

    group.bench_function("separate_64_heavy_glyph_widgets", |b| {
        b.iter(|| {
            let mut elements: Vec<Element<'_, Message>> = Vec::with_capacity(num_items);
            for (idx, text) in items_text.iter().enumerate() {
                let widget: SelectableText<'_, Message> =
                    SelectableText::new(vec![Span::new(text.as_str())], 14.0)
                        .width(Length::Fill)
                        .block_index(idx);
                let cont = container(widget)
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 4.0,
                        left: 0.0,
                    })
                    .width(Length::Fill);
                elements.push(cont.into());
            }
            let col: Element<'_, Message> = column(elements).width(Length::Fill).into();
            col
        })
    });

    group.bench_function("coalesced_1_heavy_glyph_widget", |b| {
        b.iter(|| {
            let mut spans = Vec::with_capacity(num_items * 2);
            for text in &items_text {
                spans.push(Span::new(text.as_str()));
                spans.push(Span::new("\n"));
            }
            let widget: SelectableText<'_, Message> = SelectableText::new(spans, 14.0)
                .width(Length::Fill)
                .block_index(0);
            let cont: Element<'_, Message> = container(widget)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 4.0,
                    left: 0.0,
                })
                .width(Length::Fill)
                .into();
            cont
        })
    });

    group.finish();
}

criterion_group!(
    solutions_benches,
    bench_solution_retained_spans,
    bench_solution_chunk_coalescing,
    bench_solution_dynamic_overscan,
    bench_solution_incremental_measured_layout,
    bench_solution_retained_spans_heavy_glyphs,
    bench_solution_chunk_coalescing_heavy_glyphs,
);
criterion_main!(solutions_benches);
