//! Vector similarity performance benchmarks
//!
//! Run with: `cargo bench --bench similarity_benchmark`

use criterion::{Criterion, criterion_group, criterion_main};
use do_memory_core::embeddings::cosine_similarity;
use rand::RngExt;
use std::hint::black_box;

/// Generate a random vector of size n with values in [-1, 1]
fn generate_random_vector(n: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    (0..n).map(|_| rng.random_range(-1.0..1.0)).collect()
}

/// Benchmark cosine similarity for various vector sizes
fn bench_cosine_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_similarity");

    for n in [128, 256, 384, 512, 768, 1024, 1536] {
        let v1 = generate_random_vector(n);
        let v2 = generate_random_vector(n);

        group.bench_function(format!("size_{n}"), |b| {
            b.iter(|| black_box(cosine_similarity(black_box(&v1), black_box(&v2))));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_cosine_similarity);
criterion_main!(benches);
