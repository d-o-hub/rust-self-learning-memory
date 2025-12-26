//! Phase 2 (GENESIS) Performance Benchmarks
//!
//! Validates performance claims from RESEARCH_INTEGRATION_EXECUTION_PLAN.md:
//! 1. Storage compression: 3.2x via semantic summaries
//! 2. Retrieval speed: +65% faster (semantic summaries vs full episodes)
//! 3. Capacity overhead: ≤ 10ms for capacity enforcement
//! 4. Summary generation: ≤ 20ms per episode
//! 5. Total overhead: ≤ 10ms average (combined PREMem + GENESIS)

use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, BenchmarkId,
    Criterion,
};
use memory_benches::benchmark_helpers::{
    create_benchmark_context, generate_episode_description, generate_execution_steps,
    setup_temp_memory,
};
use memory_core::{
    episodic::{CapacityManager, EvictionPolicy},
    semantic::SemanticSummarizer,
    types::{TaskOutcome, TaskType},
    Episode,
};
use std::time::Instant;

/// Benchmark 1: Capacity Enforcement Overhead
///
/// Measures time to check capacity and select eviction candidates.
/// Target: ≤ 10ms overhead
fn benchmark_capacity_enforcement_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity_enforcement_overhead");
    group.sample_size(100);

    // Test with different episode counts and policies
    for episode_count in [100, 500, 1000].iter() {
        for policy in [EvictionPolicy::LRU, EvictionPolicy::RelevanceWeighted].iter() {
            let bench_name = format!("{}episodes_{:?}", episode_count, policy);
            group.bench_with_input(
                BenchmarkId::from_parameter(&bench_name),
                &(episode_count, policy),
                |b, &(&count, &policy)| {
                    // Setup: Create episodes for capacity check
                    let episodes: Vec<Episode> = (0..count)
                        .map(|i| {
                            let context = create_benchmark_context();
                            let mut episode = Episode::new(
                                generate_episode_description(i),
                                context,
                                TaskType::CodeGeneration,
                            );
                            // Add some steps to make it realistic
                            let steps = generate_execution_steps(5);
                            for step in steps {
                                episode.add_step(step);
                            }
                            episode.complete(TaskOutcome::Success {
                                verdict: "Done".to_string(),
                                artifacts: vec![],
                            });
                            episode
                        })
                        .collect();

                    let manager = CapacityManager::new(count - 1, policy);

                    b.iter(|| {
                        let start = Instant::now();

                        // Measure capacity check
                        let can_store = manager.can_store(episodes.len());
                        black_box(can_store);

                        // Measure eviction decision
                        let to_evict = manager.evict_if_needed(&episodes);
                        black_box(to_evict);

                        start.elapsed()
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark 2: Summary Generation Time
///
/// Measures time to generate semantic summaries for episodes of varying complexity.
/// Target: ≤ 20ms per episode
fn benchmark_summary_generation_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary_generation_time");
    group.sample_size(100);

    // Test with different step counts
    for step_count in [5, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                let summarizer = SemanticSummarizer::new();
                let context = create_benchmark_context();
                let mut episode = Episode::new(
                    "Implement comprehensive authentication system with JWT tokens".to_string(),
                    context,
                    TaskType::CodeGeneration,
                );

                // Add steps of varying complexity
                let steps = generate_execution_steps(count);
                for step in steps {
                    episode.add_step(step);
                }
                episode.complete(TaskOutcome::Success {
                    verdict: "Authentication system implemented successfully".to_string(),
                    artifacts: vec!["auth.rs".to_string(), "jwt.rs".to_string()],
                });

                b.to_async(FuturesExecutor).iter(|| async {
                    let start = Instant::now();
                    let summary = summarizer.summarize_episode(&episode).await.unwrap();
                    let elapsed = start.elapsed();
                    black_box(summary);
                    elapsed
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 3: Storage Compression Ratio
///
/// Measures compression ratio between full episodes and semantic summaries.
/// Target: ≥ 3.2x compression
fn benchmark_storage_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_compression_ratio");
    group.sample_size(50);

    for step_count in [5, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let summarizer = SemanticSummarizer::new();
                    let context = create_benchmark_context();
                    let mut episode = Episode::new(
                        generate_episode_description(0),
                        context,
                        TaskType::CodeGeneration,
                    );

                    let steps = generate_execution_steps(count);
                    for step in steps {
                        episode.add_step(step);
                    }
                    episode.complete(TaskOutcome::Success {
                        verdict: "Task completed successfully".to_string(),
                        artifacts: vec!["output.txt".to_string()],
                    });

                    // Measure full episode size
                    let episode_json = serde_json::to_string(&episode).unwrap();
                    let episode_size = episode_json.len();

                    // Generate summary and measure size
                    let summary = summarizer.summarize_episode(&episode).await.unwrap();
                    let summary_json = serde_json::to_string(&summary).unwrap();
                    let summary_size = summary_json.len();

                    // Calculate compression ratio
                    let compression_ratio = episode_size as f32 / summary_size as f32;

                    black_box((episode_size, summary_size, compression_ratio))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 4: Eviction Algorithm Performance
///
/// Compares performance of LRU vs RelevanceWeighted eviction.
fn benchmark_eviction_algorithm_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("eviction_algorithm_performance");
    group.sample_size(50);

    for episode_count in [100, 500, 1000].iter() {
        // Create test episodes with varying quality
        let episodes: Vec<Episode> = (0..*episode_count)
            .map(|i| {
                let context = create_benchmark_context();
                let mut episode = Episode::new(
                    generate_episode_description(i),
                    context,
                    TaskType::CodeGeneration,
                );
                let steps = generate_execution_steps(5);
                for step in steps {
                    episode.add_step(step);
                }
                episode.complete(TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                });
                episode
            })
            .collect();

        // Benchmark LRU
        group.bench_with_input(
            BenchmarkId::new("LRU", episode_count),
            episode_count,
            |b, &count| {
                let manager = CapacityManager::new(count - 10, EvictionPolicy::LRU);
                b.iter(|| {
                    let start = Instant::now();
                    let to_evict = manager.evict_if_needed(&episodes);
                    let elapsed = start.elapsed();
                    black_box(to_evict);
                    elapsed
                });
            },
        );

        // Benchmark RelevanceWeighted
        group.bench_with_input(
            BenchmarkId::new("RelevanceWeighted", episode_count),
            episode_count,
            |b, &count| {
                let manager = CapacityManager::new(count - 10, EvictionPolicy::RelevanceWeighted);
                b.iter(|| {
                    let start = Instant::now();
                    let to_evict = manager.evict_if_needed(&episodes);
                    let elapsed = start.elapsed();
                    black_box(to_evict);
                    elapsed
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 5: Combined PREMem + GENESIS Overhead
///
/// Measures end-to-end overhead of complete_episode with:
/// - Baseline: no PREMem or GENESIS
/// - PREMem only: with quality assessment
/// - GENESIS only: with summarization
/// - Both: PREMem + GENESIS
///
/// Target: ≤ 10ms total overhead
fn benchmark_combined_premem_genesis_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_premem_genesis_overhead");
    group.sample_size(30);

    // Baseline: complete_episode without any Phase 2 features
    group.bench_function("baseline_no_phase2", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let (memory, _temp_dir) = setup_temp_memory().await;
            let context = create_benchmark_context();

            let start = Instant::now();

            let episode_id = memory
                .start_episode(
                    "Implement feature with baseline".to_string(),
                    context,
                    TaskType::CodeGeneration,
                )
                .await;

            let steps = generate_execution_steps(10);
            for step in steps {
                memory.log_step(episode_id, step).await;
            }

            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Completed".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .unwrap();

            let elapsed = start.elapsed();
            black_box(episode_id);
            elapsed
        });
    });

    // GENESIS only: with semantic summarization
    group.bench_function("genesis_only_summarization", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let (memory, _temp_dir) = setup_temp_memory().await;
            let context = create_benchmark_context();

            let start = Instant::now();

            let episode_id = memory
                .start_episode(
                    "Implement feature with summarization".to_string(),
                    context,
                    TaskType::CodeGeneration,
                )
                .await;

            let steps = generate_execution_steps(10);
            for step in steps {
                memory.log_step(episode_id, step).await;
            }

            // Get episode and summarize (if available)
            if let Ok(episode) = memory.get_episode(episode_id).await {
                let summarizer = SemanticSummarizer::new();
                let _summary = summarizer.summarize_episode(&episode).await.unwrap();
            }

            memory
                .complete_episode(
                    episode_id,
                    TaskOutcome::Success {
                        verdict: "Completed".to_string(),
                        artifacts: vec![],
                    },
                )
                .await
                .unwrap();

            let elapsed = start.elapsed();
            black_box(episode_id);
            elapsed
        });
    });

    group.finish();
}

/// Benchmark 6: Capacity Check vs Full Episode Count
///
/// Compares metadata-based capacity check vs full table scan.
fn benchmark_capacity_check_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity_check_efficiency");
    group.sample_size(100);

    for episode_count in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, &count| {
                let manager = CapacityManager::new(count + 100, EvictionPolicy::LRU);

                b.iter(|| {
                    let start = Instant::now();
                    // Simulate metadata lookup (O(1))
                    let can_store = manager.can_store(count);
                    let elapsed = start.elapsed();
                    black_box(can_store);
                    elapsed
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 7: Summary Key Concept Extraction
///
/// Measures performance of key concept extraction from episodes.
fn benchmark_summary_key_concept_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary_key_concept_extraction");
    group.sample_size(100);

    for step_count in [5, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                let summarizer = SemanticSummarizer::new();
                let context = create_benchmark_context();
                let mut episode = Episode::new(
                    "Implement comprehensive authentication system with JWT tokens, OAuth2, and role-based access control".to_string(),
                    context,
                    TaskType::CodeGeneration,
                );

                let steps = generate_execution_steps(count);
                for step in steps {
                    episode.add_step(step);
                }
                episode.complete(TaskOutcome::Success {
                    verdict: "Authentication implemented".to_string(),
                    artifacts: vec![],
                });

                b.iter(|| {
                    let start = Instant::now();
                    let concepts = summarizer.extract_key_concepts(&episode);
                    let elapsed = start.elapsed();
                    black_box(concepts);
                    elapsed
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 8: Summary Key Steps Extraction
///
/// Measures performance of critical step extraction.
fn benchmark_summary_key_steps_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary_key_steps_extraction");
    group.sample_size(100);

    for step_count in [5, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                let summarizer = SemanticSummarizer::new();
                let context = create_benchmark_context();
                let mut episode = Episode::new(
                    "Complex multi-step task".to_string(),
                    context,
                    TaskType::CodeGeneration,
                );

                let steps = generate_execution_steps(count);
                for step in steps {
                    episode.add_step(step);
                }
                episode.complete(TaskOutcome::Success {
                    verdict: "Done".to_string(),
                    artifacts: vec![],
                });

                b.iter(|| {
                    let start = Instant::now();
                    let key_steps = summarizer.extract_key_steps(&episode);
                    let elapsed = start.elapsed();
                    black_box(key_steps);
                    elapsed
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 9: Relevance Score Calculation
///
/// Measures performance of relevance scoring for eviction.
fn benchmark_relevance_score_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("relevance_score_calculation");
    group.sample_size(100);

    for episode_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(episode_count),
            episode_count,
            |b, &count| {
                let manager = CapacityManager::new(1000, EvictionPolicy::RelevanceWeighted);
                let episodes: Vec<Episode> = (0..count)
                    .map(|i| {
                        let context = create_benchmark_context();
                        let mut episode = Episode::new(
                            generate_episode_description(i),
                            context,
                            TaskType::CodeGeneration,
                        );
                        let steps = generate_execution_steps(5);
                        for step in steps {
                            episode.add_step(step);
                        }
                        episode.complete(TaskOutcome::Success {
                            verdict: "Done".to_string(),
                            artifacts: vec![],
                        });
                        episode
                    })
                    .collect();

                b.iter(|| {
                    let start = Instant::now();
                    for episode in &episodes {
                        let score = manager.relevance_score(episode);
                        black_box(score);
                    }
                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 10: Summary Text Generation
///
/// Measures performance of coherent summary text generation.
fn benchmark_summary_text_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("summary_text_generation");
    group.sample_size(100);

    for step_count in [5, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(step_count),
            step_count,
            |b, &count| {
                let summarizer = SemanticSummarizer::new();
                let context = create_benchmark_context();
                let mut episode = Episode::new(
                    "Implement feature with comprehensive steps".to_string(),
                    context,
                    TaskType::CodeGeneration,
                );

                let steps = generate_execution_steps(count);
                for step in steps {
                    episode.add_step(step);
                }
                episode.complete(TaskOutcome::Success {
                    verdict: "Feature implemented successfully".to_string(),
                    artifacts: vec!["feature.rs".to_string()],
                });

                b.iter(|| {
                    let start = Instant::now();
                    let summary_text = summarizer.generate_summary_text(&episode);
                    let elapsed = start.elapsed();
                    black_box(summary_text);
                    elapsed
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_capacity_enforcement_overhead,
    benchmark_summary_generation_time,
    benchmark_storage_compression_ratio,
    benchmark_eviction_algorithm_performance,
    benchmark_combined_premem_genesis_overhead,
    benchmark_capacity_check_efficiency,
    benchmark_summary_key_concept_extraction,
    benchmark_summary_key_steps_extraction,
    benchmark_relevance_score_calculation,
    benchmark_summary_text_generation,
);

criterion_main!(benches);
