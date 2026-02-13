//! Benchmarks for pattern extraction

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_benches::TokioExecutor;
use memory_core::{
    episode::ExecutionStep,
    extraction::PatternExtractor,
    memory::SelfLearningMemory,
    types::{TaskOutcome, TaskType},
};
use regex::Regex;
use std::hint::black_box;
use uuid::Uuid;

async fn log_steps(memory: &SelfLearningMemory, episode_id: Uuid, steps: Vec<ExecutionStep>) {
    for step in steps {
        memory.log_step(episode_id, step).await;
    }
}

fn benchmark_regex_pattern_matching(c: &mut Criterion) {
    let pattern = Regex::new(r"pattern_\d+").unwrap();
    let text = "This text contains pattern_123 and pattern_456 and other content. ".repeat(100);

    c.bench_function("regex_pattern_matching", |b| {
        b.iter(|| {
            let matches: Vec<_> = pattern.find_iter(&text).collect();
            black_box(matches.len());
        });
    });
}

fn benchmark_text_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_analysis_by_size");

    for size in [1000, 10000, 100000].iter() {
        let text = format!("This is sample text with patterns and data structures. Word_{} appears multiple times. ", "pattern").repeat(*size / 100);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let words: Vec<&str> = text.split_whitespace().collect();
                let patterns: Vec<_> = words
                    .iter()
                    .filter(|word| word.contains("pattern") || word.contains("Word_"))
                    .collect();
                black_box(patterns.len());
            });
        });
    }

    group.finish();
}

fn benchmark_bulk_pattern_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_pattern_extraction");
    group.sample_size(10);

    for episode_count in [5, 20, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, &count| {
                b.to_async(TokioExecutor).iter(|| async {
                    let (memory, _temp_dir) = setup_temp_memory().await;
                    let context = create_benchmark_context();
                    let mut episode_ids = Vec::new();

                    // Create episodes with steps
                    for i in 0..count {
                        let episode_id = memory
                            .start_episode(
                                generate_episode_description(i),
                                context.clone(),
                                TaskType::CodeGeneration,
                            )
                            .await;

                        let steps = generate_execution_steps(3);
                        log_steps(&memory, episode_id, steps).await;

                        memory
                            .complete_episode(
                                episode_id,
                                TaskOutcome::Success {
                                    verdict: format!("Episode {} completed", i),
                                    artifacts: vec![format!("result_{}.rs", i)],
                                },
                            )
                            .await
                            .expect("Failed to complete episode");

                        episode_ids.push(episode_id);
                    }

                    // Extract patterns from all episodes
                    let extractor = PatternExtractor::new();
                    let mut total_patterns = 0;

                    for &episode_id in &episode_ids {
                        let episode = memory
                            .get_episode(episode_id)
                            .await
                            .expect("Failed to get episode");
                        let patterns = extractor.extract(&episode);
                        total_patterns += patterns.len();
                    }

                    black_box(total_patterns);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_data_filtering(c: &mut Criterion) {
    let data: Vec<String> = (0..10000)
        .map(|i| format!("item_{}_pattern_{}", i, i % 100))
        .collect();

    c.bench_function("data_filtering", |b| {
        b.iter(|| {
            let filtered: Vec<_> = data
                .iter()
                .filter(|item| item.contains("pattern_5") || item.contains("pattern_10"))
                .map(|item| item.to_uppercase())
                .collect();
            black_box(filtered.len());
        });
    });
}

criterion_group!(
    benches,
    benchmark_regex_pattern_matching,
    benchmark_text_analysis,
    benchmark_data_filtering,
    benchmark_bulk_pattern_extraction
);
criterion_main!(benches);
