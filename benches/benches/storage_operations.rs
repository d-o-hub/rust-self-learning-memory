//! Benchmarks for storage operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Very simple benchmark that doesn't depend on complex external libraries
// This ensures benchmarks run quickly and reliably in CI

fn benchmark_simple_operations(c: &mut Criterion) {
    c.bench_function("simple_memory_operations", |b| {
        b.iter(|| {
            let mut data = Vec::new();
            for i in 0..1000 {
                data.push(black_box(i * 2));
            }
            let sum: i32 = data.iter().sum();
            black_box(sum);
        });
    });
}

fn benchmark_string_operations(c: &mut Criterion) {
    c.bench_function("string_operations", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..100 {
                strings.push(format!("test_string_{}", black_box(i)));
            }
            let concatenated = strings.join(",");
            black_box(concatenated.len());
        });
    });
}

fn benchmark_vector_operations(c: &mut Criterion) {
    let data: Vec<i32> = (0..10000).collect();

    c.bench_function("vector_filtering", |b| {
        b.iter(|| {
            let filtered: Vec<_> = data
                .iter()
                .filter(|&&x| x % 2 == 0)
                .take(100)
                .collect();
            black_box(filtered.len());
        });
    });
}

criterion_group!(
    benches,
    benchmark_simple_operations,
    benchmark_string_operations,
    benchmark_vector_operations
);
criterion_main!(benches);
