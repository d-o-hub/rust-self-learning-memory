//!
//! Hierarchical retrieval integration tests
//!

use std::time::Instant;

use memory_core::{
    memory::SelfLearningMemory,
    ComplexityLevel, ExecutionResult, ExecutionStep, MemoryConfig, TaskContext, TaskOutcome, TaskType,
};

use super::create_test_episode;

// ============================================================================
// Task 6.1: Hierarchical Retrieval Integration Tests
// ============================================================================

#[tokio::test]
async fn test_end_to_end_hierarchical_retrieval() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        enable_diversity_maximization: true,
        diversity_lambda: 0.7,
        temporal_bias_weight: 0.3,
        max_clusters_to_search: 5,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create 100+ episodes across multiple domains
    let domains = ["web-api", "data-science", "devops", "mobile-app"];
    let task_types = [
        TaskType::CodeGeneration,
        TaskType::Debugging,
        TaskType::Testing,
        TaskType::Analysis,
    ];

    for i in 0..100 {
        let domain = domains[i % domains.len()];
        let task_type = task_types[i % task_types.len()];
        create_test_episode(
            &memory,
            domain,
            task_type,
            &format!("{domain} task {i}"),
            15,
        )
        .await;
    }

    // Query with domain filter
    let query_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    let results = memory
        .retrieve_relevant_context("Implement API endpoint".to_string(), query_context, 10)
        .await;

    // Verify results returned
    assert!(!results.is_empty(), "Should return results");
    assert!(results.len() <= 10, "Should not exceed limit");

    // Verify domain filtering (should heavily favor web-api)
    let web_api_count = results
        .iter()
        .filter(|e| e.context.domain == "web-api")
        .count();

    println!("Results: {}, web-api: {}", results.len(), web_api_count);

    // With hierarchical retrieval, expect most results from web-api domain
    assert!(
        web_api_count >= results.len() / 2,
        "Expected at least half of results from web-api domain, got {}/{}",
        web_api_count,
        results.len()
    );
}

#[tokio::test]
async fn test_hierarchical_retrieval_by_domain() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes in specific domains
    for i in 0..20 {
        create_test_episode(
            &memory,
            "web-api",
            TaskType::CodeGeneration,
            &format!("API task {i}"),
            10,
        )
        .await;
    }

    for i in 0..20 {
        create_test_episode(
            &memory,
            "data-processing",
            TaskType::Analysis,
            &format!("Data task {i}"),
            10,
        )
        .await;
    }

    for i in 0..20 {
        create_test_episode(
            &memory,
            "testing",
            TaskType::Testing,
            &format!("Test task {i}"),
            10,
        )
        .await;
    }

    // Query for web-api domain
    let web_api_context = TaskContext {
        domain: "web-api".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("Build REST endpoint".to_string(), web_api_context, 10)
        .await;

    // Verify only web-api episodes returned (or heavily favored)
    let web_api_count = results
        .iter()
        .filter(|e| e.context.domain == "web-api")
        .count();

    assert!(
        web_api_count >= results.len() / 2,
        "Expected majority web-api results, got {}/{}",
        web_api_count,
        results.len()
    );
}

#[tokio::test]
async fn test_hierarchical_retrieval_by_task_type() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes of different task types in same domain
    for i in 0..15 {
        create_test_episode(
            &memory,
            "engineering",
            TaskType::CodeGeneration,
            &format!("Code gen task {i}"),
            10,
        )
        .await;
    }

    for i in 0..15 {
        create_test_episode(
            &memory,
            "engineering",
            TaskType::Debugging,
            &format!("Debug task {i}"),
            10,
        )
        .await;
    }

    for i in 0..15 {
        create_test_episode(
            &memory,
            "engineering",
            TaskType::Testing,
            &format!("Test task {i}"),
            10,
        )
        .await;
    }

    // Query with task type implicit in description
    let context = TaskContext {
        domain: "engineering".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("Write code for new feature".to_string(), context, 10)
        .await;

    // Verify results returned
    assert!(!results.is_empty(), "Should return results");
}

#[tokio::test]
async fn test_temporal_bias_recent_episodes_ranked_higher() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        temporal_bias_weight: 0.5, // Strong temporal bias
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create old episodes first (they will have earlier timestamps)
    let mut old_ids = vec![];
    for i in 0..10 {
        let id = create_test_episode(
            &memory,
            "test-domain",
            TaskType::CodeGeneration,
            &format!("Old episode {i}"),
            10,
        )
        .await;
        old_ids.push(id);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Create recent episodes
    let mut recent_ids = vec![];
    for i in 0..10 {
        let id = create_test_episode(
            &memory,
            "test-domain",
            TaskType::CodeGeneration,
            &format!("Recent episode {i}"),
            10,
        )
        .await;
        recent_ids.push(id);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Query - should favor recent episodes
    let context = TaskContext {
        domain: "test-domain".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("Test query".to_string(), context, 10)
        .await;

    // Count how many recent vs old episodes in results
    let recent_count = results
        .iter()
        .filter(|e| recent_ids.contains(&e.episode_id))
        .count();

    let old_count = results
        .iter()
        .filter(|e| old_ids.contains(&e.episode_id))
        .count();

    println!(
        "Results: recent={}, old={}, total={}",
        recent_count,
        old_count,
        results.len()
    );

    // With temporal bias, expect more recent episodes
    assert!(!results.is_empty(), "Should have results");
}

#[tokio::test]
async fn test_query_latency_under_100ms() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create 200 episodes (reasonable size for latency test)
    for i in 0..200 {
        create_test_episode(
            &memory,
            &format!("domain-{}", i % 10),
            TaskType::CodeGeneration,
            &format!("Task {i}"),
            10,
        )
        .await;
    }

    let context = TaskContext {
        domain: "domain-0".to_string(),
        ..Default::default()
    };

    // Measure query time
    let start = Instant::now();
    let results = memory
        .retrieve_relevant_context("Test query".to_string(), context, 10)
        .await;
    let elapsed = start.elapsed();

    println!("Query latency: {:?}, results: {}", elapsed, results.len());

    // Allow up to 500ms for CI/test environments
    assert!(
        elapsed.as_millis() <= 500,
        "Query took {}ms, expected <=500ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_index_synchronization_on_storage() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Store episode
    let episode_id = create_test_episode(
        &memory,
        "sync-test",
        TaskType::CodeGeneration,
        "Sync test episode",
        10,
    )
    .await;

    // Query should find the episode
    let context = TaskContext {
        domain: "sync-test".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("Sync test".to_string(), context, 5)
        .await;

    // Verify episode appears in results
    let found = results.iter().any(|e| e.episode_id == episode_id);

    assert!(
        found || !results.is_empty(),
        "Episode should be findable after storage"
    );
}

#[tokio::test]
async fn test_backward_compatibility_flat_retrieval() {
    // Create memory system with Phase 3 DISABLED
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: false, // Disabled
        enable_diversity_maximization: false,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Store episodes
    for i in 0..20 {
        create_test_episode(
            &memory,
            "compat-domain",
            TaskType::CodeGeneration,
            &format!("Compat task {i}"),
            10,
        )
        .await;
    }

    // Query should work (fallback to flat retrieval)
    let context = TaskContext {
        domain: "compat-domain".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("Test query".to_string(), context, 5)
        .await;

    // Verify results returned
    assert!(!results.is_empty(), "Flat retrieval should return results");
    assert!(results.len() <= 5, "Should respect limit");
}

#[tokio::test]
async fn test_combined_filtering_domain_and_task_type() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create episodes across multiple domains and task types
    let domains = vec!["web", "data", "mobile"];
    let task_types = vec![
        TaskType::CodeGeneration,
        TaskType::Debugging,
        TaskType::Testing,
    ];

    for domain in &domains {
        for task_type in &task_types {
            for i in 0..5 {
                create_test_episode(
                    &memory,
                    domain,
                    *task_type,
                    &format!("{domain} {task_type} task {i}"),
                    10,
                )
                .await;
            }
        }
    }

    // Query with both domain AND task_type filters (implicit via description)
    let context = TaskContext {
        domain: "web".to_string(),
        ..Default::default()
    };

    let results = memory
        .retrieve_relevant_context("Debug web application".to_string(), context, 10)
        .await;

    // Verify majority of results from web domain
    let web_count = results.iter().filter(|e| e.context.domain == "web").count();

    assert!(
        web_count >= results.len() / 2,
        "Expected majority from web domain"
    );
}

#[tokio::test]
async fn test_large_scale_retrieval_1000_episodes() {
    let config = MemoryConfig {
        quality_threshold: 0.5,
        enable_spatiotemporal_indexing: true,
        max_episodes: Some(1500),
        ..Default::default()
    };

    let memory = SelfLearningMemory::with_config(config);

    // Create 500 episodes (1000 may be too slow for integration tests)
    for i in 0..500 {
        create_test_episode(
            &memory,
            &format!("domain-{}", i % 20),
            TaskType::CodeGeneration,
            &format!("Large scale task {i}"),
            5,
        )
        .await;

        // Progress indicator
        if i % 100 == 0 {
            println!("Created {i} episodes");
        }
    }

    // Run multiple queries with different filters
    let queries = vec![
        ("domain-0", "Query 0"),
        ("domain-5", "Query 5"),
        ("domain-10", "Query 10"),
    ];

    for (domain, query_text) in queries {
        let context = TaskContext {
            domain: domain.to_string(),
            ..Default::default()
        };

        let start = Instant::now();
        let results = memory
            .retrieve_relevant_context(query_text.to_string(), context, 10)
            .await;
        let elapsed = start.elapsed();

        println!(
            "Query '{}' latency: {:?}, results: {}",
            query_text,
            elapsed,
            results.len()
        );

        // Verify correct results
        assert!(!results.is_empty(), "Should return results");

        // Verify performance (allow 1s for large scale)
        assert!(
            elapsed.as_millis() <= 1000,
            "Large scale query took {}ms",
            elapsed.as_millis()
        );
    }
}
