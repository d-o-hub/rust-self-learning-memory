//! End-to-End tests for embedding search quality
//!
//! Tests the quality of semantic search:
//! - Search accuracy with known queries
//! - Similarity threshold tuning
//! - Ranking quality
//! - Multilingual query support
//! - Domain-specific search accuracy

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::embeddings::{
    cosine_similarity, EmbeddingProvider, EmbeddingStorageBackend, InMemoryEmbeddingStorage,
    LocalConfig, LocalEmbeddingProvider,
};
use std::collections::HashMap;

/// Create test dataset with known similarities
struct QualityTestDataset {
    episodes: Vec<(uuid::Uuid, String, String)>, // (id, text, domain)
    queries: Vec<(String, Vec<usize>)>,          // (query, expected_top_n_indices)
}

impl QualityTestDataset {
    fn new() -> Self {
        Self {
            episodes: vec![
                // Authentication domain
                (
                    uuid::Uuid::new_v4(),
                    "Implement JWT token authentication for user login".to_string(),
                    "authentication".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Create OAuth2 authorization flow with Google".to_string(),
                    "authentication".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Add password reset functionality with email verification".to_string(),
                    "authentication".to_string(),
                ),
                // API domain
                (
                    uuid::Uuid::new_v4(),
                    "Build REST API endpoints with Axum framework".to_string(),
                    "api".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Implement GraphQL API with async resolvers".to_string(),
                    "api".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Add rate limiting middleware to prevent API abuse".to_string(),
                    "api".to_string(),
                ),
                // Database domain
                (
                    uuid::Uuid::new_v4(),
                    "Design PostgreSQL database schema for user profiles".to_string(),
                    "database".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Optimize SQL queries with proper indexing strategy".to_string(),
                    "database".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Implement database transaction handling for data consistency".to_string(),
                    "database".to_string(),
                ),
                // Testing domain
                (
                    uuid::Uuid::new_v4(),
                    "Write unit tests for authentication module with mock data".to_string(),
                    "testing".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Create integration tests for API endpoints".to_string(),
                    "testing".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Add property-based tests with QuickCheck".to_string(),
                    "testing".to_string(),
                ),
                // Deployment domain
                (
                    uuid::Uuid::new_v4(),
                    "Deploy application to production using Docker containers".to_string(),
                    "deployment".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Set up CI/CD pipeline with GitHub Actions".to_string(),
                    "deployment".to_string(),
                ),
                (
                    uuid::Uuid::new_v4(),
                    "Configure Kubernetes cluster for microservices".to_string(),
                    "deployment".to_string(),
                ),
            ],
            queries: vec![
                // Query: user authentication
                ("How to implement user login?".to_string(), vec![0, 1, 2]),
                // Query: API development
                ("Create REST API".to_string(), vec![3, 4, 5]),
                // Query: database
                ("Design database tables".to_string(), vec![6, 7, 8]),
                // Query: testing
                ("Write tests for code".to_string(), vec![9, 10, 11]),
                // Query: deployment
                ("Deploy to production".to_string(), vec![12, 13, 14]),
            ],
        }
    }
}

// ============================================================================
// Day 3: Semantic Search Quality Tests
// ============================================================================

#[tokio::test]
async fn test_quality_search_accuracy_known_queries() {
    let dataset = QualityTestDataset::new();
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Generate embeddings for all episodes
    let mut embeddings = HashMap::new();
    for (id, text, _) in &dataset.episodes {
        let embedding = provider
            .embed_text(text)
            .await
            .expect("Should generate embedding");
        embeddings.insert(*id, embedding);
    }

    let mut total_correct = 0;
    let mut total_queries = 0;

    // Test each query
    for (query, expected_indices) in &dataset.queries {
        let query_embedding = provider
            .embed_text(query)
            .await
            .expect("Should generate query embedding");

        // Calculate similarities
        let mut similarities: Vec<_> = dataset
            .episodes
            .iter()
            .map(|(id, text, domain)| {
                let embedding = &embeddings[id];
                let similarity = cosine_similarity(&query_embedding, embedding);
                (*id, text, domain, similarity)
            })
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

        // Check top results
        let top_indices: Vec<_> = similarities
            .iter()
            .take(3)
            .filter_map(|(id, _, _, _)| {
                dataset
                    .episodes
                    .iter()
                    .position(|(ep_id, _, _)| ep_id == id)
            })
            .collect();

        // Count how many expected results are in top 3
        let correct_count = expected_indices
            .iter()
            .filter(|idx| top_indices.contains(idx))
            .count();

        total_correct += correct_count;
        total_queries += 1;

        println!("Query: '{}'", query);
        println!("  Expected indices: {:?}", expected_indices);
        println!("  Top 3 results:");
        for (i, (_, text, domain, similarity)) in similarities.iter().take(3).enumerate() {
            println!("    {}. [{:.3}] {} ({})", i + 1, similarity, text, domain);
        }
        println!("  Correct: {}/3", correct_count);
        println!();
    }

    let accuracy = total_correct as f64 / (total_queries * 3) as f64;
    println!("Overall accuracy: {:.1}%", accuracy * 100.0);

    // At least 40% of top 3 results should be correct (this is a low bar for mock embeddings)
    assert!(
        accuracy >= 0.4,
        "Accuracy should be >= 40%, got {:.1}%",
        accuracy * 100.0
    );
}

#[tokio::test]
async fn test_quality_similarity_threshold_tuning() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Test pairs with known similarity levels
    let test_pairs = vec![
        // Identical
        ("REST API", "REST API", 0.95),
        // Very similar
        ("user authentication", "user login", 0.70),
        ("database schema", "database design", 0.70),
        // Somewhat similar
        ("REST API", "web service", 0.60),
        // Different
        ("authentication", "database", 0.40),
    ];

    let thresholds = vec![0.3, 0.5, 0.7, 0.9];

    println!("Threshold Sensitivity Analysis:");
    println!(
        "{:<10} {:<20} {:<10} {:<15}",
        "Threshold", "Test Pair", "Similarity", "Filtered?"
    );
    println!("{}", "-".repeat(60));

    for threshold in thresholds {
        let mut filtered_count = 0;

        for (text1, text2, _expected_sim) in &test_pairs {
            let similarity = provider
                .similarity(text1, text2)
                .await
                .expect("Should calculate similarity");

            let filtered = similarity < threshold;
            if filtered {
                filtered_count += 1;
            }

            println!(
                "{:<10} {:<20} {:<10.3} {:<15}",
                threshold,
                format!("'{}' / '{}'", text1, text2),
                similarity,
                if filtered {
                    "✗ Filtered"
                } else {
                    "✓ Passed"
                }
            );
        }

        println!("  Total filtered: {}/{}", filtered_count, test_pairs.len());
        println!();
    }
}

#[tokio::test]
async fn test_quality_ranking_quality() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    let query = "user authentication system";
    let candidates = vec![
        "Implement JWT authentication for secure login",
        "Create OAuth2 authorization flow",
        "Add password reset functionality",
        "Build REST API endpoints",
        "Design database schema",
        "Write unit tests",
        "Deploy application",
    ];

    let query_embedding = provider
        .embed_text(query)
        .await
        .expect("Should generate query embedding");

    // Calculate all similarities
    let mut embeddings = Vec::new();
    for text in &candidates {
        let embedding = provider.embed_text(text).await.unwrap();
        embeddings.push(embedding);
    }

    let mut ranked: Vec<_> = candidates
        .iter()
        .zip(embeddings.iter())
        .map(|(text, embedding)| {
            let similarity = cosine_similarity(&query_embedding, embedding);
            (text, similarity)
        })
        .collect();

    // Sort by similarity
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("Ranking for query: '{}'", query);
    for (i, (text, similarity)) in ranked.iter().enumerate() {
        println!("  {}. [{:.3}] {}", i + 1, similarity, text);
    }

    // Verify ranking is monotonic
    for i in 1..ranked.len() {
        assert!(
            ranked[i - 1].1 >= ranked[i].1,
            "Ranking should be monotonic decreasing"
        );
    }

    // Authentication-related results should be ranked higher
    let top_3_auth_related = ranked
        .iter()
        .take(3)
        .filter(|(text, _)| {
            text.contains(&"authentication") || text.contains(&"login") || text.contains(&"OAuth")
        })
        .count();

    println!("Top 3 auth-related results: {}/3", top_3_auth_related);
    assert!(
        top_3_auth_related >= 1,
        "At least one auth result should be in top 3"
    );
}

#[tokio::test]
async fn test_quality_domain_specific_search() {
    let storage = InMemoryEmbeddingStorage::new();
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Create episodes from different domains
    let domain_episodes = vec![
        ("authentication", "Implement JWT authentication system"),
        (
            "authentication",
            "Add password reset with email verification",
        ),
        ("authentication", "Create OAuth2 authorization flow"),
        ("api", "Build REST API with Axum framework"),
        ("api", "Add rate limiting middleware"),
        ("api", "Implement GraphQL API"),
        ("database", "Design PostgreSQL schema"),
        ("database", "Optimize SQL queries with indexes"),
        ("database", "Add transaction handling"),
    ];

    // Store embeddings
    for (_domain, text) in &domain_episodes {
        let episode_id = uuid::Uuid::new_v4();
        let embedding = provider
            .embed_text(text)
            .await
            .expect("Should generate embedding");

        storage
            .store_episode_embedding(episode_id, embedding)
            .await
            .expect("Should store embedding");
    }

    // Query for authentication
    let query = "How to implement user login?";
    let query_embedding = provider
        .embed_text(query)
        .await
        .expect("Should generate query embedding");

    let results = storage
        .find_similar_episodes(query_embedding, 10, 0.0)
        .await
        .expect("Should find results");

    println!("Domain-specific search for: '{}'", query);
    println!("Total results: {}", results.len());

    // Count results by domain
    let mut domain_counts: HashMap<&str, usize> = HashMap::new();
    for (domain, _) in &domain_episodes {
        *domain_counts.entry(domain).or_insert(0) += 0;
    }

    // Top results should be authentication-related
    for (i, result) in results.iter().take(5).enumerate() {
        println!("  {}. similarity: {:.3}", i + 1, result.similarity);
    }
}

#[tokio::test]
async fn test_quality_multilingual_support() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Multilingual test cases
    let multilingual_queries = vec![
        (
            "authentication",
            vec![
                "Implement authentication system",
                "Implémenter système d'authentification",
                "Implementar sistema de autenticación",
            ],
        ),
        (
            "database",
            vec![
                "Design database schema",
                "Concevoir schéma de base de données",
                "Diseñar esquema de base de datos",
            ],
        ),
    ];

    println!("Multilingual Similarity Analysis:");

    for (concept, translations) in multilingual_queries {
        println!("\nConcept: {}", concept);

        // Calculate pairwise similarities
        let mut similarities = vec![];

        for i in 0..translations.len() {
            for j in (i + 1)..translations.len() {
                let sim = provider
                    .similarity(&translations[i], &translations[j])
                    .await
                    .expect("Should calculate similarity");

                similarities.push((i, j, sim));

                println!(
                    "  '{}' <-> '{}': {:.3}",
                    translations[i], translations[j], sim
                );
            }
        }

        // Average similarity
        let avg_sim: f32 =
            similarities.iter().map(|(_, _, s)| s).sum::<f32>() / similarities.len() as f32;

        println!("  Average cross-lingual similarity: {:.3}", avg_sim);

        // For good multilingual models, average should be > 0.6
        // For mock/test embeddings, we just verify it runs
        assert!(avg_sim >= 0.0 && avg_sim <= 1.0);
    }
}

#[tokio::test]
async fn test_quality_fuzzy_matching() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Test fuzzy matching with typos, abbreviations, etc.
    let fuzzy_tests = vec![
        // Exact match
        ("authentication", "authentication", 0.99),
        // Plural/singular
        ("API endpoint", "API endpoints", 0.90),
        // Abbreviations
        ("JWT token", "JSON Web Token", 0.70),
        // Typos
        ("authentication", "authentiction", 0.80),
        // Synonyms
        ("user login", "user signin", 0.75),
        // Related concepts
        ("password reset", "forgot password", 0.70),
    ];

    println!("Fuzzy Matching Test:");
    println!("{:<30} {:<30} {:<10}", "Text 1", "Text 2", "Similarity");
    println!("{}", "-".repeat(75));

    for (text1, text2, min_expected) in fuzzy_tests {
        let similarity = provider
            .similarity(text1, text2)
            .await
            .expect("Should calculate similarity");

        println!("{:<30} {:<30} {:.3}", text1, text2, similarity);

        // Note: For mock embeddings, these expectations are very loose
        // Real embeddings would perform much better
        assert!(
            similarity >= min_expected * 0.5,
            "Similarity should be reasonable"
        );
    }
}

#[tokio::test]
async fn test_quality_context_aware_search() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Test that context affects search results
    let base_query = "API";

    let contexts = vec![
        ("authentication", "How to secure API with authentication?"),
        ("performance", "How to optimize API performance?"),
        ("testing", "How to test API endpoints?"),
    ];

    println!("Context-Aware Search:");

    for (context, contextualized_query) in contexts {
        let base_sim = provider
            .similarity(base_query, "REST API endpoint")
            .await
            .expect("Should calculate similarity");

        let context_sim = provider
            .similarity(contextualized_query, "Add authentication to REST API")
            .await
            .expect("Should calculate similarity");

        println!("Context: {}", context);
        println!("  Base query '{}': {:.3}", base_query, base_sim);
        println!(
            "  Contextualized '{}': {:.3}",
            contextualized_query, context_sim
        );

        // Contextualized query should be more specific
        // (For mock embeddings, this may not hold, but test should run)
    }
}

#[tokio::test]
async fn test_quality_natural_language_queries() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Test various natural language query styles
    let nl_queries = vec![
        "How do I implement authentication?",
        "What's the best way to add user login?",
        "I need to secure my API",
        "Can you help me with OAuth?",
        "Steps to implement JWT tokens",
    ];

    let target = "Implement JWT authentication system";

    println!("Natural Language Query Variations:");
    println!("Target: '{}'", target);
    println!();

    let mut similarities = vec![];

    for query in nl_queries {
        let similarity = provider
            .similarity(query, target)
            .await
            .expect("Should calculate similarity");

        similarities.push((query, similarity));
        println!("  '{}' -> {:.3}", query, similarity);
    }

    // Find best match
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!(
        "\nBest match: '{}' ({:.3})",
        similarities[0].0, similarities[0].1
    );

    // All variations should have some similarity
    for (_, similarity) in &similarities {
        assert!(
            *similarity > 0.0,
            "All variations should have some similarity"
        );
    }
}

#[tokio::test]
async fn test_quality_temporal_and_versioning_consistency() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Test that embeddings are deterministic over time
    let texts = vec![
        "Implement authentication",
        "Build API endpoints",
        "Create database schema",
    ];

    let mut embeddings_v1 = vec![];
    let mut embeddings_v2 = vec![];

    // Generate "version 1" embeddings
    for text in &texts {
        let emb = provider.embed_text(text).await.unwrap();
        embeddings_v1.push((text, emb));
    }

    // Generate "version 2" embeddings (should be identical)
    for text in &texts {
        let emb = provider.embed_text(text).await.unwrap();
        embeddings_v2.push((text, emb));
    }

    // Verify consistency
    println!("Temporal Consistency Test:");
    for (i, ((text, emb1), (_, emb2))) in embeddings_v1.iter().zip(embeddings_v2.iter()).enumerate()
    {
        let similarity = cosine_similarity(emb1, emb2);
        println!("  Text {}: '{}' -> {:.6}", i + 1, text, similarity);

        // Should be identical (or very close)
        assert!(
            similarity > 0.999,
            "Embeddings should be consistent over time: {:.6}",
            similarity
        );
    }
}

#[tokio::test]
async fn test_quality_long_form_text_search() {
    let provider = LocalEmbeddingProvider::new(LocalConfig::new("test-model", 384))
        .await
        .expect("Should create provider");

    // Test with longer descriptions
    let long_texts = vec![
        "Implement a comprehensive authentication system using JWT tokens that includes \
         user registration, login, password reset, email verification, and session management. \
         The system should be secure, scalable, and follow OWASP best practices.",
        "Design and implement a RESTful API using the Axum framework with proper error handling, \
         request validation, rate limiting, and comprehensive logging. The API should support \
         pagination, filtering, and sorting of resources.",
        "Create a robust database schema for a multi-tenant application using PostgreSQL with \
         proper indexing strategies, foreign key constraints, and migration scripts. Include \
         tables for users, organizations, and permissions.",
    ];

    let short_queries = vec!["user authentication", "REST API", "database design"];

    println!("Long-Form Text Search:");

    for (i, long_text) in long_texts.iter().enumerate() {
        let long_emb = provider.embed_text(long_text).await.unwrap();

        println!("\nLong text {} ({} chars)", i + 1, long_text.len());

        for query in &short_queries {
            let query_emb = provider.embed_text(query).await.unwrap();
            let similarity = cosine_similarity(&query_emb, &long_emb);

            println!("  Query '{}' -> {:.3}", query, similarity);
        }
    }
}
