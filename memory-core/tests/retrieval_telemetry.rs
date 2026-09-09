//! Issue #962: retrieval telemetry is recorded on the real paths with
//! bounded labels and no query text.
//!
//! All tests use serial delta assertions against the process-global
//! registry (production call sites record there).

#![allow(clippy::unwrap_used, clippy::panic)] // test-only wiring checks

use do_memory_core::TaskOutcome;
use do_memory_core::monitoring::metrics::global_retrieval_metrics;
use do_memory_core::{
    MemoryConfig, RecommendationFeedback, RecommendationSession, SelfLearningMemory, TaskContext,
    TaskType,
};
use serial_test::serial;

/// Counters snapshot for delta assertions: (requests, fallbacks, feedback).
fn counters() -> (u64, u64, u64) {
    let snapshot = global_retrieval_metrics().snapshot();
    let requests: u64 = snapshot["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["count"].as_u64().unwrap())
        .sum();
    let fallbacks: u64 = snapshot["fallbacks"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    let feedback: u64 = snapshot["feedback"]
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_u64().unwrap())
        .sum();
    (requests, fallbacks, feedback)
}

fn test_config() -> MemoryConfig {
    MemoryConfig {
        quality_threshold: 0.0,
        pattern_extraction_threshold: 1.0,
        enable_summarization: false,
        enable_embeddings: false,
        batch_config: None,
        ..MemoryConfig::default()
    }
}

#[tokio::test]
#[serial]
async fn query_path_records_miss_then_cache_hit() {
    let memory = SelfLearningMemory::with_config(test_config());
    let episode_id = memory
        .start_episode(
            "telemetry query task".into(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "done".into(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let context = TaskContext::default();
    let (requests_before, _, _) = counters();

    // First call misses the cache and runs retrieval.
    let first = memory
        .retrieve_relevant_context("telemetry query task".into(), context.clone(), 5)
        .await;
    assert!(!first.is_empty());
    // Second identical call is served from the query cache.
    let second = memory
        .retrieve_relevant_context("telemetry query task".into(), context, 5)
        .await;
    assert!(!second.is_empty());

    let (requests_after, _, _) = counters();
    assert_eq!(requests_after - requests_before, 2);

    let text = global_retrieval_metrics().export_prometheus();
    assert!(text.contains("tier=\"cache\""));
}

#[tokio::test]
#[serial]
async fn feedback_signal_recorded_on_accept() {
    use chrono::Utc;

    let memory = SelfLearningMemory::with_config(test_config());
    let session = RecommendationSession {
        session_id: uuid::Uuid::new_v4(),
        episode_id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        recommended_pattern_ids: vec!["p1".to_string()],
        recommended_playbook_ids: vec![],
    };
    memory.record_recommendation_session(session.clone()).await;

    let (_, _, feedback_before) = counters();
    memory
        .record_recommendation_feedback(RecommendationFeedback {
            session_id: session.session_id,
            applied_pattern_ids: vec!["p1".to_string()],
            consulted_episode_ids: vec![],
            outcome: TaskOutcome::Failure {
                reason: "did not apply".into(),
                error_details: None,
            },
            agent_rating: Some(0.2),
        })
        .await
        .unwrap();

    let (_, _, feedback_after) = counters();
    assert_eq!(feedback_after - feedback_before, 1);
    assert!(
        global_retrieval_metrics()
            .export_prometheus()
            .contains("memory_recommendation_feedback_total{signal=\"failure\"} 1")
    );
}

#[tokio::test]
#[serial]
async fn exposition_carries_no_query_text() {
    // Distinctive marker that must never appear in metric output.
    let marker = "zz9quux-telemetry-marker";
    let memory = SelfLearningMemory::with_config(test_config());
    memory
        .retrieve_relevant_context(marker.into(), TaskContext::default(), 5)
        .await;

    let registry = global_retrieval_metrics();
    let text = registry.export_prometheus();
    let snapshot = serde_json::to_string(&registry.snapshot()).unwrap();
    assert!(!text.contains(marker));
    assert!(!snapshot.contains(marker));
}

/// Cascade recording only exists behind the `csm` feature (the hook lives
/// in the csm-gated retrieve path).
#[tokio::test]
#[serial]
#[cfg(feature = "csm")]
async fn cascade_retrievals_record_tier_and_fallback() {
    use do_memory_core::retrieval::{CascadeConfig, CascadeRetriever};

    let mut retriever = CascadeRetriever::new(CascadeConfig::default());
    retriever.add_episode("ep-1", "authentication JWT token implementation");
    retriever.add_episode("ep-2", "authentication session management");

    let (requests_before, fallbacks_before, _) = counters();
    for _ in 0..2 {
        retriever
            .retrieve("authentication JWT")
            .expect("csm retrieve");
    }

    let (requests_after, fallbacks_after, _) = counters();
    assert_eq!(requests_after - requests_before, 2);
    assert_eq!(fallbacks_after - fallbacks_before, 2);
    assert!(
        global_retrieval_metrics()
            .export_prometheus()
            .contains("memory_retrieval_fallback_total{reason=\"local_tier_sufficient\"} 2")
            || global_retrieval_metrics()
                .export_prometheus()
                .contains("reason=\"local_confident\"")
    );
}
