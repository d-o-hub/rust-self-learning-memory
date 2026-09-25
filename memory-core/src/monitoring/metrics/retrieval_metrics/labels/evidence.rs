//! Evidence-classification vocabulary (issue #1032): status and disposition labels.
//!
//! Kept beside `labels.rs` so the retrieval-telemetry vocabulary stays under the
//! per-file LOC ceiling; re-exported from `labels`.

pub(crate) const N_EVIDENCE_STATUSES: usize = 6;
pub(crate) const N_DISPOSITIONS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Evidence-classification outcome dimension. Vocabulary: `disabled`,
/// `not_configured`, `applied`, `low_confidence`, `provider_error`,
/// `invalid`.
pub enum EvidenceStatus {
    /// Configuration disabled evidence classification.
    Disabled,
    /// Classification was enabled but no judge was configured.
    NotConfigured,
    /// Judge ran and dispositions were applied to the candidate set.
    Applied,
    /// Judge ran but its confidence stayed below the threshold.
    LowConfidence,
    /// Judge provider call failed.
    ProviderError,
    /// Judge response was unusable (unparseable or incomplete).
    Invalid,
}

impl EvidenceStatus {
    /// Zero-based index for fixed-size storage.
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        match self {
            EvidenceStatus::Disabled => 0,
            EvidenceStatus::NotConfigured => 1,
            EvidenceStatus::Applied => 2,
            EvidenceStatus::LowConfidence => 3,
            EvidenceStatus::ProviderError => 4,
            EvidenceStatus::Invalid => 5,
        }
    }

    /// Bounded label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            EvidenceStatus::Disabled => "disabled",
            EvidenceStatus::NotConfigured => "not_configured",
            EvidenceStatus::Applied => "applied",
            EvidenceStatus::LowConfidence => "low_confidence",
            EvidenceStatus::ProviderError => "provider_error",
            EvidenceStatus::Invalid => "invalid",
        }
    }
}

/// Fixed evidence-disposition vocabulary in storage order. Matches
/// `EvidenceDisposition` rank: `keep` 0 < `flag` 1 < `demote` 2 < `drop` 3.
pub(crate) const DISPOSITIONS: [&str; N_DISPOSITIONS] = ["keep", "flag", "demote", "drop"];
