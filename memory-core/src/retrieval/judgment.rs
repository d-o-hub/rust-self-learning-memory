//! Provider-neutral typed semantic judgment interface.
//!
//! Provides atomic, typed assessments (relevance, useful evidence, contradiction,
//! instruction-likeness) for query/candidate pairs without coupling to specific
//! model providers or using free-form JSON outputs.
//!
//! # Trust Boundary
//!
//! Provider outputs are **inferential heuristics**, not ground truth.
//! All provider outputs MUST be strictly validated before use:
//! - Scores and confidences must be finite numbers clamped in `[0.0, 1.0]`.
//! - Output candidate IDs must match the input candidates exactly (no missing, unknown, or duplicate IDs).
//! - Provider failures (timeouts, errors, invalid responses) or low confidence must **never** corrupt,
//!   drop, or fail local retrieval candidates. Deterministic local ordering is always maintained as fallback.

use std::collections::HashSet;
use std::time::Instant;
use tracing::{debug, info};

use crate::monitoring::metrics::JudgmentOutcome;
use crate::monitoring::metrics::global_retrieval_metrics;

/// Candidate item submitted for semantic judgment.
#[derive(Debug, Clone, PartialEq)]
pub struct JudgmentCandidate<'a> {
    /// Unique candidate identifier (e.g. episode ID).
    pub id: &'a str,
    /// Candidate text content.
    pub text: &'a str,
    /// Local score prior to semantic judgment (e.g., BM25/HDC/hybrid score).
    pub local_score: f32,
}

impl<'a> JudgmentCandidate<'a> {
    /// Create a new judgment candidate.
    #[must_use]
    pub const fn new(id: &'a str, text: &'a str, local_score: f32) -> Self {
        Self {
            id,
            text,
            local_score,
        }
    }
}

/// An atomic score with value and confidence in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtomicScore {
    /// Primary scalar score value in `[0.0, 1.0]`.
    pub value: f32,
    /// Provider confidence in the assigned score in `[0.0, 1.0]`.
    pub confidence: f32,
}

impl AtomicScore {
    /// Create a new atomic score.
    #[must_use]
    pub const fn new(value: f32, confidence: f32) -> Self {
        Self { value, confidence }
    }

    /// Check if the atomic score is finite and within the `[0.0, 1.0]` bound.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.value.is_finite()
            && (0.0..=1.0).contains(&self.value)
            && self.confidence.is_finite()
            && (0.0..=1.0).contains(&self.confidence)
    }
}

/// Typed atomic judgments for a single candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateJudgment {
    /// Unique candidate identifier matching an input candidate.
    pub id: String,
    /// Query relevance judgment.
    pub relevance: AtomicScore,
    /// Usefulness as evidence for query/task completion.
    pub useful_evidence: AtomicScore,
    /// Presence of contradictory statements regarding the query.
    pub contradiction: AtomicScore,
    /// Presence of prompt-injection or instruction-like text.
    pub instruction_like: AtomicScore,
}

impl CandidateJudgment {
    /// Check if all atomic scores contained within this candidate judgment are valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.relevance.is_valid()
            && self.useful_evidence.is_valid()
            && self.contradiction.is_valid()
            && self.instruction_like.is_valid()
    }
}

/// Errors occurring during semantic judgment operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum JudgmentError {
    /// The semantic judgment provider is unavailable or disconnected.
    #[error("semantic judgment provider unavailable")]
    Unavailable,

    /// Semantic judgment operation timed out.
    #[error("semantic judgment timed out")]
    Timeout,

    /// Provider output failed validation or contract constraints.
    #[error("semantic judgment provider returned invalid output: {0}")]
    Invalid(String),

    /// Provider execution failed with internal message.
    #[error("semantic judgment provider failed: {0}")]
    Provider(String),
}

/// Provider-neutral interface for batched semantic judgments.
///
/// Implementations adapt specific LLMs, local SLMs, or heuristics to score
/// candidate passages across typed dimensions without exposing provider SDKs to `memory-core`.
pub trait RetrievalJudge: Send + Sync {
    /// Assess a query and a batch of candidates, returning typed atomic judgments.
    ///
    /// # Errors
    ///
    /// Returns [`JudgmentError`] if the provider is unavailable, times out, or yields malformed output.
    fn judge_candidates(
        &self,
        query: &str,
        candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError>;
}

/// Validates raw provider judgments and aligns them with the input candidates order.
///
/// Ensures:
/// 1. Every judgment's atomic scores are finite and within `[0.0, 1.0]`.
/// 2. Candidate counts match exactly.
/// 3. No duplicate IDs exist in judgment results.
/// 4. Input candidate IDs and judgment IDs match exactly as a set (no missing or unknown IDs).
/// 5. Output vector is re-ordered to align strictly with input `candidates` order.
///
/// # Errors
///
/// Returns [`JudgmentError::Invalid`] if any validation rule is violated.
pub fn validate_and_align_judgments(
    candidates: &[JudgmentCandidate<'_>],
    judgments: Vec<CandidateJudgment>,
) -> Result<Vec<CandidateJudgment>, JudgmentError> {
    if judgments.len() != candidates.len() {
        return Err(JudgmentError::Invalid(format!(
            "candidate count mismatch: expected {}, got {}",
            candidates.len(),
            judgments.len()
        )));
    }

    // Check individual score validity and duplicate judgment IDs
    let mut seen_ids = HashSet::with_capacity(judgments.len());
    for j in &judgments {
        if !j.is_valid() {
            return Err(JudgmentError::Invalid(format!(
                "candidate '{}' has out-of-bound, NaN, or non-finite atomic score(s)",
                j.id
            )));
        }
        if !seen_ids.insert(j.id.as_str()) {
            return Err(JudgmentError::Invalid(format!(
                "duplicate candidate judgment ID in provider response: '{}'",
                j.id
            )));
        }
    }

    // Check that every candidate ID from input is present in judgments
    for c in candidates {
        if !seen_ids.contains(c.id) {
            return Err(JudgmentError::Invalid(format!(
                "missing candidate judgment ID in provider response: '{}'",
                c.id
            )));
        }
    }

    // Re-align judgments to match input candidate order. Every lookup
    // succeeds: the set check above rejects unknown or missing IDs and the
    // duplicate check rejects repeats, so each ID is consumed exactly once.
    let mut judgment_map: std::collections::HashMap<String, CandidateJudgment> =
        judgments.into_iter().map(|j| (j.id.clone(), j)).collect();

    let mut aligned = Vec::with_capacity(candidates.len());
    for c in candidates {
        let Some(j) = judgment_map.remove(c.id) else {
            return Err(JudgmentError::Invalid(format!(
                "unknown candidate ID mismatch during alignment: '{}'",
                c.id
            )));
        };
        aligned.push(j);
    }

    Ok(aligned)
}

/// Evaluates semantic judgments using an optional judge with timing, zero-work empty batch short-circuit,
/// provider result validation, and telemetry recording.
///
/// # Returns
/// - `Ok(None)` if no judge is configured.
/// - `Ok(Some(vec![]))` if `candidates` is empty (zero provider work).
/// - `Ok(Some(aligned_judgments))` if judgment succeeds and passes validation.
/// - `Err(JudgmentError)` if provider or validation fails.
///
/// # Telemetry Safety
/// Query text, candidate text, candidate IDs, and provider error strings are **never** recorded in metric labels.
/// Only bounded metadata (outcome enum, candidate count, elapsed time, judge_configured flag) is emitted.
pub fn evaluate_judgments(
    judge: Option<&dyn RetrievalJudge>,
    query: &str,
    candidates: &[JudgmentCandidate<'_>],
) -> Result<Option<Vec<CandidateJudgment>>, JudgmentError> {
    // 1. No judge configured -> return early without provider call
    let Some(judge) = judge else {
        global_retrieval_metrics().record_judgment(
            false,
            JudgmentOutcome::NotConfigured,
            candidates.len(),
            0,
        );
        return Ok(None);
    };

    // 2. Empty candidate batch -> zero provider work!
    if candidates.is_empty() {
        debug!("empty candidate batch for semantic judgment; zero provider work performed");
        global_retrieval_metrics().record_judgment(true, JudgmentOutcome::Ok, 0, 0);
        return Ok(Some(Vec::new()));
    }

    // 3. Measure provider call execution time
    let start = Instant::now();
    let raw_result = judge.judge_candidates(query, candidates);
    let elapsed_ms = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;

    // 4. Process provider response and update telemetry
    match raw_result {
        Ok(raw_judgments) => match validate_and_align_judgments(candidates, raw_judgments) {
            Ok(aligned) => {
                info!(
                    judge_configured = true,
                    outcome = %JudgmentOutcome::Ok.as_str(),
                    candidate_count = candidates.len(),
                    elapsed_ms = elapsed_ms,
                    "semantic candidate judgment completed successfully"
                );
                global_retrieval_metrics().record_judgment(
                    true,
                    JudgmentOutcome::Ok,
                    candidates.len(),
                    elapsed_ms,
                );
                Ok(Some(aligned))
            }
            Err(err) => {
                info!(
                    judge_configured = true,
                    outcome = %JudgmentOutcome::Invalid.as_str(),
                    candidate_count = candidates.len(),
                    elapsed_ms = elapsed_ms,
                    "semantic candidate judgment returned invalid response"
                );
                global_retrieval_metrics().record_judgment(
                    true,
                    JudgmentOutcome::Invalid,
                    candidates.len(),
                    elapsed_ms,
                );
                Err(err)
            }
        },
        Err(err) => {
            let outcome = match &err {
                JudgmentError::Unavailable => JudgmentOutcome::Unavailable,
                JudgmentError::Timeout => JudgmentOutcome::Timeout,
                JudgmentError::Invalid(_) => JudgmentOutcome::Invalid,
                JudgmentError::Provider(_) => JudgmentOutcome::ProviderError,
            };
            info!(
                judge_configured = true,
                outcome = %outcome.as_str(),
                candidate_count = candidates.len(),
                elapsed_ms = elapsed_ms,
                "semantic candidate judgment provider call failed"
            );
            global_retrieval_metrics().record_judgment(true, outcome, candidates.len(), elapsed_ms);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests;
