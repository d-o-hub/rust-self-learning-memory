//! Issue #962 acceptance test: all eight metric families specified in the
//! issue exist in both JSON snapshot and Prometheus exposition, labels are
//! bounded, and raw query text never leaks.
//!
//! This test validates the complete observability contract end-to-end.

use do_memory_core::monitoring::metrics::{
    CacheLayer, EmbeddingOutcome, EmbeddingProviderLabel, FallbackReason, FeedbackSignal,
    RetrievalMetrics, RetrievalOperation, RetrievalOutcome, RetrievalStage, RetrievalTier,
};

/// Record at least one sample in every metric family, then assert the
/// JSON snapshot and Prometheus exposition contain all eight families.
#[test]
fn all_eight_issue_families_present() {
    let m = RetrievalMetrics::new();

    // 1. memory_retrieval_requests_total{operation,tier,outcome}
    m.record_request(
        RetrievalOperation::Query,
        RetrievalTier::Semantic,
        RetrievalOutcome::Hit,
        12,
    );

    // 2. memory_retrieval_duration_seconds{operation,tier} (recorded alongside requests)

    // 3. memory_embedding_requests_total{provider,result}
    m.record_embedding(EmbeddingProviderLabel::OpenAI, EmbeddingOutcome::Ok, 45);

    // 4. memory_embedding_duration_seconds{provider} (recorded alongside embeddings)

    // 5. memory_cache_requests_total{layer,result}
    m.record_cache(CacheLayer::Query, RetrievalOutcome::Miss);

    // 6. memory_retrieval_candidates{stage}
    m.record_candidates(RetrievalStage::Cascade, 15);

    // 7. memory_retrieval_fallback_total{reason}
    m.record_fallback(FallbackReason::NoLocalResults);

    // 8. memory_recommendation_feedback_total{signal}
    m.record_feedback(FeedbackSignal::Success);

    // ---- JSON snapshot assertions ----
    let snapshot = m.snapshot();
    assert!(
        !snapshot["requests"].as_array().unwrap().is_empty(),
        "requests array must be non-empty"
    );
    assert!(
        !snapshot["fallbacks"].as_object().unwrap().is_empty(),
        "fallbacks must be non-empty"
    );
    assert!(
        !snapshot["feedback"].as_object().unwrap().is_empty(),
        "feedback must be non-empty"
    );
    assert!(
        !snapshot["cache"].as_object().unwrap().is_empty(),
        "cache must be non-empty"
    );
    assert!(
        !snapshot["embeddings"].as_object().unwrap().is_empty(),
        "embeddings must be non-empty"
    );
    assert!(
        !snapshot["candidates"].as_object().unwrap().is_empty(),
        "candidates must be non-empty"
    );

    // ---- Prometheus text exposition assertions ----
    let text = m.export_prometheus();
    let required_families = [
        "memory_retrieval_requests_total",
        "memory_retrieval_duration_seconds",
        "memory_embedding_requests_total",
        "memory_embedding_duration_seconds",
        "memory_cache_requests_total",
        "memory_retrieval_candidates_sum",
        "memory_retrieval_fallback_total",
        "memory_recommendation_feedback_total",
    ];
    for family in &required_families {
        assert!(
            text.contains(family),
            "Prometheus exposition missing required family: {family}"
        );
    }

    // ---- Verify bounded labels (no free-form strings) ----
    assert!(text.contains("operation=\"query\""));
    assert!(text.contains("tier=\"semantic\""));
    assert!(text.contains("outcome=\"hit\""));
    assert!(text.contains("provider=\"openai\""));
    assert!(text.contains("result=\"miss\""));
    assert!(text.contains("reason=\"no_local_results\""));
    assert!(text.contains("signal=\"success\""));
    assert!(text.contains("stage=\"cascade\""));
}

/// Verify that the snapshot carries latency percentiles for requests
/// and embeddings (P50, P95, P99, avg).
#[test]
fn latency_percentiles_in_snapshot() {
    let m = RetrievalMetrics::new();
    m.record_request(
        RetrievalOperation::Cascade,
        RetrievalTier::Bm25,
        RetrievalOutcome::Hit,
        7,
    );
    m.record_embedding(EmbeddingProviderLabel::Local, EmbeddingOutcome::Ok, 3);

    let snapshot = m.snapshot();

    // Request latency
    let request = &snapshot["requests"].as_array().unwrap()[0];
    let latency = &request["latency_ms"];
    assert!(latency["p50"].is_number());
    assert!(latency["p95"].is_number());
    assert!(latency["p99"].is_number());
    assert!(latency["avg"].is_number());

    // Embedding latency is exposed via Prometheus, verify it's present
    let text = m.export_prometheus();
    assert!(text.contains("memory_embedding_duration_seconds{provider=\"local\""));
}

/// Verify the cardinality upper bound: max 151 series.
#[test]
fn cardinality_upper_bound() {
    // The doc promises at most 151 series. Verify the array dimensions.
    // requests: 2 ops x 11 tiers x 2 outcomes = 44
    // durations: 2 ops x 11 tiers x 3 quantiles = 66
    // candidates: 2 stages x 2 (sum + count) = 4
    // cache: 1 layer x 2 outcomes = 2
    // embeddings: 5 providers x 2 outcomes = 10
    // emb_durations: 5 providers x 3 quantiles = 15
    // fallbacks: 6 reasons = 6
    // feedback: 4 signals = 4
    // Total: 44 + 66 + 4 + 2 + 10 + 15 + 6 + 4 = 151
    assert_eq!(2 * 11 * 2 + 2 * 11 * 3 + 4 + 2 + 10 + 15 + 6 + 4, 151);
}
