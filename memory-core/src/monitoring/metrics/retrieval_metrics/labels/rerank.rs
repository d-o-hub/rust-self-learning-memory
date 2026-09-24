//! Rerank status vocabulary (issue #1031).
//!
//! Kept beside `labels.rs` so the retrieval-telemetry vocabulary stays under
//! the per-file LOC ceiling; re-exported from `labels`.

pub(crate) const N_RERANK_STATUSES: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Semantic rerank outcome dimension. Vocabulary: `disabled`,
/// `not_configured`, `applied`, `low_confidence`, `provider_error`,
/// `invalid`.
pub enum RerankStatus {
    /// Configuration disabled reranking.
    Disabled,
    /// Reranking was enabled but no judge was configured.
    NotConfigured,
    /// Candidate order was reranked from judge scores.
    Applied,
    /// Judge ran but its confidence stayed below the threshold.
    LowConfidence,
    /// Judge provider call failed.
    ProviderError,
    /// Judge response was unusable (unparseable or incomplete).
    Invalid,
}

impl RerankStatus {
    /// Zero-based index for fixed-size storage.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        match self {
            RerankStatus::Disabled => 0,
            RerankStatus::NotConfigured => 1,
            RerankStatus::Applied => 2,
            RerankStatus::LowConfidence => 3,
            RerankStatus::ProviderError => 4,
            RerankStatus::Invalid => 5,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RerankStatus::Disabled => "disabled",
            RerankStatus::NotConfigured => "not_configured",
            RerankStatus::Applied => "applied",
            RerankStatus::LowConfidence => "low_confidence",
            RerankStatus::ProviderError => "provider_error",
            RerankStatus::Invalid => "invalid",
        }
    }
}
