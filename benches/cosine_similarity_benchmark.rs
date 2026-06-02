//! Cosine similarity benchmarks.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use do_memory_core::embeddings::cosine_similarity;
use rand::distr::{Distribution, Uniform};
use std::hint::black_box;

fn generate_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::rng();
    let die = Uniform::new(-1.0, 1.0).unwrap();
    (0..dim).map(|_| die.sample(&mut rng)).collect()
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
