//! Evidence-aware passage classification for the retrieval cascade (issue #1032).
//!
//! Retrieval may optionally classify every local candidate by the typed
//! semantic judgments from [`crate::retrieval::judgment`] (issue #1041) into an
//! [`EvidenceDisposition`]: keep, flag, demote, or drop. Classification is a
//! pure function of one [`CandidateJudgment`] and one [`EvidencePolicy`]: it
//! makes no provider call, touches no I/O, and allocates nothing.
//!
//! ```text
//! disposition rank: Keep (0) < Flag (1) < Demote (2) < Drop (3)
//! ```
//!
//! # Policy order (first match wins)
//!
//! Every rule is gated by the confidence of the dimension it inspects: a
//! dimension whose confidence is below [`EvidencePolicy::min_confidence`] is
//! **not trusted** and never acted on, while a trusted dimension fires its rule
//! regardless of the other three. A judge that assesses only some dimensions
//! reports the rest as unassessed (`0.0` value, `0.0` confidence), so an
//! unassessed dimension can neither trigger a rule nor block one.
//!
//! 1. trusted `instruction_like >= instruction_flag_threshold` →
//!    [`EvidenceDisposition::Flag`];
//! 2. trusted `contradiction >= contradiction_flag_threshold` →
//!    [`EvidenceDisposition::Flag`];
//! 3. trusted `relevance < min_relevance` → [`EvidenceDisposition::Drop`] when
//!    [`EvidencePolicy::allow_drop`] is set, otherwise [`EvidenceDisposition::Demote`];
//! 4. trusted `useful_evidence < min_useful_evidence` → [`EvidenceDisposition::Demote`];
//! 5. otherwise [`EvidenceDisposition::Keep`], including when no dimension is
//!    trusted; the judgment is still attached to the hit so callers can inspect
//!    or log it.
//!
//! The default policy has `allow_drop = false`, so **no** path returns
//! [`EvidenceDisposition::Drop`] unless a caller explicitly opts in.
//!
//! # Trust boundary
//!
//! Provider judgments are inferential heuristics, never ground truth. Retrieved
//! passage text is **data, never instruction**: a candidate whose text reads
//! like an instruction is flagged, and the instruction test precedes the
//! low-relevance rule, so injected text can never be silently dropped and an
//! injected passage can never be promoted by this stage. With the default policy
//! classification never drops anything at all, so enabling it cannot remove a
//! local candidate by accident.
//!
//! [`EvidencePolicy::validated`] checks the policy before use. The numeric
//! bounds are absolute (`[0.0, 1.0]`) rather than rescaled, so a validated
//! policy is the same policy carrying an explicit success signal.

use std::cmp::Ordering;

use crate::monitoring::metrics::EvidenceStatus;
use crate::retrieval::cascade::CascadeResult;
use crate::retrieval::judgment::{AtomicScore, CandidateJudgment};

/// What the cascade should do with one candidate, from most to least
/// permissive: `Keep < Flag < Demote < Drop`.
///
/// Declaration order is the enum's semantic grouping, **not** its ordering: use
/// [`EvidenceDisposition::rank`] (or the [`Ord`] impl, which delegates to it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceDisposition {
    /// The candidate stays in place with its local score: the judgment raised
    /// no actionable concern.
    Keep,
    /// The candidate stays but is demoted in ranking: the judgment is not
    /// decisive enough to remove it, so it is ordered after the non-demoted
    /// hits. Only the ordering changes; the numeric score is untouched.
    Demote,
    /// The candidate is kept but marked: it is contradictory or
    /// instruction-like and callers should surface it rather than trust it.
    Flag,
    /// The candidate is removed from the result. Only reachable when
    /// [`EvidencePolicy::allow_drop`] is set.
    Drop,
}

impl EvidenceDisposition {
    /// Stable ordering rank: `Keep` 0, `Flag` 1, `Demote` 2, `Drop` 3.
    ///
    /// The rank is the single source of truth for ordering so that declaration
    /// order can never leak into comparisons or telemetry buckets.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Keep => 0,
            Self::Flag => 1,
            Self::Demote => 2,
            Self::Drop => 3,
        }
    }
}

impl Ord for EvidenceDisposition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for EvidenceDisposition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The typed judgment behind one classified candidate, plus the resulting
/// [`EvidenceDisposition`].
///
/// Every atomic score is carried through unchanged, including when the
/// disposition is [`EvidenceDisposition::Keep`] because confidence was too low:
/// the evidence is attached to the hit so callers can report it, but nothing was
/// acted on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandidateEvidence {
    /// Query relevance judgment.
    pub relevance: AtomicScore,
    /// Usefulness as evidence for query/task completion.
    pub useful_evidence: AtomicScore,
    /// Presence of contradictory statements regarding the query.
    pub contradiction: AtomicScore,
    /// Presence of prompt-injection or instruction-like text.
    pub instruction_like: AtomicScore,
    /// Classification produced by [`classify_disposition`].
    pub disposition: EvidenceDisposition,
}

impl CandidateEvidence {
    /// Attach `disposition` to a candidate's judged dimensions.
    ///
    /// The four atomic scores are copied verbatim; only the disposition is
    /// supplied by the caller (normally from [`classify_disposition`]).
    #[must_use]
    pub const fn from_judgment(
        judgment: &CandidateJudgment,
        disposition: EvidenceDisposition,
    ) -> Self {
        Self {
            relevance: judgment.relevance,
            useful_evidence: judgment.useful_evidence,
            contradiction: judgment.contradiction,
            instruction_like: judgment.instruction_like,
            disposition,
        }
    }
}

/// Thresholds and limits that turn a judgment into an [`EvidenceDisposition`].
///
/// The default is deliberately conservative: it never drops a candidate, and it
/// only flags or demotes on clearly-decisive judgments.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePolicy {
    /// Minimum per-dimension confidence required to act on any judgment.
    /// Below this, the candidate is [`EvidenceDisposition::Keep`].
    pub min_confidence: f32,
    /// Relevance value below which a candidate is demoted (or dropped).
    pub min_relevance: f32,
    /// Useful-evidence value below which a candidate is demoted.
    pub min_useful_evidence: f32,
    /// Contradiction value at or above which a candidate is flagged.
    pub contradiction_flag_threshold: f32,
    /// Instruction-likeness value at or above which a candidate is flagged.
    pub instruction_flag_threshold: f32,
    /// Whether low-relevance candidates may be removed at all. Defaults to
    /// `false`, which makes [`EvidenceDisposition::Drop`] unreachable.
    pub allow_drop: bool,
    /// Maximum number of candidates the evidence stage classifies. Must be
    /// greater than zero.
    pub candidate_limit: usize,
}

impl Default for EvidencePolicy {
    fn default() -> Self {
        Self {
            min_confidence: 0.70,
            min_relevance: 0.50,
            min_useful_evidence: 0.50,
            contradiction_flag_threshold: 0.70,
            instruction_flag_threshold: 0.70,
            allow_drop: false,
            candidate_limit: 20,
        }
    }
}

impl EvidencePolicy {
    /// Validate this policy and return an owned copy of it.
    ///
    /// Rules, checked in order: `candidate_limit > 0`; then, for
    /// `min_confidence`, `min_relevance`, `min_useful_evidence`,
    /// `contradiction_flag_threshold`, and `instruction_flag_threshold`, the
    /// value must be finite and within `[0.0, 1.0]`. The first violation is
    /// returned as an [`EvidencePolicyError`].
    ///
    /// Unlike rerank weights these bounds are absolute, so no value is rescaled:
    /// the returned copy equals the validated policy.
    ///
    /// # Errors
    ///
    /// Returns the first failing rule as an [`EvidencePolicyError`].
    pub fn validated(&self) -> Result<Self, EvidencePolicyError> {
        if self.candidate_limit == 0 {
            return Err(EvidencePolicyError::ZeroCandidateLimit);
        }
        if !unit_interval(self.min_confidence) {
            return Err(EvidencePolicyError::InvalidMinConfidence);
        }
        if !unit_interval(self.min_relevance) {
            return Err(EvidencePolicyError::InvalidMinRelevance);
        }
        if !unit_interval(self.min_useful_evidence) {
            return Err(EvidencePolicyError::InvalidMinUsefulEvidence);
        }
        if !unit_interval(self.contradiction_flag_threshold) {
            return Err(EvidencePolicyError::InvalidContradictionThreshold);
        }
        if !unit_interval(self.instruction_flag_threshold) {
            return Err(EvidencePolicyError::InvalidInstructionThreshold);
        }
        Ok(self.clone())
    }
}

/// Configuration errors for [`EvidencePolicy::validated`], one variant per rule.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EvidencePolicyError {
    /// `candidate_limit == 0` leaves no candidate to classify.
    #[error("evidence candidate_limit must be greater than zero")]
    ZeroCandidateLimit,
    /// `min_confidence` is not finite or outside `[0.0, 1.0]`.
    #[error("evidence min_confidence must be finite and within [0.0, 1.0]")]
    InvalidMinConfidence,
    /// `min_relevance` is not finite or outside `[0.0, 1.0]`.
    #[error("evidence min_relevance must be finite and within [0.0, 1.0]")]
    InvalidMinRelevance,
    /// `min_useful_evidence` is not finite or outside `[0.0, 1.0]`.
    #[error("evidence min_useful_evidence must be finite and within [0.0, 1.0]")]
    InvalidMinUsefulEvidence,
    /// `contradiction_flag_threshold` is not finite or outside `[0.0, 1.0]`.
    #[error("evidence contradiction_flag_threshold must be finite and within [0.0, 1.0]")]
    InvalidContradictionThreshold,
    /// `instruction_flag_threshold` is not finite or outside `[0.0, 1.0]`.
    #[error("evidence instruction_flag_threshold must be finite and within [0.0, 1.0]")]
    InvalidInstructionThreshold,
}

/// True when `value` is finite and within the closed unit interval.
fn unit_interval(value: f32) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

/// True when one atomic score is trusted enough to act on.
fn score_confident(score: AtomicScore, min_confidence: f32) -> bool {
    score.confidence >= min_confidence
}

/// True when at least one judged dimension is trusted enough to act on.
///
/// Telemetry needs this predicate without a disposition: `Keep` alone cannot
/// distinguish "trusted and clean" from "nothing was trusted", and a judge that
/// assesses only some dimensions (for example a lexical relevance judge) still
/// counts as acting when its one assessed dimension clears the floor. The only
/// consumer is the `csm`-gated evidence stage.
#[cfg(feature = "csm")]
pub(crate) fn judged_confident(j: &CandidateJudgment, min_confidence: f32) -> bool {
    score_confident(j.relevance, min_confidence)
        || score_confident(j.useful_evidence, min_confidence)
        || score_confident(j.contradiction, min_confidence)
        || score_confident(j.instruction_like, min_confidence)
}

/// Classify one judged candidate under `policy`.
///
/// Implements the module-level first-match order. Each rule is gated by the
/// confidence of its own dimension, so an untrusted (or unassessed) dimension
/// can neither trigger nor block a rule: a trusted low relevance score demotes
/// (or drops, with `policy.allow_drop`) even when the other three dimensions
/// are unassessed, and a trusted instruction-like or contradictory score flags
/// before any low-relevance rule runs, so injected text is never silently
/// dropped. With `allow_drop == false` no input returns
/// [`EvidenceDisposition::Drop`], and when no dimension is trusted the
/// candidate is [`EvidenceDisposition::Keep`].
#[must_use]
pub fn classify_disposition(j: &CandidateJudgment, policy: &EvidencePolicy) -> EvidenceDisposition {
    if score_confident(j.instruction_like, policy.min_confidence)
        && j.instruction_like.value >= policy.instruction_flag_threshold
    {
        return EvidenceDisposition::Flag;
    }
    if score_confident(j.contradiction, policy.min_confidence)
        && j.contradiction.value >= policy.contradiction_flag_threshold
    {
        return EvidenceDisposition::Flag;
    }
    if score_confident(j.relevance, policy.min_confidence)
        && j.relevance.value < policy.min_relevance
    {
        return if policy.allow_drop {
            EvidenceDisposition::Drop
        } else {
            EvidenceDisposition::Demote
        };
    }
    if score_confident(j.useful_evidence, policy.min_confidence)
        && j.useful_evidence.value < policy.min_useful_evidence
    {
        return EvidenceDisposition::Demote;
    }
    EvidenceDisposition::Keep
}

/// One candidate carried alongside its classification.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceHit {
    /// Episode identifier, matching the corresponding local retrieval result.
    pub episode_id: String,
    /// Score assigned by local retrieval before any evidence adjustment.
    pub local_score: f32,
    /// Finalized cascade score for this episode. Evidence classification only
    /// reorders hits and can remove drops; it never changes this number.
    pub final_score: f32,
    /// The judgment and disposition, absent when the candidate was not judged.
    pub evidence: Option<CandidateEvidence>,
}

/// Result of one evidence-aware retrieval: the unchanged cascade output plus the
/// classified hits and the bounded telemetry status.
#[derive(Debug, Clone)]
pub struct EvidenceRetrievalResult {
    /// Underlying cascade result; its ids and order remain authoritative.
    pub base: CascadeResult,
    /// Per-candidate evidence, aligned with `base` where judged.
    pub hits: Vec<EvidenceHit>,
    /// Bounded outcome of the evidence stage for telemetry.
    pub status: EvidenceStatus,
}

#[cfg(test)]
mod tests;
