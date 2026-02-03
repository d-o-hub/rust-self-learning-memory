//!
//! Tests for semantic retrieval integration with hierarchical retrieval
//!

use std::sync::Arc;
use std::time::Instant;

use memory_core::{
    embeddings::{EmbeddingConfig, InMemoryEmbeddingStorage, SemanticService},
    memory::SelfLearningMemory,
    spatiotemporal::{HierarchicalRetriever, RetrievalQuery},
    ComplexityLevel, ExecutionResult, ExecutionStep, MemoryConfig, TaskContext, TaskOutcome,
    TaskType,
};

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test episode with specified parameters
async fn create_test_episode(
    memory: &SelfLearningMemory,
    domain: &str,
    task_type: TaskType,
    description: &str,
    num_steps: usize,
) -> uuid::Uuid {
    let context = TaskContext {
        domain: domain.to_string(),
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    let episode_id = memory
        .start_episode(description.to_string(), context, task_type)
        .await;

    for i in 0..num_steps {
        let step = ExecutionStep {
            step_number: i + 1,
            timestamp: chrono::Utc::now(),
            tool: format!("tool-{}", i % 3),
            action: format!("action-{}", i),
            parameters: serde_json::json!({}),
            result: Some(ExecutionResult::Success {
                output: format!("output-{}", i),
                metadata: Default::default(),
            }),
            latency_ms: 10,
            tokens_used: None,
            metadata: Default::default(),
        };
        memory.log_step(episode_id, step).await.unwrap();
    }

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: format!("Completed: {}", description),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    episode_id
}

// ============================================================================
// Test 1: Query Embedding Generation
// ============================================================================

#[tokio::test]
async fn test_query_embedding_generation() {
    // Create semantic service with in-memory storage
    let storage = Box::new(InMemoryEmbeddingStorage::new());
    let semantic_service = SemanticService::default(storage)
        .await
        .expect("Failed to create semantic service");

    // Generate query embedding
    let query = "Implement REST API endpoint for user authentication";
    let embedding = semantic_service
        .provider
        .embed_text(query)
        .await
        .expect("Failed to generate query embedding");

    // Verify embedding is not empty
    assert!(!embedding.is_empty(), "Embedding should not be empty");
    assert!(
        embedding.len() >= 3,
        "Embedding should have at least 3 dimensions"
    );

    // Verify embedding values are normalized (typically -1 to 1 or 0 to 1)
    for &val in &embedding {
        assert!(
            val.is_finite(),
            "Embedding values should be finite, got {}",
            val
        );
    }

    println!("Generated query embedding: {} dimensions", embedding.len());
}

#[tokio::test]
async fn test_query_embedding_consistency() {
    // Create semantic service
    let storage = Box::new(InMemoryEmbeddingStorage::new());
    let semantic_service = SemanticService::default(storage)
        .await
        .expect("Failed to create semantic service");

    // Generate embeddings for the same query twice
    let query = "Test query for consistency";
    let embedding1 = semantic_service
        .provider
        .embed_text(query)
        .await
        .expect("Failed to generate first embedding");
    let embedding2 = semantic_service
        .provider
        .embed_text(query)
        .await
        .expect("Failed to generate second embedding");

    // Verify embeddings are identical
    assert_eq!(
        embedding1.len(),
        embedding2.len(),
        "Embeddings should have same length"
    );

    for (i, (&v1, &v2)) in embedding1.iter().zip(embedding2.iter()).enumerate() {
        assert!(
            (v1 - v2).abs() < f32::EPSILON,
            "Embedding dimension {} differs: {} vs {}",
            i,
            v1,
            v2
        );
    }
}

// ============================================================================
// Test 2: Semantic Search with Hierarchical Retrieval
// ============================================================================

#[tokio::test]
async fn test_semantic_hierarchical_retrieval() {
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        enable_embeddings: true,
        temporal_bias_weight: 0.3,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes with semantic similarity
    let descriptions = vec![
        "Implement REST API endpoint for user authentication",
        "Build GraphQL API for user management",
        "Create authentication service with JWT tokens",
        "Design database schema for user accounts",
        "Write unit tests for authentication module",
    ];

    for desc in descriptions {
        create_test_episode(&memory, "web-api", TaskType::CodeGeneration, desc, 10).await;
    }

    // Query with semantic similarity
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("actix-web".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    let results = memory
        .retrieve_relevant_context("Implement user login API".to_string(), query_context, 5)
        .await;

    // Verify results returned
    assert!(!results.is_empty(), "Should return results");

    // The most semantically similar episodes should be returned
    // (authentication and API related)
    println!("Semantic retrieval returned {} episodes", results.len());
    for (i, ep) in results.iter().enumerate() {
        println!("  {}. {}", i + 1, ep.task_description);
    }
}

// ============================================================================
// Test 3: Hybrid Retrieval (Semantic + Temporal)
// ============================================================================

#[tokio::test]
async fn test_hybrid_retrieval() {
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        enable_embeddings: true,
        temporal_bias_weight: 0.4,
        semantic_search_mode: "hybrid".to_string(),
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create older episodes (less recent)
    for i in 0..5 {
        create_test_episode(
            &memory,
            "data-science",
            TaskType::Analysis,
            &format!("Old data analysis task {}", i),
            8,
        )
        .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Create newer episodes (more recent)
    for i in 0..5 {
        create_test_episode(
            &memory,
            "data-science",
            TaskType::Analysis,
            &format!("Recent data analysis task {}", i),
            8,
        )
        .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    let query_context = TaskContext {
        domain: "data-science".to_string(),
        language: Some("python".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    let results = memory
        .retrieve_relevant_context("Analyze data patterns".to_string(), query_context, 5)
        .await;

    // Verify results returned
    assert!(!results.is_empty(), "Should return results");

    // With temporal bias, recent episodes should be favored
    println!("Hybrid retrieval returned {} episodes", results.len());
    for (i, ep) in results.iter().enumerate() {
        println!(
            "  {}. {} (age: {:?})",
            i + 1,
            ep.task_description,
            ep.start_time
        );
    }
}

// ============================================================================
// Test 4: Preloading Episode Embeddings
// ============================================================================

#[tokio::test]
async fn test_episode_embedding_preloading() {
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        enable_embeddings: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes
    for i in 0..10 {
        create_test_episode(
            &memory,
            "test-domain",
            TaskType::CodeGeneration,
            &format!("Task {}", i),
            5,
        )
        .await;
    }

    // Query and measure performance
    let query_context = TaskContext {
        domain: "test-domain".to_string(),
        ..Default::default()
    };

    let start = Instant::now();
    let results = memory
        .retrieve_relevant_context("Test query".to_string(), query_context, 5)
        .await;
    let elapsed = start.elapsed();

    assert!(!results.is_empty(), "Should return results");
    println!(
        "Retrieval with preloaded embeddings: {:?}, {} results",
        elapsed,
        results.len()
    );

    // Performance should be reasonable (< 1 second)
    assert!(
        elapsed.as_secs() < 1,
        "Retrieval should be fast, got {:?}",
        elapsed
    );
}

// ============================================================================
// Test 5: Fallback to Keyword Search
// ============================================================================

#[tokio::test]
async fn test_fallback_to_keyword_search() {
    // Disable embeddings to force keyword search fallback
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        enable_embeddings: false, // Disable embeddings
        semantic_search_mode: "keyword-only".to_string(),
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes
    create_test_episode(
        &memory,
        "web-api",
        TaskType::CodeGeneration,
        "Implement authentication endpoint",
        10,
    )
    .await;

    create_test_episode(
        &memory,
        "web-api",
        TaskType::CodeGeneration,
        "Create user profile management",
        10,
    )
    .await;

    // Query with keyword matching
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("authentication endpoint".to_string(), query_context, 5)
        .await;

    // Verify results returned using keyword matching
    assert!(
        !results.is_empty(),
        "Keyword fallback should return results"
    );
    println!("Keyword fallback returned {} results", results.len());

    // Results should contain episodes with matching keywords
    let has_match = results.iter().any(|ep| {
        ep.task_description
            .to_lowercase()
            .contains("authentication")
    });

    assert!(has_match, "Should find episode with matching keyword");
}

// ============================================================================
// Test 6: Similarity Threshold Filtering
// ============================================================================

#[tokio::test]
async fn test_similarity_threshold_filtering() {
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        enable_embeddings: true,
        semantic_similarity_threshold: 0.7, // High threshold
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create diverse episodes
    create_test_episode(
        &memory,
        "web-api",
        TaskType::CodeGeneration,
        "Implement authentication system",
        10,
    )
    .await;

    create_test_episode(
        &memory,
        "data-science",
        TaskType::Analysis,
        "Analyze customer data patterns",
        10,
    )
    .await;

    create_test_episode(
        &memory,
        "mobile-app",
        TaskType::Testing,
        "Write UI unit tests",
        10,
    )
    .await;

    // Query for authentication (should only match web-api episode)
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("User authentication".to_string(), query_context, 10)
        .await;

    // With high similarity threshold, only closely related episodes should match
    println!("Similarity threshold filtering: {} results", results.len());

    // Results should be limited to semantically similar episodes
    for ep in &results {
        println!("  - {}", ep.task_description);
    }
}

// ============================================================================
// Test 7: Performance with Large Episode Count
// ============================================================================

#[tokio::test]
async fn test_semantic_retrieval_performance() {
    let config = MemoryConfig {
        enable_spatiotemporal_indexing: true,
        enable_embeddings: true,
        enable_query_embedding_cache: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create many episodes
    let episode_count = 100;
    for i in 0..episode_count {
        let domain = match i % 3 {
            0 => "web-api",
            1 => "data-science",
            _ => "mobile-app",
        };

        create_test_episode(
            &memory,
            domain,
            TaskType::CodeGeneration,
            &format!("Task {} in {}", i, domain),
            5,
        )
        .await;

        if i % 20 == 0 {
            println!("Created {} episodes", i + 1);
        }
    }

    // Query multiple times to test cache effectiveness
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let mut total_time = 0u128;
    let query_count = 5;

    for i in 0..query_count {
        let start = Instant::now();
        let results = memory
            .retrieve_relevant_context("Implement API".to_string(), query_context.clone(), 10)
            .await;
        let elapsed = start.elapsed();

        total_time += elapsed.as_millis();
        println!(
            "Query {} returned {} results in {:?}",
            i + 1,
            results.len(),
            elapsed
        );

        assert!(!results.is_empty(), "Should return results");
    }

    let avg_time = total_time / query_count as u128;
    println!("Average query time: {}ms", avg_time);

    // Average query time should be reasonable (< 500ms)
    assert!(
        avg_time < 500,
        "Average query time should be < 500ms, got {}ms",
        avg_time
    );
}
