use criterion::{Criterion, criterion_group, criterion_main};
use kglance::core::scroll::velocity::{
    EstimatorStrategy, LinearRegression, PolynomialRegression2, Sample, VelocityEstimator,
    VelocityEstimatorAlgorithm, WeightedRecentRegression,
};
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

    let samples: Vec<Sample> = (0..8)
        .map(|i| Sample {
            time: now + Duration::from_millis(i * 8),
            delta_y: -25.0,
        })
        .collect();

    c.bench_function("velocity_estimator_polynomial2_default", |b| {
        b.iter(|| {
            black_box(estimator.compute_velocity(black_box(query_time)));
        });
    });

    estimator.set_strategy(EstimatorStrategy::Linear);
    c.bench_function("velocity_estimator_linear", |b| {
        b.iter(|| {
            black_box(estimator.compute_velocity(black_box(query_time)));
        });
    });

    estimator.set_strategy(EstimatorStrategy::WeightedRecent);
    c.bench_function("velocity_estimator_weighted_recent", |b| {
        b.iter(|| {
            black_box(estimator.compute_velocity(black_box(query_time)));
        });
    });

    c.bench_function("direct_lsq1_regression", |b| {
        b.iter(|| {
            black_box(LinearRegression.estimate(black_box(&samples), black_box(query_time)));
        });
    });

    c.bench_function("direct_lsq2_polynomial_regression", |b| {
        b.iter(|| {
            black_box(PolynomialRegression2.estimate(black_box(&samples), black_box(query_time)));
        });
    });

    c.bench_function("direct_wlsq2_weighted_regression", |b| {
        let wlsq = WeightedRecentRegression::default();
        b.iter(|| {
            black_box(wlsq.estimate(black_box(&samples), black_box(query_time)));
        });
    });
}

criterion_group!(benches, bench_velocity_estimation);
criterion_main!(benches);
