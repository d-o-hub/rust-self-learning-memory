//! Cosine Similarity Performance Benchmarks
//!
//! Benchmarks for the cosine_similarity function across various vector dimensions.

#![allow(clippy::doc_markdown)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use do_memory_core::embeddings::cosine_similarity;
use rand::RngExt;
use rand::rng;
use std::hint::black_box;

fn generate_vector(dim: usize) -> Vec<f32> {
    let mut rng = rng();
    (0..dim).map(|_| rng.random_range(-1.0..1.0)).collect()
}

fn bench_cosine_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("cosine_similarity");

    for dim in [384, 768, 1536, 3072] {
        let v1 = generate_vector(dim);
        let v2 = generate_vector(dim);

        group.bench_with_input(BenchmarkId::new("scalar", dim), &dim, |b, _| {
            b.iter(|| black_box(cosine_similarity(black_box(&v1), black_box(&v2))));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_cosine_similarity);
criterion_main!(benches);
