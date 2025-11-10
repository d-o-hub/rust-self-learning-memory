//! Benchmarks for storage operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_core::*;
use test_utils::*;

// Simplified benchmark that focuses on core operations without database I/O
// This avoids timeout issues in CI environments while still measuring performance

fn benchmark_episode_serialization(c: &mut Criterion) {
    let episode = create_completed_episode("Benchmark", true);

    c.bench_function("episode_serialization", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&episode)).unwrap();
            let _: Episode = serde_json::from_str(black_box(&serialized)).unwrap();
        });
    });
}

fn benchmark_episode_memory_operations(c: &mut Criterion) {
    let mut episode = create_test_episode("Benchmark");

    c.bench_function("episode_memory_operations", |b| {
        b.iter(|| {
            // Simulate memory operations without I/O
            for i in 0..10 {
                let step = create_test_step(i + 1);
                episode.add_step(black_box(step));
            }
            let outcome = TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            };
            episode.complete(black_box(outcome));
        });
    });
}

fn benchmark_pattern_matching(c: &mut Criterion) {
    let episodes: Vec<Episode> = (0..100)
        .map(|i| create_completed_episode(&format!("Task {}", i), true))
        .collect();

    c.bench_function("pattern_matching_simulation", |b| {
        b.iter(|| {
            // Simulate pattern matching logic without database queries
            let mut matches = 0;
            for episode in &episodes {
                if episode.task_description.contains("Task 5") {
                    matches += 1;
                }
            }
            black_box(matches);
        });
    });
}

fn benchmark_context_filtering(c: &mut Criterion) {
    let episodes: Vec<Episode> = (0..1000)
        .map(|i| {
            let domain = if i % 2 == 0 { "web-api" } else { "cli-tool" };
            let context = create_test_context(domain, Some("rust"));
            create_test_episode_with_context(
                &format!("Task {}", i),
                context,
                TaskType::Testing,
            )
        })
        .collect();

    c.bench_function("context_filtering", |b| {
        b.iter(|| {
            // Simulate filtering by context without database queries
            let filtered: Vec<_> = episodes
                .iter()
                .filter(|episode| episode.context.domain == "web-api")
                .take(10)
                .collect();
            black_box(filtered.len());
        });
    });
}

criterion_group!(
    benches,
    benchmark_episode_serialization,
    benchmark_episode_memory_operations,
    benchmark_pattern_matching,
    benchmark_context_filtering
);
criterion_main!(benches);
