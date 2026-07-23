# Performance Specification & Benchmarking — Kglance

> **Strict SLA Target:** Every preview action must complete in **< 100 ms** from keypress to full content rendering.

---

## 1. Latency Time Budget

Processing pipeline breakdown when pressing `Space` in Dolphin (Daemon mode):

```text
[Space Key] → KIO Service Menu → busctl call → DBus transport → Parser → UI update → Window show
```

| Phase | Budget Target | Notes |
| --- | --: | --- |
| **KIO dispatch & IPC** | 15 ms | Fixed system overhead |
| **DBus Transport (`zbus`)** | 5 ms | Zero-copy Unix socket IPC |
| **Parsing Engine** | 50 ms | Format parsing (file size dependent) |
| **UI Render Update** | 20 ms | Iced state update & first frame render |
| **Window Compositor Show** | 10 ms | Wayland compositor latency |
| **Total Target** | **100 ms** | **End-to-End Latency Target** |

---

## 2. Benchmark Benchmarking & Thresholds

### Parser Benchmarks

Run Criterion benchmarks using Cargo:

```bash
cargo bench
```

### Performance SLA Thresholds

| File Category | Input Specification | Threshold (ms) | Notes |
| --- | --- | --: | --- |
| **Text / Source Code** | < 1,000 lines | 5 ms | Read & syntax highlight |
| **Text / Source Code** | 10,000 lines | 20 ms | File I/O bound |
| **Image (PNG/JPEG)** | 1920×1080 (FHD) | 30 ms | Decode & RGBA buffer copy |
| **Image (PNG/JPEG)** | 3840×2160 (4K) | 50 ms | Downscale optimization |
| **SVG Vector** | < 100 elements | 10 ms | `resvg` hardware rendering |
| **PDF Document** | Single page render | 15-50 ms | MuPDF 150 DPI render |
| **Markdown** | 100 lines with code | 30 ms | `pulldown-cmark` + `syntect` |
| **Archive (ZIP/TAR)** | 100 entries | 5 ms | Read central directory only |
| **Directory** | 50 items | 5 ms | Directory scan & metadata stat |

---

## 3. High-Priority Optimization Targets

1. **Avoid PNG Re-encoding:** Pass raw RGBA pixel buffers directly to Iced widgets rather than re-encoding to PNG format.
2. **Efficient Image Scaling:** Use `CatmullRom` scaling instead of heavy `Lanczos3` during real-time zoom interactions.
3. **Buffer Memory Copies:** Use `copy_from_slice` instead of element-by-element loop copying for pixel buffers.
4. **Direct DBus Calls:** Use native Rust `zbus` connection handles instead of spawning external `busctl` processes.
