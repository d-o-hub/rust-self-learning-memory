//! Top-K Selection Performance Benchmarks
//!
//! Compares O(n) select_nth_unstable_by vs O(n log n) full sort.
//! Part of WG-104: select_nth_unstable_by for top-k retrieval.
//!
//! Run with: `cargo bench --bench top_k_benchmark`

// Benchmarks have relaxed clippy rules
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::stable_sort_primitive)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::let_with_type_underscore)]

use criterion::{Criterion, criterion_group, criterion_main};
use rand::RngExt;
use rand::rng;
use std::hint::black_box;

/// Generate random scores for benchmarking
fn generate_random_scores(n: usize) -> Vec<f32> {
    let mut rng = rng();
    (0..n).map(|_| rng.random_range(0.0..1.0)).collect()
}

/// O(n log n) approach: full sort then take top k
fn top_k_full_sort(scores: &mut [f32], k: usize) -> Vec<f32> {
    scores.sort_by(|a, b| b.partial_cmp(a).unwrap());
    scores[..k].to_vec()
}

/// O(n) approach: select_nth_unstable_by + partial sort
fn top_k_partial_sort(scores: &mut [f32], k: usize) -> Vec<f32> {
    if k == 0 || scores.is_empty() {
        return vec![];
    }
    let k = k.min(scores.len());

    // Partition: top k elements are now in scores[..k] (unsorted)
    scores.select_nth_unstable_by(k - 1, |a, b| b.partial_cmp(a).unwrap());

    // Sort only the top k
    let mut top_k: Vec<f32> = scores[..k].to_vec();
    top_k.sort_by(|a, b| b.partial_cmp(a).unwrap());
    top_k
}

/// Benchmark: Compare full sort vs partial sort for various data sizes
fn bench_top_k_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("top_k_selection");

    for n in [100, 1_000, 10_000, 100_000] {
        let k = n / 10; // 10% top-k

        // Full sort approach
        let scores_full = generate_random_scores(n);
        group.bench_function(format!("full_sort_n{}_k{}", n, k), |b| {
            b.iter_batched(
                || scores_full.clone(),
                |mut s| black_box(top_k_full_sort(&mut s, k)),
                criterion::BatchSize::SmallInput,
            )
        });

        // Partial sort approach (WG-104)
        let scores_partial = generate_random_scores(n);
        group.bench_function(format!("partial_sort_n{}_k{}", n, k), |b| {
            b.iter_batched(
                || scores_partial.clone(),
                |mut s| black_box(top_k_partial_sort(&mut s, k)),
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// Benchmark: Vary k for fixed n
fn bench_top_k_varying_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("top_k_varying_k");
    let n = 10_000;
    let scores = generate_random_scores(n);

    for k in [10, 100, 1_000, 5_000] {
        // Partial sort (our implementation)
        group.bench_function(format!("partial_n{}_k{}", n, k), |b| {
            b.iter_batched(
                || scores.clone(),
                |mut s| black_box(top_k_partial_sort(&mut s, k)),
                criterion::BatchSize::SmallInput,
            )
        });

        // Full sort (baseline)
        group.bench_function(format!("full_n{}_k{}", n, k), |b| {
            b.iter_batched(
                || scores.clone(),
                |mut s| black_box(top_k_full_sort(&mut s, k)),
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// Benchmark: Using the actual select_top_k from the crate
fn bench_select_top_k_crate(c: &mut Criterion) {
    let mut group = c.benchmark_group("select_top_k_crate");
    let mut rng = rng();

    for n in [100, 1_000, 10_000, 100_000] {
        let k = n / 10;

        // Generate items with scores
        let items: Vec<(String, f32)> = (0..n)
            .map(|i| (format!("item_{}", i), rng.random_range(0.0..1.0)))
            .collect();

        group.bench_function(format!("crate_n{}_k{}", n, k), |b| {
            b.iter_batched(
                || items.clone(),
                |mut items| {
                    black_box(do_memory_core::search::top_k::select_top_k(
                        items.as_mut_slice(),
                        k,
                        |a, b| b.1.partial_cmp(&a.1).unwrap(),
                    ))
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_top_k_comparison,
    bench_top_k_varying_k,
    bench_select_top_k_crate,
);
criterion_main!(benches);
