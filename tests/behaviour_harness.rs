//! Behaviour harness — snapshot tests for memory search/recall behaviour.
//!
//! These tests establish a regression baseline for the memory system's core value:
//! search ranking, recall accuracy, and similarity scoring.
//!
//! Run: cargo nextest run -p e2e-tests --test `behaviour_harness`
//! Approve snapshots: cargo insta review

use std::collections::HashMap;

use do_memory_core::types::config::MemoryConfig;
use do_memory_core::types::enums::{ComplexityLevel, TaskOutcome, TaskType};
use do_memory_core::types::structs::TaskContext;
use do_memory_core::{ExecutionStep, SelfLearningMemory};

fn test_context(domain: &str) -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec!["test".to_string()],
    }
}

fn test_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0,
        ..Default::default()
    }
}

/// Store + recall exact match returns the stored entry.
#[tokio::test]
async fn snapshot_store_and_recall_exact_match() {
    let memory = SelfLearningMemory::with_config(test_config());

    let ctx = test_context("web-api");
    let ep_id = memory
        .start_episode(
            "Implement JWT authentication middleware".to_string(),
            ctx,
            TaskType::CodeGeneration,
        )
        .await;

    let step = ExecutionStep {
        step_number: 1,
        timestamp: chrono::Utc::now(),
        tool: "coder".to_string(),
        action: "Implement auth handler".to_string(),
        parameters_json: "{}".to_string(),
        result: Some(do_memory_core::ExecutionResult::Success {
            output: "auth.rs created".to_string(),
        }),
        latency_ms: 100,
        tokens_used: Some(50),
        metadata: HashMap::default(),
    };
    memory.log_step(ep_id, step).await;

    memory
        .complete_episode(
            ep_id,
            TaskOutcome::Success {
                verdict: "JWT auth done".to_string(),
                artifacts: vec!["auth.rs".to_string()],
            },
        )
        .await
        .unwrap();

    let recalled = memory.get_episode(ep_id).await.unwrap();

    // Format reward with fixed precision so f32 Debug display is platform-stable
    // (Linux may print 1.885494 while macOS prints 1.8854939 for the same bits).
    let reward_str = recalled
        .reward
        .as_ref()
        .map_or_else(|| "None".to_string(), |r| format!("{:.4}", r.total));

    insta::assert_snapshot!(
        "exact_match_recall",
        format!(
            "task={}, steps={}, complete={}, reward={}",
            recalled.task_description,
            recalled.steps.len(),
            recalled.outcome.is_some(),
            reward_str
        )
    );
}

/// Store multiple entries + search returns results ordered by relevance.
#[tokio::test]
async fn snapshot_multi_entry_search_ordering() {
    let memory = SelfLearningMemory::with_config(test_config());

    let descriptions = vec![
        "Fix database connection pooling bug",
        "Implement user authentication",
        "Add rate limiting to API endpoints",
        "Refactor error handling in storage layer",
        "Write unit tests for search module",
    ];

    for desc in &descriptions {
        let ctx = test_context("backend");
        let ep_id = memory
            .start_episode((*desc).to_string(), ctx, TaskType::CodeGeneration)
            .await;
        memory
            .complete_episode(
                ep_id,
                TaskOutcome::Success {
                    verdict: "done".to_string(),
                    artifacts: vec![],
                },
            )
            .await
            .unwrap();
    }

    // Retrieve context for a database-related task
    let query_ctx = test_context("backend");
    let results = memory
        .retrieve_relevant_context("Fix database connection timeout".to_string(), query_ctx, 5)
        .await;

    // Fixed-precision scores keep snapshots platform-stable across f32 Debug variants.
    let result_summary: Vec<String> = results
        .iter()
        .map(|ep| {
            let score = ep
                .reward
                .as_ref()
                .map_or_else(|| "None".to_string(), |r| format!("{:.4}", r.total));
            format!("{} (score={})", ep.task_description, score)
        })
        .collect();

    insta::assert_snapshot!(
        "multi_entry_search_ordering",
        format!(
            "query='Fix database connection timeout', results={:?}, total_stored={}",
            result_summary,
            descriptions.len()
        )
    );
}

/// Empty store search returns empty result set.
#[tokio::test]
async fn snapshot_empty_store_search() {
    let memory = SelfLearningMemory::with_config(test_config());

    let ctx = test_context("general");
    let results = memory
        .retrieve_relevant_context("Anything at all".to_string(), ctx, 10)
        .await;

    insta::assert_snapshot!(
        "empty_store_search",
        format!(
            "query='Anything at all', results_count={}, results_empty={}",
            results.len(),
            results.is_empty()
        )
    );
}

/// Duplicate entry store is handled deterministically.
#[tokio::test]
async fn snapshot_duplicate_entry_behaviour() {
    let memory = SelfLearningMemory::with_config(test_config());

    let ctx = test_context("backend");
    let desc = "Implement caching layer for database queries".to_string();

    let id1 = memory
        .start_episode(desc.clone(), ctx.clone(), TaskType::CodeGeneration)
        .await;
    memory
        .complete_episode(
            id1,
            TaskOutcome::Success {
                verdict: "done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let id2 = memory
        .start_episode(desc, ctx, TaskType::CodeGeneration)
        .await;
    memory
        .complete_episode(
            id2,
            TaskOutcome::Success {
                verdict: "done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let all = memory.get_all_episodes().await.unwrap();

    insta::assert_snapshot!(
        "duplicate_entry_behaviour",
        format!(
            "duplicate_desc_stored_twice={}, total_episodes={}",
            all.len() == 2,
            all.len(),
        )
    );
}
