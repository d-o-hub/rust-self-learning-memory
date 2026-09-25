use super::*;

/// Every label value must be Prometheus-safe (bounded vocabulary guard).
fn assert_label_value(value: &str) {
    assert!(
        value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
        "unbounded label value: {value}"
    );
}

#[test]
fn label_vocabularies_are_bounded() {
    for value in RetrievalTier::all() {
        assert_label_value(value);
    }
    assert_eq!(RetrievalTier::all().len(), 11);
    for value in FALLBACK_REASONS {
        assert_label_value(value);
    }
    assert_eq!(FALLBACK_REASONS.len(), 6);
    for op in [RetrievalOperation::Query, RetrievalOperation::Cascade] {
        assert_label_value(op.as_str());
    }
    for signal in [
        FeedbackSignal::Success,
        FeedbackSignal::Partial,
        FeedbackSignal::Failure,
        FeedbackSignal::Abstained,
    ] {
        assert_label_value(signal.as_str());
    }
    for provider in [
        EmbeddingProviderLabel::Local,
        EmbeddingProviderLabel::OpenAI,
        EmbeddingProviderLabel::Mistral,
        EmbeddingProviderLabel::AzureOpenAI,
        EmbeddingProviderLabel::Custom,
    ] {
        assert_label_value(provider.as_str());
    }
    for outcome in [
        JudgmentOutcome::NotConfigured,
        JudgmentOutcome::Ok,
        JudgmentOutcome::Unavailable,
        JudgmentOutcome::Timeout,
        JudgmentOutcome::Invalid,
        JudgmentOutcome::ProviderError,
    ] {
        assert_label_value(outcome.as_str());
    }
    let rerank_statuses = [
        RerankStatus::Disabled,
        RerankStatus::NotConfigured,
        RerankStatus::Applied,
        RerankStatus::LowConfidence,
        RerankStatus::ProviderError,
        RerankStatus::Invalid,
    ];
    assert_eq!(rerank_statuses.len(), N_RERANK_STATUSES);
    for (expected, status) in rerank_statuses.iter().enumerate() {
        // `index()` is the storage slot and the exposition walk order.
        assert_eq!(status.index(), expected);
        assert_label_value(status.as_str());
    }
}

#[test]
fn feedback_signal_mapping_covers_outcome_kinds() {
    use TaskOutcome::{Abstained, Failure, PartialSuccess, Success};
    assert_eq!(
        FeedbackSignal::from_outcome(&Success {
            verdict: "v".into(),
            artifacts: vec![]
        }),
        FeedbackSignal::Success
    );
    assert_eq!(
        FeedbackSignal::from_outcome(&PartialSuccess {
            verdict: "v".into(),
            completed: vec![],
            failed: vec![]
        }),
        FeedbackSignal::Partial
    );
    assert_eq!(
        FeedbackSignal::from_outcome(&Failure {
            reason: "r".into(),
            error_details: None
        }),
        FeedbackSignal::Failure
    );
    assert_eq!(
        FeedbackSignal::from_outcome(&Abstained {
            reason: "r".into(),
            stopped_at_step: 0,
            infeasibility_signals: vec![]
        }),
        FeedbackSignal::Abstained
    );
}
