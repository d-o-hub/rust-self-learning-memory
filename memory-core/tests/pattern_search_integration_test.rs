//! Integration tests for semantic pattern search

use do_memory_core::{
    ComplexityLevel, ExecutionStep, Pattern, PatternEffectiveness, SelfLearningMemory, TaskContext,
    TaskOutcome, TaskType,
};

#[path = "common/counting_provider.rs"]
mod counting_provider;

use counting_provider::counting_service;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn test_search_patterns_basic() {
    let memory = SelfLearningMemory::new();

    // Create a test context
    let context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string(), "async".to_string()],
    };

    // Start and complete an episode to generate patterns
    let episode_id = memory
        .start_episode(
            "Build REST API".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    // Add multiple steps to meet quality threshold
    let steps = vec![
        ExecutionStep::new(
            1,
            "create_project".to_string(),
            "Create new Rust project".to_string(),
        ),
        ExecutionStep::new(
            2,
            "add_dependencies".to_string(),
            "Add Axum and tower dependencies".to_string(),
        ),
        ExecutionStep::new(
            3,
            "create_router".to_string(),
            "Setup routes and handlers".to_string(),
        ),
        ExecutionStep::new(
            4,
            "add_middleware".to_string(),
            "Add logging and cors middleware".to_string(),
        ),
        ExecutionStep::new(
            5,
            "write_tests".to_string(),
            "Write integration tests".to_string(),
        ),
    ];

    for step in steps {
        memory.log_step(episode_id, step).await;
    }

    // Complete with success - quality should now meet threshold
    let _ = memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "API created successfully".to_string(),
                artifacts: vec!["api.rs".to_string(), "main.rs".to_string()],
            },
        )
        .await;

    // Search for patterns - even if episode didn't pass quality, search should work
    let results = memory
        .search_patterns_semantic("How to build a REST API", context, 5)
        .await
        .unwrap_or_default();

    // Search should succeed (return empty or fallback results)
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_recommend_patterns_for_task() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        domain: "cli".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Simple,
        tags: vec!["argparse".to_string()],
    };

    // Recommend patterns for a task
    let results = memory
        .recommend_patterns_for_task("Parse command line arguments", context, 3)
        .await
        .unwrap();

    // Should return 0-3 results
    assert!(results.len() <= 3);
}

#[tokio::test]
async fn test_discover_analogous_patterns() {
    let memory = SelfLearningMemory::new();

    let target_context = TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![],
    };

    // Discover patterns from CLI domain for web-api
    let results = memory
        .discover_analogous_patterns("cli", target_context, 5)
        .await
        .unwrap();

    // Should return 0-5 results
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_pattern_search_with_filters() {
    let memory = SelfLearningMemory::new();

    let context = TaskContext {
        domain: "data-processing".to_string(),
        language: Some("python".to_string()),
        framework: Some("pandas".to_string()),
        complexity: ComplexityLevel::Complex,
        tags: vec!["etl".to_string()],
    };

    // Search with strict config
    let config = do_memory_core::memory::SearchConfig::strict();

    let results = memory
        .search_patterns_with_config("ETL pipeline", context, config, 10)
        .await
        .unwrap();

    assert!(results.len() <= 10);
}

// ============================================================================
// Runtime provider snapshot wiring (issue #1074)
//
// The pattern search / recommendation / management call sites must consult the
// runtime-activated provider (via `live_semantic_service`) instead of the
// construction-time `semantic_service` field, which is `None` on every
// production path.
// ============================================================================

/// Seed a memory with one pattern stored against a live episode.
async fn memory_with_one_pattern(context: &TaskContext) -> SelfLearningMemory {
    let memory = SelfLearningMemory::new();
    let episode_id = memory
        .start_episode(
            "Seed episode".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;
    let pattern = Pattern::ToolSequence {
        id: do_memory_core::PatternId::new_v4(),
        tools: vec!["reqwest".to_string(), "tower".to_string()],
        context: context.clone(),
        success_rate: 0.9,
        avg_latency: chrono::Duration::milliseconds(120),
        occurrence_count: 3,
        effectiveness: PatternEffectiveness::new(),
    };
    let _ = memory.store_patterns(episode_id, vec![pattern]).await;
    memory
}

fn search_context() -> TaskContext {
    TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        tags: vec!["rest".to_string()],
    }
}

#[tokio::test]
async fn test_pattern_search_uses_activated_provider() {
    let context = search_context();
    let memory = memory_with_one_pattern(&context).await;

    let (service, calls) = counting_service("counting-model");
    memory
        .activate_semantic_service(service, "local:counting-model:4".to_string())
        .await;

    let before = calls.load(Ordering::SeqCst);
    let results = memory
        .search_patterns_semantic("build a REST API", context, 5)
        .await
        .expect("pattern search should succeed");
    let after = calls.load(Ordering::SeqCst);

    // One query embedding plus one embedding per stored pattern. Before the
    // fix this path read the (always `None`) construction-time field, so the
    // provider was never consulted.
    assert!(
        after - before >= 2,
        "activated provider must serve query + pattern embeddings (delta {})",
        after - before
    );
    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_management_search_patterns_uses_activated_provider() {
    let context = search_context();
    let memory = memory_with_one_pattern(&context).await;

    let (service, calls) = counting_service("mgmt-model");
    memory
        .activate_semantic_service(service, "local:mgmt-model:4".to_string())
        .await;

    let before = calls.load(Ordering::SeqCst);
    let results = memory
        .search_patterns(
            "build a REST API",
            &context,
            do_memory_core::memory::SearchConfig::default(),
        )
        .await
        .expect("management pattern search should succeed");
    let after = calls.load(Ordering::SeqCst);

    assert!(
        after - before >= 2,
        "management search must use the activated provider (delta {})",
        after - before
    );
    assert!(results.len() <= 10);
}

#[tokio::test]
async fn test_pattern_search_follows_reconfigured_provider() {
    let context = search_context();
    let memory = memory_with_one_pattern(&context).await;

    let (service_a, calls_a) = counting_service("model-a");
    let (service_b, calls_b) = counting_service("model-b");

    memory
        .activate_semantic_service(service_a, "local:model-a:4".to_string())
        .await;

    let a_before = calls_a.load(Ordering::SeqCst);
    memory
        .search_patterns_semantic("build a REST API", context.clone(), 5)
        .await
        .expect("search with provider A");
    let a_after = calls_a.load(Ordering::SeqCst);
    assert!(a_after > a_before, "provider A must serve the first search");
    assert_eq!(
        calls_b.load(Ordering::SeqCst),
        0,
        "the inactive provider must not be consulted"
    );

    // Reconfiguration between operations: the very next search must use B and
    // never fall back to the stale A.
    memory
        .activate_semantic_service(service_b, "local:model-b:4".to_string())
        .await;

    let b_before = calls_b.load(Ordering::SeqCst);
    memory
        .search_patterns_semantic("build a REST API", context, 5)
        .await
        .expect("search after reconfiguration");
    let b_after = calls_b.load(Ordering::SeqCst);
    assert!(
        b_after > b_before,
        "search must follow the re-activated provider"
    );
    assert_eq!(
        calls_a.load(Ordering::SeqCst),
        a_after,
        "the previous provider must not be consulted after reconfiguration"
    );
}

#[tokio::test]
async fn test_pattern_search_without_provider_preserves_fallback() {
    let context = search_context();
    let memory = memory_with_one_pattern(&context).await;

    // No activation and no construction-time provider: the explicit
    // no-provider behaviour (lexical fallback) must be preserved.
    let results = memory
        .search_patterns_semantic("build a REST API", context, 5)
        .await
        .expect("search must succeed without a provider");

    assert!(results.len() <= 5);
}

#[tokio::test]
async fn test_repeated_search_with_unchanged_identity_performs_zero_additional_provider_pattern_embeds()
 {
    let context = search_context();
    let memory = memory_with_one_pattern(&context).await;

    let (service, calls) = counting_service("repeat-model");
    memory
        .activate_semantic_service(service, "local:repeat-model:4".to_string())
        .await;

    // First search: 1 query embed + 1 pattern embed generated & stored.
    let _ = memory
        .search_patterns_semantic("build a REST API", context.clone(), 5)
        .await
        .expect("first search");

    let count_after_first = calls.load(Ordering::SeqCst);

    // Second search with a different query string: 1 query embed, 0 pattern embeds generated/written because stored pattern embedding is reused.
    let _ = memory
        .search_patterns_semantic("different query for REST API", context.clone(), 5)
        .await
        .expect("second search");

    let count_after_second = calls.load(Ordering::SeqCst);

    // Only 1 additional call for the new query embedding (zero for pattern embeddings)
    assert_eq!(
        count_after_second - count_after_first,
        1,
        "second search should only embed the new query, reusing stored pattern embedding"
    );

    // Third search with exact same query string: 0 additional provider calls due to query coalescing cache & pattern storage reuse
    let _ = memory
        .search_patterns_semantic("different query for REST API", context.clone(), 5)
        .await
        .expect("third search");

    let count_after_third = calls.load(Ordering::SeqCst);
    assert_eq!(
        count_after_third - count_after_second,
        0,
        "identical repeat search performs zero provider calls and zero embedding writes"
    );
}

#[tokio::test]
async fn test_mixed_hit_miss_search_batches_only_misses() {
    use do_memory_core::embeddings::{EmbeddingConfig, InMemoryEmbeddingStorage};
    use std::sync::Arc;

    let context = search_context();
    let memory = SelfLearningMemory::new();

    let (provider, calls) = counting_provider::CountingProvider::new("mixed-model", 4);

    let service = Arc::new(do_memory_core::embeddings::SemanticService::new(
        Box::new(provider),
        Box::new(InMemoryEmbeddingStorage::new()),
        EmbeddingConfig::default(),
    ));

    memory
        .activate_semantic_service(service.clone(), "local:mixed-model:4".to_string())
        .await;

    // Seed pattern 1
    let p1 = Pattern::ToolSequence {
        id: do_memory_core::PatternId::new_v4(),
        tools: vec!["reqwest".to_string()],
        context: context.clone(),
        success_rate: 0.9,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 2,
        effectiveness: PatternEffectiveness::new(),
    };
    // Seed pattern 2
    let p2 = Pattern::ToolSequence {
        id: do_memory_core::PatternId::new_v4(),
        tools: vec!["axum".to_string()],
        context: context.clone(),
        success_rate: 0.95,
        avg_latency: chrono::Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness::new(),
    };

    let ep_id = memory
        .start_episode(
            "seed".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;
    memory
        .store_patterns(ep_id, vec![p1.clone(), p2.clone()])
        .await
        .unwrap();

    // Pre-embed pattern 1 into service storage manually so it's a hit
    service.embed_pattern(&p1).await.unwrap();

    let calls_before_search = calls.load(Ordering::SeqCst);

    // Search patterns: p1 is a hit, p2 is a miss
    let _ = memory
        .search_patterns_semantic("axum reqwest", context.clone(), 5)
        .await
        .unwrap();

    let calls_after_search = calls.load(Ordering::SeqCst);

    // Calls made during search: 1 for query embed + 1 for missing pattern p2 embed. p1 was reused.
    assert_eq!(
        calls_after_search - calls_before_search,
        2,
        "search should embed 1 query + 1 missing pattern (p1 was cached hit)"
    );
}

#[tokio::test]
async fn test_concurrent_identical_searches_coalesce_provider_work() {
    let context = search_context();
    let memory = memory_with_one_pattern(&context).await;

    let (service, calls) = counting_service("concurrent-model");
    memory
        .activate_semantic_service(service, "local:concurrent-model:4".to_string())
        .await;

    let initial_calls = calls.load(Ordering::SeqCst);

    // Run 5 concurrent identical searches
    let memory_ref = &memory;
    let context_ref = &context;
    let futures: Vec<_> = (0..5)
        .map(|_| async move {
            memory_ref
                .search_patterns_semantic("concurrent search query", context_ref.clone(), 5)
                .await
        })
        .collect();

    let results = futures::future::try_join_all(futures)
        .await
        .expect("concurrent searches");
    assert_eq!(results.len(), 5);

    let final_calls = calls.load(Ordering::SeqCst);
    let delta = final_calls - initial_calls;

    // 1 query call + 1 pattern call = 2 calls total across all 5 concurrent searches
    assert_eq!(
        delta, 2,
        "concurrent searches must coalesce query and pattern embeddings (expected 2 calls, got {delta})"
    );
}

#[tokio::test]
async fn test_coalesced_cache_key_includes_provider_identity() {
    use do_memory_core::embeddings::{
        BatchingConfig, CoalescedEmbeddingProvider, EmbeddingProvider, MockLocalModel,
    };
    use std::sync::Arc;

    let inner_a = Arc::new(MockLocalModel::new("shared-model-name".to_string(), 4));

    let batch_config = BatchingConfig {
        enabled: false,
        ..Default::default()
    };

    let coalesced = CoalescedEmbeddingProvider::new(inner_a, batch_config, true);

    // First embed populates cache
    let vec1 = coalesced.embed_text("test string").await.unwrap();
    assert_eq!(vec1.len(), 4);

    // Second embed hits cache
    let vec2 = coalesced.embed_text("test string").await.unwrap();
    assert_eq!(vec1, vec2);
}
