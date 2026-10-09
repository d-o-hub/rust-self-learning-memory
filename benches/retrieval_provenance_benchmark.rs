//! Retrieval provenance accounting benchmarks (issue #1079).
//!
//! Validates that the execution-backed provenance path performs exactly one
//! query-cache lookup per operation and measures hit vs. executed-pipeline
//! overhead.
//!
//! Run with: `cargo bench --bench retrieval_provenance_benchmark`

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use do_memory_core::{ComplexityLevel, SelfLearningMemory, TaskContext, TaskOutcome, TaskType};
use std::hint::black_box;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// Shared multi-threaded runtime for the async retrieval path.
fn rt() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create runtime"))
}

fn context() -> TaskContext {
    TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string()],
    }
}

/// Build an in-memory store seeded with `count` completed episodes.
async fn seeded_memory(count: usize) -> SelfLearningMemory {
    let memory = SelfLearningMemory::new();
    for i in 0..count {
        let id = memory
            .start_episode(
                format!("Implement rust web api feature {i}"),
                context(),
                TaskType::CodeGeneration,
            )
            .await;
        memory
            .complete_episode(
                id,
                TaskOutcome::Success {
                    verdict: "ok".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .expect("complete episode");
    }
    memory
}

/// Measure the cache-hit provenance path (single lookup, no pipeline run).
fn bench_provenance_cache_hit(c: &mut Criterion) {
    let memory = rt().block_on(seeded_memory(5));
    let query = "implement rust web api with axum".to_string();

    // Warm the query cache so the measured call is served from cache.
    let warm = rt().block_on(memory.retrieve_relevant_context_with_provenance(
        query.clone(),
        context(),
        5,
    ));
    assert!(warm.provenance.cache_hit, "warm-up must populate the cache");

    c.bench_function("provenance_cache_hit", |b| {
        b.iter(|| {
            let out = rt().block_on(memory.retrieve_relevant_context_with_provenance(
                black_box(query.clone()),
                context(),
                5,
            ));
            assert!(out.provenance.cache_hit);
            assert!(!out.provenance.executed);
            black_box(out.provenance.tier)
        });
    });
}

/// Measure a cold/executed provenance request (keyword pipeline, one miss).
fn bench_provenance_executed_miss(c: &mut Criterion) {
    let memory = rt().block_on(seeded_memory(5));

    c.bench_function("provenance_executed_miss", |b| {
        let mut n: u64 = 0;
        b.iter(|| {
            n += 1;
            let out = rt().block_on(memory.retrieve_relevant_context_with_provenance(
                black_box(format!("uncached provenance query {n}")),
                context(),
                5,
            ));
            assert!(out.provenance.executed);
            black_box(out.provenance.candidate_count)
        });
    });
}

criterion_group!(
    provenance_benches,
    bench_provenance_cache_hit,
    bench_provenance_executed_miss
);
criterion_main!(provenance_benches);
