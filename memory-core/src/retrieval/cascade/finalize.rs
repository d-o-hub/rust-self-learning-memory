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
use crate::retrieval::judgment::JudgmentCandidate;
use crate::retrieval::rerank::{JudgedShortlist, fuse_with_judgments, semantic_rerank};
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
            Some(self.reranked_results(query, results.to_vec(), None))
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

    /// Finalize the local results, reusing an already-judged shortlist when the
    /// caller holds one (issue #1032).
    ///
    /// `batch` is the shared judgment batch for this exact `results` list (see
    /// `judge_shortlist_once`): when present, the rerank decision is taken from
    /// it and no provider call is made, so one validated batch can serve both
    /// this stage and a following evidence stage. Without a batch the provider
    /// path ([`Self::rerank_local`]) runs unchanged, so a caller that has no
    /// batch behaves exactly as before.
    ///
    /// The failure policy is the one of [`Self::rerank_local`]: only
    /// [`RerankStatus::Applied`] replaces the list, a shortlist of at most one
    /// candidate returns the leading local order capped at `output_k`, and every
    /// other status, or a fusion in which no judgment meets the confidence
    /// floor, preserves the original local results.
    pub(super) fn reranked_results(
        &self,
        query: &str,
        mut results: Vec<(String, f32)>,
        batch: Option<&JudgedShortlist>,
    ) -> Vec<(String, f32)> {
        if !self.rerank_active() {
            return results;
        }
        let Some(batch) = batch else {
            return self.rerank_local(query, results);
        };
        if batch.status != RerankStatus::Applied {
            return results;
        }

        let shortlist_len = batch.shortlist_len.min(results.len());
        // A shortlist of at most one candidate has no competing order, so the
        // leading local order stands, capped at `output_k`.
        if shortlist_len <= 1 {
            let output_len = self.semantic_rerank.output_k.min(results.len());
            results.truncate(output_len);
            return results;
        }

        // The judged shortlist is this list's prefix, so it is rebuilt from the
        // local scores; a batch judged over any other list cannot be fused.
        // Fusion reads the id and the local score only, never the passage text
        // the provider was called with.
        debug_assert_eq!(batch.judgments.len(), shortlist_len);
        let mut shortlist: Vec<JudgmentCandidate<'_>> = Vec::with_capacity(shortlist_len);
        for (id, score) in results.iter().take(shortlist_len) {
            shortlist.push(JudgmentCandidate::new(id, "", *score));
        }
        let (ranked, _avg_confidence, confident_count) = fuse_with_judgments(
            &shortlist,
            &batch.judgments,
            &self.semantic_rerank,
            self.semantic_rerank.output_k,
        );
        if confident_count == 0 {
            return results;
        }
        ranked
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::monitoring::metrics::RerankStatus;
    use crate::retrieval::judgment::{
        AtomicScore, CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
    };
    use crate::retrieval::rerank::{SemanticRerankConfig, judge_shortlist_once};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const QUERY: &str = "authentication JWT";

    /// Episodes backing both the local results and the shortlist texts.
    const CORPUS: [(&str, &str); 3] = [
        ("ep-1", "authentication JWT token"),
        ("ep-2", "authentication JWT validation"),
        ("ep-3", "authentication JWT parsing"),
    ];

    /// Deterministic judge: counts provider calls, promotes `promote` to full
    /// relevance at `confidence`, and can fail like an unavailable provider.
    struct StubJudge {
        calls: Arc<AtomicUsize>,
        promote: String,
        confidence: f32,
        fail: bool,
    }

    impl RetrievalJudge for StubJudge {
        fn judge_candidates(
            &self,
            _query: &str,
            candidates: &[JudgmentCandidate<'_>],
        ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(JudgmentError::Unavailable);
            }
            Ok(candidates
                .iter()
                .map(|candidate| CandidateJudgment {
                    id: candidate.id.to_string(),
                    relevance: if candidate.id == self.promote {
                        AtomicScore::new(1.0, self.confidence)
                    } else {
                        AtomicScore::new(0.0, self.confidence)
                    },
                    useful_evidence: AtomicScore::new(0.5, self.confidence),
                    contradiction: AtomicScore::new(0.0, self.confidence),
                    instruction_like: AtomicScore::new(0.0, self.confidence),
                })
                .collect())
        }
    }

    /// Local ranking handed to the finalizer: descending local scores.
    fn local_results() -> Vec<(String, f32)> {
        vec![
            ("ep-1".to_string(), 0.9),
            ("ep-2".to_string(), 0.6),
            ("ep-3".to_string(), 0.5),
        ]
    }

    fn corpus_texts() -> HashMap<&'static str, &'static str> {
        CORPUS.iter().copied().collect()
    }

    fn enabled_rerank() -> SemanticRerankConfig {
        SemanticRerankConfig {
            enabled: true,
            ..SemanticRerankConfig::default()
        }
    }

    fn build_retriever(
        calls: &Arc<AtomicUsize>,
        promote: &str,
        confidence: f32,
        fail: bool,
        config: SemanticRerankConfig,
    ) -> CascadeRetriever {
        let mut retriever = CascadeRetriever::default_config()
            .with_judge(Arc::new(StubJudge {
                calls: Arc::clone(calls),
                promote: promote.to_string(),
                confidence,
                fail,
            }))
            .with_semantic_rerank(config)
            .expect("the test rerank configuration is valid");
        for (id, text) in CORPUS {
            retriever.add_episode(id, text);
        }
        retriever
    }

    /// The supplied batch reranks exactly like the provider path and pays for
    /// no second judgment call.
    #[test]
    fn test_supplied_batch_matches_the_provider_path_without_extra_calls() {
        let calls = Arc::new(AtomicUsize::new(0));
        let retriever = build_retriever(&calls, "ep-3", 0.9, false, enabled_rerank());
        let results = local_results();
        let judge = retriever.judge().expect("judge is attached");

        let batch = judge_shortlist_once(
            QUERY,
            &results,
            &corpus_texts(),
            Some(&**judge),
            retriever.semantic_rerank_config().shortlist_k,
        );
        assert_eq!(batch.status, RerankStatus::Applied);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // A supplied batch must not judge a second time.
        let supplied = retriever.reranked_results(QUERY, results.clone(), Some(&batch));
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let provider = retriever.rerank_local(QUERY, results);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(supplied, provider);
        assert_eq!(supplied.first().map(|(id, _)| id.as_str()), Some("ep-3"));
    }

    /// Judgments below the confidence floor keep the local ids and scores.
    #[test]
    fn test_supplied_batch_keeps_local_results_below_the_confidence_floor() {
        let calls = Arc::new(AtomicUsize::new(0));
        let retriever = build_retriever(&calls, "ep-3", 0.2, false, enabled_rerank());
        let results = local_results();
        let judge = retriever.judge().expect("judge is attached");

        let batch = judge_shortlist_once(
            QUERY,
            &results,
            &corpus_texts(),
            Some(&**judge),
            retriever.semantic_rerank_config().shortlist_k,
        );
        assert_eq!(batch.status, RerankStatus::Applied);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // Untrusted judgments must not replace the local ranking.
        let reranked = retriever.reranked_results(QUERY, results.clone(), Some(&batch));
        assert_eq!(reranked, results);

        // Without a batch the provider path still decides.
        let fallback = retriever.reranked_results(QUERY, results.clone(), None);
        let provider = retriever.rerank_local(QUERY, results);
        assert_eq!(fallback, provider);
    }

    /// A failed batch preserves the local list and is never re-judged.
    #[test]
    fn test_supplied_batch_keeps_local_results_on_provider_error() {
        let calls = Arc::new(AtomicUsize::new(0));
        let retriever = build_retriever(&calls, "ep-3", 0.9, true, enabled_rerank());
        let results = local_results();
        let judge = retriever.judge().expect("judge is attached");

        let batch = judge_shortlist_once(
            QUERY,
            &results,
            &corpus_texts(),
            Some(&**judge),
            retriever.semantic_rerank_config().shortlist_k,
        );
        assert_eq!(batch.status, RerankStatus::ProviderError);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // A provider failure must keep the local ranking without a retry.
        let reranked = retriever.reranked_results(QUERY, results.clone(), Some(&batch));
        assert_eq!(reranked, results);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    /// A single-candidate shortlist keeps the leading local order, capped at
    /// `output_k`, with no provider work on either path.
    #[test]
    fn test_supplied_batch_respects_the_single_candidate_shortlist_cap() {
        let calls = Arc::new(AtomicUsize::new(0));
        let config = SemanticRerankConfig {
            enabled: true,
            shortlist_k: 1,
            output_k: 1,
            ..SemanticRerankConfig::default()
        };
        let retriever = build_retriever(&calls, "ep-3", 0.9, false, config);
        let results = local_results();
        let judge = retriever.judge().expect("judge is attached");

        let batch = judge_shortlist_once(
            QUERY,
            &results,
            &corpus_texts(),
            Some(&**judge),
            retriever.semantic_rerank_config().shortlist_k,
        );
        assert_eq!(batch.status, RerankStatus::Applied);
        assert_eq!(batch.shortlist_len, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let supplied = retriever.reranked_results(QUERY, results.clone(), Some(&batch));
        let provider = retriever.rerank_local(QUERY, results);
        assert_eq!(supplied, provider);
        assert_eq!(supplied, vec![("ep-1".to_string(), 0.9)]);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
