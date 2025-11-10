//! Benchmarks for pattern extraction

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Simplified benchmarks for pattern-like operations

fn benchmark_regex_matching(c: &mut Criterion) {
    use regex::Regex;

    let pattern = Regex::new(r"test_\d+").unwrap();
    let text = "This contains test_123 and test_456 and other content. ".repeat(100);

    c.bench_function("regex_matching", |b| {
        b.iter(|| {
            let matches: Vec<_> = pattern.find_iter(&text).collect();
            black_box(matches.len());
        });
    });
}

fn benchmark_data_processing(c: &mut Criterion) {
    let data: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();

    c.bench_function("data_processing", |b| {
        b.iter(|| {
            let processed: Vec<_> = data
                .iter()
                .filter(|item| item.contains("5"))
                .map(|item| item.to_uppercase())
                .collect();
            black_box(processed.len());
        });
    });
}

fn benchmark_pattern_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_search_by_size");

    for size in [100, 1000, 10000].iter() {
        let haystack: String = (0..*size).map(|i| format!("word{} ", i)).collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    let count = haystack.split_whitespace()
                        .filter(|word| word.contains("5"))
                        .count();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_regex_matching,
    benchmark_data_processing,
    benchmark_pattern_search
);
criterion_main!(benches);
