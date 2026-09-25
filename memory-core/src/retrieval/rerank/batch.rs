//! Judgment-batch mechanics shared by every judgment-consuming stage
//! (issue #1032).
//!
//! Holds the pieces one bounded provider call and its deterministic fusion
//! need — bounded shortlist construction, the single batched judge call, and
//! the fused ordering — so the seam that yields a validated batch lives in one
//! place. Every item is `pub(super)`: the `rerank` module owns the public
//! surface, and the evidence stage consumes the same batch without a second
//! provider call.
//!
//! The trust boundary is the one documented on the `rerank` module: provider
//! output is inferential, so Rust computes the order and every failure path
//! keeps the caller's local ranking unchanged.

use std::collections::HashMap;
use std::time::Instant;

use tracing::debug;

use super::{SemanticRerankConfig, normalize_min_max};
use crate::monitoring::metrics::RerankStatus;
use crate::retrieval::judgment::{
    CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge, evaluate_judgments,
};

/// Builds the one judgment batch from the leading local candidates.
///
/// `None` means a shortlisted candidate has no passage text, which must skip the
/// provider call rather than judge a candidate without text.
pub(super) fn build_shortlist<'a>(
    candidates: &'a [(String, f32)],
    texts: &HashMap<&'a str, &'a str>,
    shortlist_len: usize,
) -> Option<Vec<JudgmentCandidate<'a>>> {
    let mut shortlist: Vec<JudgmentCandidate<'a>> = Vec::with_capacity(shortlist_len);
    for (id, local_score) in candidates.iter().take(shortlist_len) {
        let text = texts.get(id.as_str()).copied()?;
        shortlist.push(JudgmentCandidate::new(id, text, *local_score));
    }
    Some(shortlist)
}

/// Calls the judge once and aligns its judgments with the shortlist.
///
/// # Errors
///
/// Returns the bounded [`RerankStatus`] the failed attempt maps to plus the
/// provider duration, so the caller keeps the local order without inspecting
/// provider error strings.
pub(super) fn judge_shortlist(
    query: &str,
    shortlist: &[JudgmentCandidate<'_>],
    judge: &dyn RetrievalJudge,
) -> Result<(Vec<CandidateJudgment>, u64), (RerankStatus, u64)> {
    let provider_start = Instant::now();
    let result = evaluate_judgments(Some(judge), query, shortlist);
    let provider_ms = provider_start
        .elapsed()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;

    match result {
        Ok(Some(judgments)) if judgments.len() == shortlist.len() => Ok((judgments, provider_ms)),
        Ok(Some(judgments)) => {
            let count = judgments.len();
            debug!(
                shortlist_len = shortlist.len(),
                judgment_count = count,
                "count mismatch"
            );
            Err((RerankStatus::Invalid, provider_ms))
        }
        // Unreachable while a judge is configured: never invent a ranking.
        Ok(None) => Err((RerankStatus::NotConfigured, provider_ms)),
        Err(err) => {
            let status = match err {
                JudgmentError::Invalid(_) => RerankStatus::Invalid,
                JudgmentError::Unavailable
                | JudgmentError::Timeout
                | JudgmentError::Provider(_) => RerankStatus::ProviderError,
            };
            debug!(status = %status.as_str(), shortlist_len = shortlist.len(), "provider call failed");
            Err((status, provider_ms))
        }
    }
}

/// One candidate during deterministic fused ordering.
pub(super) struct FusedCandidate {
    /// Candidate id, stable across the local and provider views.
    pub(super) id: String,
    /// Fused score (fusion formula on the `rerank` module).
    pub(super) score: f32,
    /// Position in the pre-rerank local order (primary tie-break).
    pub(super) local_rank: usize,
}

/// Fuses local scores with aligned relevance judgments, orders the result
/// deterministically, and truncates it to `output_k`.
///
/// Returns the ordered entries, the mean relevance confidence over the
/// shortlist, and how many judgments met the confidence floor. `shortlist` and
/// `judgments` are aligned one-to-one and non-empty. Ordering is descending
/// fused score, ties resolved by original local rank, then candidate id.
pub(super) fn fuse_shortlist(
    shortlist: &[JudgmentCandidate<'_>],
    judgments: &[CandidateJudgment],
    config: &SemanticRerankConfig,
    output_k: usize,
) -> (Vec<FusedCandidate>, f32, usize) {
    let local_scores: Vec<f32> = shortlist
        .iter()
        .map(|candidate| candidate.local_score)
        .collect();
    let normalized = normalize_min_max(&local_scores);

    let mut fused: Vec<FusedCandidate> = Vec::with_capacity(shortlist.len());
    let mut confidence_sum = 0.0_f32;
    let mut confident_count = 0_usize;

    for (local_rank, (candidate, judgment)) in shortlist.iter().zip(judgments.iter()).enumerate() {
        let relevance = judgment.relevance;
        confidence_sum += relevance.confidence;

        let local = normalized[local_rank];
        let score = if relevance.confidence >= config.min_judgment_confidence {
            confident_count += 1;
            config.local_weight * local
                + config.semantic_weight * (relevance.value * relevance.confidence)
        } else {
            // Below the confidence floor the provider signal is not trustworthy
            // enough to outweigh a deterministic local score, so the candidate
            // keeps its normalized local score and, with it, its local ordering.
            local
        };

        fused.push(FusedCandidate {
            id: candidate.id.to_string(),
            score,
            local_rank,
        });
    }

    fused.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.local_rank.cmp(&b.local_rank))
            .then_with(|| a.id.cmp(&b.id))
    });
    fused.truncate(output_k);

    let avg_confidence = confidence_sum / shortlist.len() as f32;
    (fused, avg_confidence, confident_count)
}
