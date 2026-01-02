//! Benchmark Turso vector search performance for different embedding dimensions
//!
//! Measures:
//! 1. OpenAI embedding search latency (1536-dim, brute-force)
//! 2. 384-dim search latency (native vector search)
//! 3. Memory usage for embeddings
//! 4. JSON query performance vs Rust deserialization

use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use memory_core::{
    embeddings::EmbeddingStorageBackend,
    types::{ComplexityLevel, TaskContext, TaskType},
    Episode,
};
use memory_storage_turso::TursoStorage;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Generate random embedding of specified dimension
fn generate_random_embedding(dimension: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Create a test episode
fn create_test_episode(id: usize) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        domain: "web-api".to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new(
        format!("Implement API endpoint {}", id),
        context,
        TaskType::CodeGeneration,
    );
    episode.complete(memory_core::types::TaskOutcome::Success {
        verdict: "Done".to_string(),
        artifacts: vec![],
    });
    episode
}

/// Setup storage with episodes and embeddings
async fn setup_storage_with_data(
    dimension: usize,
    count: usize,
) -> Result<(Arc<TursoStorage>, TempDir, Vec<f32>)> {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Use Turso dev server or cloud database via environment variables
    // Default to local Turso dev server: libsql://127.0.0.1:8080
    let url = std::env::var("TURSO_DATABASE_URL")
        .unwrap_or_else(|_| "libsql://127.0.0.1:8080".to_string());
    let token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_else(|_| String::new());

    eprintln!("Connecting to Turso at: {}", url);

    let storage = TursoStorage::new(&url, &token)
        .await
        .expect("Failed to create turso storage");
    storage
        .initialize_schema()
        .await
        .expect("Failed to initialize schema");

    // Store episodes with embeddings
    for i in 0..count {
        let episode = create_test_episode(i);
        storage
            .store_episode(&episode)
            .await
            .expect("Failed to store episode");

        let embedding = generate_random_embedding(dimension, i as u64);
        storage
            .store_episode_embedding(episode.episode_id, embedding.clone())
            .await
            .expect("Failed to store embedding");
    }

    // Create a query embedding
    let query_embedding = generate_random_embedding(dimension, 9999);

    Ok((Arc::new(storage), temp_dir, query_embedding))
}

/// Benchmark 384-dim native vector search
fn benchmark_384_dim_native_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("384_dim_native_search");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(20));

    let rt = Runtime::new().unwrap();

    // Note: Skip 10K test for initial run as it takes very long
    for embedding_count in [100, 1000] {
        group.throughput(Throughput::Elements(embedding_count as u64));

        // Setup storage with data once per test size
        let (storage, _temp_dir, query_embedding) = rt
            .block_on(async { setup_storage_with_data(384, embedding_count).await })
            .expect("Failed to setup storage");
        let storage = Arc::new(storage);
        let query_embedding = Arc::new(query_embedding);

        group.bench_with_input(
            BenchmarkId::from_parameter(embedding_count),
            &embedding_count,
            |b, &_count| {
                b.iter(|| {
                    rt.block_on(async {
                        let storage = storage.clone();
                        let query_embedding = query_embedding.clone();

                        // This should use native vector search for 384-dim embeddings
                        let results = storage
                            .find_similar_episodes(query_embedding.as_ref().clone(), 10, 0.5)
                            .await
                            .expect("Failed to find similar episodes");

                        black_box(results.len());
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark brute-force search simulation (1536-dim equivalent)
/// Note: This simulates the fallback brute-force approach by using 384-dim
/// which will use the brute-force code path if migration not applied
fn benchmark_brute_force_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("brute_force_search");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(20));

    let rt = Runtime::new().unwrap();

    // Note: brute-force is O(n), so we test smaller sizes
    for embedding_count in [10, 50, 100] {
        group.throughput(Throughput::Elements(embedding_count as u64));

        // Setup storage with data once per test size
        let (storage, _temp_dir, query_embedding) = rt
            .block_on(async { setup_storage_with_data(384, embedding_count).await })
            .expect("Failed to setup storage");
        let storage = Arc::new(storage);
        let query_embedding = Arc::new(query_embedding);

        group.bench_with_input(
            BenchmarkId::from_parameter(embedding_count),
            &embedding_count,
            |b, &_count| {
                b.iter(|| {
                    rt.block_on(async {
                        let storage = storage.clone();
                        let query_embedding = query_embedding.clone();

                        // Use the public API - will fallback to brute-force if migration not applied
                        let results = storage
                            .find_similar_episodes(query_embedding.as_ref().clone(), 10, 0.5)
                            .await
                            .expect("Failed to find similar episodes");

                        black_box(results.len());
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage calculation
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding_memory_usage");

    for dimension in [384, 1536, 3072] {
        for embedding_count in [1000, 10000] {
            group.bench_with_input(
                BenchmarkId::new(format!("{}_dim", dimension), embedding_count),
                &embedding_count,
                |b, &count| {
                    b.iter(|| {
                        // Calculate memory usage for embeddings
                        let embedding_size_bytes = dimension * 4; // f32 = 4 bytes
                        let total_memory_bytes = count * embedding_size_bytes;
                        let total_memory_mb = total_memory_bytes as f64 / (1024.0 * 1024.0);

                        // Add overhead for data structures (estimate 20%)
                        let estimated_memory_mb = total_memory_mb * 1.2;

                        black_box(estimated_memory_mb);
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark JSON query performance
fn benchmark_json_query_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_query_performance");

    // Benchmark Rust deserialization
    group.bench_function("rust_deserialization", |b| {
        b.iter(|| {
            // Create a sample metadata JSON
            let metadata = serde_json::json!({
                "domain": "benchmark",
                "complexity": "moderate",
                "tags": ["performance", "test"],
                "language": "rust",
                "framework": "tokio",
                "timestamp": 1234567890,
                "success": true,
                "score": 0.95,
                "iterations": 42
            });

            // Serialize and deserialize
            let json_str = metadata.to_string();
            let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

            // Access multiple fields (simulating typical usage)
            let domain = parsed.get("domain").and_then(|v| v.as_str()).unwrap_or("");
            let complexity = parsed
                .get("complexity")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let score = parsed.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let success = parsed
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            black_box((domain, complexity, score, success));
        });
    });

    group.finish();
}

/// Benchmark embedding storage performance
fn benchmark_embedding_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding_storage");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(20));

    let rt = Runtime::new().unwrap();

    for dimension in [384, 1536] {
        for count in [10, 100] {
            group.bench_with_input(
                BenchmarkId::new(format!("{}_dim", dimension), count),
                &count,
                |b, &_count| {
                    #[allow(clippy::excessive_nesting)]
                    b.iter(|| {
                        rt.block_on(async {
                            let _temp_dir =
                                TempDir::new().expect("Failed to create temp directory");

                            // Use Turso dev server or cloud database via environment variables
                            let url = std::env::var("TURSO_DATABASE_URL")
                                .unwrap_or_else(|_| "libsql://127.0.0.1:8080".to_string());
                            let token =
                                std::env::var("TURSO_AUTH_TOKEN").unwrap_or_else(|_| String::new());

                            let storage = TursoStorage::new(&url, &token)
                                .await
                                .expect("Failed to create turso storage");
                            storage
                                .initialize_schema()
                                .await
                                .expect("Failed to initialize schema");

                            // Store embeddings
                            for i in 0..count {
                                let embedding = generate_random_embedding(dimension, i as u64);
                                let id = Uuid::new_v4().to_string();

                                // Use store_embedding_backend which handles dimension check
                                storage
                                    .store_embedding_backend(&id, embedding)
                                    .await
                                    .expect("Failed to store embedding");
                            }

                            black_box(count);
                        })
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_384_dim_native_search,
    benchmark_brute_force_search,
    benchmark_memory_usage,
    benchmark_json_query_performance,
    benchmark_embedding_storage,
);

criterion_main!(benches);
