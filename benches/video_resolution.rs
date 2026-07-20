use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

/// Generate a simulated RGBA video frame buffer at the given dimensions.
fn gen_frame(w: u32, h: u32) -> Vec<u8> {
    let len = (w * h * 4) as usize;
    let mut buf = vec![0u8; len];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize * 4;
            buf[i] = (x ^ y) as u8;
            buf[i + 1] = ((x * 3 + y * 5) & 0xff) as u8;
            buf[i + 2] = ((x * 7 + y * 11) & 0xff) as u8;
            buf[i + 3] = 255;
        }
    }
    buf
}

/// Compute scaled dimensions using the same logic as `ui/handlers/video.rs`.
fn scaled_dims(src_w: u32, src_h: u32, max_dim: f64) -> (u32, u32) {
    let scale = (max_dim / (src_w.max(src_h) as f64)).min(1.0);
    let tw = (((src_w as f64 * scale) as u32) & !1).max(16);
    let th = (((src_h as f64 * scale) as u32) & !1).max(16);
    (tw, th)
}

// ── Benchmark groups ────────────────────────────────────────────────────

/// Measure clone and image-handle creation cost per resolution.
fn bench_frame_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("video/frame_pipeline");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(50);

    let src_res: &[(u32, u32, &str)] = &[(1920, 1080, "1080p_source")];

    let max_dims: &[(f64, &str)] = &[
        (480.0, "max480"),
        (720.0, "max720"),
        (900.0, "max900"),
        (1080.0, "max1080"),
        (1440.0, "max1440"),
    ];

    for &(src_w, src_h, src_label) in src_res {
        for &(max_dim, dim_label) in max_dims {
            let (tw, th) = scaled_dims(src_w, src_h, max_dim);
            let frame = gen_frame(tw, th);
            let label = format!("{src_label}/{dim_label}_{tw}x{th}");

            // Clone cost
            group.bench_with_input(BenchmarkId::new("clone", &label), &frame, |b, data| {
                b.iter(|| black_box(data.clone()))
            });

            // Image handle creation cost
            group.bench_with_input(
                BenchmarkId::new("handle_from_rgba", &label),
                &frame,
                |b, data| {
                    b.iter(|| {
                        let h = iced::widget::image::Handle::from_rgba(tw, th, data.clone());
                        black_box(h)
                    })
                },
            );
        }
    }
    group.finish();
}

/// Measure channel send/recv latency for each frame size.
/// Uses tokio::sync::mpsc to mimic the real pipeline.
fn bench_channel_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("video/channel");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(50);

    let rt = tokio::runtime::Runtime::new().unwrap();

    let src_res: &[(u32, u32, &str)] = &[(1920, 1080, "1080p_source")];
    let max_dims: &[(f64, &str)] = &[(480.0, "max480"), (720.0, "max720"), (1080.0, "max1080")];

    for &(src_w, src_h, src_label) in src_res {
        for &(max_dim, dim_label) in max_dims {
            let (tw, th) = scaled_dims(src_w, src_h, max_dim);
            let frame = gen_frame(tw, th);
            let label = format!("{src_label}/{dim_label}_{tw}x{th}");

            group.bench_with_input(
                BenchmarkId::new("try_send_recv", &label),
                &frame,
                |b, data| {
                    b.iter_batched(
                        || {
                            let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
                            (tx, rx, data.clone())
                        },
                        |(tx, mut rx, d)| {
                            rt.block_on(async {
                                let _ = tx.try_send(d);
                                if let Some(received) = rx.recv().await {
                                    black_box(received);
                                }
                            });
                        },
                        criterion::BatchSize::SmallInput,
                    )
                },
            );
        }
    }
    group.finish();
}

/// Measure total memory throughput per second at each resolution.
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("video/throughput");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(30);

    let src_res: &[(u32, u32, &str)] = &[(1920, 1080, "1080p_source")];
    let max_dims: &[(f64, &str)] = &[
        (480.0, "max480"),
        (720.0, "max720"),
        (900.0, "max900"),
        (1080.0, "max1080"),
        (1440.0, "max1440"),
    ];

    for &(src_w, src_h, src_label) in src_res {
        for &(max_dim, dim_label) in max_dims {
            let (tw, th) = scaled_dims(src_w, src_h, max_dim);
            let frame = gen_frame(tw, th);
            let frame_size = frame.len();
            let label = format!("{src_label}/{dim_label}_{tw}x{th}");

            // Simulate the full per-frame overhead: clone + handle
            group.bench_with_input(
                BenchmarkId::new("full_overhead", &label),
                &frame,
                |b, data| {
                    b.iter(|| {
                        let cloned = data.clone();
                        let handle = iced::widget::image::Handle::from_rgba(tw, th, cloned);
                        black_box(handle);
                    })
                },
            );

            // Theoretical max FPS based on full overhead
            group.bench_with_input(
                BenchmarkId::new("max_fps_estimate", &label),
                &(frame_size, tw, th),
                |b, &(size, w, h)| {
                    b.iter(|| {
                        let d = vec![0u8; size];
                        let h = iced::widget::image::Handle::from_rgba(w, h, d);
                        black_box(h);
                    })
                },
            );
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_frame_pipeline,
    bench_channel_latency,
    bench_throughput,
);

criterion_main!(benches);
