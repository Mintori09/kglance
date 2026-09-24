use criterion::{Criterion, criterion_group, criterion_main};
use iced::advanced::graphics::text::Paragraph;
use iced::advanced::text::{Paragraph as _, Text};
use iced::{Font, Pixels, Size, alignment};
use kglance::features::markdown::parser::layout_constants::{
    CODE_BUTTON_PADDING_V, CODE_LABEL_BUTTON_FONT_SIZE, CODE_LINE_FONT_SIZE, CODE_PADDING,
    CODE_TOP_BAR_PADDING_V, DIVIDER_HEIGHT, PARAGRAPH_PADDING_V, QUOTE_CONTENT_PADDING_V,
    SECTION_SPACING, heading_layout, scale_size,
};
use kglance::features::markdown::parser::{
    Block, block_margin, estimated_block_height, intrinsic_block_height, parse_to_blocks,
};
use kglance::features::markdown::state::compute_block_layouts;
use kglance::features::markdown::view::components::inline_spans::{SpanCtx, inlines_to_spans};
use kglance::ui::theme::AppTheme;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn measure_intrinsic_block_height(
    block: &Block,
    font_size: f32,
    content_width: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
    block_index: usize,
) -> f32 {
    let span_counter = Cell::new(0);
    let span_ctx = SpanCtx {
        font_family: None,
        font_family_mono: None,
        search_query: "",
        active_match: 0,
        counter: &span_counter,
        theme: AppTheme::Light,
    };

    match block {
        Block::Heading { level, content } => {
            let layout = heading_layout(*level, font_size);
            let spans = inlines_to_spans(content, &span_ctx);
            let text_input = Text {
                content: &spans[..],
                bounds: Size::new(content_width, f32::INFINITY),
                size: Pixels(layout.font_size),
                line_height: iced::advanced::text::LineHeight::default(),
                font: Font::default(),
                align_x: alignment::Horizontal::Left.into(),
                align_y: alignment::Vertical::Top,
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::Word,
            };
            let p = Paragraph::with_spans(text_input);
            let text_h = p.min_bounds().height;
            let div = if *level == 1 || *level == 2 {
                DIVIDER_HEIGHT + SECTION_SPACING
            } else {
                0.0
            };
            layout.padding_top + text_h + layout.padding_bottom + div
        }
        Block::Paragraph(inlines) => {
            let spans = inlines_to_spans(inlines, &span_ctx);
            let text_input = Text {
                content: &spans[..],
                bounds: Size::new(content_width, f32::INFINITY),
                size: Pixels(font_size),
                line_height: iced::advanced::text::LineHeight::default(),
                font: Font::default(),
                align_x: alignment::Horizontal::Left.into(),
                align_y: alignment::Vertical::Top,
                shaping: iced::advanced::text::Shaping::Advanced,
                wrapping: iced::advanced::text::Wrapping::Word,
            };
            let p = Paragraph::with_spans(text_input);
            let pad_v = (PARAGRAPH_PADDING_V * 2) as f32;
            p.min_bounds().height + pad_v
        }
        Block::CodeBlock { code, .. } => {
            let n = code.lines().count().max(1) as f32;
            let button_font_h = scale_size(CODE_LABEL_BUTTON_FONT_SIZE, font_size);
            let top_bar =
                button_font_h + (CODE_BUTTON_PADDING_V * 2).max(CODE_TOP_BAR_PADDING_V * 2) as f32;
            let code_font_size = scale_size(CODE_LINE_FONT_SIZE, font_size);
            let code_line_h = code_font_size * 1.35;
            let pad_v = (CODE_PADDING * 2) as f32;
            top_bar + pad_v + n * code_line_h
        }
        Block::Quote(blocks) => {
            let inner_w = (content_width - 24.0).max(100.0);
            let h: f32 = blocks
                .iter()
                .map(|b| {
                    measure_intrinsic_block_height(b, font_size, inner_w, image_sizes, block_index)
                })
                .sum();
            let spacing = if blocks.len() > 1 {
                (blocks.len() - 1) as f32 * SECTION_SPACING
            } else {
                0.0
            };
            let pad_v = (QUOTE_CONTENT_PADDING_V * 2) as f32;
            h + spacing + pad_v
        }
        Block::Alert { content, .. } => {
            let inner_w = (content_width - 32.0).max(100.0);
            let h: f32 = content
                .iter()
                .map(|b| {
                    measure_intrinsic_block_height(b, font_size, inner_w, image_sizes, block_index)
                })
                .sum();
            let spacing = if content.len() > 1 {
                (content.len() - 1) as f32 * SECTION_SPACING
            } else {
                0.0
            };
            let header_h = font_size * 0.95 + 6.0;
            header_h + h + spacing + 20.0
        }
        _ => intrinsic_block_height(block, font_size, block_index, image_sizes, content_width),
    }
}

fn measure_block_height(
    block: &Block,
    font_size: f32,
    content_width: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
    block_index: usize,
) -> f32 {
    let margin = block_margin(block, font_size);
    measure_intrinsic_block_height(block, font_size, content_width, image_sizes, block_index)
        + margin
}

#[derive(Default)]
struct ErrorSummary {
    count: usize,
    total_est: f32,
    total_real: f32,
}

pub fn run_multi_size_matrix_analysis() {
    let target_path = Path::new("/home/mintori/Desktop/05.md");
    let content = if target_path.exists() {
        fs::read_to_string(target_path).expect("Read 05.md")
    } else {
        "# Header\nParagraph with CJK text\n".to_string()
    };

    let blocks = parse_to_blocks(&content);
    let total_blocks = blocks.len();
    let image_sizes = HashMap::new();

    let font_sizes: [f32; 5] = [12.0, 14.0, 16.0, 18.0, 20.0];
    let widths: [f32; 4] = [600.0, 760.0, 960.0, 1200.0];
    let vh: f32 = 800.0;

    println!(
        "=========================================================================================================="
    );
    println!(
        "  MULTI-SIZE MATRIX BENCHMARK: VIRTUAL DRIFT ANALYSIS (/home/mintori/Desktop/05.md, {total_blocks} Blocks)"
    );
    println!(
        "=========================================================================================================="
    );
    println!(
        "{:<9} | {:<7} | {:<12} | {:<12} | {:<13} | {:<13} | {:<12} | Status",
        "Font Size", "Width", "Est Height", "Real Height", "Total Drift", "Drift %", "Max Jump"
    );
    println!(
        "{:-<9}-+-{:-<7}-+-{:-<12}-+-{:-<12}-+-{:-<13}-+-{:-<13}-+-{:-<12}-+-----------",
        "", "", "", "", "", "", ""
    );

    for &fs in &font_sizes {
        for &w in &widths {
            let (_, est_offsets, est_h) = compute_block_layouts(&blocks, fs, &image_sizes, w);
            let mut real_offsets = Vec::with_capacity(total_blocks);
            let mut real_h = 0.0;
            for (i, b) in blocks.iter().enumerate() {
                real_offsets.push(real_h);
                real_h += measure_block_height(b, fs, w, &image_sizes, i);
            }

            let drift = real_h - est_h;
            let drift_pct = (drift / real_h) * 100.0;

            const CHUNK_SIZE: usize = 12;
            const OVERSCAN_CHUNKS: usize = 3;
            let overscan_px = vh * 1.5;

            let mut prev_first = 0;
            let mut max_jump = 0.0_f32;
            let mut scroll_y = 0.0_f32;

            while scroll_y < est_h {
                let view_top = (scroll_y - overscan_px).max(0.0);
                let raw_first = est_offsets
                    .partition_point(|&y| y < view_top)
                    .saturating_sub(1);
                let first_visible = if scroll_y <= overscan_px
                    || raw_first <= OVERSCAN_CHUNKS * CHUNK_SIZE
                {
                    0
                } else {
                    raw_first.saturating_sub(OVERSCAN_CHUNKS * CHUNK_SIZE) / CHUNK_SIZE * CHUNK_SIZE
                };

                if first_visible != prev_first {
                    let est_top = if first_visible > 0 {
                        est_offsets[first_visible]
                    } else {
                        0.0
                    };
                    let real_top = if first_visible > 0 {
                        real_offsets[first_visible]
                    } else {
                        0.0
                    };
                    let jump = real_top - est_top;
                    if jump.abs() > max_jump.abs() {
                        max_jump = jump;
                    }
                    prev_first = first_visible;
                }
                scroll_y += 20.0;
            }

            let status = if max_jump.abs() < 50.0 {
                "EXCELLENT"
            } else if max_jump.abs() < 150.0 {
                "MODERATE"
            } else {
                "HIGH DRIFT"
            };

            println!(
                "{:<9.1} | {:<7.0} | {:<12.1} | {:<12.1} | {:<+13.1} | {:<+12.2}% | {:<+12.1} | {}",
                fs, w, est_h, real_h, drift, drift_pct, max_jump, status
            );
        }
        println!(
            "{:-<9}-+-{:-<7}-+-{:-<12}-+-{:-<12}-+-{:-<13}-+-{:-<13}-+-{:-<12}-+-----------",
            "", "", "", "", "", "", ""
        );
    }

    println!("\n--- BLOCK TYPE ERROR PROFILE (Across all tested sizes) ---");
    println!(
        "{:<14} | {:<6} | {:<12} | {:<12} | {:<12} | {:<10}",
        "Block Type", "Count", "Avg Est (px)", "Avg Real(px)", "Avg Diff(px)", "Avg Err %"
    );
    println!(
        "{:-<14}-+-{:-<6}-+-{:-<12}-+-{:-<12}-+-{:-<12}-+-{:-<10}",
        "", "", "", "", "", ""
    );

    let mut type_stats: HashMap<&'static str, ErrorSummary> = HashMap::new();
    for &fs in &[14.0_f32, 18.0] {
        for &w in &[760.0_f32, 960.0] {
            for (i, b) in blocks.iter().enumerate() {
                let est = estimated_block_height(b, fs, i, &image_sizes, w);
                let real = measure_block_height(b, fs, w, &image_sizes, i);
                let type_name = match b {
                    Block::Heading { level, .. } => match level {
                        1 => "Heading 1",
                        2 => "Heading 2",
                        3 => "Heading 3",
                        _ => "Heading 4+",
                    },
                    Block::Paragraph(_) => "Paragraph",
                    Block::Alert { .. } => "Alert",
                    Block::CodeBlock { .. } => "CodeBlock",
                    Block::Quote(_) => "Quote",
                    Block::List { .. } => "List",
                    Block::Table(_) => "Table",
                    Block::Image { .. } => "Image",
                    Block::HorizontalRule => "Divider",
                    _ => "Other",
                };
                let entry = type_stats.entry(type_name).or_default();
                entry.count += 1;
                entry.total_est += est;
                entry.total_real += real;
            }
        }
    }

    let mut sorted_types: Vec<(&'static str, ErrorSummary)> = type_stats.into_iter().collect();
    sorted_types.sort_by(|a, b| {
        let diff_a = (a.1.total_real - a.1.total_est).abs();
        let diff_b = (b.1.total_real - b.1.total_est).abs();
        diff_b.partial_cmp(&diff_a).unwrap()
    });

    for (name, s) in sorted_types {
        let avg_est = s.total_est / s.count as f32;
        let avg_real = s.total_real / s.count as f32;
        let avg_diff = avg_real - avg_est;
        let avg_err_pct = (avg_diff / avg_real) * 100.0;
        println!(
            "{:<14} | {:<6} | {:<12.1} | {:<12.1} | {:<+12.1} | {:<+9.2}%",
            name,
            s.count / 4,
            avg_est,
            avg_real,
            avg_diff,
            avg_err_pct
        );
    }
    println!(
        "==========================================================================================================\n"
    );
}

fn bench_virtual_drift(c: &mut Criterion) {
    run_multi_size_matrix_analysis();

    let target_path = Path::new("/home/mintori/Desktop/05.md");
    let content = if target_path.exists() {
        fs::read_to_string(target_path).unwrap_or_default()
    } else {
        "# Header\nParagraph\n".into()
    };
    let blocks = parse_to_blocks(&content);
    let image_sizes = HashMap::new();

    let mut group = c.benchmark_group("virtual_matrix_drift");
    for &fs in &[12.0, 14.0, 18.0] {
        for &w in &[600.0, 760.0, 1200.0] {
            group.bench_function(format!("layout_fs{fs:.0}_w{w:.0}"), |b| {
                b.iter(|| {
                    compute_block_layouts(&blocks, fs, &image_sizes, w);
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_virtual_drift);
criterion_main!(benches);
