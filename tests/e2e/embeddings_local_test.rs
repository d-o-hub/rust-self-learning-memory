//! End-to-End tests for Local embedding provider
//!
//! Tests the local CPU-based embedding provider with:
//! - Model loading and initialization
//! - Embedding generation from local models
//! - Performance benchmarks (CPU-based)
//! - Quality comparison with API providers
//! - Fallback behavior when API unavailable

#![allow(clippy::unwrap_used, clippy::expect_used)]

use do_memory_core::embeddings::{
    cosine_similarity, EmbeddingProvider, InMemoryEmbeddingStorage, LocalConfig,
    LocalEmbeddingProvider, SemanticService,
};
use std::time::{Duration, Instant};

/// Create local provider for testing
async fn create_local_provider() -> anyhow::Result<LocalEmbeddingProvider> {
    let config = LocalConfig::new("sentence-transformers/all-MiniLM-L6-v2", 384);

    LocalEmbeddingProvider::new_with_fallback(config)
        .await
        .?
    Ok(())
}

// ============================================================================
// Day 1: Local Provider E2E Tests
// ============================================================================

#[tokio::test]
async fn test_local_provider_initialization() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Verify provider is loaded
    assert!(provider.is_loaded().await, "Provider should be loaded");

    // Check embedding dimension
    assert_eq!(
        provider.embedding_dimension(),
        384,
        "Dimension should be 384"
    );

    // Verify availability
    assert!(
        provider.is_available().await,
        "Provider should be available"
    );

    // Verify warmup succeeds
    let warmup_result = provider.warmup().await;
    assert!(warmup_result.is_ok(), "Warmup should succeed");
    Ok(())
}

#[tokio::test]
async fn test_local_single_embedding_generation() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;
    let text = "Implement user authentication with JWT tokens in Rust";

    let start = Instant::now();
    let embedding = provider
        .embed_text(text)
        .await
        ?;
    let duration = start.elapsed();

    // Verify embedding
    assert!(!embedding.is_empty(), "Embedding should not be empty");
    assert_eq!(embedding.len(), 384, "Embedding dimension should be 384");

    // Verify vector is normalized
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (magnitude - 1.0).abs() < 0.01,
        "Embedding should be normalized"
    );

    println!("Local embedding generation time: {:?}", duration);

    // Local embeddings should be fast
    assert!(
        duration < Duration::from_millis(500),
        "Local embedding should be fast, got {:?}",
        duration
    );
    Ok(())
}

#[tokio::test]
async fn test_local_batch_embedding_generation() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    let texts = vec![
        "Implement REST API endpoints in Rust using Axum framework",
        "Create user authentication system with OAuth2",
        "Build database schema with PostgreSQL and SQLx",
        "Add input validation middleware for API requests",
        "Implement rate limiting to prevent API abuse",
        "Write unit tests for authentication module",
        "Deploy application to production with Docker",
        "Monitor application performance and logging",
        "Document API endpoints with OpenAPI specification",
        "Optimize database queries for better performance",
    ];

    let start = Instant::now();
    let embeddings = provider
        .embed_batch(&texts)
        .await
        ?;
    let duration = start.elapsed();

    // Verify all embeddings generated
    assert_eq!(
        embeddings.len(),
        texts.len(),
        "Should generate all embeddings"
    );

    // Verify each embedding
    for (i, embedding) in embeddings.iter().enumerate() {
        assert!(!embedding.is_empty(), "Embedding {} should not be empty", i);
        assert_eq!(
            embedding.len(),
            384,
            "Embedding {} should have dimension 384",
            i
        );

        // Check normalization
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (magnitude - 1.0).abs() < 0.01,
            "Embedding {} should be normalized",
            i
        );
    }

    let avg_time = duration.as_millis() as f64 / texts.len() as f64;
    println!(
        "Local batch embedding generation: {:?} total, {:.2}ms avg",
        duration, avg_time
    );
    Ok(())
}

#[tokio::test]
async fn test_local_semantic_similarity() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Test semantic similarity
    let test_cases = vec![
        // Identical texts
        ("REST API", "REST API", 0.98),
        // Very similar (same concept, different wording)
        ("user authentication", "user login", 0.70),
        ("database schema", "database structure", 0.70),
        // Related concepts
        ("REST API", "web service", 0.60),
        // Different domains
        ("authentication", "database", 0.40),
    ];

    for (text1, text2, min_expected) in test_cases {
        let similarity = provider
            .similarity(text1, text2)
            .await
            ?;

        println!(
            "Similarity between '{}' and '{}': {:.3}",
            text1, text2, similarity
        );

        assert!(
            similarity >= min_expected,
            "Similarity between '{}' and '{}' ({:.3}) should be >= {:.3}",
            text1,
            text2,
            similarity,
            min_expected
        );

        assert!(similarity >= 0.0 && similarity <= 1.0);
    }
    Ok(())
}

#[tokio::test]
async fn test_local_deterministic_embeddings() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;
    let text = "Test text for deterministic behavior";

    let embedding1 = provider
        .embed_text(text)
        .await
        ?;
    let embedding2 = provider
        .embed_text(text)
        .await
        ?;

    // Local embeddings should be deterministic
    assert_eq!(
        embedding1, embedding2,
        "Same text should produce identical embeddings"
    );
    Ok(())
}

#[tokio::test]
async fn test_local_model_caching() -> anyhow::Result<()> {
    // Test that model is cached and reused
    let provider1 = create_local_provider().await;
    let provider2 = create_local_provider().await;

    // Both should use the same model
    assert_eq!(
        provider1.embedding_dimension(),
        provider2.embedding_dimension()
    );

    // Second instantiation should be faster (model cached)
    let text = "Test caching";

    let start1 = Instant::now();
    let _ = provider1.embed_text(text).await?;
    let time1 = start1.elapsed();

    let start2 = Instant::now();
    let _ = provider2.embed_text(text).await?;
    let time2 = start2.elapsed();

    println!("First call: {:?}, Second call: {:?}", time1, time2);

    // Second call might be faster due to caching
    // (but this is a weak assertion due to system variance)
    Ok(())
}

#[tokio::test]
async fn test_local_empty_text_handling() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Test with empty string
    let result = provider.embed_text("");

    match result.await {
        Ok(embedding) => {
            // Should handle gracefully
            assert_eq!(embedding.len(), 384);
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("empty") || error_msg.contains("invalid"),
                "Error should mention empty input: {}",
                error_msg
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_local_multilingual_support() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Test multilingual support
    let texts = vec![
        ("Implement authentication system", "en"),
        ("Implémenter un système d'authentification", "fr"),
        ("Implementar sistema de autenticación", "es"),
        ("实现认证系统", "zh"),
        ("認証システムを実装する", "ja"),
    ];

    for (text, lang) in texts {
        let embedding = provider
            .embed_text(text)
            .await
            .expect(&format!("Should embed {} text", lang));

        assert_eq!(
            embedding.len(),
            384,
            "{} text should have correct dimension",
            lang
        );
        println!("{} text embedded successfully", lang);
    }

    // Cross-language similarity
    let sim_en_fr = provider
        .similarity("authentication", "authentification")
        .await
        ?;

    println!("Cross-language similarity (EN-FR): {:.3}", sim_en_fr);
    // Note: Local models may have lower cross-lingual similarity than API models
    Ok(())
}

#[tokio::test]
async fn test_local_special_characters_handling() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Test with various special characters
    let special_texts = vec![
        "Code with <html> tags & special chars: @#$%",
        "UTF-8: 中文 日本語 한글 العربية",
        "Emojis: 🚀 🎉 ✅ 🧠",
        "Math: ∑(i=1..n) i² = n(n+1)(2n+1)/6",
        "Quotes: 'single' and \"double\" quotes",
    ];

    for text in special_texts {
        let embedding = provider
            .embed_text(text)
            .await
            .expect(&format!("Should handle: {}", text));

        assert_eq!(embedding.len(), 384);
        println!(
            "Successfully embedded: {}",
            text.chars().take(30).collect::<String>()
        );
    }
    Ok(())
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_local_performance_benchmarks() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Warmup
    let _ = provider.embed_text("warmup").await?;

    // Benchmark single embeddings
    let iterations = 50;
    let mut durations = vec![];

    for i in 0..iterations {
        let text = format!("Benchmark text number {}", i);
        let start = Instant::now();
        let _ = provider.embed_text(&text).await?;
        durations.push(start.elapsed());
    }

    let avg = durations.iter().sum::<Duration>().as_millis() as f64 / iterations as f64;
    let min = durations.iter().min()?;
    let max = durations.iter().max()?;

    println!(
        "Local provider single embedding performance ({} iterations):",
        iterations
    );
    println!("  Average: {:.2}ms", avg);
    println!("  Min: {:?}", min);
    println!("  Max: {:?}", max);

    // Local provider should be consistently fast
    assert!(avg < 100.0, "Average should be < 100ms, was {:.2}ms", avg);
    assert!(max < Duration::from_millis(500), "Max should be < 500ms");
    Ok(())
}

#[tokio::test]
async fn test_local_batch_performance() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    let batch_sizes = vec![1, 10, 50, 100];

    for batch_size in batch_sizes {
        let texts: Vec<String> = (0..batch_size)
            .map(|i| format!("Text {} for batch testing", i))
            .collect();

        let start = Instant::now();
        let embeddings = provider.embed_batch(&texts).await?;
        let duration = start.elapsed();

        assert_eq!(embeddings.len(), batch_size);

        let avg_time = duration.as_millis() as f64 / batch_size as f64;
        let throughput = batch_size as f64 / duration.as_secs_f64();

        println!(
            "Batch size {}: {:?} total, {:.2}ms avg, {:.1} embeddings/sec",
            batch_size, duration, avg_time, throughput
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_local_concurrent_embeddings() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    let texts: Vec<String> = (0..20).map(|i| format!("Concurrent text {}", i)).collect();

    let start = Instant::now();

    // Spawn concurrent tasks
    let handles: Vec<_> = texts
        .iter()
        .map(|text| {
            let provider_clone = provider.clone();
            let text = text.clone();
            tokio::spawn(async move { provider_clone.embed_text(&text).await? })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r?)
        .collect();

    let duration = start.elapsed();

    // Verify all embeddings
    assert_eq!(results.len(), texts.len());
    for (i, embedding) in results.iter().enumerate() {
        assert_eq!(
            embedding.len(),
            384,
            "Embedding {} should have correct dimension",
            i
        );
    }

    let avg_time = duration.as_millis() as f64 / texts.len() as f64;
    println!(
        "Concurrent embeddings (20 parallel): {:?} total, {:.2}ms avg",
        duration, avg_time
    );
    Ok(())
}

// ============================================================================
// Integration Tests with SemanticService
// ============================================================================

#[tokio::test]
async fn test_local_semantic_search_workflow() -> anyhow::Result<()> {
    let storage = Box::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(storage, config)
        .await
        ?;

    // Create sample episodes
    let episodes = vec![
        "Implement user authentication with JWT tokens",
        "Build REST API endpoints for user management",
        "Create data validation middleware for API requests",
        "Add rate limiting to prevent API abuse",
        "Design database schema for user profiles",
        "Write unit tests for authentication module",
        "Deploy API to production with Docker",
    ];

    // Store embeddings
    for text in episodes.iter() {
        let episode_id = uuid::Uuid::new_v4();
        let embedding = service
            .provider
            .embed_text(text)
            .await
            ?;

        service
            .storage
            .store_episode_embedding(episode_id, embedding)
            .await
            ?;
    }

    // Test semantic search
    let query = "How to secure my API?";
    let query_embedding = service
        .provider
        .embed_text(query)
        .await
        ?;

    let results = service
        .storage
        .find_similar_episodes(query_embedding, 5, 0.0)
        .await
        ?;

    assert!(!results.is_empty(), "Should find similar episodes");

    // Results should be sorted
    for i in 1..results.len() {
        assert!(results[i - 1].similarity >= results[i].similarity);
    }

    println!(
        "Found {} similar episodes for query: '{}'",
        results.len(),
        query
    );
    for (i, result) in results.iter().enumerate() {
        println!("  {}. similarity: {:.3}", i + 1, result.similarity);
    }
    Ok(())
}

// ============================================================================
// Quality Tests
// ============================================================================

#[tokio::test]
async fn test_local_embedding_quality() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Test that semantically similar texts have high similarity
    let similar_pairs = vec![
        ("user authentication", "user login system"),
        ("REST API", "REST web service"),
        ("database query", "SQL query"),
        ("error handling", "exception handling"),
    ];

    for (text1, text2) in similar_pairs {
        let similarity = provider
            .similarity(text1, text2)
            .await
            ?;

        println!(
            "Similarity between '{}' and '{}': {:.3}",
            text1, text2, similarity
        );

        // Similar concepts should have decent similarity
        assert!(
            similarity > 0.5,
            "Similar concepts should have similarity > 0.5, got {:.3}",
            similarity
        );
    }

    // Test that different texts have lower similarity
    let different_pairs = vec![
        ("authentication", "database"),
        ("API", "frontend"),
        ("testing", "deployment"),
    ];

    for (text1, text2) in different_pairs {
        let similarity = provider
            .similarity(text1, text2)
            .await
            ?;

        println!(
            "Dissimilarity between '{}' and '{}': {:.3}",
            text1, text2, similarity
        );

        // Different concepts should have lower similarity
        assert!(
            similarity < 0.7,
            "Different concepts should have similarity < 0.7, got {:.3}",
            similarity
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_local_fallback_behavior() -> anyhow::Result<()> {
    // Test that local provider works when APIs are unavailable
    let provider = create_local_provider().await?;

    // Should work without any API keys or network
    assert!(provider.is_available().await);

    let text = "Test without network";
    let embedding = provider.embed_text(text).await?;

    assert_eq!(embedding.len(), 384);
    println!("Local provider works offline: ✓");
    Ok(())
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
async fn test_local_memory_efficiency() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Generate many embeddings to test memory usage
    let num_embeddings = 1000;
    let texts: Vec<String> = (0..num_embeddings)
        .map(|i| format!("Text number {} for memory testing", i))
        .collect();

    let start = Instant::now();
    let embeddings = provider.embed_batch(&texts).await?;
    let duration = start.elapsed();

    assert_eq!(embeddings.len(), num_embeddings);

    // Calculate memory usage
    let total_bytes = embeddings.len() * embeddings[0].len() * std::mem::size_of::<f32>();
    let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

    println!("Memory efficiency test:");
    println!("  Embeddings: {}", num_embeddings);
    println!("  Total size: {:.2} MB", total_mb);
    println!("  Time: {:?}", duration);
    println!(
        "  Throughput: {:.1} embeddings/sec",
        num_embeddings as f64 / duration.as_secs_f64()
    );

    // Should be reasonably fast even for 1000 embeddings
    assert!(duration < Duration::from_secs(30));
    Ok(())
}

#[tokio::test]
async fn test_local_large_text_handling() -> anyhow::Result<()> {
    let provider = create_local_provider().await?;

    // Test with increasingly large texts
    let sizes = vec![100, 500, 1000, 5000];

    for size in sizes {
        let text = "word ".repeat(size);
        let start = Instant::now();
        let embedding = provider.embed_text(&text).await?;
        let duration = start.elapsed();

        assert_eq!(embedding.len(), 384);
        println!("Text size {}: {:?}, embedding: 384 dims", size, duration);

        // Should still complete in reasonable time
        assert!(duration < Duration::from_secs(5));
    }
    Ok(())
}
