//! Local-tier finalization for the cascading retrieval pipeline (issue #1031).
//!
//! Holds the single rerank-aware finalization path shared by every
//! local-success branch (BM25, merged BM25+HDC, HDC, ConceptGraph) together
//! with the optional semantic rerank it applies. Split out of `mod.rs` to keep
//! individual source files under the 500 LOC quality gate (WG-185).
//!
//! The module is compiled only with the `csm` feature, matching the gating the
//! methods had inside `mod.rs`.

use crate::monitoring::metrics::RerankStatus;
use crate::retrieval::rerank::semantic_rerank;
use std::collections::HashMap;

use super::{CascadeResult, CascadeRetriever, FallbackPolicy, FallbackReason, local_confidence};

impl CascadeRetriever {
    /// Package sufficient local-tier results as the final [`CascadeResult`],
    /// optionally reranking them first (issue #1031).
    ///
    /// This is the single finalization path shared by every local-success
    /// branch (BM25, merged BM25+HDC, HDC, ConceptGraph): it reranks the local
    /// shortlist once when configured, then applies the unchanged local
    /// accounting. Cascade telemetry is recorded by `retrieve()` after this
    /// returns (see `record_cascade`). The
    /// only policy with an effect here is [`FallbackPolicy::AlwaysEmbed`],
    /// which counts a Tier 4 call on top of the local hit for baseline
    /// comparisons.
    pub(super) fn finish_ranked(
        &self,
        query: &str,
        results: &[(String, f32)],
        tiers: Vec<String>,
    ) -> CascadeResult {
        // Disabled (the default) borrows the local results untouched: no clone,
        // no text index, no provider call.
        let reranked = if self.rerank_active() {
            Some(self.rerank_local(query, results.to_vec()))
        } else {
            None
        };
        let ranked: &[(String, f32)] = reranked.as_deref().unwrap_or(results);

        let (top_score, score_margin, _) = local_confidence(
            ranked,
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
            scores: ranked.iter().map(|(_, s)| *s).collect(),
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

    /// Whether semantic reranking can do any work for this retriever.
    ///
    /// Both conditions are required: the stage is opt-in and a judge must be
    /// attached, so the default configuration never reaches the provider.
    pub(super) fn rerank_active(&self) -> bool {
        self.semantic_rerank.enabled && self.judge.is_some()
    }

    /// Optionally rerank a bounded local shortlist against `query`.
    ///
    /// Local-first and failure-safe:
    /// - when reranking is inactive the results are returned verbatim and no
    ///   shortlist text index is built;
    /// - only [`RerankStatus::Applied`] replaces the list (the deterministic
    ///   fused ordering, with its fused scores);
    /// - every other outcome (provider error, invalid output, low confidence,
    ///   text lookup mismatch) preserves the original local results, so a
    ///   provider problem can never fail or empty a successful local retrieval.
    pub(super) fn rerank_local(
        &self,
        query: &str,
        results: Vec<(String, f32)>,
    ) -> Vec<(String, f32)> {
        if !self.rerank_active() {
            return results;
        }

        let texts: HashMap<&str, &str> = self
            .episode_data
            .iter()
            .map(|(id, text)| (id.as_str(), text.as_str()))
            .collect();
        let outcome = semantic_rerank(
            query,
            &results,
            &texts,
            self.judge.as_deref(),
            &self.semantic_rerank,
        );

        if matches!(outcome.status, RerankStatus::Applied)
            && outcome.ids.len() == outcome.scores.len()
        {
            return outcome.ids.into_iter().zip(outcome.scores).collect();
        }

        // Failure policy: the local results are preserved as-is.
        results
    }
}
