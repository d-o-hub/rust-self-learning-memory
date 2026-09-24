//! Shared local cascade pipeline (issue #1032).
//!
//! Holds the csm-only local retrieval pipeline: tier selection
//! ([`CascadeRetriever::local_plan`]) and the packaging that turns a plan into
//! the final [`CascadeResult`] ([`CascadeRetriever::finalize_local`], including
//! the batch-aware rerank seam it shares with the evidence stage). Split out of
//! `mod.rs` to keep individual source files under the 500 LOC quality gate
//! (WG-185), matching `types.rs`, `fallback.rs` and `finalize.rs`.
//!
//! The module is compiled only with the `csm` feature.

use super::{
    CascadeResult, CascadeRetriever, FallbackPolicy, FallbackReason, decide_fallback,
    local_confidence,
};
use crate::retrieval::rerank::JudgedShortlist;

/// The local cascade's result for one query, before rerank and evidence (issue
/// #1032).
///
/// Selection ([`CascadeRetriever::local_plan`]) is split from packaging
/// ([`CascadeRetriever::finalize_local`]) so the plain and evidence-aware
/// retrieval paths share one implementation of the tier and fallback
/// accounting.
pub(super) enum LocalPlan {
    /// A local tier satisfied the query:
    /// [`CascadeRetriever::finish_ranked`] accounting applies.
    Sufficient {
        /// Pre-rerank local ranking, best first.
        results: Vec<(String, f32)>,
        /// Contributing tier names.
        tiers: Vec<String>,
    },
    /// No tier sufficed: the Tier 4 decision reads the finalized best results.
    Fallback {
        /// Pre-rerank best local results.
        results: Vec<(String, f32)>,
        /// Whether Tier 1 produced any result.
        bm25_nonempty: bool,
        /// Whether Tier 2 produced any result.
        hdc_nonempty: bool,
    },
}

impl LocalPlan {
    /// The pre-rerank local ranking this plan finalizes.
    pub(super) fn results(&self) -> &[(String, f32)] {
        match self {
            Self::Sufficient { results, .. } | Self::Fallback { results, .. } => results,
        }
    }
}

impl CascadeRetriever {
    /// Full cascade implementation using CSM components.
    ///
    /// Tiers 1-3 return their local results once the count-based sufficiency
    /// rules hold. When no tier suffices, the Tier 4 fallback decision is
    /// governed by
    /// [`CascadeConfig::fallback_policy`](crate::retrieval::cascade::CascadeConfig::fallback_policy):
    /// `Adaptive` (default) returns confident local results without counting an
    /// API call and only escalates genuinely uncertain queries (see
    /// [`decide_fallback`]).
    ///
    /// Selection ([`Self::local_plan`]) is split from packaging
    /// ([`Self::finalize_local`]) so the evidence stage reuses the same
    /// pre-rerank ranking without re-running the cascade.
    pub(super) fn retrieve_with_csm(&self, query: &str) -> CascadeResult {
        let plan = self.local_plan(query);
        self.finalize_local(query, plan, None)
    }

    /// Select the local cascade's result for `query` (issue #1032).
    ///
    /// Tiers 1-3 return their local results once the count-based sufficiency
    /// rules hold; otherwise the best available local results are carried for
    /// the Tier 4 fallback decision. The plan holds only what finalization
    /// needs, so the same selection serves plain and evidence-aware retrieval.
    pub(super) fn local_plan(&self, query: &str) -> LocalPlan {
        use crate::retrieval::{compute_weights, merge_results};

        // Tier 1: BM25 keyword search
        let bm25_results = self.retrieve_bm25(query);

        // Check if BM25 produced sufficient results
        if bm25_results.sufficient {
            return LocalPlan::Sufficient {
                results: bm25_results.results,
                tiers: vec!["bm25".to_string()],
            };
        }

        // Tier 2: HDC similarity search
        let hdc_results = self.retrieve_hdc(query);

        // Check if HDC produced sufficient results (or merge with BM25)
        if self.config.merge_results && !bm25_results.is_empty() {
            // Merge BM25 and HDC results with query-length-dependent weights
            let weights = compute_weights(query.len());
            let merged = merge_results(
                &bm25_results.results,
                &hdc_results.results,
                weights,
                self.config.top_k,
            );

            // Check if merged results are sufficient
            if merged.len() >= self.config.min_results {
                return LocalPlan::Sufficient {
                    results: merged,
                    tiers: vec!["bm25".to_string(), "hdc".to_string()],
                };
            }
        } else if hdc_results.sufficient {
            return LocalPlan::Sufficient {
                results: hdc_results.results,
                tiers: vec!["hdc".to_string()],
            };
        }

        // Tier 3: ConceptGraph expansion (optional)
        if self.config.enable_concept_expansion {
            let concept_results = self.retrieve_concept_graph(query);

            if concept_results.sufficient {
                return LocalPlan::Sufficient {
                    results: concept_results.results,
                    tiers: vec!["concept_graph".to_string()],
                };
            }
        }

        // Tier 4: confidence-gated API fallback (issue #968).
        // Return best available results; whether they count as an API call
        // is decided by the fallback policy, not by result counts alone.
        let results: Vec<(String, f32)> = if self.config.merge_results {
            let weights = compute_weights(query.len());
            merge_results(
                &bm25_results.results,
                &hdc_results.results,
                weights,
                self.config.top_k,
            )
        } else if !hdc_results.is_empty() {
            hdc_results.results.clone()
        } else {
            bm25_results.results.clone()
        };

        LocalPlan::Fallback {
            results,
            bm25_nonempty: !bm25_results.is_empty(),
            hdc_nonempty: !hdc_results.is_empty(),
        }
    }

    /// Package a [`LocalPlan`] as the final [`CascadeResult`] (issue #1032).
    ///
    /// With `batch == None` this is exactly the path plain `retrieve()` takes:
    /// [`Self::finish_ranked`] for a satisfied local tier and the Tier 4
    /// decision over the reranked best results otherwise. With a batch, the
    /// rerank stage fuses the supplied judgments instead of calling the
    /// provider again, so one batch serves both the rerank and evidence stages
    /// without changing which tier accounting is reported.
    pub(super) fn finalize_local(
        &self,
        query: &str,
        plan: LocalPlan,
        batch: Option<&JudgedShortlist>,
    ) -> CascadeResult {
        match plan {
            LocalPlan::Sufficient { results, tiers } => {
                let Some(batch) = batch else {
                    return self.finish_ranked(query, &results, tiers);
                };
                self.finish_sufficient(query, results, tiers, batch)
            }
            LocalPlan::Fallback {
                results,
                bm25_nonempty,
                hdc_nonempty,
            } => self.finish_fallback(query, results, bm25_nonempty, hdc_nonempty, batch),
        }
    }

    /// Package a satisfied local tier from a shared judgment batch.
    ///
    /// The accounting is the one [`Self::finish_ranked`] applies — same tier
    /// list, same `AlwaysEmbed` handling, same confidence inputs — but the
    /// ranking comes from the caller's batch, so the provider is not called
    /// again.
    fn finish_sufficient(
        &self,
        query: &str,
        results: Vec<(String, f32)>,
        tiers: Vec<String>,
        batch: &JudgedShortlist,
    ) -> CascadeResult {
        let ranked = self.reranked_results(query, results, Some(batch));
        let (top_score, score_margin, _) = local_confidence(
            &ranked,
            self.config.local_confidence_threshold,
            self.config.minimum_score_margin,
        );
        let forced = self.config.fallback_policy == FallbackPolicy::AlwaysEmbed;
        let mut contributing_tiers = tiers;
        if forced {
            contributing_tiers.push("api".to_string());
        }
        CascadeResult {
            episode_ids: ranked.iter().map(|(id, _)| id.clone()).collect(),
            scores: ranked.iter().map(|(_, score)| *score).collect(),
            contributing_tiers,
            api_calls: u32::from(forced),
            fallback_reason: if forced {
                FallbackReason::AlwaysEmbedPolicy
            } else {
                FallbackReason::LocalTierSufficient
            },
            top_score,
            score_margin,
        }
    }

    /// Package the Tier 4 branch: finalize the best local results, decide the
    /// fallback over them, and report the tier accounting (issue #1032).
    fn finish_fallback(
        &self,
        query: &str,
        results: Vec<(String, f32)>,
        bm25_nonempty: bool,
        hdc_nonempty: bool,
        batch: Option<&JudgedShortlist>,
    ) -> CascadeResult {
        // The same single rerank call runs before the fallback decision, so
        // the decision, the tier accounting, and the returned literal all see
        // the final ordering.
        let best_results = self.reranked_results(query, results, batch);

        let decision = decide_fallback(
            self.config.fallback_policy,
            self.config.local_confidence_threshold,
            self.config.minimum_score_margin,
            &best_results,
        );
        // No query text, IDs, or scores-as-labels: bounded enums plus
        // numerics only (telemetry contract in
        // `plans/GOAP_FEATURE_WAVE_2026-09-04.md`).
        tracing::info!(
            policy = %self.config.fallback_policy,
            reason = %decision.reason,
            top_score = decision.top_score,
            score_margin = decision.score_margin,
            candidates = best_results.len(),
            api_calls = decision.api_calls,
            "cascade Tier 4 fallback decision"
        );

        let mut tiers = Vec::new();
        if bm25_nonempty {
            tiers.push("bm25".to_string());
        }
        if hdc_nonempty {
            tiers.push("hdc".to_string());
        }
        if decision.api_calls == 1 {
            if best_results.is_empty() {
                tiers = vec!["none".to_string()];
            } else {
                tiers.push("api_fallback_needed".to_string());
            }
        } else if tiers.is_empty() {
            tiers = vec!["none".to_string()];
        }

        CascadeResult {
            episode_ids: best_results.iter().map(|(id, _)| id.clone()).collect(),
            scores: best_results.iter().map(|(_, s)| *s).collect(),
            contributing_tiers: tiers,
            api_calls: decision.api_calls,
            fallback_reason: decision.reason,
            top_score: decision.top_score,
            score_margin: decision.score_margin,
        }
    }
}
