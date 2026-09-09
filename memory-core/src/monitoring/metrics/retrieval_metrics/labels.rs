//! Bounded label vocabularies for retrieval telemetry (issue #962).
//!
//! Every dimension is a fieldless enum: label values are fixed
//! snake_case strings, so raw queries, IDs, tags, and provider error
//! strings can never enter metric series.

use crate::types::TaskOutcome;

/// Storage bounds for the fixed-size telemetry arrays.
pub(super) const N_OPERATIONS: usize = 2;
pub(super) const N_TIERS: usize = 11;
pub(super) const N_OUTCOMES: usize = 2;
pub(super) const N_LAYERS: usize = 1;
pub(super) const N_PROVIDERS: usize = 5;
pub(super) const N_EMB_OUTCOMES: usize = 2;
pub(super) const N_STAGES: usize = 2;
pub(super) const N_FALLBACK_REASONS: usize = 6;
pub(super) const N_SIGNALS: usize = 4;

/// Retrieval operation dimension. Vocabulary: `query`, `cascade`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalOperation {
    /// `retrieve_relevant_context` query path.
    Query,
    /// `CascadeRetriever` 4-tier path.
    Cascade,
}

impl RetrievalOperation {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            RetrievalOperation::Query => 0,
            RetrievalOperation::Cascade => 1,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RetrievalOperation::Query => "query",
            RetrievalOperation::Cascade => "cascade",
        }
    }
}

/// Serving-tier dimension. Vocabulary: `cache`, `hybrid`, `semantic`,
/// `hierarchical`, `keyword`, `bm25`, `hdc`, `concept_graph`, `api`,
/// `blended`, `none`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalTier {
    /// Served from the query cache.
    Cache,
    /// Hybrid ANN-backed retrieval.
    Hybrid,
    /// Semantic similarity retrieval.
    Semantic,
    /// Hierarchical spatiotemporal retrieval.
    Hierarchical,
    /// Legacy keyword retrieval.
    Keyword,
    /// Cascade Tier 1 only.
    Bm25,
    /// Cascade Tier 2 only.
    Hdc,
    /// Cascade Tier 3 only.
    ConceptGraph,
    /// Policy-forced API tier.
    Api,
    /// Multiple cascade tiers contributed.
    Blended,
    /// No tier served (empty index / no episodes).
    None,
}

impl RetrievalTier {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            RetrievalTier::Cache => 0,
            RetrievalTier::Hybrid => 1,
            RetrievalTier::Semantic => 2,
            RetrievalTier::Hierarchical => 3,
            RetrievalTier::Keyword => 4,
            RetrievalTier::Bm25 => 5,
            RetrievalTier::Hdc => 6,
            RetrievalTier::ConceptGraph => 7,
            RetrievalTier::Api => 8,
            RetrievalTier::Blended => 9,
            RetrievalTier::None => 10,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RetrievalTier::Cache => "cache",
            RetrievalTier::Hybrid => "hybrid",
            RetrievalTier::Semantic => "semantic",
            RetrievalTier::Hierarchical => "hierarchical",
            RetrievalTier::Keyword => "keyword",
            RetrievalTier::Bm25 => "bm25",
            RetrievalTier::Hdc => "hdc",
            RetrievalTier::ConceptGraph => "concept_graph",
            RetrievalTier::Api => "api",
            RetrievalTier::Blended => "blended",
            RetrievalTier::None => "none",
        }
    }

    /// All vocabulary values, for cardinality assertions.
    #[must_use]
    pub const fn all() -> [&'static str; 11] {
        [
            "cache",
            "hybrid",
            "semantic",
            "hierarchical",
            "keyword",
            "bm25",
            "hdc",
            "concept_graph",
            "api",
            "blended",
            "none",
        ]
    }
}

/// Request outcome dimension. Vocabulary: `hit`, `miss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalOutcome {
    /// At least one episode returned.
    Hit,
    /// Empty result.
    Miss,
}

impl RetrievalOutcome {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            RetrievalOutcome::Hit => 0,
            RetrievalOutcome::Miss => 1,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RetrievalOutcome::Hit => "hit",
            RetrievalOutcome::Miss => "miss",
        }
    }

    /// Outcome from a returned-item count.
    #[must_use]
    pub const fn from_count(count: usize) -> Self {
        if count == 0 {
            RetrievalOutcome::Miss
        } else {
            RetrievalOutcome::Hit
        }
    }
}

/// Cache layer dimension. Vocabulary: `query`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLayer {
    /// Episodic query-result cache.
    Query,
}

impl CacheLayer {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            CacheLayer::Query => 0,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            CacheLayer::Query => "query",
        }
    }
}

/// Embedding provider dimension. Vocabulary: `local`, `openai`, `mistral`,
/// `azure_openai`, `custom`. Mirrors `ProviderConfig` variants 1:1 so new
/// providers fail compilation here instead of widening cardinality silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingProviderLabel {
    /// CPU-local provider.
    Local,
    /// OpenAI provider.
    OpenAI,
    /// Mistral provider.
    Mistral,
    /// Azure OpenAI provider.
    AzureOpenAI,
    /// Custom provider.
    Custom,
}

impl EmbeddingProviderLabel {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            EmbeddingProviderLabel::Local => 0,
            EmbeddingProviderLabel::OpenAI => 1,
            EmbeddingProviderLabel::Mistral => 2,
            EmbeddingProviderLabel::AzureOpenAI => 3,
            EmbeddingProviderLabel::Custom => 4,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            EmbeddingProviderLabel::Local => "local",
            EmbeddingProviderLabel::OpenAI => "openai",
            EmbeddingProviderLabel::Mistral => "mistral",
            EmbeddingProviderLabel::AzureOpenAI => "azure_openai",
            EmbeddingProviderLabel::Custom => "custom",
        }
    }
}

/// Embedding call outcome dimension. Vocabulary: `ok`, `error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingOutcome {
    /// Provider returned an embedding.
    Ok,
    /// Provider call failed.
    Error,
}

impl EmbeddingOutcome {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            EmbeddingOutcome::Ok => 0,
            EmbeddingOutcome::Error => 1,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            EmbeddingOutcome::Ok => "ok",
            EmbeddingOutcome::Error => "error",
        }
    }
}

/// Candidate-set stage dimension. Vocabulary: `cascade`, `scored`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalStage {
    /// Cascade final set size.
    Cascade,
    /// Hierarchically scored candidate count.
    Scored,
}

impl RetrievalStage {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            RetrievalStage::Cascade => 0,
            RetrievalStage::Scored => 1,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RetrievalStage::Cascade => "cascade",
            RetrievalStage::Scored => "scored",
        }
    }
}

/// Recommendation-feedback signal dimension. Vocabulary: `success`,
/// `partial`, `failure`, `abstained`. Mirrors `TaskOutcome` kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackSignal {
    /// Task succeeded.
    Success,
    /// Task partially succeeded.
    Partial,
    /// Task failed.
    Failure,
    /// Agent abstained.
    Abstained,
}

impl FeedbackSignal {
    /// Zero-based index for fixed-size storage.
    pub(super) const fn index(self) -> usize {
        match self {
            FeedbackSignal::Success => 0,
            FeedbackSignal::Partial => 1,
            FeedbackSignal::Failure => 2,
            FeedbackSignal::Abstained => 3,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            FeedbackSignal::Success => "success",
            FeedbackSignal::Partial => "partial",
            FeedbackSignal::Failure => "failure",
            FeedbackSignal::Abstained => "abstained",
        }
    }

    /// Signal from a feedback outcome. Counted only for accepted feedback
    /// (hydration replays bypass recording to avoid double counting).
    #[must_use]
    pub const fn from_outcome(outcome: &TaskOutcome) -> Self {
        match outcome {
            TaskOutcome::Success { .. } => FeedbackSignal::Success,
            TaskOutcome::PartialSuccess { .. } => FeedbackSignal::Partial,
            TaskOutcome::Failure { .. } => FeedbackSignal::Failure,
            TaskOutcome::Abstained { .. } => FeedbackSignal::Abstained,
        }
    }
}

/// Zero-based index for [`FallbackReason`] (display order is fixed).
pub(super) const fn fallback_index(reason: FallbackReason) -> usize {
    match reason {
        FallbackReason::LocalTierSufficient => 0,
        FallbackReason::LocalConfident => 1,
        FallbackReason::InsufficientConfidence => 2,
        FallbackReason::NoLocalResults => 3,
        FallbackReason::AlwaysEmbedPolicy => 4,
        FallbackReason::LocalOnlyPolicy => 5,
    }
}

/// Tier-4 fallback reason dimension. Vocabulary: `local_tier_sufficient`,
/// `local_confident`, `insufficient_confidence`, `no_local_results`,
/// `always_embed_policy`, `local_only_policy`.
///
/// This is the canonical vocabulary (telemetry contract). The confidence
/// policy (#968) reuses it for `CascadeResult::fallback_reason` rather than
/// defining its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackReason {
    /// A CPU-local tier already satisfied the query.
    LocalTierSufficient,
    /// Confident local result, no Tier-4 call.
    LocalConfident,
    /// Local result exists but is not confident.
    InsufficientConfidence,
    /// No local results at all.
    NoLocalResults,
    /// `always_embed` policy forced the call.
    AlwaysEmbedPolicy,
    /// `local_only` policy suppressed the call.
    LocalOnlyPolicy,
}

impl FallbackReason {
    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            FallbackReason::LocalTierSufficient => "local_tier_sufficient",
            FallbackReason::LocalConfident => "local_confident",
            FallbackReason::InsufficientConfidence => "insufficient_confidence",
            FallbackReason::NoLocalResults => "no_local_results",
            FallbackReason::AlwaysEmbedPolicy => "always_embed_policy",
            FallbackReason::LocalOnlyPolicy => "local_only_policy",
        }
    }
}

/// Fixed fallback-reason vocabulary in storage order.
pub(super) const FALLBACK_REASONS: [&str; N_FALLBACK_REASONS] = [
    "local_tier_sufficient",
    "local_confident",
    "insufficient_confidence",
    "no_local_results",
    "always_embed_policy",
    "local_only_policy",
];

#[cfg(test)]
mod tests {
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
}
