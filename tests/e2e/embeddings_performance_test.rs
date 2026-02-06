//! End-to-End tests for embedding performance
//!
//! Comprehensive performance benchmarks:
//! - Embedding generation latency (single and batch)
//! - Search latency
//! - Large episode sets (1000+)
//! - Provider comparison
//! - Memory usage
//! - Throughput benchmarks

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::embeddings::{
    cosine_similarity, EmbeddingProvider, EmbeddingStorageBackend, InMemoryEmbeddingStorage,
    LocalConfig, LocalEmbeddingProvider,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics
#[derive(Debug)]
struct PerformanceMetrics {
    operation: String,
    count: usize,
    total_duration: Duration,
    avg_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
    throughput: f64, // operations per second
}

impl PerformanceMetrics {
    fn new(operation: String, durations: Vec<Duration>) -> Self {
        let count = durations.len();
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / count as u32;
        let min_duration = *durations.iter().min().unwrap();
        let max_duration = *durations.iter().max().unwrap();
        let throughput = count as f64 / total_duration.as_secs_f64();

        Self {
            operation,
            count,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            throughput,
        }
    }

    fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Performance Metrics: {}", self.operation);
        println!("{}", "=".repeat(60));
        println!("  Operations: {}", self.count);
        println!("  Total time: {:?}", self.total_duration);
        println!("  Average: {:.2}ms", self.avg_duration.as_millis());
        println!("  Min: {:?}", self.min_duration);
        println!("  Max: {:?}", self.max_duration);
        println!("  Throughput: {:.2} ops/sec", self.throughput);
        println!("{}", "=".repeat(60));
    }
}

// ============================================================================
// Day 3: Performance Tests
// ============================================================================

#[tokio::test]
async fn test_performance_single_embedding_latency() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Warmup
    for _ in 0..5 {
        let _ = provider.embed_text("warmup text").await.unwrap();
    }

    let iterations = 100;
    let mut durations = vec![];

    for i in 0..iterations {
        let text = format!("Performance test text number {}", i);
        let start = Instant::now();
        let _ = provider.embed_text(&text).await.unwrap();
        durations.push(start.elapsed());
    }

    let metrics = PerformanceMetrics::new("Single Embedding Generation".to_string(), durations);
    metrics.print();

    // Performance assertions
    assert!(
        metrics.avg_duration < Duration::from_millis(100),
        "Average should be < 100ms, got {:?}",
        metrics.avg_duration
    );
    assert!(
        metrics.max_duration < Duration::from_millis(500),
        "Max should be < 500ms, got {:?}",
        metrics.max_duration
    );
    assert!(
        metrics.throughput > 10.0,
        "Throughput should be > 10 ops/sec, got {:.2}",
        metrics.throughput
    );
}

#[tokio::test]
async fn test_performance_batch_embedding_latency() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    let batch_sizes = vec![1, 10, 50, 100, 200];

    println!("\nBatch Embedding Performance:");
    println!(
        "{:<10} {:<12} {:<12} {:<12} {:<15}",
        "Batch", "Total", "Avg", "Throughput", "Speedup"
    );
    println!("{}", "-".repeat(70));

    let mut prev_avg = None;

    for batch_size in batch_sizes {
        let texts: Vec<String> = (0..batch_size)
            .map(|i| format!("Batch test text {}", i))
            .collect();

        let start = Instant::now();
        let embeddings = provider.embed_batch(&texts).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(embeddings.len(), batch_size);

        let avg = duration.as_millis() as f64 / batch_size as f64;
        let throughput = batch_size as f64 / duration.as_secs_f64();

        let speedup = if let Some(prev) = prev_avg {
            format!("{:.2}x", prev / avg)
        } else {
            "1.00x".to_string()
        };

        println!(
            "{:<10} {:<12?} {:<12.2}ms {:<12.2}/s {:<15}",
            batch_size, duration, avg, throughput, speedup
        );

        prev_avg = Some(avg);
    }
}

#[tokio::test]
async fn test_performance_search_latency() {
    let storage = Arc::new(InMemoryEmbeddingStorage::new());
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Create a dataset of 1000 episodes
    let num_episodes = 1000;
    println!("Creating dataset with {} episodes...", num_episodes);

    for i in 0..num_episodes {
        let episode_id = uuid::Uuid::new_v4();
        let text = format!("Episode {} about various topics like authentication, API development, database design, testing, and deployment", i);
        let embedding = provider.embed_text(&text).await.unwrap();

        storage
            .store_episode_embedding(episode_id, embedding)
            .await
            .unwrap();
    }

    // Benchmark search performance
    let query = "authentication and API security";
    let query_embedding = provider.embed_text(query).await.unwrap();

    let iterations = 100;
    let mut durations = vec![];

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = storage
            .find_similar_episodes(query_embedding.clone(), 10, 0.5)
            .await
            .unwrap();
        durations.push(start.elapsed());
    }

    let metrics = PerformanceMetrics::new(format!("Search ({} episodes)", num_episodes), durations);
    metrics.print();

    // Search should be fast even with 1000 episodes
    assert!(
        metrics.avg_duration < Duration::from_millis(50),
        "Average search should be < 50ms, got {:?}",
        metrics.avg_duration
    );
}

#[tokio::test]
async fn test_performance_large_dataset_scaling() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    let dataset_sizes = vec![100, 500, 1000, 2000, 5000];

    println!("\nScalability Analysis:");
    println!(
        "{:<10} {:<15} {:<15} {:<15}",
        "Episodes", "Build Time", "Search Time", "Memory"
    );
    println!("{}", "-".repeat(60));

    for size in dataset_sizes {
        let storage = Arc::new(InMemoryEmbeddingStorage::new());

        // Build dataset
        let build_start = Instant::now();
        for i in 0..size {
            let episode_id = uuid::Uuid::new_v4();
            let text = format!("Episode {} with various technical content", i);
            let embedding = provider.embed_text(&text).await.unwrap();
            storage
                .store_episode_embedding(episode_id, embedding)
                .await
                .unwrap();
        }
        let build_time = build_start.elapsed();

        // Search
        let query = "technical content about APIs and databases";
        let query_embedding = provider.embed_text(query).await.unwrap();

        let search_start = Instant::now();
        let _ = storage
            .find_similar_episodes(query_embedding, 10, 0.5)
            .await
            .unwrap();
        let search_time = search_start.elapsed();

        // Estimate memory (384 floats * 4 bytes per float)
        let memory_mb = (size * 384 * 4) as f64 / (1024.0 * 1024.0);

        println!(
            "{:<10} {:<15?} {:<15?} {:<15.2}MB",
            size, build_time, search_time, memory_mb
        );

        // Assert reasonable performance
        assert!(
            search_time < Duration::from_millis(500),
            "Search should be fast for {} episodes: {:?}",
            size,
            search_time
        );
    }
}

#[tokio::test]
#[allow(clippy::excessive_nesting)]
async fn test_performance_concurrent_embeddings() {
    let provider = Arc::new(
        LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
            .await
            .unwrap(),
    );

    let concurrent_tasks = vec![1, 5, 10, 20];

    println!("\nConcurrent Embedding Performance:");
    println!(
        "{:<15} {:<15} {:<15} {:<15}",
        "Concurrent", "Total Time", "Avg Time", "Efficiency"
    );
    println!("{}", "-".repeat(65));

    for num_tasks in concurrent_tasks {
        let texts: Vec<String> = (0..(num_tasks * 10))
            .map(|i| format!("Concurrent test text {}", i))
            .collect();

        let start = Instant::now();

        let handles: Vec<_> = texts
            .chunks(10)
            .map(|chunk| {
                let provider = Arc::clone(&provider);
                let chunk = chunk.to_vec();
                tokio::spawn(async move {
                    let mut local_durations = vec![];
                    for text in chunk {
                        let t_start = Instant::now();
                        let _ = provider.embed_text(&text).await.unwrap();
                        local_durations.push(t_start.elapsed());
                    }
                    local_durations
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .flat_map(|r| r.unwrap())
            .collect();

        let total_time = start.elapsed();
        let avg_time = total_time / results.len() as u32;

        // Efficiency = (sequential time / parallel time)
        let sequential_estimate = Duration::from_millis(10) * results.len() as u32;
        let efficiency = sequential_estimate.as_secs_f64() / total_time.as_secs_f64();

        println!(
            "{:<15} {:<15?} {:<15?} {:.1}%",
            num_tasks,
            total_time,
            avg_time,
            efficiency * 100.0
        );

        // Concurrent should be faster
        assert!(efficiency > 0.5, "Efficiency should be > 50%");
    }
}

#[tokio::test]
async fn test_performance_similarity_calculation() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    let embedding1 = provider.embed_text("text one").await.unwrap();
    let embedding2 = provider.embed_text("text two").await.unwrap();

    let iterations = 10_000;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = cosine_similarity(&embedding1, &embedding2);
    }
    let duration = start.elapsed();

    let avg = duration.as_micros() as f64 / iterations as f64;
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!("\nCosine Similarity Performance:");
    println!("  Iterations: {}", iterations);
    println!("  Total time: {:?}", duration);
    println!("  Average: {:.2}μs", avg);
    println!("  Throughput: {:.0} ops/sec", throughput);

    // Similarity calculation should be very fast
    assert!(avg < 100.0, "Average should be < 100μs, got {:.2}μs", avg);
}

#[tokio::test]
async fn test_performance_memory_efficiency() {
    let storage = Arc::new(InMemoryEmbeddingStorage::new());
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    println!("\nMemory Efficiency Test:");

    let test_sizes = vec![100, 1000, 5000, 10000];

    for size in test_sizes {
        let start = Instant::now();

        for i in 0..size {
            let episode_id = uuid::Uuid::new_v4();
            let text = format!("Episode {} for memory testing", i);
            let embedding = provider.embed_text(&text).await.unwrap();
            storage
                .store_episode_embedding(episode_id, embedding)
                .await
                .unwrap();
        }

        let duration = start.elapsed();

        // Calculate memory usage
        let bytes_per_embedding = 384 * std::mem::size_of::<f32>();
        let total_bytes = size * bytes_per_embedding;
        let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

        // Get actual memory usage if possible
        let (rss_mb, _) = get_memory_usage();

        println!(
            "  Size: {:<6} | Time: {:<8?} | Est: {:<8.2}MB | RSS: {:<8.2}MB | Throughput: {:.0}/s",
            size,
            duration,
            total_mb,
            rss_mb,
            size as f64 / duration.as_secs_f64()
        );

        // Should complete in reasonable time
        assert!(
            duration < Duration::from_secs(60),
            "Should handle {} embeddings in < 60s",
            size
        );
    }
}

#[tokio::test]
async fn test_performance_provider_comparison() {
    let local_provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .unwrap();

    let text = "Implement authentication system with JWT tokens";
    let iterations = 50;

    println!("\nProvider Performance Comparison:");
    println!(
        "{:<15} {:<15} {:<15} {:<15}",
        "Provider", "Avg Time", "Min Time", "Max Time"
    );
    println!("{}", "-".repeat(65));

    // Test local provider
    let mut local_durations = vec![];
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = local_provider.embed_text(text).await.unwrap();
        local_durations.push(start.elapsed());
    }

    let local_avg = local_durations.iter().sum::<Duration>() / iterations as u32;
    let local_min = *local_durations.iter().min().unwrap();
    let local_max = *local_durations.iter().max().unwrap();

    println!(
        "{:<15} {:<15?} {:<15?} {:<15?}",
        "Local", local_avg, local_min, local_max
    );

    // Note: OpenAI provider testing would require API key
    // This is a placeholder for comparison
    println!(
        "{:<15} {:<15} {:<15} {:<15}",
        "OpenAI", "(requires API key)", "-", "-"
    );
}

#[tokio::test]
async fn test_performance_cache_effectiveness() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    let texts = vec![
        "authentication",
        "REST API",
        "database design",
        "unit testing",
        "deployment",
    ];

    println!("\nCache Effectiveness Test:");

    // First pass (cold cache)
    let mut cold_durations = vec![];
    for text in &texts {
        let start = Instant::now();
        let _ = provider.embed_text(text).await.unwrap();
        cold_durations.push(start.elapsed());
    }

    // Second pass (warm cache - if caching is implemented)
    let mut warm_durations = vec![];
    for text in &texts {
        let start = Instant::now();
        let _ = provider.embed_text(text).await.unwrap();
        warm_durations.push(start.elapsed());
    }

    println!(
        "{:<20} {:<15} {:<15} {:<10}",
        "Text", "Cold (ms)", "Warm (ms)", "Speedup"
    );
    println!("{}", "-".repeat(60));

    for (i, text) in texts.iter().enumerate() {
        let cold = cold_durations[i].as_millis();
        let warm = warm_durations[i].as_millis();
        let speedup = if warm > 0 {
            format!("{:.2}x", cold as f64 / warm as f64)
        } else {
            "N/A".to_string()
        };

        println!("{:<20} {:<15} {:<15} {:<10}", text, cold, warm, speedup);
    }
}

#[tokio::test]
async fn test_performance_text_length_impact() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Create owned strings to avoid temporary value issues
    let medium_text = "Medium length text with some details. ".repeat(3);
    let longer_text = "Longer text description with more content. ".repeat(7);
    let very_long_text = "Very long text with extensive content and details. ".repeat(15);
    let extremely_long_text =
        "Extremely long text document with comprehensive information. ".repeat(35);

    let text_lengths: Vec<(usize, &str)> = vec![
        (10, "Short text"),
        (50, &medium_text),
        (100, &longer_text),
        (200, &very_long_text),
        (500, &extremely_long_text),
    ];

    println!("\nText Length Impact on Performance:");
    println!(
        "{:<10} {:<15} {:<15}",
        "Length", "Time (ms)", "Time/char (μs)"
    );
    println!("{}", "-".repeat(45));

    for (_expected_len, text) in text_lengths {
        let actual_len = text.len();
        let iterations = 10;

        let mut durations = vec![];
        for _ in 0..iterations {
            let start = Instant::now();
            let _ = provider.embed_text(text).await.unwrap();
            durations.push(start.elapsed());
        }

        let avg = durations.iter().sum::<Duration>() / iterations as u32;
        let per_char = avg.as_micros() as f64 / actual_len as f64;

        println!(
            "{:<10} {:<15.2} {:<15.2}",
            actual_len,
            avg.as_millis(),
            per_char
        );
    }
}

#[tokio::test]
async fn test_performance_realistic_workload() {
    let storage = Arc::new(InMemoryEmbeddingStorage::new());
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    println!("\nRealistic Workload Simulation:");

    // Simulate a realistic mix of operations
    let total_operations = 1000;
    let mut add_count = 0;
    let mut search_count = 0;

    let start = Instant::now();

    for i in 0..total_operations {
        // 70% adds, 30% searches
        if i % 10 < 7 {
            // Add new episode
            let episode_id = uuid::Uuid::new_v4();
            let text = format!("Episode {} about various technical topics", i);
            let embedding = provider.embed_text(&text).await.unwrap();
            storage
                .store_episode_embedding(episode_id, embedding)
                .await
                .unwrap();
            add_count += 1;
        } else {
            // Search
            let query = "technical topics and implementations";
            let query_embedding = provider.embed_text(query).await.unwrap();
            let _ = storage
                .find_similar_episodes(query_embedding, 5, 0.5)
                .await
                .unwrap();
            search_count += 1;
        }
    }

    let duration = start.elapsed();
    let throughput = total_operations as f64 / duration.as_secs_f64();

    println!("  Total operations: {}", total_operations);
    println!("  Add operations: {}", add_count);
    println!("  Search operations: {}", search_count);
    println!("  Total time: {:?}", duration);
    println!("  Throughput: {:.2} ops/sec", throughput);
    println!(
        "  Avg time: {:.2}ms/op",
        duration.as_millis() as f64 / total_operations as f64
    );

    // Should complete in reasonable time
    assert!(
        duration < Duration::from_secs(30),
        "Workload should complete in < 30s"
    );
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get memory usage (RSS and total)
#[cfg(unix)]
fn get_memory_usage() -> (f64, f64) {
    use std::fs;

    let mut rss = 0.0;
    let mut total = 0.0;

    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if let Some((prefix, value_str)) = line.split_once(':') {
                if let Ok(kb_val) = value_str.trim().parse::<f64>() {
                    match prefix {
                        "VmRSS" => rss = kb_val / 1024.0,
                        "VmSize" => total = kb_val / 1024.0,
                        _ => {}
                    }
                }
            }
        }
    }

    (rss, total)
}

#[cfg(not(unix))]
fn get_memory_usage() -> (f64, f64) {
    (0.0, 0.0) // Not implemented for non-Unix
}
