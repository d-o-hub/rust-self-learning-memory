//! Benchmarks for spatiotemporal hierarchical indexing.
//!
//! Run with: cargo bench --package memory-core -- indexing

use chrono::{DateTime, Duration, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use memory_core::indexing::{HierarchicalIndex, HierarchicalQuery, SpatiotemporalIndex};
use memory_core::types::{ComplexityLevel, TaskContext, TaskType};
use memory_core::Episode;
use uuid::Uuid;

/// Create a test episode with a specific timestamp.
fn create_test_episode_with_time(
    domain: &str,
    task_type: TaskType,
    timestamp: DateTime<Utc>,
) -> Episode {
    let context = TaskContext {
        domain: domain.to_string(),
        complexity: ComplexityLevel::Simple,
        tags: vec![],
        ..Default::default()
    };
    let mut episode = Episode::new("Test episode".to_string(), context, task_type);
    episode.start_time = timestamp;
    episode
}

/// Linear scan implementation for comparison.
struct LinearScanIndex {
    episodes: Vec<(Uuid, DateTime<Utc>)>,
}

impl LinearScanIndex {
    fn new() -> Self {
        Self {
            episodes: Vec::new(),
        }
    }

    fn insert(&mut self, episode: &Episode) {
        self.episodes.push((episode.episode_id, episode.start_time));
    }

    fn query_range(&self, start: DateTime<Utc>, end: DateTime<Utc>, limit: usize) -> Vec<Uuid> {
        self.episodes
            .iter()
            .filter(|(_, timestamp)| *timestamp >= start && *timestamp < end)
            .take(limit)
            .map(|(id, _)| *id)
            .collect()
    }
}

fn bench_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion");

    for size in &[100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("spatiotemporal", size), size, |b, _| {
            let now = Utc::now();
            let episodes: Vec<_> = (0..*size)
                .map(|i| {
                    create_test_episode_with_time(
                        "web-api",
                        TaskType::CodeGeneration,
                        now - Duration::hours(i64::from(i)),
                    )
                })
                .collect();

            b.iter(|| {
                let mut index = SpatiotemporalIndex::new();
                for episode in &episodes {
                    index.insert(black_box(episode));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("hierarchical", size), size, |b, _| {
            let now = Utc::now();
            let episodes: Vec<_> = (0..*size)
                .map(|i| {
                    create_test_episode_with_time(
                        "web-api",
                        TaskType::CodeGeneration,
                        now - Duration::hours(i64::from(i)),
                    )
                })
                .collect();

            b.iter(|| {
                let mut index = HierarchicalIndex::new();
                for episode in &episodes {
                    index.insert(black_box(episode));
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("linear_scan", size), size, |b, _| {
            let now = Utc::now();
            let episodes: Vec<_> = (0..*size)
                .map(|i| {
                    create_test_episode_with_time(
                        "web-api",
                        TaskType::CodeGeneration,
                        now - Duration::hours(i64::from(i)),
                    )
                })
                .collect();

            b.iter(|| {
                let mut index = LinearScanIndex::new();
                for episode in &episodes {
                    index.insert(black_box(episode));
                }
            });
        });
    }

    group.finish();
}

fn bench_range_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_query");

    for size in &[100, 1000, 10000] {
        // Setup spatiotemporal index
        let now = Utc::now();
        let mut spatiotemporal = SpatiotemporalIndex::new();
        let episodes: Vec<_> = (0..*size)
            .map(|i| {
                create_test_episode_with_time(
                    "web-api",
                    TaskType::CodeGeneration,
                    now - Duration::hours(i64::from(i)),
                )
            })
            .collect();
        for episode in &episodes {
            spatiotemporal.insert(episode);
        }

        // Setup linear scan index
        let mut linear = LinearScanIndex::new();
        for episode in &episodes {
            linear.insert(episode);
        }

        let start = now - Duration::hours(10);
        let end = now;

        group.bench_with_input(BenchmarkId::new("spatiotemporal", size), size, |b, _| {
            b.iter(|| {
                let results = spatiotemporal.query_range(black_box(start), black_box(end), 100);
                black_box(results);
            });
        });

        group.bench_with_input(BenchmarkId::new("linear_scan", size), size, |b, _| {
            b.iter(|| {
                let results = linear.query_range(black_box(start), black_box(end), 100);
                black_box(results);
            });
        });
    }

    group.finish();
}

fn bench_hierarchical_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchical_query");

    for size in &[100, 1000, 10000] {
        let now = Utc::now();
        let mut index = HierarchicalIndex::new();

        // Insert episodes across multiple domains and task types
        for i in 0..*size {
            let domain = if i % 3 == 0 {
                "web-api"
            } else if i % 3 == 1 {
                "data-processing"
            } else {
                "cli"
            };
            let task_type = if i % 4 == 0 {
                TaskType::CodeGeneration
            } else if i % 4 == 1 {
                TaskType::Debugging
            } else if i % 4 == 2 {
                TaskType::Analysis
            } else {
                TaskType::Testing
            };

            let episode = create_test_episode_with_time(
                domain,
                task_type,
                now - Duration::hours(i64::from(i)),
            );
            index.insert(&episode);
        }

        // Query by domain only
        group.bench_with_input(BenchmarkId::new("query_by_domain", size), size, |b, _| {
            b.iter(|| {
                let results = index.query_by_domain(black_box("web-api"), 100);
                black_box(results);
            });
        });

        // Query by domain and task type
        group.bench_with_input(
            BenchmarkId::new("query_by_task_type", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = index.query_by_task_type(
                        black_box("web-api"),
                        TaskType::CodeGeneration,
                        100,
                    );
                    black_box(results);
                });
            },
        );

        // Query with hierarchical query builder
        let query = HierarchicalQuery::new()
            .with_domain("web-api")
            .with_task_type(TaskType::CodeGeneration)
            .with_limit(100);

        group.bench_with_input(
            BenchmarkId::new("hierarchical_query_builder", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = index.query(black_box(&query));
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    for size in &[100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("spatiotemporal_memory", size),
            size,
            |b, _| {
                let now = Utc::now();
                let episodes: Vec<_> = (0..*size)
                    .map(|i| {
                        create_test_episode_with_time(
                            "web-api",
                            TaskType::CodeGeneration,
                            now - Duration::hours(i64::from(i)),
                        )
                    })
                    .collect();

                b.iter(|| {
                    let mut index = SpatiotemporalIndex::new();
                    for episode in &episodes {
                        index.insert(episode);
                    }
                    black_box(index.memory_usage_estimate());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hierarchical_memory", size),
            size,
            |b, _| {
                let now = Utc::now();
                let episodes: Vec<_> = (0..*size)
                    .map(|i| {
                        create_test_episode_with_time(
                            "web-api",
                            TaskType::CodeGeneration,
                            now - Duration::hours(i64::from(i)),
                        )
                    })
                    .collect();

                b.iter(|| {
                    let mut index = HierarchicalIndex::new();
                    for episode in &episodes {
                        index.insert(episode);
                    }
                    black_box(index.memory_usage_estimate());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_insertion,
    bench_range_query,
    bench_hierarchical_query,
    bench_memory_overhead
);
criterion_main!(benches);
