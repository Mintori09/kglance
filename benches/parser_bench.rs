use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::io::Write;
use std::path::PathBuf;

use image::GenericImageView;
use kglance::parsers::PreviewParser;

fn create_text_file(lines: usize) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.rs");
    let mut f = std::fs::File::create(&path).unwrap();
    for i in 0..lines {
        writeln!(f, "fn func_{i}() {{ let x = {i}; println!(\"{{x}}\"); }}").unwrap();
    }
    (dir, path)
}

fn create_image_file(width: u32, height: u32) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.png");
    let img = image::DynamicImage::new_rgba8(width, height);
    img.save(&path).unwrap();
    (dir, path)
}

fn create_svg_file(elements: usize) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.svg");
    let mut svg =
        String::from(r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">"#);
    for i in 0..elements {
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="10" height="10" fill="blue"/>"#,
            (i % 80) * 10,
            (i / 80) * 10,
        ));
    }
    svg.push_str("</svg>");
    std::fs::write(&path, &svg).unwrap();
    (dir, path)
}

fn create_zip_file(file_count: usize) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.zip");
    let f = std::fs::File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..file_count {
        zip.start_file(format!("file_{i}.txt"), opts).unwrap();
        zip.write_all(b"content").unwrap();
    }
    zip.finish().unwrap();
    (dir, path)
}

fn create_folder(item_count: usize) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("sample_dir");
    std::fs::create_dir(&folder).unwrap();
    for i in 0..item_count {
        std::fs::write(folder.join(format!("file_{i}.txt")), "data").unwrap();
    }
    (dir, folder)
}

fn create_markdown_file(lines: usize) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.md");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "# Sample Markdown\n").unwrap();
    for i in 0..lines {
        writeln!(f, "## Section {i}").unwrap();
        writeln!(f, "This is **bold** and *italic* text with `inline code`.").unwrap();
        writeln!(f, "- list item {i}a\n- list item {i}b").unwrap();
        writeln!(f, "```rust\nfn hello_{i}() {{ println!(\"hi\"); }}\n```\n").unwrap();
    }
    (dir, path)
}

fn create_pdf_file(page_count: usize) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.pdf");
    let mut buf = Vec::new();

    let total_objects = 3 + 2 * page_count;

    buf.extend_from_slice(b"%PDF-1.4\n");

    let mut offsets = vec![0u64; total_objects];

    // 1: Catalog
    offsets[0] = buf.len() as u64;
    buf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // 2: Pages
    offsets[1] = buf.len() as u64;
    let kids: Vec<String> = (0..page_count).map(|i| format!("{} 0 R", 3 + i)).collect();
    let _ = write!(
        buf,
        "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
        kids.join(" "),
        page_count
    );

    // Pages + Content streams
    let font_obj = 3 + page_count;
    for i in 0..page_count {
        let page_obj = 3 + i;
        let content_obj = 3 + page_count + 1 + i;
        offsets[page_obj - 1] = buf.len() as u64;
        let _ = write!(
            buf,
            "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> >>\nendobj\n",
            page_obj, content_obj, font_obj
        );
    }

    // Font
    offsets[font_obj - 1] = buf.len() as u64;
    let _ = write!(
        buf,
        "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
        font_obj
    );

    // Content streams
    for i in 0..page_count {
        let content_obj = 3 + page_count + 1 + i;
        let content = format!("BT /F1 24 Tf 100 700 Td (Page {}) Tj ET\n", i);
        offsets[content_obj - 1] = buf.len() as u64;
        let _ = write!(
            buf,
            "{} 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
            content_obj,
            content.len(),
            content
        );
    }

    // xref
    let xref_offset = buf.len() as u64;
    let _ = write!(buf, "xref\n0 {}\n", total_objects + 1);
    let _ = writeln!(buf, "{:010} 65535 f ", 0u64);
    for off in &offsets {
        let _ = writeln!(buf, "{:010} 00000 n ", off);
    }
    let _ = write!(
        buf,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        total_objects + 1,
        xref_offset
    );

    std::fs::write(&path, &buf).unwrap();
    (dir, path)
}

fn build_registry() -> kglance::parsers::ParserRegistry {
    let mut r = kglance::parsers::ParserRegistry::new();
    r.register(Box::new(kglance::parsers::markdown::MarkdownParser::new()));
    r.register(Box::new(kglance::parsers::text::TextParser::new()));
    r.register(Box::new(kglance::parsers::image::ImageParser));
    r.register(Box::new(kglance::parsers::svg::SvgParser));
    r.register(Box::new(kglance::parsers::pdf::PdfParser));
    r.register(Box::new(kglance::parsers::archive::ArchiveParser));
    r.register(Box::new(kglance::parsers::folder::FolderParser));
    r.register(Box::new(kglance::parsers::typst::TypstParser));
    r
}

fn bench_text_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/text");
    for lines in [100, 500, 1_000, 5_000, 10_000] {
        let (_dir, path) = create_text_file(lines);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{lines}_lines")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_image_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/image");
    for (w, h) in [(256, 256), (1920, 1080), (3840, 2160)] {
        let (_dir, path) = create_image_file(w, h);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{w}x{h}")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_svg_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/svg");
    for elements in [10, 100, 1_000] {
        let (_dir, path) = create_svg_file(elements);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{elements}_elements")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_markdown_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/markdown");
    for lines in [10, 100, 500] {
        let (_dir, path) = create_markdown_file(lines);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{lines}_lines")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_archive_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/archive");
    for count in [10, 100, 1_000] {
        let (_dir, path) = create_zip_file(count);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{count}_files")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_folder_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/folder");
    for count in [10, 50, 200] {
        let (_dir, path) = create_folder(count);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{count}_items")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_pdf_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser/pdf");
    for count in [1, 5, 25] {
        let (_dir, path) = create_pdf_file(count);
        let registry = build_registry();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{count}_pages")),
            &path,
            |b, path| b.iter(|| registry.parse(path).unwrap()),
        );
    }
    group.finish();
}

fn bench_registry_dispatch(c: &mut Criterion) {
    let (_dir, path) = create_text_file(100);
    let registry = build_registry();
    let direct_parser = kglance::parsers::text::TextParser::new();

    let mut group = c.benchmark_group("dispatch_overhead");
    group.bench_function("via_registry", |b| {
        b.iter(|| registry.parse(&path).unwrap());
    });
    group.bench_function("direct_parser", |b| {
        b.iter(|| direct_parser.parse(&path).unwrap());
    });
    group.finish();
}

// ── Image operations benchmarks ─────────────────────────────────────

fn create_image_rgba(width: u32, height: u32) -> image::DynamicImage {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            buf[i] = ((x ^ y) & 0xff) as u8;
            buf[i + 1] = ((x * 3 + y * 5) & 0xff) as u8;
            buf[i + 2] = ((x * 7 + y * 11) & 0xff) as u8;
            buf[i + 3] = 255;
        }
    }
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(width, height, buf).unwrap())
}

fn create_image_file_png_bytes(width: u32, height: u32) -> Vec<u8> {
    let img = create_image_rgba(width, height);
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

fn create_image_file_jpeg_bytes(width: u32, height: u32) -> Vec<u8> {
    let img = create_image_rgba(width, height);
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
    buf.into_inner()
}

fn bench_image_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("image/decode");

    for (w, h, label) in [
        (256, 256, "256x256"),
        (1920, 1080, "1920x1080"),
        (3840, 2160, "4K"),
    ] {
        let png_bytes = create_image_file_png_bytes(w, h);
        group.bench_with_input(BenchmarkId::new("png", label), &png_bytes, |b, data| {
            b.iter(|| image::load_from_memory(data).unwrap())
        });

        let jpeg_bytes = create_image_file_jpeg_bytes(w, h);
        group.bench_with_input(BenchmarkId::new("jpeg", label), &jpeg_bytes, |b, data| {
            b.iter(|| image::load_from_memory(data).unwrap())
        });
    }
    group.finish();
}

fn bench_image_zoom(c: &mut Criterion) {
    let mut group = c.benchmark_group("image/zoom");

    for (w, h, label) in [(1920, 1080, "1920x1080"), (3840, 2160, "4K")] {
        let img = create_image_rgba(w, h);

        for (factor, zoom_label) in [(1.0, "100pct"), (2.0, "200pct"), (4.0, "400pct")] {
            group.bench_with_input(
                BenchmarkId::new(format!("lanczos3_{label}"), zoom_label),
                &(img.clone(), factor),
                |b, (im, f)| {
                    b.iter(|| {
                        let (iw, ih) = im.dimensions();
                        let nw = (iw as f32 * f).max(1.0) as u32;
                        let nh = (ih as f32 * f).max(1.0) as u32;
                        im.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3)
                    })
                },
            );
        }
    }
    group.finish();
}

fn bench_image_rotate(c: &mut Criterion) {
    let mut group = c.benchmark_group("image/rotate");

    let img = create_image_rgba(3840, 2160);

    group.bench_function("rotate90_4K", |b| {
        b.iter(|| {
            let mut p = img.clone();
            p = p.rotate90();
            p = p.rotate90();
            p
        })
    });

    group.bench_function("rotate180_4K", |b| {
        b.iter(|| {
            let mut p = img.clone();
            for _ in 0..2 {
                p = p.rotate90();
            }
            p
        })
    });

    let small = create_image_rgba(1920, 1080);
    group.bench_function("rotate90_FHD", |b| b.iter(|| small.clone().rotate90()));

    group.finish();
}

fn bench_image_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("image/pipeline");

    let sizes: &[(u32, u32, &str)] = &[(1920, 1080, "FHD"), (3840, 2160, "4K")];
    for (w, h, label) in sizes {
        // Full pipeline: decode -> rotate -> resize(Lanczos3) -> to_rgba8 -> pixel copy
        let png_bytes = create_image_file_png_bytes(*w, *h);

        group.bench_function(format!("full_{label}"), |b| {
            b.iter(|| {
                let img = image::load_from_memory(&png_bytes).unwrap();
                let mut processed = img;
                for _ in 0..1 {
                    processed = processed.rotate90();
                }
                let (pw, ph) = processed.dimensions();
                let nw = (pw as f32 * 0.75).max(1.0) as u32;
                let nh = (ph as f32 * 0.75).max(1.0) as u32;
                let scaled = processed.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
                let rgba = scaled.to_rgba8();
                let raw = rgba.into_raw();
                let data_len = raw.len();
                // Simulate pixel copy into SharedPixelBuffer
                let mut copy = vec![0u8; data_len];
                for (i, p) in copy.chunks_exact_mut(4).enumerate() {
                    let offset = i * 4;
                    p[0] = raw[offset];
                    p[1] = raw[offset + 1];
                    p[2] = raw[offset + 2];
                    p[3] = raw[offset + 3];
                }
                std::hint::black_box(copy);
            })
        });
    }
    group.finish();
}

fn bench_image_pixel_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("image/pixel_copy");

    for (w, h, label) in [
        (256, 256, "256x256"),
        (1920, 1080, "FHD"),
        (3840, 2160, "4K"),
    ] {
        let img = create_image_rgba(w, h);
        let rgba = img.to_rgba8();
        let raw = rgba.into_raw();

        // Current slow path: byte-by-byte via enumerate
        group.bench_function(format!("bytewise_{label}"), |b| {
            b.iter(|| {
                let len = raw.len();
                let mut copy = vec![0u8; len];
                for (i, chunk) in copy.chunks_exact_mut(4).enumerate() {
                    let offset = i * 4;
                    chunk[0] = raw[offset];
                    chunk[1] = raw[offset + 1];
                    chunk[2] = raw[offset + 2];
                    chunk[3] = raw[offset + 3];
                }
                std::hint::black_box(copy);
            })
        });

        // Fast path: copy_from_slice
        group.bench_function(format!("memcpy_{label}"), |b| {
            b.iter(|| {
                let mut copy = vec![0u8; raw.len()];
                copy.copy_from_slice(&raw);
                std::hint::black_box(copy);
            })
        });
    }
    group.finish();
}

fn bench_image_exif(c: &mut Criterion) {
    use std::io::Read;

    let mut group = c.benchmark_group("image/exif");
    let exif_bytes = create_minimal_jpeg_with_exif();

    group.bench_function("extract", |b| {
        b.iter(|| {
            let mut file = std::io::Cursor::new(&exif_bytes);
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            let reader = exif::Reader::new();
            let _ = reader.read_raw(buf).ok();
        })
    });
    group.finish();
}

fn create_minimal_jpeg_with_exif() -> Vec<u8> {
    // Build a minimal JPEG with EXIF using APP1 marker
    // JPEG SOI + APP1 (EXIF) + minimal image data + EOI
    let mut jpeg = Vec::new();

    // SOI
    jpeg.extend_from_slice(b"\xff\xd8");

    // APP1 marker + EXIF header
    let make = b"KGLANCE";
    let model = b"BenchCam";

    let mut ifd_data = Vec::new();
    // IFD0 entries: Make (0x010F), Model (0x0110)
    // Tag(2) Type(2) Count(4) Offset/Data(4)
    // Make tag
    ifd_data.extend_from_slice(&0x010fu16.to_be_bytes()); // tag
    ifd_data.extend_from_slice(&2u16.to_be_bytes()); // ASCII
    ifd_data.extend_from_slice(&(make.len() as u32).to_be_bytes());
    // Write string at end, store offset placeholder
    let make_offset: u32 = 8 + 2 * 12 + 4; // after IFD entries + next IFD pointer
    ifd_data.extend_from_slice(&make_offset.to_be_bytes());

    // Model tag
    ifd_data.extend_from_slice(&0x0110u16.to_be_bytes());
    ifd_data.extend_from_slice(&2u16.to_be_bytes());
    ifd_data.extend_from_slice(&(model.len() as u32).to_be_bytes());
    let model_offset = make_offset + make.len() as u32;
    ifd_data.extend_from_slice(&model_offset.to_be_bytes());

    // Next IFD pointer = 0 (no IFD1)
    ifd_data.extend_from_slice(&0u32.to_be_bytes());

    // String data
    ifd_data.extend_from_slice(make);
    ifd_data.extend_from_slice(model);

    let entry_count = 2u16;
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II"); // little-endian
    tiff.extend_from_slice(&0x002au16.to_be_bytes()); // TIFF magic
    tiff.extend_from_slice(&8u32.to_be_bytes()); // offset to IFD0
    tiff.extend_from_slice(&entry_count.to_be_bytes());
    tiff.extend_from_slice(&ifd_data);

    let mut app1_data = Vec::new();
    app1_data.extend_from_slice(b"Exif\x00\x00");
    app1_data.extend_from_slice(&tiff);

    let app1_len = app1_data.len() as u16 + 2;
    jpeg.extend_from_slice(b"\xff\xe1");
    jpeg.extend_from_slice(&app1_len.to_be_bytes());
    jpeg.extend_from_slice(&app1_data);

    // SOS + minimal scan data
    jpeg.extend_from_slice(b"\xff\xda\x00\x08\x01\x01\x00\x00?\x00\xff\xd9");

    jpeg
}

criterion_group!(
    benches,
    bench_text_parser,
    bench_image_parser,
    bench_svg_parser,
    bench_markdown_parser,
    bench_pdf_parser,
    bench_archive_parser,
    bench_folder_parser,
    bench_registry_dispatch,
    bench_image_decode,
    bench_image_zoom,
    bench_image_rotate,
    bench_image_pipeline,
    bench_image_pixel_copy,
    bench_image_exif,
);

criterion_main!(benches);
