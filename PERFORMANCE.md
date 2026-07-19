# Performance Testing — Kglance

> **Mục tiêu cứng:** Mọi thao tác preview phải hoàn thành trong **< 100 ms** tính từ lúc người dùng bấm phím đến lúc cửa sổ hiển thị nội dung.

---

## 1. Phân rã ngân sách thời gian (Time Budget)

Luồng xử lý khi người dùng bấm Space trong Dolphin (Daemon mode):

```
[Space] → KIO Service Menu → busctl call → DBus transport → Parser → UI update → Window show
```

| Giai đoạn | Ngân sách | Ghi chú |
| --- | --: | --- |
| KIO dispatch + busctl | 15 ms | Chi phí cố định, nằm ngoài tầm kiểm soát |
| DBus transport (zbus) | 5 ms | IPC qua Unix socket, gần như zero-copy |
| Parser (đọc + xử lý) | 50 ms | **Phần lớn nhất** — thay đổi theo loại file |
| UI update (Slint) | 20 ms | Set properties + render frame đầu tiên |
| Window show | 10 ms | Wayland compositor latency |
| **Tổng** | **100 ms** |  |

Standalone mode bỏ qua bước DBus (~20 ms), nhưng phải khởi tạo Slint window từ đầu (~30 ms), nên tổng tương đương.

---

## 2. Benchmark Rust (cargo bench)

### 2.1 Thiết lập

Thêm `criterion` vào `Cargo.toml`:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3"

[[bench]]
name = "parser_bench"
harness = false
```

### 2.2 File benchmark: `benches/parser_bench.rs`

````rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::io::Write;
use std::path::PathBuf;

// ── Helpers tạo file test ──────────────────────────────────────────

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
    let mut svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">"#,
    );
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

// ── Hàm build registry giống main.rs ───────────────────────────────

fn build_registry() -> Kglance::parser::ParserRegistry {
    let mut r = Kglance::parser::ParserRegistry::new();
    r.register(Box::new(Kglance::parser::markdown::MarkdownParser::new()));
    r.register(Box::new(Kglance::parser::text::TextParser::new()));
    r.register(Box::new(Kglance::parser::image::ImageParser));
    r.register(Box::new(Kglance::parser::svg::SvgParser));
    r.register(Box::new(Kglance::parser::pdf::PdfParser));
    r.register(Box::new(Kglance::parser::archive::ArchiveParser));
    r.register(Box::new(Kglance::parser::folder::FolderParser));
    r
}

// ── Benchmark groups ───────────────────────────────────────────────

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

fn bench_registry_dispatch(c: &mut Criterion) {
    // Đo overhead của ParserRegistry::parse (chọn parser + dispatch)
    // so với gọi parser trực tiếp
    let (_dir, path) = create_text_file(100);
    let registry = build_registry();
    let direct_parser = Kglance::parser::text::TextParser::new();

    let mut group = c.benchmark_group("dispatch_overhead");
    group.bench_function("via_registry", |b| {
        b.iter(|| registry.parse(&path).unwrap());
    });
    group.bench_function("direct_parser", |b| {
        b.iter(|| direct_parser.parse(&path).unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_text_parser,
    bench_image_parser,
    bench_svg_parser,
    bench_markdown_parser,
    bench_archive_parser,
    bench_folder_parser,
    bench_registry_dispatch,
);
criterion_main!(benches);
````

### 2.3 Chạy benchmark

```bash
# Chạy toàn bộ benchmark
cargo bench

# Chạy một group cụ thể
cargo bench -- parser/text
cargo bench -- parser/image

# Xem báo cáo HTML (tự động mở trình duyệt)
open target/criterion/report/index.html
```

---

## 2.4 Image Operations Benchmarks (Phase 3)

### 2.4.1 Benchmark groups mới

Thêm vào `benches/parser_bench.rs`:

```rust
// Gọi các hàm helper có sẵn: create_image_rgba(), create_image_file_png_bytes()

fn bench_image_decode(c: &mut Criterion)           // image/decode
fn bench_image_zoom(c: &mut Criterion)             // image/zoom
fn bench_image_rotate(c: &mut Criterion)           // image/rotate
fn bench_image_pipeline(c: &mut Criterion)         // image/pipeline
fn bench_image_pixel_copy(c: &mut Criterion)       // image/pixel_copy
fn bench_image_exif(c: &mut Criterion)             // image/exif
```

### 2.4.2 Ngưỡng mục tiêu (image operations)

| Benchmark | Input | Ngưỡng (ms) | Ghi chú |
| --- | --- | --: | --- |
| **image/decode/png** | 1920×1080 | 15 | `image::load_from_memory` |
| **image/decode/png** | 3840×2160 (4K) | 30 | Decode 4K PNG |
| **image/decode/jpeg** | 1920×1080 | 10 | JPEG decode nhanh hơn PNG |
| **image/decode/jpeg** | 3840×2160 | 20 | |
| **image/zoom/lanczos3** | 1920×1080 → 200% | 80 | **Bottleneck #1** — Lanczos3 rất chậm |
| **image/zoom/lanczos3** | 1920×1080 → 400% | 200 | Zoom lớn = resize ảnh lớn |
| **image/zoom/lanczos3** | 4K → 100% | 150 | 4K gốc không cần resize, chỉ clone |
| **image/zoom/lanczos3** | 4K → 200% | 350 | **Rất chậm** — cần downscale trước |
| **image/rotate/rotate90** | 4K (1 lần) | 5 | `DynamicImage::rotate90()` |
| **image/rotate/rotate90** | 4K (2 lần) | 10 | Xoay 180° |
| **image/pipeline/full** | FHD decode+rotate+resize | 60 | Pipeline đầy đủ (75% zoom) |
| **image/pipeline/full** | 4K decode+rotate+resize | 120 | Pipeline đầy đủ |
| **image/pixel_copy/bytewise** | 1920×1080 | 8 | **Bottleneck #2** — copy từng pixel qua Rust loop |
| **image/pixel_copy/bytewise** | 4K | 35 | 33 triệu pixel → loop chậm |
| **image/pixel_copy/memcpy** | 1920×1080 | 1 | `copy_from_slice` nhanh hơn ~8 lần |
| **image/pixel_copy/memcpy** | 4K | 4 | |
| **image/exif/extract** | JPEG có EXIF header | 0.5 | EXIF parse rất nhanh |

### 2.4.3 Bottleneck chi tiết

| Vị trí | Chi phí (FHD) | Chi phí (4K) | Nguyên nhân | Giải pháp |
| --- | --: | --: | --- | --- |
| `image::open` → `write_to(PNG)` trong parser | 10-30ms | 20-50ms | Decode rồi re-encode PNG trước khi gửi lên UI | Truyền raw `Vec<u8>` RGBA qua `ParsedContent::ImageRaw` |
| `image::load_from_memory` trong `show()` | 5-15ms | 15-30ms | Giải mã lại PNG từ parser | Bỏ nếu UI nhận raw RGBA |
| `DynamicImage::clone()` | 0.5ms | 2ms | Clone toàn bộ ảnh gốc mỗi lần zoom/rotate | Dùng `Arc<DynamicImage>` thay `Option<DynamicImage>` |
| `resize_exact(Lanczos3)` | 20-80ms | 80-350ms | **Chậm nhất** — Lanczos3 convolution | • Dùng `FilterType::CatmullRom` hoặc `Triangle` cho realtime<br>• Giới hạn zoom ≤ 2x<br>• Cache kết quả resize |
| `rotate90()` | 2-5ms | 5-10ms | Xoay pixel toàn bộ ảnh | Cache rotation state, chỉ rotate khi cần render |
| `to_rgba8()` → `into_raw()` | 2-5ms | 10-20ms | Chuyển đổi pixel buffer | Có thể tối ưu với `as_bytes()` hoặc raw buffer |
| Pixel copy loop (bytewise) | 8ms | 35ms | `for (i, pixel) in pixel_slice.iter_mut().enumerate()` | Thay bằng `copy_from_slice` trên toàn bộ slice |

### 2.4.4 Tối ưu ưu tiên

1. **Thay `copy_from_slice` cho pixel copy** — Tiết kiệm ~8ms (FHD) / ~35ms (4K). Dùng `buffer.make_mut_slice().copy_from_slice(&raw)` thay vì vòng lặp từng pixel.

2. **Giảm chất lượng resize realtime** — Chuyển từ `Lanczos3` sang `CatmullRom` hoặc `Triangle` khi zoom > 1.5×. Tiết kiệm 40-60% thời gian resize.

3. **Bỏ re-encode PNG trong parser** — Gửi raw RGBA thay vì PNG, tiết kiệm 10-30ms ở parser + 5-15ms ở UI decode lại.

4. **Dùng `Arc<DynamicImage>`** — Tránh clone toàn bộ ảnh mỗi lần zoom. Chỉ clone khi cần rotate (ảnh gốc immutable).

5. **Cache resize kết quả** — Nếu zoom với cùng factor và rotation, dùng ảnh đã resize từ lần trước.

---

## 3. Đo End-to-End (shell script)

### 3.1 Đo thời gian Standalone (toàn bộ pipeline trừ Slint render)

Tạo file `scripts/bench_e2e.sh`:

```bash
#!/usr/bin/env bash
# Đo thời gian end-to-end cho Kglance standalone mode.
# Sử dụng: ./scripts/bench_e2e.sh [số lần lặp]
set -euo pipefail

BINARY="${BINARY:-target/release/kglance}"
ITERATIONS="${1:-10}"
PASS_THRESHOLD_MS=100

if [[ ! -x "$BINARY" ]]; then
    echo "Binary not found. Run: cargo build --release"
    exit 1
fi

# Tạo file test
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Text file (1000 dòng Rust)
python3 -c "
for i in range(1000):
    print(f'fn func_{i}() {{ let x = {i}; }}')
" > "$TMPDIR/sample.rs"

# Image file (1920x1080 PNG)
python3 -c "
from struct import pack
import zlib

width, height = 1920, 1080
raw = b''
for y in range(height):
    raw += b'\x00' + b'\xff\x00\x00\xff' * width

def chunk(ctype, data):
    c = ctype + data
    return pack('>I', len(data)) + c + pack('>I', zlib.crc32(c) & 0xffffffff)

ihdr = pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
png = b'\x89PNG\r\n\x1a\n'
png += chunk(b'IHDR', ihdr)
png += chunk(b'IDAT', zlib.compress(raw))
png += chunk(b'IEND', b'')
with open('$TMPDIR/sample.png', 'wb') as f:
    f.write(png)
" 2>/dev/null || convert -size 1920x1080 xc:red "$TMPDIR/sample.png" 2>/dev/null || echo "skip image bench"

# SVG file
cat > "$TMPDIR/sample.svg" << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="800" height="600">
  <rect width="800" height="600" fill="#1a1a2e"/>
  <circle cx="400" cy="300" r="200" fill="#e94560"/>
</svg>
EOF

# ZIP file
cd "$TMPDIR" && zip -q sample.zip sample.rs sample.svg 2>/dev/null && cd - > /dev/null

echo "╔══════════════════════════════════════════════════════════╗"
echo "║           Kglance End-to-End Performance Test           ║"
echo "╠══════════════════════════════════════════════════════════╣"
echo "║  Threshold: ${PASS_THRESHOLD_MS}ms   Iterations: ${ITERATIONS}                    ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

OVERALL_PASS=true

run_bench() {
    local label="$1"
    local file="$2"
    local total_ns=0
    local min_ns=999999999999
    local max_ns=0
    local failed=0

    if [[ ! -f "$file" ]] && [[ ! -d "$file" ]]; then
        printf "  %-20s  SKIP (file not found)\n" "$label"
        return
    fi

    for i in $(seq 1 "$ITERATIONS"); do
        # Đo bằng timeout + time; standalone mode sẽ cần UI nên ta đo parse-only
        # thông qua strace hoặc đo binary startup
        local start_ns
        start_ns=$(date +%s%N)
        timeout 5 "$BINARY" --standalone "$file" &
        local pid=$!
        # Đợi cửa sổ xuất hiện (hoặc timeout)
        sleep 0.2
        kill $pid 2>/dev/null || true
        wait $pid 2>/dev/null || true
        local end_ns
        end_ns=$(date +%s%N)
        local elapsed_ns=$((end_ns - start_ns))

        total_ns=$((total_ns + elapsed_ns))
        if (( elapsed_ns < min_ns )); then min_ns=$elapsed_ns; fi
        if (( elapsed_ns > max_ns )); then max_ns=$elapsed_ns; fi
    done

    local avg_ms=$(( total_ns / ITERATIONS / 1000000 ))
    local min_ms=$(( min_ns / 1000000 ))
    local max_ms=$(( max_ns / 1000000 ))
    local status="PASS"

    if (( avg_ms > PASS_THRESHOLD_MS )); then
        status="FAIL"
        OVERALL_PASS=false
    fi

    printf "  %-20s  avg=%4dms  min=%4dms  max=%4dms  [%s]\n" \
        "$label" "$avg_ms" "$min_ms" "$max_ms" "$status"
}

echo "Parser benchmarks (standalone mode):"
echo ""
run_bench "Text (1000 lines)"  "$TMPDIR/sample.rs"
run_bench "SVG (simple)"       "$TMPDIR/sample.svg"
run_bench "Archive (ZIP)"      "$TMPDIR/sample.zip"
run_bench "Folder"             "$TMPDIR"

echo ""
if [[ "$OVERALL_PASS" == true ]]; then
    echo "Result: ALL PASSED (< ${PASS_THRESHOLD_MS}ms)"
else
    echo "Result: SOME FAILED (> ${PASS_THRESHOLD_MS}ms)"
    exit 1
fi
```

### 3.2 Đo thời gian DBus round-trip (Daemon mode)

```bash
#!/usr/bin/env bash
# Đo latency DBus call khi daemon đang chạy.
# Yêu cầu: kglance daemon đang chạy.
set -euo pipefail

FILE="${1:-/etc/hostname}"
ITERATIONS="${2:-50}"

if ! busctl --user status org.mintori.Kglance &>/dev/null; then
    echo "Daemon not running. Start with: kglance daemon"
    exit 1
fi

echo "DBus round-trip benchmark (${ITERATIONS} iterations)"
echo "File: $FILE"
echo ""

total_us=0
for i in $(seq 1 "$ITERATIONS"); do
    start=$(date +%s%N)
    busctl --user call org.mintori.Kglance /org/mintori/Kglance \
        org.mintori.Kglance ShowPreview s "$FILE" >/dev/null
    end=$(date +%s%N)
    elapsed_us=$(( (end - start) / 1000 ))
    total_us=$((total_us + elapsed_us))
    printf "\r  iteration %d/%d  %d µs" "$i" "$ITERATIONS" "$elapsed_us"
done

avg_us=$((total_us / ITERATIONS))
avg_ms=$((avg_us / 1000))
echo ""
echo ""
echo "Average: ${avg_us} µs (${avg_ms} ms)"

if (( avg_ms < 100 )); then
    echo "Result: PASS (< 100ms)"
else
    echo "Result: FAIL (>= 100ms)"
    exit 1
fi
```

---

## 4. Đo bằng Instrumentation trong Code

Thêm macro đo thời gian nội bộ (chỉ active trong debug build):

```rust
/// Đo thời gian thực thi một block code.
/// In ra stderr trong debug build, không làm gì trong release.
macro_rules! perf_trace {
    ($label:expr, $block:expr) => {{
        #[cfg(debug_assertions)]
        let _start = std::time::Instant::now();

        let result = $block;

        #[cfg(debug_assertions)]
        eprintln!(
            "[perf] {}: {:.2}ms",
            $label,
            _start.elapsed().as_secs_f64() * 1000.0
        );

        result
    }};
}
```

Sử dụng trong `dbus/service.rs`:

```rust
async fn show_preview(&mut self, file_path: &str) -> zbus::fdo::Result<()> {
    let path = std::path::Path::new(file_path);
    let content = perf_trace!("parse", {
        self.parser_registry.parse(path).map_err(/* ... */)
    })?;
    perf_trace!("channel_send", {
        self.tx.send(DaemonCommand::ShowPreview {
            path: file_path.to_string(),
            content,
        })
    }).map_err(/* ... */)?;
    Ok(())
}
```

Output mẫu (debug build):

```
[perf] parse: 2.34ms
[perf] channel_send: 0.01ms
```

---

## 5. Ngưỡng hiệu năng theo loại file

Bảng dưới đây là ngưỡng **parser-only** (không tính UI/DBus) cho các kịch bản phổ biến. Đây là mục tiêu cho `cargo bench`:

| Loại file | Kích thước / Độ phức tạp | Ngưỡng (ms) | Ghi chú |
| --- | --- | --: | --- |
| Text / Source code | < 1,000 dòng | 5 | Đọc file + detect language |
| Text / Source code | 10,000 dòng | 20 | I/O là bottleneck chính |
| Image PNG/JPEG | 1920×1080 | 30 | Decode + re-encode PNG |
| Image PNG/JPEG | 3840×2160 (4K) | 50 | Cân nhắc downscale trước |
| SVG | < 100 elements | 10 | resvg render |
| SVG | 1,000 elements | 40 | Complex path tốn CPU |
| PDF | Bất kỳ | 15-150 | MuPDF render 150 DPI |
| Markdown               | 10 lines                 |          10 | pulldown-cmark + syntect       |
| Markdown               | 100 lines                |          30 | Include code blocks            |
| Markdown               | 500 lines                |          80 | Nhiều code blocks → chậm hơn  |
| Archive ZIP | 100 entries | 5 | Chỉ đọc central directory |
| Archive ZIP | 1,000 entries | 15 | Tỷ lệ tuyến tính |
| Folder | 50 items | 5 | readdir + stat |
| Folder | 200 items | 15 | Nhiều syscall |

### Quy tắc vàng

- Nếu parser > **50 ms** → xem xét lazy loading hoặc downscale
- Nếu tổng pipeline > **100 ms** → đây là **regression**, phải sửa trước khi merge

---

## 6. Profiling chuyên sâu

### 6.1 Flamegraph

```bash
# Cài đặt
cargo install flamegraph

# Chạy (cần quyền perf hoặc dtrace)
cargo flamegraph --bin kglance -- --standalone /path/to/large_file.rs
```

### 6.2 perf stat (Linux)

```bash
# Đếm cache miss, branch miss, IPC
perf stat -e cache-misses,branch-misses,instructions,cycles \
    target/release/kglance --standalone /path/to/file.rs &
PID=$!
sleep 0.5
kill $PID
```

### 6.3 Valgrind / DHAT (heap profiling)

```bash
# Đo heap allocation
valgrind --tool=dhat target/release/kglance --standalone /path/to/file.rs &
PID=$!
sleep 0.5
kill $PID
# Kết quả: dhat-out-<pid>.txt → mở bằng dhat viewer
```

### 6.4 Tracing với tokio-console (cho async)

```bash
# Thêm vào Cargo.toml:
# console-subscriber = "0.4"

# Trong main.rs (daemon mode):
# console_subscriber::init();

# Chạy:
tokio-console
```

---

## 7. CI Performance Gate

Thêm vào CI pipeline (GitHub Actions / GitLab CI) để chặn regression:

```yaml
# .github/workflows/perf.yml
name: Performance Gate

on:
  pull_request:
    paths: ["src/**", "Cargo.toml", "Cargo.lock"]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system deps
        run: |
          sudo apt-get update
          sudo apt-get install -y libfontconfig-dev libfreetype-dev libxkbcommon-dev

      - name: Build release
        run: cargo build --release

      - name: Run benchmarks
        run: cargo bench -- --output-format bencher | tee bench_output.txt

      - name: Check parser thresholds
        run: |
          # Trích xuất kết quả và so sánh với ngưỡng
          python3 - << 'PYEOF'
          import re, sys

          THRESHOLDS = {
              "parser/text/1000_lines": 10_000_000,    # 10ms in ns
              "parser/text/10000_lines": 30_000_000,   # 30ms
              "parser/image/1920x1080": 50_000_000,    # 50ms
              "parser/image/3840x2160": 80_000_000,    # 80ms
              "parser/svg/100_elements": 15_000_000,   # 15ms
              "parser/markdown/10_lines": 10_000_000,  # 10ms (actual ~1.5ms)
              "parser/markdown/100_lines": 30_000_000, # 30ms (actual ~15ms)
              "parser/markdown/500_lines": 100_000_000, # 100ms (actual ~75ms)
              "parser/archive/100_files": 10_000_000,  # 10ms
              "parser/folder/50_items": 10_000_000,    # 10ms
          }

          failed = False
          with open("bench_output.txt") as f:
              for line in f:
                  m = re.match(r"test (\S+)\s+bench:\s+([\d,]+) ns/iter", line)
                  if not m:
                      continue
                  name = m.group(1)
                  ns = int(m.group(2).replace(",", ""))
                  threshold = THRESHOLDS.get(name)
                  if threshold and ns > threshold:
                      ms = ns / 1_000_000
                      threshold_ms = threshold / 1_000_000
                      print(f"FAIL: {name} = {ms:.1f}ms (threshold: {threshold_ms:.0f}ms)")
                      failed = True

          if failed:
              sys.exit(1)
          print("All benchmarks within thresholds")
          PYEOF

      - name: Upload benchmark results
        uses: actions/upload-artifact@v4
        with:
          name: criterion-report
          path: target/criterion/
```

## 8. Checklist trước khi release

- [ ] `cargo bench` — tất cả parser dưới ngưỡng bảng mục 5
- [ ] `cargo bench` — image operations dưới ngưỡng bảng 2.4.2
- [ ] `scripts/bench_e2e.sh` — tất cả loại file PASS (< 100ms)
- [ ] `scripts/bench_dbus.sh` — DBus round-trip trung bình < 100ms
- [ ] Flamegraph — không có hotspot bất thường
- [ ] Heap profiling — không có allocation đột biến (> 50MB cho file thông thường)
- [ ] CI perf gate — xanh trên PR

---

## 9. Bottleneck đã biết & Hướng tối ưu

| Bottleneck | Ảnh hưởng | Hướng xử lý |
| --- | --- | --- |
| `image::open` decode ảnh lớn | 20-80ms | Downscale trước khi decode, hoặc dùng `turbojpeg` |
| `image::write_to(PNG)` re-encode | 10-40ms | Truyền raw RGBA trực tiếp cho Slint, bỏ bước encode |
| `image::load_from_memory` trong UI show() | 5-30ms | Bỏ nếu UI nhận raw RGBA từ parser |
| `resize_exact(Lanczos3)` khi zoom | 20-350ms | **Bottleneck nặng nhất** — dùng CatmullRom cho realtime, hoặc cache |
| `DynamicImage::clone()` mỗi lần zoom | 1-5ms | Dùng `Arc<DynamicImage>` thay `Option<DynamicImage>` |
| Pixel copy loop (bytewise) | 8-35ms | Thay bằng `copy_from_slice` — nhanh hơn 8× |
| `rotate90()` trên ảnh lớn | 2-10ms | Cache rotation state, chỉ rotate 1 lần trước resize |
| `SyntaxSet::load_defaults_newlines()` | ~15ms | Đã cache trong `TextParser::new()` — OK |
| `resvg::render` SVG phức tạp | 10-50ms | Giới hạn viewport, hoặc cache kết quả render |
| `mupdf::Document::open` + render | 15-150ms | Phụ thuộc số trang; lazy render từng trang |
| `pulldown_cmark::Parser` parse full file | <1ms | Không đáng kể; bottleneck là syntect highlighting |
| `syntect::HighlightLines` code block | 1-5ms/block | Cache kết quả cho code block giống nhau |
| `StyledText::from_markdown` trong Slint | ~5ms | Chỉ gọi 1 lần khi preview |
| Slint first frame render | ~20ms | Daemon mode giữ window ẩn — chỉ tốn 1 lần |
| `busctl` fork+exec | ~10ms | Chuyển sang gọi `zbus` client trực tiếp (pure Rust) |

### Cải thiện ưu tiên cao nhất

1. **Giảm chất lượng resize (Lanczos3 → CatmullRom)**: Zoom realtime không cần Lanczos3. Chuyển sang `CatmullRom` hoặc `Triangle` tiết kiệm 40-60% thời gian resize (từ 80ms → 30ms cho FHD 200%).

2. **Thay pixel copy loop bằng `copy_from_slice`**: Dòng 1334-1339 trong `src/ui/mod.rs` đang copy từng pixel qua vòng lặp `enumerate()`. Thay bằng `buffer.make_mut_slice().copy_from_slice(&raw)` tiết kiệm 8-35ms.

3. **Bỏ re-encode PNG trong `ImageParser`**: Decode rồi encode lại PNG trước khi gửi cho UI. Truyền raw `Vec<u8>` (RGBA) trực tiếp cho Slint qua variant `ImageRaw` sẽ tiết kiệm 10-40ms + 5-15ms decode lại trong UI.

4. **Dùng `Arc<DynamicImage>`**: Clone toàn bộ ảnh mỗi lần zoom/rotate là lãng phí. `Arc<DynamicImage>` tránh clone, chỉ clone khi thực sự cần modify (rotate).

5. **Thay `busctl` bằng `zbus` client**: `send_via_dbus()` trong `main.rs` đang fork process `busctl`. Gọi `zbus::Connection` trực tiếp sẽ nhanh hơn ~10ms.

6. **Lazy parse cho file lớn**: File text > 5000 dòng chỉ parse 200 dòng đầu, load thêm khi scroll.
