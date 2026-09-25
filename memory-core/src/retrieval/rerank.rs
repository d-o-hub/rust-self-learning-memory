//! Optional semantic reranking over a bounded local shortlist (issue #1031).
//!
//! Local retrieval already produces a ranking; this stage adds an *optional*
//! typed semantic relevance judgment over only the leading `shortlist_k`
//! candidates and fuses it with that ranking in deterministic Rust code:
//!
//! ```text
//! local ranking (descending local score)
//!   -> bounded shortlist: candidates[..min(shortlist_k, len)]
//!   -> optional typed judge (at most one batched provider call)
//!   -> deterministic fusion: min-max local + relevance * confidence
//!   -> top output_k candidates
//! ```
//!
//! The provider never supplies the ranking itself: it returns atomic
//! per-candidate relevance judgments, and Rust computes the order.
//!
//! # Trust Boundary
//!
//! Provider output is **inferential, not ground truth**, so the caller's
//! candidate list stays authoritative. Every path that is not a successful
//! fusion returns it unchanged (same ids, same order, same scores):
//!
//! - disabled ([`RerankStatus::Disabled`]) or no judge
//!   ([`RerankStatus::NotConfigured`]): no provider call;
//! - invalid configuration, empty candidates, a shortlisted candidate without
//!   text in `texts`, or a shortlist of at most one candidate: no provider call
//!   ([`RerankStatus::Invalid`] / [`RerankStatus::Applied`]); a shortlist of at
//!   most one returns the leading local order capped at `output_k`;
//! - provider error, timeout, unavailability, or malformed output: the original
//!   list with [`RerankStatus::ProviderError`] / [`RerankStatus::Invalid`];
//! - no judgment meeting [`SemanticRerankConfig::min_judgment_confidence`]: the
//!   original list with [`RerankStatus::LowConfidence`].
//!
//! A provider failure can therefore never turn a successful local retrieval
//! into an error, never drop a candidate, and never reorder local results.
//!
//! Telemetry is one
//! [`crate::monitoring::metrics::RetrievalMetrics::record_rerank`] call per
//! invocation, bounded values only (status, counts, provider duration, top-1
//! changed, mean relevance confidence). Query text, candidate ids, and candidate
//! text are never logged or emitted.

use std::collections::HashMap;

use tracing::{debug, info};

use crate::monitoring::metrics::{RerankStatus, global_retrieval_metrics};
use crate::retrieval::judgment::{CandidateJudgment, JudgmentCandidate, RetrievalJudge};

use batch::{build_shortlist, fuse_shortlist, judge_shortlist};

/// Configuration for the optional semantic rerank stage, disabled by default so
/// retrieval ordering, cost, and provider usage are unchanged unless a caller
/// explicitly opts in with a judge.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRerankConfig {
    /// Whether the semantic rerank stage runs at all.
    pub enabled: bool,
    /// Maximum number of leading local candidates sent to the judge in one batch.
    pub shortlist_k: usize,
    /// Maximum number of candidates returned after fusion (`<= shortlist_k`).
    pub output_k: usize,
    /// Weight of the min-max normalized local score in the fused score.
    pub local_weight: f32,
    /// Weight of the provider relevance judgment in the fused score.
    pub semantic_weight: f32,
    /// Minimum per-candidate relevance confidence required to use a judgment.
    pub min_judgment_confidence: f32,
}

impl Default for SemanticRerankConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            shortlist_k: 20,
            output_k: 10,
            local_weight: 0.4,
            semantic_weight: 0.6,
            min_judgment_confidence: 0.70,
        }
    }
}

impl SemanticRerankConfig {
    /// Validate this configuration and return it with weights normalized to sum
    /// to exactly `1.0` (keeping the fused score a convex combination).
    ///
    /// Rules: `shortlist_k > 0`; `output_k > 0 && output_k <= shortlist_k`; both
    /// weights finite, non-negative, not both zero; confidence floor finite and
    /// within `[0.0, 1.0]`. Violations return the first failing rule as a
    /// [`RerankConfigError`].
    pub fn validated(&self) -> Result<Self, RerankConfigError> {
        if self.shortlist_k == 0 {
            return Err(RerankConfigError::ZeroShortlist);
        }
        if self.output_k == 0 || self.output_k > self.shortlist_k {
            return Err(RerankConfigError::InvalidOutputK {
                output_k: self.output_k,
                shortlist_k: self.shortlist_k,
            });
        }
        if !self.local_weight.is_finite() || !self.semantic_weight.is_finite() {
            return Err(RerankConfigError::NonFiniteWeight);
        }
        if self.local_weight < 0.0 || self.semantic_weight < 0.0 {
            return Err(RerankConfigError::NegativeWeight);
        }
        let weight_sum = self.local_weight + self.semantic_weight;
        if weight_sum <= 0.0 {
            return Err(RerankConfigError::ZeroWeightSum);
        }
        if !self.min_judgment_confidence.is_finite()
            || !(0.0..=1.0).contains(&self.min_judgment_confidence)
        {
            return Err(RerankConfigError::InvalidConfidence);
        }
        Ok(Self {
            local_weight: self.local_weight / weight_sum,
            semantic_weight: self.semantic_weight / weight_sum,
            ..self.clone()
        })
    }
}

/// Configuration errors for [`SemanticRerankConfig::validated`]: `ZeroShortlist`
/// (`shortlist_k == 0`), `InvalidOutputK` (`output_k == 0` or `> shortlist_k`),
/// `NonFiniteWeight`, `NegativeWeight`, `ZeroWeightSum` (no fusion signal), and
/// `InvalidConfidence` (floor not finite or outside `[0.0, 1.0]`).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RerankConfigError {
    #[error("rerank shortlist_k must be greater than zero")]
    ZeroShortlist,
    #[error("rerank output_k must be greater than zero and at most shortlist_k ({shortlist_k})")]
    InvalidOutputK { output_k: usize, shortlist_k: usize },
    #[error("rerank weights must be finite")]
    NonFiniteWeight,
    #[error("rerank weights must be non-negative")]
    NegativeWeight,
    #[error("rerank weights must not sum to zero")]
    ZeroWeightSum,
    #[error("rerank min_judgment_confidence must be finite and within [0.0, 1.0]")]
    InvalidConfidence,
}

/// Result of one [`semantic_rerank`] invocation.
///
/// `ids`/`scores` are the returned ranking: the fused shortlist truncated to
/// `output_k` when reranking applied, or the caller's original list when the
/// stage was skipped or fell back (see the module trust boundary).
#[derive(Debug, Clone, PartialEq)]
pub struct RerankOutcome {
    pub ids: Vec<String>,
    pub scores: Vec<f32>,
    pub status: RerankStatus,
    /// Number of shortlisted candidates; `0` when nothing was shortlisted.
    pub shortlist_len: usize,
    pub output_len: usize,
    /// Whether the returned top-1 id differs from the pre-rerank top-1 id.
    pub top1_changed: bool,
}

/// One validated judgment batch over a bounded shortlist, shared by every
/// judgment-consuming stage.
///
/// Produced by [`judge_shortlist_once`], which makes at most one provider call.
/// When `status` is [`RerankStatus::Applied`] and `shortlist_len > 1`, the
/// `judgments` align one-to-one with the leading `shortlist_len` candidates of
/// the input list; every other path carries no judgments. Callers that already
/// hold a batch fuse it with [`fuse_with_judgments`] instead of paying for a
/// second provider call.
#[derive(Debug)]
pub(crate) struct JudgedShortlist {
    /// Number of leading local candidates the batch covers (`0` when nothing
    /// was shortlisted).
    pub shortlist_len: usize,
    /// Validated, shortlist-aligned judgments (empty unless `status` is
    /// [`RerankStatus::Applied`]).
    pub judgments: Vec<CandidateJudgment>,
    /// Provider duration in milliseconds (`0` when no provider call was made).
    pub provider_ms: u64,
    /// Bounded outcome of the shared provider seam.
    pub status: RerankStatus,
}

/// Runs the shared provider seam at most once over the leading `shortlist_k`
/// candidates and returns the validated batch.
///
/// Precedence (first match wins), mirroring the decisions the rerank stage takes
/// on top of it: no judge yields [`RerankStatus::NotConfigured`] without touching
/// the provider; a shortlisted candidate without passage text yields
/// [`RerankStatus::Invalid`]; a shortlist of at most one candidate yields
/// [`RerankStatus::Applied`] because there is no competing order to establish;
/// otherwise the judge is called at most once and its aligned judgments are
/// returned with [`RerankStatus::Applied`], or the bounded failure status is
/// returned with no judgments.
///
/// The caller owns the stage gates (`enabled`, validated configuration), so this
/// seam never validates configuration or records telemetry.
pub(crate) fn judge_shortlist_once<'a>(
    query: &str,
    candidates: &'a [(String, f32)],
    texts: &HashMap<&'a str, &'a str>,
    judge: Option<&dyn RetrievalJudge>,
    shortlist_k: usize,
) -> JudgedShortlist {
    let Some(judge) = judge else {
        return JudgedShortlist {
            shortlist_len: 0,
            judgments: Vec::new(),
            provider_ms: 0,
            status: RerankStatus::NotConfigured,
        };
    };
    let shortlist_len = shortlist_k.min(candidates.len());
    let Some(shortlist) = build_shortlist(candidates, texts, shortlist_len) else {
        debug!(
            shortlist_len,
            "judgment shortlist text missing; keeping local order"
        );
        return JudgedShortlist {
            shortlist_len,
            judgments: Vec::new(),
            provider_ms: 0,
            status: RerankStatus::Invalid,
        };
    };
    if shortlist_len <= 1 {
        return JudgedShortlist {
            shortlist_len,
            judgments: Vec::new(),
            provider_ms: 0,
            status: RerankStatus::Applied,
        };
    }

    match judge_shortlist(query, &shortlist, judge) {
        Ok((judgments, provider_ms)) => JudgedShortlist {
            shortlist_len,
            judgments,
            provider_ms,
            status: RerankStatus::Applied,
        },
        Err((status, provider_ms)) => JudgedShortlist {
            shortlist_len,
            judgments: Vec::new(),
            provider_ms,
            status,
        },
    }
}

/// Fuses a judged shortlist into the ranked `(id, score)` pairs a caller
/// returns, truncated to `output_k`.
///
/// `shortlist` and `judgments` align one-to-one and are non-empty. Returns the
/// ranked entries, the mean relevance confidence over the shortlist, and how
/// many judgments met [`SemanticRerankConfig::min_judgment_confidence`]. The
/// ordering rule and the below-floor rule are the ones documented on the
/// internal fusion helper; this wrapper lets callers outside this module fuse a
/// shared judgment batch without naming the private ordered-entry type.
pub(crate) fn fuse_with_judgments(
    shortlist: &[JudgmentCandidate<'_>],
    judgments: &[CandidateJudgment],
    config: &SemanticRerankConfig,
    output_k: usize,
) -> (Vec<(String, f32)>, f32, usize) {
    let (fused, avg_confidence, confident_count) =
        fuse_shortlist(shortlist, judgments, config, output_k);
    let ranked = fused
        .into_iter()
        .map(|candidate| (candidate.id, candidate.score))
        .collect();
    (ranked, avg_confidence, confident_count)
}

/// Semantically rerank the leading local candidates against `query`.
///
/// `candidates` MUST be the caller's local ranking, best first (descending local
/// score); the leading `config.shortlist_k` entries form the shortlist and the
/// provider is called at most once with that batch. `texts` maps candidate id to
/// the passage text submitted to the judge. When the shortlist holds at most one
/// candidate there is no competing order to establish, so the stage makes no
/// provider call and returns the leading local order capped at `config.output_k`.
///
/// Every other skip or fallback path returns the caller's candidates unchanged;
/// see the module documentation for the trust boundary, fusion formula, the
/// confidence floor, the deterministic tie-break, and the telemetry contract.
pub fn semantic_rerank(
    query: &str,
    candidates: &[(String, f32)],
    texts: &HashMap<&str, &str>,
    judge: Option<&dyn RetrievalJudge>,
    config: &SemanticRerankConfig,
) -> RerankOutcome {
    let report = rerank_report(query, candidates, texts, judge, config);
    let status = report.outcome.status;

    global_retrieval_metrics().record_rerank(
        status,
        report.outcome.shortlist_len,
        report.outcome.output_len,
        report.provider_ms,
        report.outcome.top1_changed,
        report.avg_confidence,
    );

    if matches!(status, RerankStatus::Disabled | RerankStatus::NotConfigured) {
        debug!(status = %status.as_str(), "semantic rerank skipped; local order unchanged");
    } else {
        info!(
            status = %status.as_str(),
            shortlist_len = report.outcome.shortlist_len,
            output_len = report.outcome.output_len,
            top1_changed = report.outcome.top1_changed,
            provider_ms = report.provider_ms,
            avg_relevance_confidence = f64::from(report.avg_confidence),
            "semantic rerank finished"
        );
    }

    report.outcome
}

/// Ranking returned to the caller plus the bounded telemetry values.
struct RerankReport {
    outcome: RerankOutcome,
    /// Mean relevance confidence over the shortlist (`0.0` when no judgment).
    avg_confidence: f32,
    provider_ms: u64,
}

/// Runs the rerank decision tree, returning the outcome plus telemetry values.
fn rerank_report(
    query: &str,
    candidates: &[(String, f32)],
    texts: &HashMap<&str, &str>,
    judge: Option<&dyn RetrievalJudge>,
    config: &SemanticRerankConfig,
) -> RerankReport {
    // The caller's candidates, unchanged: the authoritative fallback ranking.
    let verbatim = |status: RerankStatus, shortlist_len: usize| {
        local_order_outcome(candidates, candidates.len(), shortlist_len, status)
    };
    let report = |outcome: RerankOutcome, avg_confidence: f32, provider_ms: u64| RerankReport {
        outcome,
        avg_confidence,
        provider_ms,
    };

    if !config.enabled {
        return report(verbatim(RerankStatus::Disabled, 0), 0.0, 0);
    }
    let Some(judge) = judge else {
        return report(verbatim(RerankStatus::NotConfigured, 0), 0.0, 0);
    };
    let Ok(config) = config.validated() else {
        debug!("semantic rerank configuration invalid; local retrieval order unchanged");
        return report(verbatim(RerankStatus::Invalid, 0), 0.0, 0);
    };
    if candidates.is_empty() {
        return report(verbatim(RerankStatus::Applied, 0), 0.0, 0);
    }

    // The shared provider seam: at most one judgment call over the bounded
    // leading shortlist, reused by every judgment-consuming stage.
    let batch = judge_shortlist_once(query, candidates, texts, Some(judge), config.shortlist_k);
    if batch.status != RerankStatus::Applied {
        let outcome = verbatim(batch.status, batch.shortlist_len);
        return report(outcome, 0.0, batch.provider_ms);
    }

    // A shortlist of at most one candidate has no competing order to establish:
    // the leading local order stands, capped at `output_k`.
    if batch.shortlist_len <= 1 {
        let output_len = config.output_k.min(candidates.len());
        let outcome = local_order_outcome(
            candidates,
            output_len,
            batch.shortlist_len,
            RerankStatus::Applied,
        );
        return report(outcome, 0.0, 0);
    }

    // The batch covered exactly this shortlist, so fusion reuses its judgments
    // against the leading local ids and scores. Only those two fields feed the
    // fusion, so the provider-facing passage text is not rebuilt here.
    let mut shortlist: Vec<JudgmentCandidate<'_>> = Vec::with_capacity(batch.shortlist_len);
    for (id, score) in candidates.iter().take(batch.shortlist_len) {
        shortlist.push(JudgmentCandidate::new(id, "", *score));
    }
    let (fused, avg_confidence, confident_count) =
        fuse_with_judgments(&shortlist, &batch.judgments, &config, config.output_k);
    if confident_count == 0 {
        debug!(
            shortlist_len = batch.shortlist_len,
            "no judgment above the confidence floor; keeping local order"
        );
        let outcome = verbatim(RerankStatus::LowConfidence, batch.shortlist_len);
        return report(outcome, avg_confidence, batch.provider_ms);
    }

    let output_len = fused.len();
    let top1_changed = fused
        .first()
        .zip(candidates.first())
        .is_some_and(|((top, _), (local_top_id, _))| top.as_str() != local_top_id.as_str());
    report(
        RerankOutcome {
            ids: fused.iter().map(|(id, _)| id.clone()).collect(),
            scores: fused.iter().map(|(_, score)| *score).collect(),
            status: RerankStatus::Applied,
            shortlist_len: batch.shortlist_len,
            output_len,
            top1_changed,
        },
        avg_confidence,
        batch.provider_ms,
    )
}

/// The leading local candidates in unchanged order, capped at `output_len`.
fn local_order_outcome(
    candidates: &[(String, f32)],
    output_len: usize,
    shortlist_len: usize,
    status: RerankStatus,
) -> RerankOutcome {
    let mut ids = Vec::with_capacity(output_len);
    let mut scores = Vec::with_capacity(output_len);
    for (id, score) in candidates.iter().take(output_len) {
        ids.push(id.clone());
        scores.push(*score);
    }
    RerankOutcome {
        ids,
        scores,
        status,
        shortlist_len,
        output_len,
        top1_changed: false,
    }
}

/// Min-max normalizes `scores` into `[0.0, 1.0]`.
///
/// Equal (or otherwise degenerate) inputs normalize to `1.0` for every entry:
/// with no spread there is no local preference to weaken. Non-finite inputs can
/// never produce a non-finite normalized value, so fusion stays finite and
/// ordering stays total.
fn normalize_min_max(scores: &[f32]) -> Vec<f32> {
    let (min, max) = scores
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &s| {
            (lo.min(s), hi.max(s))
        });

    let span = max - min;
    if span <= 0.0 || !span.is_finite() {
        return vec![1.0; scores.len()];
    }

    scores
        .iter()
        .map(|&score| {
            let normalized = (score - min) / span;
            if normalized.is_finite() {
                normalized.clamp(0.0, 1.0)
            } else {
                1.0
            }
        })
        .collect()
}

mod batch;

#[cfg(test)]
mod tests;
