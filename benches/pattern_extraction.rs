//! Benchmarks for pattern extraction

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_core::*;
use test_utils::*;

fn benchmark_extract_patterns(c: &mut Criterion) {
    let episode = create_completed_episode("Test task with patterns", true);
    let extractor = memory_core::extraction::PatternExtractor::new();

    c.bench_function("extract_patterns_single_episode", |b| {
        b.iter(|| extractor.extract(black_box(&episode)));
    });
}

fn benchmark_extract_patterns_varying_steps(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_extraction_by_step_count");
    let extractor = memory_core::extraction::PatternExtractor::new();

    for step_count in [5, 10, 20, 50].iter() {
        let mut episode = create_test_episode("Test");
        for i in 0..*step_count {
            episode.add_step(create_test_step(i + 1));
        }
        episode.complete(TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, _| {
                b.iter(|| extractor.extract(black_box(&episode)));
            },
        );
    }

    group.finish();
}

fn benchmark_pattern_relevance_check(c: &mut Criterion) {
    let pattern = create_test_pattern("tool_sequence", 0.9);
    let context = create_test_context("web-api", Some("rust"));

    c.bench_function("pattern_relevance_check", |b| {
        b.iter(|| pattern.is_relevant_to(black_box(&context)));
    });
}

criterion_group!(
    benches,
    benchmark_extract_patterns,
    benchmark_extract_patterns_varying_steps,
    benchmark_pattern_relevance_check
);
criterion_main!(benches);
