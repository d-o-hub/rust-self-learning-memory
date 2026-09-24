//! Evidence-aware cascade retrieval (issue #1032).
//!
//! Holds the opt-in evidence stage: one shared judgment batch over the local
//! candidates, the bounded disposition classification, and the ordered
//! [`EvidenceHit`] view returned by
//! [`CascadeRetriever::retrieve_with_evidence`]. Split out of `mod.rs` to keep
//! individual source files under the 500 LOC quality gate (WG-185), matching
//! `types.rs`, `fallback.rs` and `finalize.rs`.
//!
//! The module is compiled only with the `csm` feature.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::monitoring::metrics::{EvidenceStatus, RerankStatus, global_retrieval_metrics};
use crate::retrieval::EvidenceRetrievalResult;
use crate::retrieval::evidence::{
    CandidateEvidence, EvidenceDisposition, EvidenceHit, EvidencePolicy, classify_disposition,
};
use crate::retrieval::judgment::CandidateJudgment;

use super::{CascadeResult, CascadeRetriever};
use crate::retrieval::rerank::{JudgedShortlist, judge_shortlist_once};

impl CascadeRetriever {
    /// Evidence-aware cascade implementation (issue #1032).
    ///
    /// Runs the unchanged local cascade once, then makes at most one provider
    /// call shared by both judgment-consuming stages:
    ///
    /// 1. [`Self::local_plan`] selects the local ranking and the tier
    ///    accounting plain `retrieve()` would use;
    /// 2. the id -> passage text index and the pre-rerank scores are built once;
    /// 3. one [`judge_shortlist_once`] call covers the widest prefix either
    ///    stage needs: the rerank stage's `shortlist_k` or the evidence policy's
    ///    `candidate_limit`;
    /// 4. the rerank stage fuses its own bounded view of that batch, so `base`
    ///    is the exact [`CascadeResult`] plain `retrieve()` returns;
    /// 5. the evidence stage classifies the leading
    ///    `min(candidate_limit, shortlist_len)` judgments.
    ///
    /// Exactly one
    /// [`RetrievalMetrics::record_evidence`](crate::monitoring::metrics::RetrievalMetrics::record_evidence)
    /// call is made, and no query text, candidate id, or candidate text crosses
    /// into telemetry.
    pub(super) fn retrieve_evidence_with_csm(&self, query: &str) -> EvidenceRetrievalResult {
        let policy = self.evidence_policy.as_ref();
        let plan = self.local_plan(query);

        // The pre-rerank ranking feeds the judgment batch and every hit's
        // `local_score`. A disabled evidence stage on a rerank-disabled
        // retriever needs neither, so it skips the copy.
        let local: Vec<(String, f32)> = if policy.is_some() || self.rerank_active() {
            plan.results().to_vec()
        } else {
            Vec::new()
        };

        let (batch, stage) = match policy {
            None => (None, EvidenceStage::disabled()),
            Some(policy) => {
                // One provider call covers whichever stage asks for more.
                let shortlist_k = if self.rerank_active() {
                    self.semantic_rerank.shortlist_k.max(policy.candidate_limit)
                } else {
                    policy.candidate_limit
                };
                let texts: HashMap<&str, &str> = self
                    .episode_data
                    .iter()
                    .map(|(id, text)| (id.as_str(), text.as_str()))
                    .collect();
                let batch =
                    judge_shortlist_once(query, &local, &texts, self.judge.as_deref(), shortlist_k);
                let stage = classify_evidence(&batch, &local, policy);
                (Some(batch), stage)
            }
        };

        let narrowed = batch
            .as_ref()
            .and_then(|batch| self.narrow_rerank_batch(batch));
        let base = self.finalize_local(query, plan, narrowed.as_ref().or(batch.as_ref()));

        let hits = evidence_hits(&base, &local, &stage);

        let status = stage.status;
        global_retrieval_metrics().record_evidence(
            status,
            stage.considered,
            stage.dispositions,
            stage.provider_ms,
        );

        EvidenceRetrievalResult { base, hits, status }
    }

    /// The rerank stage's bounded view of a shared judgment batch.
    ///
    /// The shared batch can cover more candidates than the rerank stage's own
    /// `shortlist_k` when the evidence policy asks for a wider prefix. Fusing
    /// the wider prefix would rank a different candidate set than plain
    /// [`Self::retrieve`] does, so the rerank stage is bounded back to its
    /// configured shortlist here. `None` means the batch already fits.
    fn narrow_rerank_batch(&self, batch: &JudgedShortlist) -> Option<JudgedShortlist> {
        let shortlist_len = self.semantic_rerank.shortlist_k;
        if !self.rerank_active()
            || batch.status != RerankStatus::Applied
            || batch.judgments.len() <= shortlist_len
        {
            return None;
        }
        Some(JudgedShortlist {
            shortlist_len,
            judgments: batch.judgments[..shortlist_len].to_vec(),
            provider_ms: batch.provider_ms,
            status: batch.status,
        })
    }
}

/// Outcome of one evidence-classification pass over a shared judgment batch.
struct EvidenceStage {
    /// Bounded status recorded for telemetry.
    status: EvidenceStatus,
    /// Disposition counts in rank order: keep, flag, demote, drop.
    dispositions: [usize; 4],
    /// Provider duration reported by the shared batch.
    provider_ms: u64,
    /// Number of candidates the evidence stage classified.
    considered: usize,
    /// Evidence aligned with the leading `considered` local candidates.
    evidence: Vec<CandidateEvidence>,
}

impl EvidenceStage {
    /// The stage that ran no classification at all.
    fn disabled() -> Self {
        Self {
            status: EvidenceStatus::Disabled,
            dispositions: [0; 4],
            provider_ms: 0,
            considered: 0,
            evidence: Vec::new(),
        }
    }
}

/// Classify the shared batch's leading judgments under `policy`.
///
/// Only the first `min(candidate_limit, shortlist_len)` judged candidates are
/// classified: the batch may cover a wider prefix when the rerank stage asked
/// for more. The status is bounded — a failed or unusable batch maps straight
/// through, and an `Applied` batch in which no classified judgment is confident
/// enough (including an empty batch) is [`EvidenceStatus::LowConfidence`].
fn classify_evidence(
    batch: &JudgedShortlist,
    local: &[(String, f32)],
    policy: &EvidencePolicy,
) -> EvidenceStage {
    let limit = policy
        .candidate_limit
        .min(batch.shortlist_len)
        .min(local.len())
        .min(batch.judgments.len());
    let mut dispositions = [0usize; 4];
    let mut evidence = Vec::with_capacity(limit);
    let mut any_confident = false;

    for (judgment, (candidate_id, _)) in batch.judgments.iter().zip(local.iter()).take(limit) {
        debug_assert_eq!(
            judgment.id, *candidate_id,
            "validated judgments align with the local candidates"
        );
        let disposition = classify_disposition(judgment, policy);
        dispositions[usize::from(disposition.rank())] += 1;
        any_confident |= judgment_confident(judgment, policy.min_confidence);
        evidence.push(CandidateEvidence::from_judgment(judgment, disposition));
    }

    let status = match batch.status {
        RerankStatus::NotConfigured => EvidenceStatus::NotConfigured,
        RerankStatus::Invalid => EvidenceStatus::Invalid,
        RerankStatus::ProviderError => EvidenceStatus::ProviderError,
        RerankStatus::Applied if !any_confident => EvidenceStatus::LowConfidence,
        // The shared seam only reports the statuses above; an applied batch
        // with at least one trusted judgment is the only remaining outcome.
        _ => EvidenceStatus::Applied,
    };

    EvidenceStage {
        status,
        dispositions,
        provider_ms: batch.provider_ms,
        considered: limit,
        evidence,
    }
}

/// Ordering rank of a hit's disposition; an unjudged candidate ranks as `Keep`.
fn hit_rank(hit: &EvidenceHit) -> u8 {
    hit.evidence
        .map_or(EvidenceDisposition::Keep, |evidence| evidence.disposition)
        .rank()
}

/// Build the evidence view over a finalized ranking.
///
/// Every candidate of `base` becomes a hit carrying its pre-rerank
/// `local_score` (falling back to the final score when the ranking was not
/// reranked) and the evidence classified for it, if any. Hits are ordered by
/// disposition rank, then final score, then the stable retrieval index, and
/// `Drop` hits are removed — `Drop` is only ever produced when the caller set
/// [`EvidencePolicy::allow_drop`].
fn evidence_hits(
    base: &CascadeResult,
    local: &[(String, f32)],
    stage: &EvidenceStage,
) -> Vec<EvidenceHit> {
    let local_scores: HashMap<&str, f32> = local
        .iter()
        .map(|(id, score)| (id.as_str(), *score))
        .collect();
    let evidence_by_id: HashMap<&str, CandidateEvidence> = local
        .iter()
        .zip(stage.evidence.iter())
        .map(|((id, _), evidence)| (id.as_str(), *evidence))
        .collect();

    let mut ordered: Vec<(usize, EvidenceHit)> = base
        .episode_ids
        .iter()
        .zip(base.scores.iter())
        .enumerate()
        .map(|(index, (id, final_score))| {
            (
                index,
                EvidenceHit {
                    episode_id: id.clone(),
                    local_score: local_scores
                        .get(id.as_str())
                        .copied()
                        .unwrap_or(*final_score),
                    final_score: *final_score,
                    evidence: evidence_by_id.get(id.as_str()).copied(),
                },
            )
        })
        .collect();

    // Disposition first, then the finalized ranking, then the stable retrieval
    // index: the ordering is total and reproducible.
    ordered.sort_by(|(a_index, a), (b_index, b)| {
        hit_rank(a)
            .cmp(&hit_rank(b))
            .then_with(|| {
                b.final_score
                    .partial_cmp(&a.final_score)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| a_index.cmp(b_index))
    });

    ordered
        .into_iter()
        .map(|(_, hit)| hit)
        .filter(|hit| {
            !hit.evidence
                .is_some_and(|evidence| evidence.disposition == EvidenceDisposition::Drop)
        })
        .collect()
}

/// True when every judged dimension is confident enough to act on.
///
/// Mirrors the confidence guard inside
/// [`classify_disposition`](crate::retrieval::evidence::classify_disposition),
/// which the evidence module keeps private: the telemetry status needs the
/// predicate without a disposition, and `Keep` alone cannot distinguish "trusted
/// and clean" from "not trusted".
fn judgment_confident(judgment: &CandidateJudgment, min_confidence: f32) -> bool {
    judgment.relevance.confidence >= min_confidence
        && judgment.useful_evidence.confidence >= min_confidence
        && judgment.contradiction.confidence >= min_confidence
        && judgment.instruction_like.confidence >= min_confidence
}
