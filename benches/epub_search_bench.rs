use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kglance::core::types::EpubChapterInfo;
use kglance::features::common::parser::traits::PreviewParser;
use kglance::features::common::parser::types::ParsedContent;
use kglance::features::epub::parser::{EpubParser, load_chapter_from_epub};
use kglance::features::epub::search::{block_to_searchable_text, search_epub_chapters};
use kglance::parsers::markdown::{Block, Inline};
use std::hint::black_box;
use std::path::{Path, PathBuf};

fn find_sample_epubs() -> Vec<PathBuf> {
    let candidate_paths = [
        "/home/mintori/Downloads/Telegram Desktop/Automate_the_Boring_Stuff_with_Python,_3rd_Edition_Al_Sweigart_3.epub",
        "/home/mintori/Downloads/Telegram Desktop/Atomic_Habits_Thay_Đổi_Tí_Hon,_Hiệu_Quả_Bất_Ngờ_James.epub",
        "/home/mintori/Downloads/Tư Duy Chiến Lược - Avinash K. Dixit & Bary J. Nalebuff.epub",
        "/home/mintori/Downloads/Telegram Desktop/Secure_Coding_in_TypeScript_Best_Practices_and_Baldurs_L_2025_1 (3).epub",
    ];

    candidate_paths
        .iter()
        .map(Path::new)
        .filter(|p| p.exists())
        .map(Path::to_path_buf)
        .collect()
}

fn create_synthetic_chapters(
    num_chapters: usize,
    blocks_per_chapter: usize,
) -> Vec<EpubChapterInfo> {
    let mut chapters = Vec::with_capacity(num_chapters);

    for ch in 0..num_chapters {
        let mut blocks = Vec::with_capacity(blocks_per_chapter);
        blocks.push(Block::Heading {
            level: 1,
            content: vec![Inline::Text(format!(
                "Chapter {ch}: Deep Architecture and Design"
            ))],
        });

        for b in 1..blocks_per_chapter {
            match b % 4 {
                0 => {
                    blocks.push(Block::Heading {
                        level: 2,
                        content: vec![Inline::Text(format!(
                            "Section {ch}.{b}: Type Inference and Generic Parameters"
                        ))],
                    });
                }
                1 => {
                    blocks.push(Block::Paragraph(vec![
                        Inline::Text("TypeScript provides a static type system for JavaScript. When writing high performance code, understanding variance and covariance is essential. ".repeat(4)),
                        Inline::Bold(vec![Inline::Text("Performance Tip: avoid excessive nested conditional types.".to_string())]),
                    ]));
                }
                2 => {
                    blocks.push(Block::CodeBlock {
                        lang: Some("rust".to_string()),
                        title: None,
                        code: "pub fn compute_hash<T: Hash>(val: &T) -> u64 {\n    let mut s = DefaultHasher::new();\n    val.hash(&mut s);\n    s.finish()\n}\n".to_string(),
                    });
                }
                _ => {
                    blocks.push(Block::Paragraph(vec![
                        Inline::Text("Cuốn sách này hướng dẫn chi tiết cách thay đổi thói quen tí hon nhằm mang lại hiệu quả bất ngờ trong cuộc sống hàng ngày. ".repeat(3)),
                    ]));
                }
            }
        }

        chapters.push(EpubChapterInfo {
            title: format!("Chapter {ch}"),
            level: 1,
            anchor: None,
            file_href: format!("ch_{ch}.xhtml"),
            blocks,
        });
    }

    chapters
}

fn bench_block_to_searchable_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_block_to_text");

    let para_block = Block::Paragraph(vec![
        Inline::Text("This is regular plain text with ".into()),
        Inline::Bold(vec![Inline::Text("bold text inside".into())]),
        Inline::Text(" and a ".into()),
        Inline::Link {
            text: vec![Inline::Text("clickable link".into())],
            url: "https://example.com".into(),
        },
        Inline::Code("inline_code()".into()),
    ]);

    group.bench_function("paragraph_flatten", |b| {
        b.iter(|| {
            black_box(block_to_searchable_text(&para_block));
        });
    });

    let code_block = Block::CodeBlock {
        lang: Some("rust".into()),
        title: None,
        code: "fn main() { println!(\"Hello Benchmark!\"); }".repeat(50),
    };

    group.bench_function("code_block_extract", |b| {
        b.iter(|| {
            black_box(block_to_searchable_text(&code_block));
        });
    });

    group.finish();
}

fn bench_search_query_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_search_query_scaling");
    let synthetic_book = create_synthetic_chapters(30, 100); // 3,000 blocks

    // High frequency common term (hits MAX_SEARCH_RESULTS limit)
    group.bench_with_input(
        BenchmarkId::new("high_frequency_capped", "type"),
        &synthetic_book,
        |b, chapters| {
            b.iter(|| {
                let matches = search_epub_chapters(chapters, "type");
                black_box(matches);
            });
        },
    );

    // Medium frequency phrase
    group.bench_with_input(
        BenchmarkId::new("medium_frequency", "variance and covariance"),
        &synthetic_book,
        |b, chapters| {
            b.iter(|| {
                let matches = search_epub_chapters(chapters, "variance and covariance");
                black_box(matches);
            });
        },
    );

    // Low frequency technical symbol
    group.bench_with_input(
        BenchmarkId::new("low_frequency_code", "compute_hash"),
        &synthetic_book,
        |b, chapters| {
            b.iter(|| {
                let matches = search_epub_chapters(chapters, "compute_hash");
                black_box(matches);
            });
        },
    );

    // Unicode / Vietnamese diacritics search
    group.bench_with_input(
        BenchmarkId::new("unicode_vietnamese", "thay đổi thói quen"),
        &synthetic_book,
        |b, chapters| {
            b.iter(|| {
                let matches = search_epub_chapters(chapters, "thay đổi thói quen");
                black_box(matches);
            });
        },
    );

    // No match found across all chapters
    group.bench_with_input(
        BenchmarkId::new("no_match", "nonexistent_cryptic_term_xyz"),
        &synthetic_book,
        |b, chapters| {
            b.iter(|| {
                let matches = search_epub_chapters(chapters, "nonexistent_cryptic_term_xyz");
                black_box(matches);
            });
        },
    );

    group.finish();
}

fn bench_real_epub_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_search_real_books");
    let sample_paths = find_sample_epubs();

    for path in sample_paths {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let parser = EpubParser;
        let chapters = match parser.parse(&path) {
            Ok(ParsedContent::Epub { chapters, .. }) => chapters,
            _ => continue,
        };

        let mut loaded_chapters = Vec::new();
        for (title, level, anchor, file_href, initial_blocks) in chapters.into_iter().take(30) {
            let blocks = if !initial_blocks.is_empty() {
                initial_blocks
            } else {
                load_chapter_from_epub(&path, &file_href, anchor.as_deref()).unwrap_or_default()
            };
            loaded_chapters.push(EpubChapterInfo {
                title,
                level,
                anchor,
                file_href,
                blocks,
            });
        }

        let short_name = if file_name.len() > 30 {
            &file_name[..30]
        } else {
            &file_name
        };

        group.bench_with_input(
            BenchmarkId::new("common_word", short_name),
            &loaded_chapters,
            |b, chs| {
                b.iter(|| {
                    let matches = search_epub_chapters(chs, "the");
                    black_box(matches);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("chapter_word", short_name),
            &loaded_chapters,
            |b, chs| {
                b.iter(|| {
                    let matches = search_epub_chapters(chs, "chapter");
                    black_box(chs);
                    black_box(matches);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_block_to_searchable_text,
    bench_search_query_scaling,
    bench_real_epub_search,
);
criterion_main!(benches);
