use super::*;

#[test]
fn requests_and_durations_accumulate_per_series() {
    let metrics = RetrievalMetrics::new();
    metrics.record_request(
        RetrievalOperation::Query,
        RetrievalTier::Cache,
        RetrievalOutcome::Hit,
        5,
    );
    metrics.record_request(
        RetrievalOperation::Query,
        RetrievalTier::Cache,
        RetrievalOutcome::Hit,
        15,
    );

    let snapshot = metrics.snapshot();
    let requests = snapshot["requests"].as_array().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["count"], 2);
    assert_eq!(requests[0]["latency_ms"]["avg"], 10);
    // Zero series are omitted from both renderings.
    let text = metrics.export_prometheus();
    assert!(text.contains(
        "memory_retrieval_requests_total{operation=\"query\",tier=\"cache\",outcome=\"hit\"} 2"
    ));
    assert!(!text.contains("tier=\"semantic\""));
}

#[test]
fn reset_clears_every_series() {
    let metrics = RetrievalMetrics::new();
    metrics.record_request(
        RetrievalOperation::Cascade,
        RetrievalTier::Bm25,
        RetrievalOutcome::Hit,
        3,
    );
    metrics.record_fallback(FallbackReason::NoLocalResults);
    metrics.reset();

    assert!(
        metrics.snapshot()["requests"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!metrics.export_prometheus().contains("tier=\"bm25\""));
}

#[test]
fn rerank_samples_follow_judge_backed_statuses() {
    let metrics = RetrievalMetrics::new();
    metrics.record_rerank(RerankStatus::Disabled, 9, 4, 12, true, 0.9);
    metrics.record_rerank(RerankStatus::NotConfigured, 9, 4, 12, true, 0.9);
    metrics.record_rerank(RerankStatus::Applied, 20, 10, 30, true, 0.8125);
    metrics.record_rerank(RerankStatus::LowConfidence, 20, 10, 10, false, 0.5);
    metrics.record_rerank(RerankStatus::ProviderError, 9, 4, 12, true, 0.9);
    metrics.record_rerank(RerankStatus::Invalid, 9, 4, 12, true, 0.9);

    // Every status counts, including the ones that never reached the
    // judge.
    assert_eq!(
        metrics.rerank[RerankStatus::Disabled.index()].load(Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics.rerank[RerankStatus::Invalid.index()].load(Ordering::Relaxed),
        1
    );

    // Only judge-backed invocations carry timing, sizes, and confidence.
    assert_eq!(metrics.rerank_durations_ms.lock().count(), 2);
    assert_eq!(metrics.rerank_shortlist_sum.load(Ordering::Relaxed), 40);
    assert_eq!(metrics.rerank_shortlist_count.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.rerank_output_sum.load(Ordering::Relaxed), 20);
    assert_eq!(metrics.rerank_output_count.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.rerank_top1_changed.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.rerank_confidence_count.load(Ordering::Relaxed), 2);
    assert_eq!(
        metrics.rerank_confidence_sum_micro.load(Ordering::Relaxed),
        1_312_500
    );

    metrics.reset();
    assert_eq!(metrics.rerank_durations_ms.lock().count(), 0);
    assert_eq!(
        metrics.rerank_confidence_sum_micro.load(Ordering::Relaxed),
        0
    );
    assert!(!metrics.export_prometheus().contains("memory_rerank"));
}

#[test]
fn evidence_samples_follow_judge_backed_statuses() {
    let metrics = RetrievalMetrics::new();
    metrics.record_evidence(EvidenceStatus::Disabled, 9, [9, 0, 0, 0], 12);
    metrics.record_evidence(EvidenceStatus::NotConfigured, 9, [9, 0, 0, 0], 12);
    metrics.record_evidence(EvidenceStatus::Applied, 20, [12, 3, 4, 1], 30);
    metrics.record_evidence(EvidenceStatus::LowConfidence, 20, [18, 2, 0, 0], 10);
    metrics.record_evidence(EvidenceStatus::ProviderError, 9, [9, 0, 0, 0], 12);
    metrics.record_evidence(EvidenceStatus::Invalid, 9, [9, 0, 0, 0], 12);

    // Every status counts, including the ones that never reached the
    // judge.
    assert_eq!(
        metrics.evidence[EvidenceStatus::Disabled.index()].load(Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics.evidence[EvidenceStatus::Invalid.index()].load(Ordering::Relaxed),
        1
    );

    // Only judge-backed invocations carry timing, sizes, and dispositions.
    assert_eq!(metrics.evidence_durations_ms.lock().count(), 2);
    assert_eq!(metrics.evidence_candidates_sum.load(Ordering::Relaxed), 40);
    assert_eq!(metrics.evidence_candidates_count.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.evidence_dispositions[0].load(Ordering::Relaxed), 30);
    assert_eq!(metrics.evidence_dispositions[1].load(Ordering::Relaxed), 5);
    assert_eq!(metrics.evidence_dispositions[2].load(Ordering::Relaxed), 4);
    assert_eq!(metrics.evidence_dispositions[3].load(Ordering::Relaxed), 1);

    metrics.reset();
    assert_eq!(metrics.evidence_durations_ms.lock().count(), 0);
    assert_eq!(metrics.evidence_dispositions[0].load(Ordering::Relaxed), 0);
    assert_eq!(metrics.evidence_candidates_sum.load(Ordering::Relaxed), 0);
    assert!(!metrics.export_prometheus().contains("memory_evidence"));
}
