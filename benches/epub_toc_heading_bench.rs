use criterion::{Criterion, criterion_group, criterion_main};
use kglance::features::common::parser::traits::PreviewParser;
use kglance::features::common::parser::types::ParsedContent;
use kglance::features::epub::parser::{EpubParser, find_block_index, load_chapter_from_epub};
use kglance::parsers::markdown::{Block, flatten_inlines};
use std::path::{Path, PathBuf};

fn find_sample_epubs() -> Vec<PathBuf> {
    let candidate_paths = [
        "/home/mintori/Downloads/Telegram Desktop/Automate_the_Boring_Stuff_with_Python,_3rd_Edition_Al_Sweigart_3.epub",
        "/home/mintori/Downloads/Telegram Desktop/Atomic_Habits_Thay_Đổi_Tí_Hon,_Hiệu_Quả_Bất_Ngờ_James.epub",
        "/home/mintori/Downloads/Tư Duy Chiến Lược - Avinash K. Dixit & Bary J. Nalebuff.epub",
        "/home/mintori/Downloads/Telegram Desktop/Hands_On_Large_Language_Models_Jay_Alammar;Maarten_Grootendorst;.epub",
        "/home/mintori/Downloads/Telegram Desktop/Secure_Coding_in_TypeScript_Best_Practices_and_Baldurs_L_2025_1 (3).epub",
    ];

    candidate_paths
        .iter()
        .map(Path::new)
        .filter(|p| p.exists())
        .map(Path::to_path_buf)
        .collect()
}

type LoadedChapter = (String, u8, Option<String>, String, Vec<Block>);

fn create_synthetic_chapters() -> Vec<LoadedChapter> {
    let mut chapters = Vec::new();
    let mut blocks = Vec::new();

    blocks.push(Block::Paragraph(vec![
        kglance::parsers::markdown::Inline::Text("Book overview and intro paragraph.".to_string()),
    ]));
    blocks.push(Block::Heading {
        level: 1,
        content: vec![kglance::parsers::markdown::Inline::Text(
            "Chapter 1: Foundations".to_string(),
        )],
    });

    for sec in 1..=20 {
        blocks.push(Block::Html(format!("<a id=\"sec-{sec}\"></a>")));
        blocks.push(Block::Heading {
            level: 2,
            content: vec![kglance::parsers::markdown::Inline::Text(format!(
                "Section 1.{sec}: Core Concept"
            ))],
        });
        for p in 0..5 {
            blocks.push(Block::Paragraph(vec![
                kglance::parsers::markdown::Inline::Text(format!(
                    "Detailed paragraph {p} discussing Section 1.{sec} with various code and notes."
                )),
            ]));
        }
    }

    chapters.push((
        "Chapter 1: Foundations".to_string(),
        1,
        None,
        "ch1.xhtml".to_string(),
        blocks.clone(),
    ));

    for sec in 1..=20 {
        chapters.push((
            format!("Section 1.{sec}: Core Concept"),
            2,
            Some(format!("sec-{sec}")),
            "ch1.xhtml".to_string(),
            blocks.clone(),
        ));
    }

    chapters
}

pub fn run_toc_heading_verification() {
    println!(
        "\n=========================================================================================================="
    );
    println!("  EPUB TOC & HEADING EXISTENCE BENCHMARK / VERIFICATION REPORT");
    println!(
        "=========================================================================================================="
    );

    let epubs = find_sample_epubs();
    if epubs.is_empty() {
        println!("No external EPUB samples found, running synthetic test suite.");
        let chapters = create_synthetic_chapters();
        verify_chapters_toc_headings("Synthetic Book (20 sub-headings)", &chapters);
    } else {
        let parser = EpubParser;
        for epub_path in &epubs {
            let filename = epub_path
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Unknown.epub".to_string());

            if let Ok(ParsedContent::Epub { chapters, .. }) = parser.parse(epub_path) {
                // Ensure blocks loaded for sampled chapters
                let mut loaded_chapters = Vec::new();
                for (title, lvl, anc, href, initial_blocks) in chapters.into_iter().take(35) {
                    let blocks = if !initial_blocks.is_empty() {
                        initial_blocks
                    } else {
                        load_chapter_from_epub(epub_path, &href, anc.as_deref()).unwrap_or_default()
                    };
                    loaded_chapters.push((title, lvl, anc, href, blocks));
                }
                verify_chapters_toc_headings(&filename, &loaded_chapters);
            }
        }
    }
    println!(
        "==========================================================================================================\n"
    );
}

fn verify_chapters_toc_headings(book_title: &str, chapters: &[LoadedChapter]) {
    println!(
        "\nBook: {book_title} (Evaluated TOC Items: {})",
        chapters.len()
    );
    println!("{:-<106}", "");
    println!(
        "{:<4} | {:<38} | {:<16} | {:<9} | {:<25}",
        "Lvl", "TOC Entry Title", "Anchor", "Block #", "Matched Target Preview"
    );
    println!("{:-<106}", "");

    let mut total = 0;
    let mut matched = 0;
    let mut matched_by_anchor = 0;
    let mut matched_by_title = 0;

    for (title, lvl, anc, _href, blocks) in chapters {
        if blocks.is_empty() {
            continue;
        }
        total += 1;
        let found = find_block_index(blocks, anc.as_deref(), Some(title));

        let truncated_title = if title.chars().count() > 36 {
            let s: String = title.chars().take(33).collect();
            format!("{s}...")
        } else {
            title.clone()
        };

        let anchor_display = anc.as_deref().unwrap_or("[none]");
        let truncated_anchor = if anchor_display.len() > 16 {
            format!("{}...", &anchor_display[..13])
        } else {
            anchor_display.to_string()
        };

        if let Some(b_idx) = found {
            matched += 1;
            let block = &blocks[b_idx];
            let block_text = match block {
                Block::Heading { content, .. } => flatten_inlines(content),
                Block::Paragraph(content) => flatten_inlines(content),
                Block::Html(h) => h.clone(),
                _ => format!("{block:?}"),
            };

            let preview: String = block_text
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(24)
                .collect();

            if anc.is_some() {
                matched_by_anchor += 1;
            } else {
                matched_by_title += 1;
            }

            println!(
                "H{:<3} | {:<38} | {:<16} | {:<9} | {:<25}",
                lvl, truncated_title, truncated_anchor, b_idx, preview
            );
        } else {
            println!(
                "H{:<3} | {:<38} | {:<16} | {:<9} | [MISSING TARGET]",
                lvl, truncated_title, truncated_anchor, "FAIL"
            );
        }
    }

    println!("{:-<106}", "");
    let match_pct = if total > 0 {
        (matched as f32 / total as f32) * 100.0
    } else {
        0.0
    };
    println!(
        "  Match Accuracy: {matched}/{total} ({match_pct:.1}%) | By Anchor: {matched_by_anchor} | By Title: {matched_by_title}"
    );
}

fn bench_toc_heading_lookup(c: &mut Criterion) {
    run_toc_heading_verification();

    let mut group = c.benchmark_group("epub_toc_heading_lookup");
    let synthetic_chapters = create_synthetic_chapters();

    group.bench_function("synthetic_20_headings_lookup", |b| {
        b.iter(|| {
            for (title, _lvl, anc, _href, blocks) in &synthetic_chapters {
                let _ = find_block_index(blocks, anc.as_deref(), Some(title));
            }
        });
    });

    let epubs = find_sample_epubs();
    if let Some(first_epub) = epubs.first() {
        let parser = EpubParser;
        if let Ok(ParsedContent::Epub { chapters, .. }) = parser.parse(first_epub) {
            let mut loaded = Vec::new();
            for (title, lvl, anc, href, initial_blocks) in chapters.into_iter().take(20) {
                let blocks = if !initial_blocks.is_empty() {
                    initial_blocks
                } else {
                    load_chapter_from_epub(first_epub, &href, anc.as_deref()).unwrap_or_default()
                };
                loaded.push((title, lvl, anc, href, blocks));
            }

            group.bench_function("real_epub_heading_lookup_20_entries", |b| {
                b.iter(|| {
                    for (title, _lvl, anc, _href, blocks) in &loaded {
                        let _ = find_block_index(blocks, anc.as_deref(), Some(title));
                    }
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_toc_heading_lookup);
criterion_main!(benches);
