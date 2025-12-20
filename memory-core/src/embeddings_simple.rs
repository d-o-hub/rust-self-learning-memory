//! Simple semantic embeddings implementation
//!
//! **⚠️ WARNING: This module contains mock/test-only implementations**
//!
//! This is a simplified version that demonstrates the concept
//! without all the complex integrations that cause compilation issues.
//! The `text_to_embedding` function uses hash-based pseudo-embeddings
//! that are NOT semantically meaningful and should only be used for:
//! - Unit testing
//! - Development/demonstration purposes
//! - Fallback when real embeddings are unavailable
//!
//! **Production Use:** Use `memory-core/src/embeddings/` module with real
//! embedding models (gte-rs, ONNX runtime) for actual semantic search.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing;

/// Configuration for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Similarity threshold for search (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum batch size for embedding generation
    pub batch_size: usize,
    /// Cache embeddings to avoid regeneration
    pub cache_embeddings: bool,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            batch_size: 32,
            cache_embeddings: true,
        }
    }
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    // Normalize from [-1, 1] to [0, 1] range
    let similarity = dot_product / (magnitude_a * magnitude_b);
    (similarity + 1.0) / 2.0
}

/// Simple text to embedding converter (mock implementation)
///
/// **⚠️ PRODUCTION WARNING: This is a mock/test-only implementation**
///
/// This function generates deterministic hash-based "embeddings" that are
/// NOT semantically meaningful. The similarity between these vectors is
/// essentially random and does not reflect actual semantic similarity.
///
/// **Use Cases:**
/// - Unit testing (deterministic, fast)
/// - Development/demonstration
/// - Fallback when real embeddings unavailable
///
/// **Do NOT Use For:**
/// - Production semantic search
/// - Real similarity calculations
/// - User-facing features
///
/// **For Production:** Use `memory-core::embeddings::LocalEmbeddingProvider`
/// with the `local-embeddings` feature enabled and real ONNX models.
pub fn text_to_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Emit production warning
    tracing::warn!(
        "PRODUCTION WARNING: Using hash-based pseudo-embeddings - semantic search will not work correctly! \
         Text: '{}'. Use real embedding models for production.",
        text.chars().take(20).collect::<String>()
    );

    // Create a deterministic embedding based on text hash
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let dimension = 384; // Standard sentence transformer dimension
    let mut embedding = Vec::with_capacity(dimension);
    let mut seed = hash;

    for _ in 0..dimension {
        // Simple PRNG to generate values
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let value = ((seed >> 16) as f32) / 32768.0 - 1.0; // Range [-1, 1]
        embedding.push(value);
    }

    // Normalize the vector
    let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }

    embedding
}

/// Test-only text to embedding converter
///
/// This function is identical to `text_to_embedding` but without the warning
/// for use in tests and internal testing scenarios.
#[cfg(test)]
pub fn text_to_embedding_test(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a deterministic embedding based on text hash
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let dimension = 384; // Standard sentence transformer dimension
    let mut embedding = Vec::with_capacity(dimension);
    let mut seed = hash;

    for _ in 0..dimension {
        // Simple PRNG to generate values
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let value = ((seed >> 16) as f32) / 32768.0 - 1.0; // Range [-1, 1]
        embedding.push(value);
    }

    // Normalize the vector
    let magnitude = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for x in &mut embedding {
            *x /= magnitude;
        }
    }

    embedding
}

/// Find most similar texts from a collection
///
/// **⚠️ Uses Mock Embeddings:** This function uses hash-based pseudo-embeddings
/// that are NOT semantically meaningful. The "similarity" results are
/// essentially random and should not be used for production semantic search.
///
/// For production semantic search, use `memory-core::embeddings::SemanticService`
/// with real embedding models.
pub fn find_similar_texts(
    query: &str,
    candidates: &[String],
    limit: usize,
    threshold: f32,
) -> Vec<(usize, f32, String)> {
    tracing::warn!(
        "Using mock embeddings for semantic search - results are not semantically meaningful!"
    );

    let query_embedding = text_to_embedding(query);

    let mut similarities: Vec<(usize, f32, String)> = candidates
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let embedding = text_to_embedding(text);
            let similarity = cosine_similarity(&query_embedding, &embedding);
            (i, similarity, text.clone())
        })
        .filter(|(_, similarity, _)| *similarity >= threshold)
        .collect();

    // Sort by similarity (highest first)
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return top results
    similarities.into_iter().take(limit).collect()
}

/// Simple semantic search demonstration
///
/// **⚠️ DEMONSTRATION ONLY:** This function uses mock hash-based embeddings
/// that are NOT semantically meaningful. The results shown here are for
/// demonstration purposes only and should NOT be used to evaluate the
/// effectiveness of semantic search.
///
/// For real semantic search demonstrations, use the proper embeddings module
/// with `cargo run --features local-embeddings`.
pub fn demonstrate_semantic_search() -> Result<()> {
    println!("🧠 Semantic Search Demonstration (Mock Embeddings)");
    println!("===================================================\n");

    println!("⚠️  WARNING: This demonstration uses hash-based pseudo-embeddings");
    println!("   that are NOT semantically meaningful. Similarity scores are");
    println!("   essentially random and do not reflect actual semantic similarity.");
    println!("\n   For production semantic search, use real embedding models.");
    println!("   Enable with: cargo run --features local-embeddings\n");

    // Sample episode descriptions
    let episodes = vec![
        "Implement user authentication with JWT tokens".to_string(),
        "Build REST API endpoints for user management".to_string(),
        "Create data validation middleware for API requests".to_string(),
        "Add rate limiting to prevent API abuse".to_string(),
        "Implement OAuth2 authentication flow".to_string(),
        "Design database schema for user profiles".to_string(),
        "Write unit tests for authentication module".to_string(),
        "Deploy API to production with Docker".to_string(),
        "Monitor API performance and error rates".to_string(),
        "Document API endpoints with OpenAPI spec".to_string(),
    ];

    // Test queries
    let queries = vec![
        "How to secure API with authentication?",
        "Need to create user management endpoints",
        "Add validation to API requests",
        "Prevent API abuse and rate limiting",
    ];

    for query in queries {
        println!("🔍 Query: \"{}\"", query);
        let results = find_similar_texts(query, &episodes, 3, 0.5);

        println!("📊 Top {} similar episodes:", results.len());
        for (i, (idx, similarity, text)) in results.iter().enumerate() {
            println!(
                "  {}. [{}] {} (similarity: {:.3})",
                i + 1,
                idx,
                text,
                similarity
            );
        }
        println!();
    }

    // Demonstrate similarity calculation
    println!("🔢 Direct Similarity Examples:");
    let pairs = vec![
        ("user authentication", "login system"),
        ("REST API", "web service endpoints"),
        ("data validation", "input verification"),
        ("rate limiting", "API throttling"),
    ];

    for (text1, text2) in pairs {
        let emb1 = text_to_embedding(text1);
        let emb2 = text_to_embedding(text2);
        let similarity = cosine_similarity(&emb1, &emb2);
        println!("  \"{}\" <-> \"{}\" = {:.3}", text1, text2, similarity);
    }

    println!("\n💡 For real semantic search, use memory-core::embeddings modules");
    println!("   with proper ONNX models and sentence transformers.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors should have similarity 1.0
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);

        // Orthogonal vectors should have similarity 0.5 (normalized from 0)
        let vec3 = vec![1.0, 0.0];
        let vec4 = vec![0.0, 1.0];
        let similarity = cosine_similarity(&vec3, &vec4);
        assert!((similarity - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_text_to_embedding() {
        let embedding1 = text_to_embedding("hello world");
        let embedding2 = text_to_embedding("hello world");
        let embedding3 = text_to_embedding("different text");

        // Same text should produce same embedding
        assert_eq!(embedding1, embedding2);

        // Different text should produce different embedding
        assert_ne!(embedding1, embedding3);

        // All embeddings should be unit vectors (magnitude ≈ 1.0)
        let magnitude1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_find_similar_texts() {
        let candidates = vec![
            "implement user authentication".to_string(),
            "create REST API endpoints".to_string(),
            "add input validation".to_string(),
            "deploy with Docker".to_string(),
        ];

        let results = find_similar_texts("user login system", &candidates, 2, 0.0);

        // Should return at most 2 results
        assert!(results.len() <= 2);

        // Results should be sorted by similarity (highest first)
        if results.len() > 1 {
            assert!(results[0].1 >= results[1].1);
        }
    }

    #[test]
    fn test_embedding_config() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.batch_size, 32);
        assert!(config.cache_embeddings);
    }
}
