use criterion::{Criterion, criterion_group, criterion_main};
use kglance::core::scroll::velocity::VelocityEstimator;
use std::hint::black_box;
use std::time::{Duration, Instant};

fn bench_velocity_estimation(c: &mut Criterion) {
    let now = Instant::now();
    let mut estimator = VelocityEstimator::new();

    // Populate with 8 samples (simulating a ~60ms fast flick)
    for i in 0..8 {
        let t = now + Duration::from_millis(i * 8);
        estimator.push_sample(t, -25.0);
    }
    let query_time = now + Duration::from_millis(60);

    c.bench_function("wlsq_velocity_compute", |b| {
        b.iter(|| {
            black_box(estimator.compute_velocity(black_box(query_time)));
        });
    });
}

criterion_group!(benches, bench_velocity_estimation);
criterion_main!(benches);
