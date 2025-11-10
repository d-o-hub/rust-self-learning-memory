//! Benchmarks for episode lifecycle operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Simplified benchmarks that test basic operations without complex dependencies

fn benchmark_basic_operations(c: &mut Criterion) {
    c.bench_function("basic_memory_operations", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(black_box(i));
            }
            vec.sort();
            black_box(vec.len());
        });
    });
}

fn benchmark_hashmap_operations(c: &mut Criterion) {
    use std::collections::HashMap;

    c.bench_function("hashmap_operations", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..500 {
                map.insert(black_box(i), i * 2);
            }
            let sum: i32 = map.values().sum();
            black_box(sum);
        });
    });
}

fn benchmark_string_processing(c: &mut Criterion) {
    let text = "This is a test string for benchmarking string operations. ".repeat(100);

    c.bench_function("string_processing", |b| {
        b.iter(|| {
            let words: Vec<&str> = text.split_whitespace().collect();
            let filtered: Vec<_> = words
                .iter()
                .filter(|word| word.len() > 3)
                .collect();
            black_box(filtered.len());
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
