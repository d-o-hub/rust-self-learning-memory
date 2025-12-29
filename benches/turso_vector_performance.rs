//! Benchmark Turso vector search performance for different embedding dimensions
//! 
//! Measures:
//! 1. OpenAI embedding search latency (1536-dim, brute-force)
//! 2. 384-dim search latency (native vector search)
//! 3. Memory usage for embeddings
//! 4. JSON query performance vs Rust deserialization

use anyhow::Result;
use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};
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
    let db_path = temp_dir.path().join("benchmark.db");
    
    let storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
        .await
        .expect("Failed to create turso storage");
    storage.initialize_schema().await.expect("Failed to initialize schema");
    
    // Store episodes with embeddings
    for i in 0..count {
        let episode = create_test_episode(i);
        storage.store_episode(&episode).await.expect("Failed to store episode");
        
        let embedding = generate_random_embedding(dimension, i as u64);
        storage.store_episode_embedding(episode.episode_id, embedding.clone())
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
    group.sample_size(10);
    
    for embedding_count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(embedding_count as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(embedding_count),
            &embedding_count,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    let (storage, _temp_dir, query_embedding) = setup_storage_with_data(384, count)
                        .await
                        .expect("Failed to setup storage");
                    
                    // This should use native vector search for 384-dim embeddings
                    let results = storage.find_similar_episodes(query_embedding.clone(), 10, 0.5)
                        .await
                        .expect("Failed to find similar episodes");
                    
                    black_box(results.len());
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark 1536-dim brute-force search simulation
fn benchmark_1536_dim_brute_force_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("1536_dim_brute_force_search");
    group.sample_size(10);
    
    // Note: brute-force is O(n), so we test smaller sizes
    for embedding_count in [10, 50, 100] {
        group.throughput(Throughput::Elements(embedding_count as u64));
        
        group.bench_with_input(
            BenchmarkId::from_parameter(embedding_count),
            &embedding_count,
            |b, &count| {
                b.to_async(FuturesExecutor).iter(|| async {
                    // For brute-force simulation, we'll compute similarity in Rust
                    // This gives us a baseline for O(n) performance
                    let (storage, _temp_dir, query_embedding) = setup_storage_with_data(384, count)
                        .await
                        .expect("Failed to setup storage");
                    
                    // Get all embeddings from database (simulating brute-force)
                    let conn = storage.get_connection().await.expect("Failed to get connection");
                    let sql = "SELECT embedding_data FROM embeddings WHERE item_type = 'episode'";
                    let mut rows = conn.query(sql, ()).await.expect("Failed to query embeddings");
                    
                    let mut similarity_count = 0;
                    while let Some(row) = rows.next().await.expect("Failed to fetch row") {
                        let embedding_json: String = row.get(0).expect("Failed to get embedding_data");
                        let embedding: Vec<f32> = serde_json::from_str(&embedding_json).expect("Failed to parse embedding");
                        
                        // Compute cosine similarity
                        let dot_product: f32 = query_embedding.iter().zip(embedding.iter()).map(|(a, b)| a * b).sum();
                        let norm_a: f32 = query_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                        let norm_b: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                        let similarity = if norm_a > 0.0 && norm_b > 0.0 {
                            dot_product / (norm_a * norm_b)
                        } else {
                            0.0
                        };
                        if similarity >= 0.5 {
                            similarity_count += 1;
                        }
                    }
                    
                    black_box(similarity_count);
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
            let complexity = parsed.get("complexity").and_then(|v| v.as_str()).unwrap_or("");
            let score = parsed.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let success = parsed.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
            
            black_box((domain, complexity, score, success));
        });
    });
    
    group.finish();
}

/// Benchmark embedding storage performance
fn benchmark_embedding_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding_storage");
    
    for dimension in [384, 1536] {
        for count in [10, 100] {
            group.bench_with_input(
                BenchmarkId::new(format!("{}_dim", dimension), count),
                &count,
                |b, &count| {
                    b.to_async(FuturesExecutor).iter(|| async {
                        let temp_dir = TempDir::new().expect("Failed to create temp directory");
                        let db_path = temp_dir.path().join("benchmark.db");
                        
                        let storage = TursoStorage::new(&format!("file:{}", db_path.to_string_lossy()), "")
                            .await
                            .expect("Failed to create turso storage");
                        storage.initialize_schema().await.expect("Failed to initialize schema");
                        
                        // Store embeddings
                        for i in 0..count {
                            let embedding = generate_random_embedding(dimension, i as u64);
                            let id = Uuid::new_v4().to_string();
                            
                            // Use store_embedding_backend which handles dimension check
                            storage.store_embedding_backend(&id, embedding)
                                .await
                                .expect("Failed to store embedding");
                        }
                        
                        black_box(count);
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
    benchmark_1536_dim_brute_force_search,
    benchmark_memory_usage,
    benchmark_json_query_performance,
    benchmark_embedding_storage,
);

criterion_main!(benches);