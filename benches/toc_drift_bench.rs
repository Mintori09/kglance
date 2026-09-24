use criterion::{Criterion, criterion_group, criterion_main};
use iced::advanced::graphics::text::Paragraph;
use iced::advanced::text::{Paragraph as _, Text};
use iced::{Font, Pixels, Size, alignment};
use kglance::features::common::parser::traits::PreviewParser;
use kglance::features::common::parser::types::ParsedContent;
use kglance::features::epub::parser::{EpubParser, load_chapter_from_epub};
use kglance::features::markdown::parser::layout_constants::{
    CODE_BUTTON_PADDING_V, CODE_LABEL_BUTTON_FONT_SIZE, CODE_LINE_FONT_SIZE, CODE_PADDING,
    CODE_TOP_BAR_PADDING_V, CONTENT_PADDING, DIVIDER_HEIGHT, PARAGRAPH_PADDING_V,
    QUOTE_CONTENT_PADDING_V, SECTION_SPACING, heading_layout, scale_size,
};
use kglance::features::markdown::parser::{
    Block, block_margin, estimated_block_height, extract_toc, intrinsic_block_height,
    parse_to_blocks,
};
use kglance::features::markdown::view::components::inline_spans::{SpanCtx, inlines_to_spans};
use kglance::ui::theme::AppTheme;
use std::cell::Cell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug)]
struct TocDriftResult {
    font_size: f32,
    width: f32,
    total_headings: usize,
    max_drift_px: f32,
    avg_drift_px: f32,
    max_drift_pct: f32,
    accurate_headings_pct: f32,
}

#[derive(Debug)]
struct TocHeadingDetail {
    title: String,
    level: u8,
    est_y: f32,
    real_y: f32,
    drift: f32,
}

fn analyze_blocks_toc_drift(
    blocks: &[Block],
    font_size: f32,
    width: f32,
    image_sizes: &HashMap<usize, (u32, u32)>,
) -> (TocDriftResult, Vec<TocHeadingDetail>) {
    let toc_entries = extract_toc(blocks, font_size, image_sizes, width);
    let mut real_positions = Vec::with_capacity(blocks.len());
    let mut current_real_y = CONTENT_PADDING;

    for (idx, block) in blocks.iter().enumerate() {
        real_positions.push(current_real_y);
        current_real_y += measure_block_height(block, font_size, width, image_sizes, idx);
    }

    let mut max_drift = 0.0f32;
    let mut total_drift = 0.0f32;
    let mut max_pct = 0.0f32;
    let mut accurate_count = 0;
    let mut details = Vec::new();

    for entry in &toc_entries {
        let real_y = real_positions
            .get(entry.block_index)
            .copied()
            .unwrap_or(0.0);
        let est_y = entry.y_offset;
        let drift = real_y - est_y;
        let abs_drift = drift.abs();

        if abs_drift > max_drift {
            max_drift = abs_drift;
        }
        total_drift += abs_drift;

        let pct = if real_y > 1.0 {
            (abs_drift / real_y) * 100.0
        } else {
            0.0
        };
        if pct > max_pct {
            max_pct = pct;
        }

        // A TOC jump is accurate if the heading lands within 60px of expected viewport top
        if abs_drift <= 60.0 {
            accurate_count += 1;
        }

        details.push(TocHeadingDetail {
            title: entry.text.clone(),
            level: entry.level,
            est_y,
            real_y,
            drift,
        });
    }

    let total = toc_entries.len();
    let avg_drift = if total > 0 {
        total_drift / total as f32
    } else {
        0.0
    };
    let accurate_pct = if total > 0 {
        (accurate_count as f32 / total as f32) * 100.0
    } else {
        100.0
    };

    (
        TocDriftResult {
            font_size,
            width,
            total_headings: total,
            max_drift_px: max_drift,
            avg_drift_px: avg_drift,
            max_drift_pct: max_pct,
            accurate_headings_pct: accurate_pct,
        },
        details,
    )
}

fn find_sample_epub() -> Option<PathBuf> {
    let candidate_paths = [
        "/home/mintori/Downloads/Phia Dong Vuon Dia Dang - John Steinbeck.epub",
        "/home/mintori/Downloads/Siêu Năng Suất - Chris Bailey & Ngô Thế Vinh (dịch).epub",
        "/home/mintori/Downloads/Telegram Desktop/Automate_the_Boring_Stuff_with_Python,_3rd_Edition_Al_Sweigart_3.epub",
        "/home/mintori/Downloads/Telegram Desktop/Atomic_Habits_Thay_Đổi_Tí_Hon,_Hiệu_Quả_Bất_Ngờ_James.epub",
    ];
    for p in &candidate_paths {
        let path = Path::new(p);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

pub fn run_full_toc_analysis() {
    println!(
        "\n=========================================================================================================="
    );
    println!("  TOC DRIFT BENCHMARK: MARKDOWN & EPUB NAVIGATION ACCURACY");
    println!(
        "=========================================================================================================="
    );

    // 1. MARKDOWN TOC ANALYSIS
    let md_path = Path::new("/home/mintori/Desktop/05.md");
    let md_content = if md_path.exists() {
        fs::read_to_string(md_path).expect("Read 05.md")
    } else {
        "# Header 1\nSome text\n## Header 2\nMore text\n".to_string()
    };
    let md_blocks = parse_to_blocks(&md_content);
    let image_sizes = HashMap::new();

    let font_sizes = [12.0f32, 14.0f32, 16.0f32, 18.0f32, 20.0f32];
    let widths = [600.0f32, 760.0f32, 960.0f32, 1200.0f32];

    println!(
        "\n[1] MARKDOWN TOC ACCURACY MATRIX (Target: 05.md, {} blocks)",
        md_blocks.len()
    );
    println!("{:-<106}", "");
    println!(
        "{:<9} | {:<7} | {:<8} | {:<12} | {:<12} | {:<12} | {:<15} | {:<10}",
        "Font Size",
        "Width",
        "Headings",
        "Avg Drift",
        "Max Drift",
        "Max Drift %",
        "Accurate (<60px)",
        "Status"
    );
    println!("{:-<106}", "");

    let mut last_details = Vec::new();

    for &fs in &font_sizes {
        for &w in &widths {
            let (res, details) = analyze_blocks_toc_drift(&md_blocks, fs, w, &image_sizes);
            let status = if res.max_drift_px <= 40.0 {
                "PERFECT"
            } else if res.max_drift_px <= 100.0 {
                "EXCELLENT"
            } else if res.max_drift_px <= 250.0 {
                "GOOD"
            } else {
                "HIGH DRIFT"
            };

            println!(
                "{:<9.1} | {:<7.0} | {:<8} | {:<+12.1} | {:<+12.1} | {:<11.2}% | {:<14.1}% | {:<10}",
                res.font_size,
                res.width,
                res.total_headings,
                res.avg_drift_px,
                res.max_drift_px,
                res.max_drift_pct,
                res.accurate_headings_pct,
                status
            );

            if (fs - 14.0).abs() < f32::EPSILON && (w - 760.0).abs() < f32::EPSILON {
                last_details = details;
            }
        }
        println!("{:-<106}", "");
    }

    if !last_details.is_empty() {
        println!("\n  --- Sample Headings Breakdown (Font: 14.0, Width: 760px) ---");
        println!("{:-<106}", "");
        println!(
            "{:<4} | {:<40} | {:<12} | {:<12} | {:<12} | {:<15}",
            "Lvl", "Heading Title", "Est Y (px)", "Real Y (px)", "Drift (px)", "Jump Precision"
        );
        println!("{:-<106}", "");
        for item in last_details.iter().take(12) {
            let truncated_title = if item.title.chars().count() > 38 {
                let s: String = item.title.chars().take(35).collect();
                format!("{s}...")
            } else {
                item.title.clone()
            };
            let precision = if item.drift.abs() <= 10.0 {
                "PINPOINT (<10px)"
            } else if item.drift.abs() <= 50.0 {
                "EXACT (<50px)"
            } else {
                "ACCEPTABLE"
            };
            println!(
                "H{:<3} | {:<40} | {:<12.1} | {:<12.1} | {:<+12.1} | {:<15}",
                item.level, truncated_title, item.est_y, item.real_y, item.drift, precision
            );
        }
        println!("{:-<106}", "");
    }

    // 2. EPUB TOC & CHAPTER DRIFT ANALYSIS
    if let Some(epub_path) = find_sample_epub() {
        println!(
            "\n[2] EPUB CHAPTER & TOC DRIFT ANALYSIS ({})",
            epub_path.file_name().unwrap().to_string_lossy()
        );
        println!("{:-<106}", "");
        println!(
            "{:<5} | {:<32} | {:<8} | {:<11} | {:<11} | {:<10} | {:<10}",
            "Ch #", "Chapter Title", "Blocks", "Est H (px)", "Real H (px)", "Diff (px)", "Err %"
        );
        println!("{:-<106}", "");

        let parser = EpubParser;
        if let Ok(ParsedContent::Epub { chapters, .. }) = parser.parse(&epub_path) {
            let sample_fs = 14.0;
            let sample_w = 760.0;
            let mut total_ch_diff = 0.0f32;
            let mut total_ch_real = 0.0f32;
            let mut tested_count = 0;
            let mut total_epub_headings = 0;
            let mut total_epub_heading_drift = 0.0f32;
            let mut max_epub_heading_drift = 0.0f32;

            for (ch_idx, (title, _lvl, anc, href, initial_blocks)) in
                chapters.iter().enumerate().take(10)
            {
                let blocks = if !initial_blocks.is_empty() {
                    initial_blocks.clone()
                } else {
                    load_chapter_from_epub(&epub_path, href, anc.as_deref()).unwrap_or_default()
                };

                if blocks.is_empty() {
                    continue;
                }

                let (ch_toc_res, _) =
                    analyze_blocks_toc_drift(&blocks, sample_fs, sample_w, &image_sizes);
                total_epub_headings += ch_toc_res.total_headings;
                total_epub_heading_drift +=
                    ch_toc_res.avg_drift_px * (ch_toc_res.total_headings as f32);
                if ch_toc_res.max_drift_px > max_epub_heading_drift {
                    max_epub_heading_drift = ch_toc_res.max_drift_px;
                }

                let est_h: f32 = blocks
                    .iter()
                    .enumerate()
                    .map(|(i, b)| estimated_block_height(b, sample_fs, i, &image_sizes, sample_w))
                    .sum();
                let real_h: f32 = blocks
                    .iter()
                    .enumerate()
                    .map(|(i, b)| measure_block_height(b, sample_fs, sample_w, &image_sizes, i))
                    .sum();

                let diff = real_h - est_h;
                let err_pct = if real_h > 0.0 {
                    (diff.abs() / real_h) * 100.0
                } else {
                    0.0
                };

                total_ch_diff += diff.abs();
                total_ch_real += real_h;
                tested_count += 1;

                let truncated_title = if title.chars().count() > 30 {
                    let s: String = title.chars().take(27).collect();
                    format!("{s}...")
                } else {
                    title.clone()
                };

                println!(
                    "{:<5} | {:<32} | {:<8} | {:<11.1} | {:<11.1} | {:<+10.1} | {:<9.2}%",
                    ch_idx + 1,
                    truncated_title,
                    blocks.len(),
                    est_h,
                    real_h,
                    diff,
                    err_pct
                );
            }
            println!("{:-<106}", "");
            if total_ch_real > 0.0 {
                let avg_err = (total_ch_diff / total_ch_real) * 100.0;
                let avg_hd_drift = if total_epub_headings > 0 {
                    total_epub_heading_drift / total_epub_headings as f32
                } else {
                    0.0
                };
                println!(
                    "  Tested {} chapters ({} total real px) | Avg Height Error: {:.2}%",
                    tested_count, total_ch_real, avg_err
                );
                println!(
                    "  Internal TOC Headings: {} | Avg Heading Drift: {:+.1}px | Max Heading Drift: {:+.1}px",
                    total_epub_headings, avg_hd_drift, max_epub_heading_drift
                );
            }
        }
    } else {
        println!(
            "\n[2] EPUB TOC ANALYSIS: No sample EPUB file found in Downloads, skipping EPUB file test."
        );
    }
    println!(
        "==========================================================================================================\n"
    );
}

fn bench_toc_extraction(c: &mut Criterion) {
    run_full_toc_analysis();

    let mut group = c.benchmark_group("toc_extraction");
    let target_path = Path::new("/home/mintori/Desktop/05.md");
    let content = if target_path.exists() {
        fs::read_to_string(target_path).expect("Read 05.md")
    } else {
        "# Header\nParagraph\n".to_string()
    };
    let blocks = parse_to_blocks(&content);
    let image_sizes = HashMap::new();

    group.bench_function("extract_toc_fs14_w760", |b| {
        b.iter(|| {
            extract_toc(&blocks, 14.0, &image_sizes, 760.0);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_toc_extraction);
criterion_main!(benches);
