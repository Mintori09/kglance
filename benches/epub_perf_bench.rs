use std::fs::File;
use std::hint::black_box;
use std::io::Write;
use std::path::{Path, PathBuf};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use kglance::core::types::EpubChapterInfo;
use kglance::features::common::parser::html::convert_html_to_markdown;
use kglance::features::common::parser::traits::PreviewParser;
use kglance::features::common::parser::types::ParsedContent;
use kglance::features::epub::parser::{EpubParser, load_chapter_content_and_images_from_epub};
use kglance::features::epub::state::build_continuous_blocks;
use kglance::parsers::markdown::{Block, Inline, parse_to_blocks};

/// Synthetic EPUB generator to guarantee deterministic, isolated benchmark without relying on external files
fn generate_synthetic_epub(path: &Path, num_chapters: usize, paragraphs_per_ch: usize) {
    let file = File::create(path).expect("Failed to create synthetic epub file");
    let mut zip = zip::ZipWriter::new(file);

    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // 1. mimetype (must be uncompressed stored)
    zip.start_file("mimetype", options).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();

    let deflated = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 2. META-INF/container.xml
    zip.start_file("META-INF/container.xml", deflated).unwrap();
    zip.write_all(
        br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )
    .unwrap();

    // 3. OEBPS/style.css
    zip.start_file("OEBPS/style.css", deflated).unwrap();
    zip.write_all(
        br#"
body { font-family: serif; }
h1 { font-size: 2em; text-align: center; }
p { margin: 1em 0; text-indent: 1.5em; line-height: 1.6; }
.highlight { font-weight: bold; color: #ff0000; }
"#,
    )
    .unwrap();

    // 4. Generate Chapter XHTML files
    let mut manifest_items = String::new();
    let mut spine_items = String::new();
    let mut ncx_navpoints = String::new();

    manifest_items.push_str(r#"<item id="css" href="style.css" media-type="text/css"/>"#);
    manifest_items.push('\n');
    manifest_items
        .push_str(r#"<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#);
    manifest_items.push('\n');

    // Add 1 test image (small 1x1 png)
    let png_1x1: [u8; 67] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    zip.start_file("OEBPS/cover.png", deflated).unwrap();
    zip.write_all(&png_1x1).unwrap();
    manifest_items.push_str(r#"<item id="cover" href="cover.png" media-type="image/png"/>"#);
    manifest_items.push('\n');

    for ch in 0..num_chapters {
        let filename = format!("chapter_{ch}.xhtml");
        let id = format!("ch_{ch}");
        manifest_items.push_str(&format!(
            r#"<item id="{id}" href="{filename}" media-type="application/xhtml+xml"/>"#
        ));
        manifest_items.push('\n');
        spine_items.push_str(&format!(r#"<itemref idref="{id}"/>"#));
        spine_items.push('\n');

        ncx_navpoints.push_str(&format!(
            r#"<navPoint id="nav_{ch}" playOrder="{ch}">
                <navLabel><text>Chapter {ch}</text></navLabel>
                <content src="{filename}"/>
            </navPoint>"#
        ));

        zip.start_file(format!("OEBPS/{filename}"), deflated)
            .unwrap();
        let mut ch_content = String::new();
        ch_content.push_str(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Chapter</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
"#,
        );
        ch_content.push_str(&format!(
            "<h1>Chapter {ch}: Benchmarking Architecture</h1>\n"
        ));
        ch_content
            .push_str(r#"<div class="image-wrapper"><img src="cover.png" alt="Cover" /></div>"#);
        ch_content.push('\n');

        for p in 0..paragraphs_per_ch {
            ch_content.push_str(&format!(
                r#"<p id="p-{p}" class="highlight">Paragraph {p}: High-performance file preview system for KDE Plasma 6 using a Client-Daemon architecture, DBus zbus, and the Iced UI toolkit. Rust provides memory safety and zero-cost abstractions.</p>"#
            ));
            ch_content.push('\n');
        }

        ch_content.push_str("</body>\n</html>");
        zip.write_all(ch_content.as_bytes()).unwrap();
    }

    // 5. OEBPS/toc.ncx
    zip.start_file("OEBPS/toc.ncx", deflated).unwrap();
    let ncx_content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
    <head><meta name="dtb:uid" content="urn:uuid:bench"/></head>
    <docTitle><text>Benchmark EPUB</text></docTitle>
    <navMap>
        {ncx_navpoints}
    </navMap>
</ncx>"#
    );
    zip.write_all(ncx_content.as_bytes()).unwrap();

    // 6. OEBPS/content.opf
    zip.start_file("OEBPS/content.opf", deflated).unwrap();
    let opf_content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="BookId">
    <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
        <dc:title>Performance Benchmark Synthetic EPUB</dc:title>
        <dc:creator>Kglance Benchmark Suite</dc:creator>
        <meta name="cover" content="cover"/>
    </metadata>
    <manifest>
        {manifest_items}
    </manifest>
    <spine toc="ncx">
        {spine_items}
    </spine>
</package>"#
    );
    zip.write_all(opf_content.as_bytes()).unwrap();

    zip.finish().unwrap();
}

fn get_or_create_test_epubs() -> (PathBuf, PathBuf) {
    let tmp_dir = std::env::temp_dir().join("kglance_bench_epubs");
    std::fs::create_dir_all(&tmp_dir).unwrap();

    let small_epub = tmp_dir.join("small_book.epub");
    let large_epub = tmp_dir.join("large_book.epub");

    if !small_epub.exists() {
        // 10 chapters, 20 paragraphs each (~200 blocks)
        generate_synthetic_epub(&small_epub, 10, 20);
    }
    if !large_epub.exists() {
        // 50 chapters, 50 paragraphs each (~2,500 blocks)
        generate_synthetic_epub(&large_epub, 50, 50);
    }

    (small_epub, large_epub)
}

/// 1. Benchmark: First Open (Metadata + Initial Chapter Parse)
fn bench_epub_first_open(c: &mut Criterion) {
    let (small_path, large_path) = get_or_create_test_epubs();
    let mut group = c.benchmark_group("epub_first_open");

    for (name, path) in [("small_10ch", &small_path), ("large_50ch", &large_path)] {
        group.bench_with_input(BenchmarkId::new("EpubParser_parse", name), path, |b, p| {
            let parser = EpubParser;
            b.iter(|| {
                let parsed = parser.parse(black_box(p)).expect("Parse failed");
                match parsed {
                    ParsedContent::Epub {
                        chapters, title, ..
                    } => {
                        assert!(!chapters.is_empty());
                        black_box((chapters, title))
                    }
                    _ => panic!("Expected ParsedContent::Epub"),
                }
            });
        });
    }
    group.finish();
}

/// 2. Benchmark: Chapter Switch Latency (Single Chapter load with Archive re-open)
fn bench_epub_chapter_switch(c: &mut Criterion) {
    let (small_path, large_path) = get_or_create_test_epubs();
    let mut group = c.benchmark_group("epub_chapter_switch");

    for (name, path) in [("small_10ch", &small_path), ("large_50ch", &large_path)] {
        group.bench_with_input(
            BenchmarkId::new("load_chapter_single", name),
            path,
            |b, p| {
                b.iter(|| {
                    let res = load_chapter_content_and_images_from_epub(
                        black_box(p),
                        black_box("chapter_5.xhtml"),
                        None,
                    )
                    .expect("Failed to load chapter content");
                    black_box(res);
                });
            },
        );
    }
    group.finish();
}

/// 3. Benchmark: HTML to Markdown string conversion vs Markdown string to AST blocks
fn bench_epub_pipeline_stages(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_pipeline_stages");

    let sample_html = r#"
    <div class="chapter-body">
        <h1>Chapter Title: Deep Architecture Exploration</h1>
        <p class="highlight">Paragraph 1: In high-performance software engineering, eliminating intermediate string allocations is crucial.</p>
        <p>Paragraph 2: Notice how Markdown AST elements require recursive parsing over raw syntax trees.</p>
        <ul>
            <li>Item 1: Zero-cost abstractions</li>
            <li>Item 2: Cache-coherent data layout</li>
            <li>Item 3: Elimination of duplicate hashing</li>
        </ul>
        <p>Final paragraph with <strong>bold text</strong> and <em>italic styling</em>.</p>
    </div>
    "#
    .repeat(20); // ~20KB HTML snippet

    group.bench_function("stage1_html_to_markdown_string", |b| {
        b.iter(|| {
            let md = convert_html_to_markdown(black_box(&sample_html));
            black_box(md)
        });
    });

    let sample_md = convert_html_to_markdown(&sample_html);
    group.bench_function("stage2_markdown_string_to_blocks", |b| {
        b.iter(|| {
            let blocks = parse_to_blocks(black_box(&sample_md));
            black_box(blocks)
        });
    });

    group.bench_function("combined_html_to_md_to_blocks", |b| {
        b.iter(|| {
            let md = convert_html_to_markdown(black_box(&sample_html));
            let blocks = parse_to_blocks(black_box(&md));
            black_box(blocks)
        });
    });

    group.finish();
}

/// 4. Benchmark: Continuous Blocks Construction (Deep clone overhead)
fn bench_epub_continuous_blocks_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("epub_continuous_clone");

    let mut chapters = Vec::with_capacity(50);
    for ch in 0..50 {
        let mut blocks = Vec::with_capacity(30);
        blocks.push(Block::Heading {
            level: 1,
            content: vec![Inline::Text(format!("Chapter {ch} Title"))],
        });
        for p in 0..29 {
            blocks.push(Block::Paragraph(vec![
                Inline::Text(format!("Content paragraph {p} of chapter {ch}. ")),
                Inline::Bold(vec![Inline::Text("Bold summary statement.".to_string())]),
            ]));
        }
        chapters.push(EpubChapterInfo {
            title: format!("Chapter {ch}"),
            level: 1,
            anchor: None,
            file_href: format!("chapter_{ch}.xhtml"),
            blocks,
        });
    }

    group.bench_function("build_continuous_blocks_deep_clone_50ch", |b| {
        b.iter(|| {
            let (all_blocks, offsets) = build_continuous_blocks(black_box(&chapters));
            black_box((all_blocks, offsets))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_epub_first_open,
    bench_epub_chapter_switch,
    bench_epub_pipeline_stages,
    bench_epub_continuous_blocks_clone
);
criterion_main!(benches);
