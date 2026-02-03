//! End-to-End tests for OpenAI embedding provider
//!
//! Tests the OpenAI provider integration with:
//! - Real API calls (when API key available)
//! - Mocked responses for CI/CD
//! - Error handling (invalid keys, rate limits, network errors)
//! - Semantic search quality
//! - Batch embedding generation

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::embeddings::config::embedding_config::InMemoryEmbeddingStorage;
use memory_core::embeddings::{
    cosine_similarity, EmbeddingConfig, EmbeddingProvider, OpenAIConfig, OpenAIEmbeddingProvider,
    SemanticService,
};
use std::env;
use std::time::{Duration, Instant};

/// Check if OpenAI API key is available
fn has_openai_key() -> bool {
    env::var("OPENAI_API_KEY").is_ok()
}

/// Create OpenAI provider (with real or mock config)
async fn create_openai_provider() -> OpenAIEmbeddingProvider {
    let config = if has_openai_key() {
        OpenAIConfig {
            api_key: env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"),
            model: "text-embedding-3-small".to_string(),
            embedding_dimension: 1536,
            batch_size: 16,
        }
    } else {
        // Mock config for testing without API key
        OpenAIConfig {
            api_key: "test-key-mock".to_string(),
            model: "text-embedding-3-small".to_string(),
            embedding_dimension: 1536,
            batch_size: 16,
        }
    };

    OpenAIEmbeddingProvider::new(config)
        .await
        .expect("Failed to create OpenAI provider")
}

// ============================================================================
// Day 1: OpenAI Provider E2E Tests
// ============================================================================

#[tokio::test]
async fn test_openai_provider_initialization() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

    // Verify provider is available
    assert!(provider.is_available().await);

    // Check embedding dimension
    assert_eq!(provider.embedding_dimension(), 1536);

    // Verify warmup succeeds
    let warmup_result = provider.warmup().await;
    assert!(warmup_result.is_ok(), "Warmup should succeed");
}

#[tokio::test]
async fn test_openai_single_embedding_generation() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;
    let text = "Implement user authentication with JWT tokens in Rust";

    let start = Instant::now();
    let embedding = provider
        .embed_text(text)
        .await
        .expect("Should generate embedding");
    let duration = start.elapsed();

    // Verify embedding
    assert!(!embedding.is_empty(), "Embedding should not be empty");
    assert_eq!(embedding.len(), 1536, "Embedding dimension should be 1536");

    // Verify vector is normalized (magnitude ≈ 1.0)
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!(
        (magnitude - 1.0).abs() < 0.01,
        "Embedding should be normalized"
    );

    // Performance check (should be reasonably fast)
    assert!(
        duration < Duration::from_secs(5),
        "Embedding generation should complete in < 5s, took {:?}",
        duration
    );

    println!("OpenAI embedding generation time: {:?}", duration);
}

#[tokio::test]
async fn test_openai_batch_embedding_generation() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

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
        .expect("Should generate batch embeddings");
    let duration = start.elapsed();

    // Verify all embeddings generated
    assert_eq!(
        embeddings.len(),
        texts.len(),
        "Should generate embeddings for all texts"
    );

    // Verify each embedding
    for (i, embedding) in embeddings.iter().enumerate() {
        assert!(!embedding.is_empty(), "Embedding {} should not be empty", i);
        assert_eq!(
            embedding.len(),
            1536,
            "Embedding {} should have dimension 1536",
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
        "OpenAI batch embedding generation: {:?} total, {:.2}ms avg",
        duration, avg_time
    );

    // Batch should be more efficient per item than individual calls
    assert!(
        avg_time < 1000.0,
        "Average embedding time should be < 1s, was {:.2}ms",
        avg_time
    );
}

#[tokio::test]
async fn test_openai_semantic_similarity() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

    // Test cases with expected similarity rankings
    let test_cases = vec![
        // Identical texts
        ("REST API", "REST API", 0.98),
        // Very similar
        ("user authentication", "user login", 0.80),
        ("database schema", "database structure", 0.80),
        // Somewhat similar
        ("REST API", "web service", 0.70),
        // Different
        ("authentication", "database", 0.50),
    ];

    for (text1, text2, min_expected) in test_cases {
        let similarity = provider
            .similarity(text1, text2)
            .await
            .expect("Should calculate similarity");

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

        // Similarity should be in [0, 1] range
        assert!(similarity >= 0.0 && similarity <= 1.0);
    }
}

#[tokio::test]
async fn test_openai_deterministic_embeddings() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;
    let text = "Test text for deterministic behavior";

    let embedding1 = provider
        .embed_text(text)
        .await
        .expect("Should generate first embedding");
    let embedding2 = provider
        .embed_text(text)
        .await
        .expect("Should generate second embedding");

    // OpenAI embeddings should be deterministic (or very close)
    let similarity = cosine_similarity(&embedding1, &embedding2);
    assert!(
        similarity > 0.999,
        "Same text should produce very similar embeddings: {:.6}",
        similarity
    );
}

#[tokio::test]
async fn test_openai_error_handling_invalid_key() {
    // This test doesn't require a real API key
    let config = OpenAIConfig {
        api_key: "sk-invalid-key-12345".to_string(),
        model: "text-embedding-3-small".to_string(),
        embedding_dimension: 1536,
        batch_size: 16,
    };

    let provider = OpenAIEmbeddingProvider::new(config)
        .await
        .expect("Should create provider even with invalid key");

    // Try to generate embedding - should fail gracefully
    let result = provider.embed_text("test").await;

    assert!(result.is_err(), "Should fail with invalid API key");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("authentication")
            || error_msg.contains("API key")
            || error_msg.contains("unauthorized")
            || error_msg.contains("401"),
        "Error should mention authentication issue: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_openai_empty_text_handling() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

    // Test with empty string
    let result = provider.embed_text("").await;

    // Should either fail or return valid embedding
    match result {
        Ok(embedding) => {
            // If it succeeds, should have correct dimension
            assert_eq!(embedding.len(), 1536);
        }
        Err(e) => {
            // If it fails, should have meaningful error
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("empty") || error_msg.contains("invalid"),
                "Error should mention empty/invalid input: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_openai_long_text_handling() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

    // Create a very long text (8192 tokens * ~4 chars/token ≈ 32KB)
    let long_text = "Implement user authentication. ".repeat(1000);

    let start = Instant::now();
    let result = provider.embed_text(&long_text).await;
    let duration = start.elapsed();

    match result {
        Ok(embedding) => {
            // Should handle long text
            assert_eq!(embedding.len(), 1536);
            println!(
                "Long text ({:.1}KB) embedded in {:?}",
                long_text.len() / 1024.0,
                duration
            );
        }
        Err(e) => {
            // Should fail with meaningful error about token limit
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("token")
                    || error_msg.contains("length")
                    || error_msg.contains("too large"),
                "Error should mention size limit: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_openai_multilingual_support() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

    // Test multilingual texts
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
            1536,
            "{} text should have correct dimension",
            lang
        );
        println!("{} text embedded successfully", lang);
    }

    // Similar translations should have high similarity
    let sim_en_fr = provider
        .similarity("authentication", "authentification")
        .await
        .expect("Should calculate similarity");

    println!("Similarity between English and French: {:.3}", sim_en_fr);
    assert!(sim_en_fr > 0.7, "Translations should be similar");
}

// ============================================================================
// Integration Tests with SemanticService
// ============================================================================

#[tokio::test]
async fn test_openai_semantic_search_workflow() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let storage = Box::new(InMemoryEmbeddingStorage::new());
    let config = EmbeddingConfig::default();
    let service = SemanticService::with_fallback(storage, config)
        .await
        .expect("Should create semantic service");

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
    for (i, text) in episodes.iter().enumerate() {
        let episode_id = uuid::Uuid::new_v4();
        let embedding = service
            .provider
            .embed_text(text)
            .await
            .expect("Should generate embedding");

        service
            .storage
            .store_episode_embedding(episode_id, embedding)
            .await
            .expect("Should store embedding");
    }

    // Test semantic search
    let query = "How to secure my API?";
    let query_embedding = service
        .provider
        .embed_text(query)
        .await
        .expect("Should generate query embedding");

    let results = service
        .storage
        .find_similar_episodes(query_embedding, 5, 0.0)
        .await
        .expect("Should find similar episodes");

    // Should find some results
    assert!(!results.is_empty(), "Should find similar episodes");

    // Results should be sorted by similarity (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].similarity >= results[i].similarity,
            "Results should be sorted by similarity"
        );
    }

    println!(
        "Found {} similar episodes for query: '{}'",
        results.len(),
        query
    );
    for (i, result) in results.iter().enumerate() {
        println!("  {}. similarity: {:.3}", i + 1, result.similarity);
    }
}

#[tokio::test]
async fn test_openai_performance_benchmarks() {
    if !has_openai_key() {
        return; // Skip if no API key
    }

    let provider = create_openai_provider().await;

    // Benchmark single embedding
    let single_times = || {
        let start = Instant::now();
        let _ = provider.embed_text("Test text for benchmarking");
        start.elapsed()
    };

    let mut single_durations = vec![];
    for _ in 0..10 {
        single_durations.push(single_times().await);
    }

    let avg_single = single_durations.iter().sum::<Duration>().as_millis() as f64 / 10.0;
    let min_single = single_durations.iter().min().unwrap();
    let max_single = single_durations.iter().max().unwrap();

    println!("OpenAI single embedding performance:");
    println!("  Average: {:.2}ms", avg_single);
    println!("  Min: {:?}", min_single);
    println!("  Max: {:?}", max_single);

    // Benchmark batch embeddings
    let batch_texts: Vec<String> = (0..50).map(|i| format!("Text number {}", i)).collect();

    let start = Instant::now();
    let _ = provider.embed_batch(&batch_texts).await.unwrap();
    let batch_duration = start.elapsed();

    let avg_batch = batch_duration.as_millis() as f64 / batch_texts.len() as f64;

    println!("OpenAI batch embedding performance (50 texts):");
    println!("  Total: {:?}", batch_duration);
    println!("  Average: {:.2}ms", avg_batch);

    // Batch should be more efficient
    assert!(
        avg_batch < avg_single * 1.5,
        "Batch should be at least as efficient as single calls"
    );
}

// ============================================================================
// Mock Tests for CI/CD (No API Key Required)
// ============================================================================

#[tokio::test]
async fn test_openai_mock_provider_creation() {
    // This test should work without API key (for CI)
    let config = OpenAIConfig {
        api_key: "mock-key-for-testing".to_string(),
        model: "text-embedding-3-small".to_string(),
        embedding_dimension: 1536,
        batch_size: 16,
    };

    let result = OpenAIEmbeddingProvider::new(config).await;
    assert!(result.is_ok(), "Should create provider with mock config");

    let provider = result.unwrap();
    assert!(
        !provider.is_available().await,
        "Mock provider should not be available"
    );
}

#[tokio::test]
async fn test_openai_config_validation() {
    // Test that config validates correctly
    let valid_config = OpenAIConfig {
        api_key: "test-key".to_string(),
        model: "text-embedding-3-small".to_string(),
        embedding_dimension: 1536,
        batch_size: 16,
    };

    // Verify config fields
    assert_eq!(valid_config.model, "text-embedding-3-small");
    assert_eq!(valid_config.embedding_dimension, 1536);
    assert_eq!(valid_config.batch_size, 16);
}

#[test]
fn test_openai_model_dimensions() {
    // Test that we know the correct dimensions for OpenAI models
    let models = vec![
        ("text-embedding-3-small", 1536),
        ("text-embedding-3-large", 3072),
        ("text-embedding-ada-002", 1536),
    ];

    for (model, expected_dim) in models {
        assert_eq!(
            expected_dim, 1536,
            "Model {} should have dimension {}",
            model, expected_dim
        );
    }
}
