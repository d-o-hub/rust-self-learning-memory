//! Retrieval Quality Benchmarks
//!
//! Benchmarks for retrieval quality using standard metrics from MTEB/BEIR methodology.
//! Retrieval-first evaluation: measures retrieval quality without LLM calls.
//!
//! Run with: `cargo bench --bench retrieval_quality_benchmark`

use criterion::{Criterion, criterion_group, criterion_main};
use do_memory_core::search::metrics::{
    hit_rate_at_k, map, mrr, ndcg_at_k, precision_at_k, recall_at_k, reciprocal_rank_fusion,
};
use rand::RngExt;
use rand::rng;
use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Duration;

/// Generate synthetic retrieval benchmark data
struct RetrievalBenchmark {
    /// Number of items in corpus (used for scaling validation)
    #[allow(dead_code)]
    corpus_size: usize,
    /// Number of queries
    num_queries: usize,
    /// Retrieved results per query (item IDs)
    retrieved: Vec<Vec<usize>>,
    /// Ground truth relevance (item ID -> relevance score 0-3)
    relevance: Vec<HashMap<usize, f64>>,
    /// Relevant item sets (binary)
    relevant_sets: Vec<HashSet<usize>>,
}

impl RetrievalBenchmark {
    fn new(corpus_size: usize, num_queries: usize, top_k: usize) -> Self {
        let mut rng = rng();
        let mut retrieved = Vec::with_capacity(num_queries);
        let mut relevance = Vec::with_capacity(num_queries);
        let mut relevant_sets = Vec::with_capacity(num_queries);

        for _ in 0..num_queries {
            // Generate retrieved results (simulating search results)
            let mut query_retrieved: Vec<usize> = (0..top_k)
                .map(|_| rng.random_range(0..corpus_size))
                .collect();
            query_retrieved.sort();
            query_retrieved.dedup();

            // Generate ground truth (simulate ~10-20% of corpus being relevant)
            let num_relevant = (corpus_size as f64 * 0.15) as usize;
            let mut query_rel = HashMap::new();
            let mut query_relevant_set = HashSet::new();

            for _ in 0..num_relevant {
                let item_id = rng.random_range(0..corpus_size);
                let rel_score: f64 = rng.random_range(1.0..4.0); // 1-3 relevance
                query_rel.insert(item_id, rel_score.floor());
                query_relevant_set.insert(item_id);
            }

            retrieved.push(query_retrieved);
            relevance.push(query_rel);
            relevant_sets.push(query_relevant_set);
        }

        Self {
            corpus_size,
            num_queries,
            retrieved,
            relevance,
            relevant_sets,
        }
    }
}

/// Benchmark Recall@k calculation
fn bench_recall_at_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("recall_at_k");
    group.measurement_time(Duration::from_secs(3));

    let bench = RetrievalBenchmark::new(10_000, 100, 100);

    for k in [10, 20, 50, 100] {
        group.bench_function(format!("k_{}", k), |b| {
            b.iter(|| {
                let mut total_recall = 0.0;
                for (retrieved, relevant) in bench.retrieved.iter().zip(bench.relevant_sets.iter())
                {
                    total_recall += recall_at_k(retrieved, relevant, k);
                }
                black_box(total_recall / bench.num_queries as f64)
            })
        });
    }

    group.finish();
}

/// Benchmark NDCG@k calculation
fn bench_ndcg_at_k(c: &mut Criterion) {
    let mut group = c.benchmark_group("ndcg_at_k");
    group.measurement_time(Duration::from_secs(3));

    let bench = RetrievalBenchmark::new(10_000, 100, 100);

    for k in [10, 20, 50, 100] {
        group.bench_function(format!("k_{}", k), |b| {
            b.iter(|| {
                let mut total_ndcg = 0.0;
                for (retrieved, rel_scores) in bench.retrieved.iter().zip(bench.relevance.iter()) {
                    total_ndcg += ndcg_at_k(retrieved, rel_scores, k);
                }
                black_box(total_ndcg / bench.num_queries as f64)
            })
        });
    }

    group.finish();
}

/// Benchmark MRR calculation
fn bench_mrr(c: &mut Criterion) {
    let mut group = c.benchmark_group("mrr");
    group.measurement_time(Duration::from_secs(3));

    for num_queries in [100, 1_000, 10_000] {
        let bench = RetrievalBenchmark::new(10_000, num_queries, 100);

        group.bench_function(format!("queries_{}", num_queries), |b| {
            b.iter(|| black_box(mrr(&bench.retrieved, &bench.relevant_sets)))
        });
    }

    group.finish();
}

/// Benchmark MAP calculation
fn bench_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("map");
    group.measurement_time(Duration::from_secs(3));

    for num_queries in [100, 1_000, 10_000] {
        let bench = RetrievalBenchmark::new(10_000, num_queries, 100);

        group.bench_function(format!("queries_{}", num_queries), |b| {
            b.iter(|| black_box(map(&bench.retrieved, &bench.relevant_sets)))
        });
    }

    group.finish();
}

/// Benchmark Reciprocal Rank Fusion
fn bench_rrf(c: &mut Criterion) {
    let mut group = c.benchmark_group("reciprocal_rank_fusion");
    group.measurement_time(Duration::from_secs(3));

    let mut rng = rng();

    for num_lists in [2, 5, 10] {
        for list_size in [100, 1_000, 10_000] {
            // Generate mock result lists
            let result_lists: Vec<Vec<(usize, f32)>> = (0..num_lists)
                .map(|_| {
                    let mut list: Vec<(usize, f32)> = (0..list_size)
                        .map(|i| {
                            let score = rng.random_range(0.0..1.0);
                            (i, score)
                        })
                        .collect();
                    list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    list
                })
                .collect();

            group.bench_function(format!("lists_{}_size_{}", num_lists, list_size), |b| {
                b.iter(|| black_box(reciprocal_rank_fusion(&result_lists, 60)))
            });
        }
    }

    group.finish();
}

/// Benchmark all metrics together (realistic workload)
fn bench_all_metrics_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_metrics_combined");
    group.measurement_time(Duration::from_secs(5));

    let bench = RetrievalBenchmark::new(10_000, 100, 100);

    group.bench_function("full_evaluation", |b| {
        b.iter(|| {
            let k = 10;
            let mut results = HashMap::new();

            for (retrieved, (rel_scores, relevant)) in bench
                .retrieved
                .iter()
                .zip(bench.relevance.iter().zip(bench.relevant_sets.iter()))
            {
                let recall = recall_at_k(retrieved, relevant, k);
                let precision = precision_at_k(retrieved, relevant, k);
                let ndcg = ndcg_at_k(retrieved, rel_scores, k);
                let hit = hit_rate_at_k(retrieved, relevant, k);

                *results.entry("recall").or_insert(0.0) += recall;
                *results.entry("precision").or_insert(0.0) += precision;
                *results.entry("ndcg").or_insert(0.0) += ndcg;
                *results.entry("hit_rate").or_insert(0.0) += hit;
            }

            // Add MRR and MAP
            results.insert("mrr", mrr(&bench.retrieved, &bench.relevant_sets));
            results.insert("map", map(&bench.retrieved, &bench.relevant_sets));

            black_box(results)
        })
    });

    group.finish();
}

/// Benchmark metric calculation at different corpus sizes
fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_scaling");

    for corpus_size in [1_000, 10_000, 100_000] {
        let bench = RetrievalBenchmark::new(corpus_size, 100, 100);

        group.bench_function(format!("recall_corpus_{}", corpus_size), |b| {
            b.iter(|| {
                let mut total = 0.0;
                for (retrieved, relevant) in bench.retrieved.iter().zip(bench.relevant_sets.iter())
                {
                    total += recall_at_k(retrieved, relevant, 10);
                }
                black_box(total)
            })
        });

        group.bench_function(format!("ndcg_corpus_{}", corpus_size), |b| {
            b.iter(|| {
                let mut total = 0.0;
                for (retrieved, rel_scores) in bench.retrieved.iter().zip(bench.relevance.iter()) {
                    total += ndcg_at_k(retrieved, rel_scores, 10);
                }
                black_box(total)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_recall_at_k,
    bench_ndcg_at_k,
    bench_mrr,
    bench_map,
    bench_rrf,
    bench_all_metrics_combined,
    bench_scaling,
);
criterion_main!(benches);
