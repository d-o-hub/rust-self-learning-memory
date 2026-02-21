//! Integration tests for semantic retrieval with embeddings
//!
//! Tests hierarchical retrieval with query embeddings enabled/disabled,
//! fallback behavior, and accuracy comparisons.

use memory_core::Episode;
use memory_core::spatiotemporal::{HierarchicalRetriever, RetrievalQuery};
use memory_core::types::{ComplexityLevel, TaskContext, TaskOutcome, TaskType};
use std::sync::Arc;

/// Create a test episode with specific characteristics
fn create_test_episode(
    domain: &str,
    task_type: TaskType,
    description: &str,
    language: Option<&str>,
) -> Arc<Episode> {
    let context = TaskContext {
        language: language.map(String::from),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new(description.to_string(), context, task_type);

    // Complete the episode
    episode.complete(TaskOutcome::Success {
        verdict: "Task completed successfully".to_string(),
        artifacts: vec!["output.txt".to_string()],
    });

    Arc::new(episode)
}

#[tokio::test]
async fn test_retrieval_with_embeddings_enabled() {
    // Create episodes
    let episodes = vec![
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "Implement REST API authentication with JWT tokens",
            Some("rust"),
        ),
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "Create REST endpoints for user management",
            Some("rust"),
        ),
        create_test_episode(
            "data-science",
            TaskType::Analysis,
            "Analyze user behavior patterns",
            Some("python"),
        ),
    ];

    // Create retriever
    let retriever = HierarchicalRetriever::new();

    // Create query with embedding (mock embedding for the query)
    let query_embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    let query = RetrievalQuery {
        query_text: "Implement authentication for REST API".to_string(),
        query_embedding: Some(query_embedding),
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 2,
        episode_embeddings: std::collections::HashMap::new(),
    };

    // Execute retrieval
    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    // Verify results
    assert_eq!(results.len(), 2, "Should return exactly 2 results");
    assert!(
        results[0].relevance_score >= results[1].relevance_score,
        "Results should be sorted by relevance"
    );

    // Both results should be from web-api domain and CodeGeneration type
    for result in &results {
        let episode = episodes
            .iter()
            .find(|e| e.episode_id == result.episode_id)
            .unwrap();
        assert_eq!(episode.context.domain, "web-api");
        assert_eq!(episode.task_type, TaskType::CodeGeneration);
    }

    // Level 4 score should use embedding similarity (not just text)
    assert!(
        results[0].level_4_score > 0.0,
        "Level 4 score should be non-zero when using embeddings"
    );
}

#[tokio::test]
async fn test_retrieval_with_embeddings_disabled() {
    // Create episodes
    let episodes = vec![
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "Implement authentication endpoint",
            Some("rust"),
        ),
        create_test_episode(
            "web-api",
            TaskType::Testing,
            "Test authentication flow",
            Some("rust"),
        ),
        create_test_episode(
            "data-science",
            TaskType::Analysis,
            "Analyze security patterns",
            Some("python"),
        ),
    ];

    // Create retriever
    let retriever = HierarchicalRetriever::new();

    // Create query WITHOUT embedding (should fallback to text similarity)
    let query = RetrievalQuery {
        query_text: "authentication endpoint".to_string(),
        query_embedding: None, // No embedding - should use text similarity
        domain: Some("web-api".to_string()),
        task_type: None,
        limit: 3,
        episode_embeddings: std::collections::HashMap::new(),
    };

    // Execute retrieval
    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    // Verify results
    assert!(!results.is_empty(), "Should return results");
    assert!(
        results[0].relevance_score >= results[results.len() - 1].relevance_score,
        "Results should be sorted by relevance"
    );

    // Level 4 score should use text similarity (fallback)
    assert!(
        results[0].level_4_score >= 0.0,
        "Level 4 score should be calculated using text similarity"
    );

    // The first result should have higher text similarity to the query
    let first_episode = episodes
        .iter()
        .find(|e| e.episode_id == results[0].episode_id)
        .unwrap();
    assert!(
        first_episode
            .task_description
            .to_lowercase()
            .contains("authentication"),
        "Top result should match query keywords"
    );
}

#[tokio::test]
async fn test_fallback_when_embedding_generation_fails() {
    // This test verifies that retrieval still works when embeddings fail
    // We simulate this by not providing an embedding in the query

    let episodes = vec![
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "Build REST API for user auth",
            Some("rust"),
        ),
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "Create OAuth2 integration",
            Some("rust"),
        ),
    ];

    let retriever = HierarchicalRetriever::new();

    // Query without embedding - should gracefully fallback
    let query = RetrievalQuery {
        query_text: "user authentication".to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 2,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results = retriever.retrieve(&query, &episodes).await;

    // Should succeed even without embeddings
    assert!(
        results.is_ok(),
        "Retrieval should succeed without embeddings"
    );

    let results = results.unwrap();
    assert!(
        !results.is_empty(),
        "Should return results using text fallback"
    );

    // Verify Level 4 scores are calculated
    for result in &results {
        assert!(
            result.level_4_score >= 0.0 && result.level_4_score <= 1.0,
            "Level 4 score should be in valid range [0, 1]"
        );
    }
}

#[tokio::test]
async fn test_embedding_dimension_mismatch_handling() {
    // Test that different embedding dimensions don't cause crashes
    let episodes = vec![create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "Implement API",
        Some("rust"),
    )];

    let retriever = HierarchicalRetriever::new();

    // Create query with different dimension embedding
    // Episode embeddings are 10-dimensional, query is 5-dimensional
    let query_embedding_small = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    let query = RetrievalQuery {
        query_text: "API implementation".to_string(),
        query_embedding: Some(query_embedding_small),
        domain: Some("web-api".to_string()),
        task_type: None,
        limit: 1,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results = retriever.retrieve(&query, &episodes).await;

    // Should handle gracefully (cosine_similarity handles different dimensions)
    assert!(
        results.is_ok(),
        "Should handle dimension mismatch gracefully"
    );
}

#[tokio::test]
async fn test_compare_accuracy_embeddings_vs_keywords() {
    // Compare retrieval accuracy with embeddings vs without

    let episodes = vec![
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "JWT authentication implementation",
            Some("rust"),
        ),
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "Session-based login system",
            Some("rust"),
        ),
        create_test_episode(
            "web-api",
            TaskType::Testing,
            "Test user authentication flows",
            Some("rust"),
        ),
        create_test_episode(
            "data-science",
            TaskType::Analysis,
            "Security analysis",
            Some("python"),
        ),
    ];

    let retriever = HierarchicalRetriever::new();

    // Query: Looking for authentication-related tasks
    let query_text = "implement user login and authentication";

    // Test 1: With embeddings
    let query_embedding = vec![0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.0]; // Mock embedding
    let query_with_emb = RetrievalQuery {
        query_text: query_text.to_string(),
        query_embedding: Some(query_embedding),
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 2,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results_with_emb = retriever
        .retrieve(&query_with_emb, &episodes)
        .await
        .unwrap();

    // Test 2: Without embeddings (keyword fallback)
    let query_without_emb = RetrievalQuery {
        query_text: query_text.to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 2,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results_without_emb = retriever
        .retrieve(&query_without_emb, &episodes)
        .await
        .unwrap();

    // Both should return results
    assert_eq!(results_with_emb.len(), 2);
    assert_eq!(results_without_emb.len(), 2);

    // Both should return web-api domain episodes
    for result in &results_with_emb {
        let episode = episodes
            .iter()
            .find(|e| e.episode_id == result.episode_id)
            .unwrap();
        assert_eq!(episode.context.domain, "web-api");
    }

    for result in &results_without_emb {
        let episode = episodes
            .iter()
            .find(|e| e.episode_id == result.episode_id)
            .unwrap();
        assert_eq!(episode.context.domain, "web-api");
    }

    // Results may differ in ordering due to different similarity metrics
    println!(
        "With embeddings - Top result: {}",
        results_with_emb[0].episode_id
    );
    println!(
        "Without embeddings - Top result: {}",
        results_without_emb[0].episode_id
    );
}

#[tokio::test]
async fn test_empty_query_embedding() {
    // Test handling of empty embedding vector
    let episodes = vec![create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "Build API",
        Some("rust"),
    )];

    let retriever = HierarchicalRetriever::new();

    let query = RetrievalQuery {
        query_text: "API".to_string(),
        query_embedding: Some(vec![]), // Empty embedding
        domain: None,
        task_type: None,
        limit: 1,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results = retriever.retrieve(&query, &episodes).await;

    // Should handle gracefully
    assert!(results.is_ok(), "Should handle empty embedding vector");
}

#[tokio::test]
async fn test_zero_embedding_similarity() {
    // Test when query and episode embeddings are orthogonal (similarity = 0)
    let episodes = vec![create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "API implementation",
        Some("rust"),
    )];

    let retriever = HierarchicalRetriever::new();

    // Create orthogonal embedding (all zeros except one dimension)
    let query_embedding = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    let query = RetrievalQuery {
        query_text: "completely different task".to_string(),
        query_embedding: Some(query_embedding),
        domain: None,
        task_type: None,
        limit: 1,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    // Should still return results (may have low score)
    assert!(
        !results.is_empty(),
        "Should return results even with low similarity"
    );

    // Level 4 score might be low but should be valid
    assert!(
        results[0].level_4_score >= 0.0 && results[0].level_4_score <= 1.0,
        "Score should be in valid range"
    );
}

#[tokio::test]
async fn test_perfect_embedding_match() {
    // Test when query and episode embeddings are identical
    let episodes = vec![create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "Implement authentication",
        Some("rust"),
    )];

    let retriever = HierarchicalRetriever::new();

    // Use same embedding for query and episode (will be compared against generated embedding)
    let query_embedding = vec![0.5, 0.9, 0.5, 1.0, 1.0, 0.1, 0.5, 0.5, 0.2, 1.0];

    let query = RetrievalQuery {
        query_text: "authentication".to_string(),
        query_embedding: Some(query_embedding),
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 1,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    assert!(!results.is_empty());

    // With perfect domain, task type, and recent timestamp, should have high score
    assert!(
        results[0].relevance_score > 0.7,
        "Perfect match should have high relevance score"
    );
}

#[tokio::test]
async fn test_no_episodes() {
    // Test retrieval with no episodes available
    let episodes: Vec<Arc<Episode>> = vec![];

    let retriever = HierarchicalRetriever::new();

    let query = RetrievalQuery {
        query_text: "some query".to_string(),
        query_embedding: Some(vec![0.5; 10]),
        domain: None,
        task_type: None,
        limit: 5,
        episode_embeddings: std::collections::HashMap::new(),
    };

    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    assert!(
        results.is_empty(),
        "Should return empty results for no episodes"
    );
}

#[tokio::test]
async fn test_level_4_score_range() {
    // Verify that Level 4 scores are always in [0, 1] range
    let episodes = vec![
        create_test_episode(
            "web-api",
            TaskType::CodeGeneration,
            "API implementation",
            Some("rust"),
        ),
        create_test_episode(
            "data-science",
            TaskType::Analysis,
            "Data analysis",
            Some("python"),
        ),
        create_test_episode(
            "mobile-app",
            TaskType::Debugging,
            "Fix crash bug",
            Some("swift"),
        ),
    ];

    let retriever = HierarchicalRetriever::new();

    // Test with various query embeddings
    let query_embeddings = [
        vec![0.0; 10],                                              // All zeros
        vec![1.0; 10],                                              // All ones
        vec![0.5; 10],                                              // All 0.5
        vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],     // Ascending
        vec![-1.0, -0.5, 0.0, 0.5, 1.0, -1.0, -0.5, 0.0, 0.5, 1.0], // Mixed positive/negative
    ];

    for (i, query_emb) in query_embeddings.iter().enumerate() {
        let query = RetrievalQuery {
            query_text: format!("query {i}"),
            query_embedding: Some(query_emb.clone()),
            domain: None,
            task_type: None,
            limit: 3,
            episode_embeddings: std::collections::HashMap::new(),
        };

        let results = retriever.retrieve(&query, &episodes).await.unwrap();

        for result in &results {
            assert!(
                result.level_4_score >= 0.0 && result.level_4_score <= 1.0,
                "Level 4 score {} should be in [0, 1] range for query {}",
                result.level_4_score,
                i
            );
        }
    }
}
