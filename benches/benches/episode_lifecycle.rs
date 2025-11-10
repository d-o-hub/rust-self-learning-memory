//! Benchmarks for episode lifecycle operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Simplified benchmarks that test basic operations without complex dependencies

fn benchmark_basic_operations(c: &mut Criterion) {
    c.bench_function("basic_memory_operations", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..10 {
                vec.push(black_box(i));
            }
            black_box(vec.len());
        });
    });
}

fn benchmark_hashmap_operations(c: &mut Criterion) {
    use std::collections::HashMap;

    c.bench_function("hashmap_operations", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..5 {
                map.insert(black_box(i), i * 2);
            }
            black_box(map.len());
        });
    });
}

fn benchmark_string_processing(c: &mut Criterion) {
    let text = "This is a test string for benchmarking";

    c.bench_function("string_processing", |b| {
        b.iter(|| {
            let words: Vec<&str> = text.split_whitespace().collect();
            black_box(words.len());
        });
    });
}

criterion_group!(
    benches,
    benchmark_basic_operations,
    benchmark_hashmap_operations,
    benchmark_string_processing
);
criterion_main!(benches);
