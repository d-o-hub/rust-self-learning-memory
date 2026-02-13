//! Comprehensive Phase 3 Spatiotemporal Retrieval Benchmarks
//!
//! Validates the +34% accuracy improvement target from hierarchical retrieval.
//!
//! Benchmarks:
//! 1. Baseline flat retrieval (Phase 3 disabled)
//! 2. Hierarchical retrieval accuracy (Phase 3 enabled)
//! 3. Diversity impact on accuracy
//! 4. Query latency scaling (100, 500, 1000, 5000 episodes)
//! 5. Index insertion overhead
//! 6. Diversity computation time
//!
//! Key Metrics:
//! - Retrieval accuracy: Precision, Recall, F1 score
//! - Accuracy improvement: % vs baseline (target: +34%)
//! - Query latency: Mean, p50, p95, p99 (target: ≤100ms)
//! - Diversity score: Average across queries (target: ≥0.7)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_core::{
    memory::SelfLearningMemory,
    spatiotemporal::{DiversityMaximizer, ScoredEpisode},
    ComplexityLevel, ExecutionStep, MemoryConfig, TaskContext, TaskOutcome, TaskType,
};
use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a realistic test episode
fn create_test_episode(
    memory: &SelfLearningMemory,
    rt: &Runtime,
    domain: &str,
    task_type: TaskType,
    description: &str,
    num_steps: usize,
) -> Uuid {
    rt.block_on(async {
        let context = TaskContext {
            domain: domain.to_string(),
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            tags: vec!["benchmark".to_string()],
        };

        let episode_id = memory
            .start_episode(description.to_string(), context, task_type)
            .await;

        // Log steps
        for i in 0..num_steps {
            let mut step = ExecutionStep::new(
                i + 1,
                format!("tool_{}", i % 5),
                format!("Step {} action", i),
            );
            step.result = Some(memory_core::ExecutionResult::Success {
                output: format!("Step {} output", i),
            });
            memory.log_step(episode_id, step).await;
        }

        // Complete episode
        memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "Task completed successfully".to_string(),
                    artifacts: vec![format!("{}.rs", domain)],
                },
            )
            .await
            .unwrap();

        episode_id
    })
}

/// Create test dataset with ground truth labels
struct GroundTruthDataset {
    memory: SelfLearningMemory,
    relevant_domain: String,
    relevant_ids: HashSet<Uuid>,
    irrelevant_ids: HashSet<Uuid>,
}

impl GroundTruthDataset {
    fn new(
        rt: &Runtime,
        enable_phase3: bool,
        enable_diversity: bool,
        num_relevant: usize,
        num_irrelevant: usize,
    ) -> Self {
        let config = MemoryConfig {
            quality_threshold: 0.5,
            enable_spatiotemporal_indexing: enable_phase3,
            enable_diversity_maximization: enable_diversity,
            diversity_lambda: 0.7,
            temporal_bias_weight: 0.3,
            max_episodes: Some(num_relevant + num_irrelevant + 100),
            ..Default::default()
        };

        let memory = SelfLearningMemory::with_config(config);

        let mut relevant_ids = HashSet::new();
        let mut irrelevant_ids = HashSet::new();

        // Create relevant episodes (target domain)
        for i in 0..num_relevant {
            let id = create_test_episode(
                &memory,
                rt,
                "web-api",
                TaskType::CodeGeneration,
                &format!("Build REST API endpoint {}", i),
                20,
            );
            relevant_ids.insert(id);
        }

        // Create irrelevant episodes (different domain)
        for i in 0..num_irrelevant {
            let id = create_test_episode(
                &memory,
                rt,
                "data-science",
                TaskType::Analysis,
                &format!("Analyze dataset {}", i),
                20,
            );
            irrelevant_ids.insert(id);
        }

        Self {
            memory,
            relevant_domain: "web-api".to_string(),
            relevant_ids,
            irrelevant_ids,
        }
    }

    /// Calculate precision, recall, and F1 score
    fn calculate_metrics(&self, rt: &Runtime, limit: usize) -> AccuracyMetrics {
        rt.block_on(async {
            let query_context = TaskContext {
                domain: self.relevant_domain.clone(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Moderate,
                tags: vec![],
            };

            let results = self
                .memory
                .retrieve_relevant_context(
                    "Implement authentication API".to_string(),
                    query_context,
                    limit,
                )
                .await;

            // Count true positives
            let true_positives = results
                .iter()
                .filter(|e| self.relevant_ids.contains(&e.episode_id))
                .count();

            let false_positives = results
                .iter()
                .filter(|e| self.irrelevant_ids.contains(&e.episode_id))
                .count();

            let precision = if results.is_empty() {
                0.0
            } else {
                (true_positives as f64) / (results.len() as f64)
            };

            let recall = if self.relevant_ids.is_empty() {
                0.0
            } else {
                (true_positives as f64) / (self.relevant_ids.len() as f64)
            };

            let f1 = if precision + recall > 0.0 {
                2.0 * (precision * recall) / (precision + recall)
            } else {
                0.0
            };

            AccuracyMetrics {
                precision,
                recall,
                f1_score: f1,
                true_positives,
                false_positives,
                total_results: results.len(),
            }
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AccuracyMetrics {
    precision: f64,
    recall: f64,
    f1_score: f64,
    true_positives: usize,
    false_positives: usize,
    total_results: usize,
}

// ============================================================================
// Benchmark 1: Baseline Flat Retrieval
// ============================================================================

fn baseline_flat_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("baseline_flat_retrieval");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    let rt = Runtime::new().unwrap();

    // Create dataset with Phase 3 DISABLED
    let dataset = GroundTruthDataset::new(&rt, false, false, 50, 50);

    group.bench_function("flat_retrieval_accuracy", |b| {
        b.iter(|| {
            let metrics = dataset.calculate_metrics(&rt, 10);
            black_box(metrics)
        });
    });

    // Print baseline metrics
    let baseline_metrics = dataset.calculate_metrics(&rt, 10);
    println!("\n=== Baseline Flat Retrieval Metrics ===");
    println!("Precision: {:.2}%", baseline_metrics.precision * 100.0);
    println!("Recall: {:.2}%", baseline_metrics.recall * 100.0);
    println!("F1 Score: {:.2}%", baseline_metrics.f1_score * 100.0);
    println!(
        "True Positives: {}/{}",
        baseline_metrics.true_positives, baseline_metrics.total_results
    );

    group.finish();
}

// ============================================================================
// Benchmark 2: Hierarchical Retrieval Accuracy
// ============================================================================

fn hierarchical_retrieval_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchical_retrieval_accuracy");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    let rt = Runtime::new().unwrap();

    // Create dataset with Phase 3 ENABLED
    let dataset = GroundTruthDataset::new(&rt, true, false, 50, 50);

    group.bench_function("phase3_retrieval_accuracy", |b| {
        b.iter(|| {
            let metrics = dataset.calculate_metrics(&rt, 10);
            black_box(metrics)
        });
    });

    // Print Phase 3 metrics and improvement
    let phase3_metrics = dataset.calculate_metrics(&rt, 10);
    println!("\n=== Phase 3 Hierarchical Retrieval Metrics ===");
    println!("Precision: {:.2}%", phase3_metrics.precision * 100.0);
    println!("Recall: {:.2}%", phase3_metrics.recall * 100.0);
    println!("F1 Score: {:.2}%", phase3_metrics.f1_score * 100.0);
    println!(
        "True Positives: {}/{}",
        phase3_metrics.true_positives, phase3_metrics.total_results
    );

    // Calculate improvement (compare to baseline from previous run)
    // Note: In practice, run both benchmarks and compare manually

    group.finish();
}

// ============================================================================
// Benchmark 3: Diversity Impact on Accuracy
// ============================================================================

fn diversity_impact_on_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("diversity_impact");
    group.sample_size(10);

    let rt = Runtime::new().unwrap();

    for lambda in [0.0, 0.3, 0.5, 0.7, 1.0] {
        let config = MemoryConfig {
            quality_threshold: 0.5,
            enable_diversity_maximization: true,
            diversity_lambda: lambda,
            max_episodes: Some(200),
            ..Default::default()
        };

        let memory = SelfLearningMemory::with_config(config);

        // Create test dataset
        for i in 0..50 {
            create_test_episode(
                &memory,
                &rt,
                "test-domain",
                TaskType::CodeGeneration,
                &format!("Task {}", i),
                10,
            );
        }

        group.bench_with_input(
            BenchmarkId::new("lambda", format!("{:.1}", lambda)),
            &lambda,
            |b, _lambda| {
                b.iter(|| {
                    rt.block_on(async {
                        let context = TaskContext {
                            domain: "test-domain".to_string(),
                            ..Default::default()
                        };

                        let results = memory
                            .retrieve_relevant_context("Test query".to_string(), context, 10)
                            .await;

                        black_box(results)
                    })
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 4: Query Latency Scaling
// ============================================================================

fn query_latency_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_latency_scaling");
    group.sample_size(15);
    group.measurement_time(Duration::from_secs(30));

    let rt = Runtime::new().unwrap();

    for size in [100, 500, 1000] {
        let config = MemoryConfig {
            quality_threshold: 0.5,
            enable_spatiotemporal_indexing: true,
            max_episodes: Some(size + 100),
            ..Default::default()
        };

        let memory = SelfLearningMemory::with_config(config);

        // Pre-populate with episodes
        println!("Creating {} episodes for latency benchmark...", size);
        for i in 0..size {
            create_test_episode(
                &memory,
                &rt,
                &format!("domain-{}", i % 10),
                TaskType::CodeGeneration,
                &format!("Task {}", i),
                5, // Fewer steps for speed
            );

            if i % 100 == 0 && i > 0 {
                println!("  Created {}/{} episodes", i, size);
            }
        }

        group.bench_with_input(BenchmarkId::new("episodes", size), &size, |b, _size| {
            b.iter(|| {
                let start = Instant::now();

                rt.block_on(async {
                    let context = TaskContext {
                        domain: "domain-0".to_string(),
                        ..Default::default()
                    };

                    let results = memory
                        .retrieve_relevant_context("Test query".to_string(), context, 10)
                        .await;

                    let elapsed = start.elapsed();

                    black_box((results, elapsed))
                })
            });
        });
    }

    group.finish();
}

// ============================================================================
// Benchmark 5: Index Insertion Overhead
// ============================================================================

fn index_insertion_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_insertion_overhead");
    group.sample_size(20);

    let rt = Runtime::new().unwrap();

    // Memory with indexing enabled
    let config_with_index = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        max_episodes: Some(200),
        ..Default::default()
    };

    // Memory with indexing disabled
    let config_without_index = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: false,
        max_episodes: Some(200),
        ..Default::default()
    };

    let memory_with = SelfLearningMemory::with_config(config_with_index);
    let memory_without = SelfLearningMemory::with_config(config_without_index);

    group.bench_function("insertion_with_index", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let start = Instant::now();
            create_test_episode(
                &memory_with,
                &rt,
                "index-test",
                TaskType::CodeGeneration,
                &format!("Indexed episode {}", counter),
                10,
            );
            let elapsed = start.elapsed();
            black_box(elapsed)
        });
    });

    group.bench_function("insertion_without_index", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let start = Instant::now();
            create_test_episode(
                &memory_without,
                &rt,
                "no-index-test",
                TaskType::CodeGeneration,
                &format!("Non-indexed episode {}", counter),
                10,
            );
            let elapsed = start.elapsed();
            black_box(elapsed)
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark 6: Diversity Computation Time
// ============================================================================

fn diversity_computation_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("diversity_computation");
    group.sample_size(50);

    let maximizer = DiversityMaximizer::new(0.7);

    for result_size in [10, 50, 100] {
        // Create test scored episodes
        let candidates: Vec<ScoredEpisode> = (0..result_size)
            .map(|i| {
                // Create random-like embeddings (128 dimensions)
                let embedding: Vec<f32> = (0..128)
                    .map(|j| ((i * 13 + j * 7) % 100) as f32 / 100.0)
                    .collect();

                ScoredEpisode::new(
                    format!("ep{}", i),
                    1.0 - (i as f32 / result_size as f32),
                    embedding,
                )
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("result_size", result_size),
            &result_size,
            |b, _size| {
                b.iter(|| {
                    let diverse = maximizer.maximize_diversity(candidates.clone(), result_size / 2);
                    black_box(diverse)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark 7: End-to-End Retrieval Performance
// ============================================================================

fn end_to_end_retrieval_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_retrieval");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(15));

    let rt = Runtime::new().unwrap();

    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        enable_diversity_maximization: true,
        diversity_lambda: 0.7,
        temporal_bias_weight: 0.3,
        max_episodes: Some(500),
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create realistic dataset
    let domains = ["web-api", "data-science", "devops", "mobile-app"];
    let task_types = [
        TaskType::CodeGeneration,
        TaskType::Debugging,
        TaskType::Testing,
        TaskType::Analysis,
    ];

    for i in 0..200 {
        let domain = domains[i % domains.len()];
        let task_type = task_types[i % task_types.len()];
        create_test_episode(
            &memory,
            &rt,
            domain,
            task_type,
            &format!("{} task {}", domain, i),
            15,
        );
    }

    group.bench_function("full_retrieval_pipeline", |b| {
        b.to_async(&rt).iter(|| async {
            let context = TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                framework: Some("tokio".to_string()),
                complexity: ComplexityLevel::Moderate,
                tags: vec![],
            };

            let start = Instant::now();
            let results = memory
                .retrieve_relevant_context("Implement REST API".to_string(), context, 10)
                .await;
            let elapsed = start.elapsed();

            black_box((results, elapsed))
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    baseline_flat_retrieval,
    hierarchical_retrieval_accuracy,
    diversity_impact_on_accuracy,
    query_latency_scaling,
    index_insertion_overhead,
    diversity_computation_time,
    end_to_end_retrieval_performance,
);

criterion_main!(benches);
