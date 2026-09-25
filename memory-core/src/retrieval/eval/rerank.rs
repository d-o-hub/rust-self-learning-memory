//! Semantic rerank comparison arm for the retrieval benchmark (issue #1031).
//!
//! [`RetrievalEvaluator::evaluate_strategy_with_rerank`] runs the same per-query
//! loop as the baseline arms — the cascade is built once with a judge attached
//! and the rerank configuration installed — and reports the judge work the run
//! actually performed: `judge_calls_per_query`, `rerank_candidates_per_query`,
//! and `top1_changed_rate`.
//!
//! Retrieval executes **once per query**. The counters are captured from the
//! judge call itself, never from a second retrieval and never from
//! process-global telemetry that concurrent retrieval elsewhere in the process
//! could pollute.

use std::sync::{Arc, Mutex};

use crate::retrieval::judgment::{
    CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
};
use crate::retrieval::rerank::SemanticRerankConfig;

use super::runner::RetrievalEvaluator;
use super::types::{BenchmarkMetrics, RetrievalStrategy};

/// Judge instrumentation for one rerank comparison run.
///
/// Wraps the caller's judge so the runner can count judge invocations, count
/// judged candidates, and remember which candidate each shortlist was locally
/// ranked by.
///
/// The judge is owned (`Arc<dyn RetrievalJudge>`), mirroring
/// `CascadeRetriever::with_judge`: the cascade stores judges as
/// `Arc<dyn RetrievalJudge + 'static>`, so a borrowing wrapper cannot be
/// attached to it.
struct CountingJudge {
    inner: Arc<dyn RetrievalJudge>,
    stats: Mutex<JudgeStats>,
}

#[derive(Default)]
struct JudgeStats {
    /// Number of judge invocations so far.
    calls: usize,
    /// Number of candidates submitted so far.
    candidates: usize,
    /// Candidate the shortlist was ranked by (its first element, i.e. the
    /// pre-rerank top-1) per judge call, in call order.
    shortlist_top1: Vec<String>,
}

impl CountingJudge {
    fn new(inner: Arc<dyn RetrievalJudge>) -> Self {
        Self {
            inner,
            stats: Mutex::new(JudgeStats::default()),
        }
    }

    /// Poison-tolerant lock: a panicking judge must not turn later metric reads
    /// into a second panic.
    fn stats(&self) -> std::sync::MutexGuard<'_, JudgeStats> {
        self.stats
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// `(judge calls, judged candidates)` observed so far.
    fn totals(&self) -> (usize, usize) {
        let stats = self.stats();
        (stats.calls, stats.candidates)
    }

    /// Pre-rerank top-1 recorded for the judge call at `call_index`.
    fn shortlist_top1_at(&self, call_index: usize) -> Option<String> {
        self.stats().shortlist_top1.get(call_index).cloned()
    }
}

impl RetrievalJudge for CountingJudge {
    fn judge_candidates(
        &self,
        query: &str,
        candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
        // Recorded before delegating: a failing judge still consumed the
        // shortlist, and `top1_changed_rate` must not treat that query as if
        // the rerank stage had never run.
        {
            let mut stats = self.stats();
            stats.calls += 1;
            stats.candidates += candidates.len();
            stats.shortlist_top1.push(
                candidates
                    .first()
                    .map_or_else(String::new, |candidate| candidate.id.to_string()),
            );
        }

        self.inner.judge_candidates(query, candidates)
    }
}

/// Judge totals captured before one retrieval execution.
pub(super) struct RerankStart {
    calls: usize,
    candidates: usize,
}

/// Judge work observed over one rerank comparison run.
#[derive(Clone, Copy, Default)]
pub(super) struct RerankCounters {
    /// Judge invocations.
    judge_calls: usize,
    /// Candidates submitted to the judge.
    candidates: usize,
    /// Queries whose top-1 changed after rerank.
    top1_changes: usize,
}

impl RerankCounters {
    /// Per-query metrics: judge calls, judged candidates, top-1 changed rate.
    ///
    /// A query whose rerank stage never reached the judge counts as unchanged
    /// in the rate, which is exactly what the non-applied paths promise: the
    /// original local list is returned intact.
    pub(super) fn per_query(self, total_queries: usize) -> (f64, f64, f64) {
        let total = total_queries as f64;
        (
            self.judge_calls as f64 / total,
            self.candidates as f64 / total,
            self.top1_changes as f64 / total,
        )
    }
}

/// Judge plus configuration for one rerank comparison run.
pub(super) struct RerankProbe {
    /// Handle installed on the cascade (`Arc<dyn RetrievalJudge>`, as
    /// `CascadeRetriever::with_judge` stores).
    judge: Arc<dyn RetrievalJudge>,
    /// Counting view of the same wrapper, used for the rerank counters.
    counting: Arc<CountingJudge>,
    /// Configuration handed to `CascadeRetriever::with_semantic_rerank`.
    config: SemanticRerankConfig,
    /// Counters accumulated over the run.
    counters: RerankCounters,
}

impl RerankProbe {
    pub(super) fn new(judge: Arc<dyn RetrievalJudge>, config: &SemanticRerankConfig) -> Self {
        let counting: Arc<CountingJudge> = Arc::new(CountingJudge::new(judge));
        // Bind the concrete handle before coercing: annotating `Arc::clone`
        // with the trait-object type makes the clone infer `dyn RetrievalJudge`
        // as its parameter and reject the concrete argument.
        let counting_handle = Arc::clone(&counting);
        let judge_handle: Arc<dyn RetrievalJudge> = counting_handle;
        Self {
            judge: judge_handle,
            counting,
            config: config.clone(),
            counters: RerankCounters::default(),
        }
    }

    /// Judge handle to install on the cascade.
    pub(super) fn judge_handle(&self) -> Arc<dyn RetrievalJudge> {
        Arc::clone(&self.judge)
    }

    /// Configuration to validate and install on the cascade.
    pub(super) fn config(&self) -> &SemanticRerankConfig {
        &self.config
    }

    /// Capture judge totals before one retrieval execution.
    pub(super) fn begin_query(&self) -> RerankStart {
        let (calls, candidates) = self.counting.totals();
        RerankStart { calls, candidates }
    }

    /// Fold one retrieval execution's judge work into the run counters.
    ///
    /// `retrieved_top1` is the id the query returned. `semantic_rerank` submits
    /// the shortlist in local descending order, so the candidate the judge saw
    /// first is the pre-rerank top-1 and the comparison is the top-1 change
    /// signal.
    pub(super) fn finish_query(&mut self, start: RerankStart, retrieved_top1: Option<&str>) {
        let (calls_after, candidates_after) = self.counting.totals();
        self.counters.judge_calls += calls_after - start.calls;
        self.counters.candidates += candidates_after - start.candidates;

        if calls_after > start.calls
            && self
                .counting
                .shortlist_top1_at(start.calls)
                .is_some_and(|local_top1| retrieved_top1 != Some(local_top1.as_str()))
        {
            self.counters.top1_changes += 1;
        }
    }

    /// Counters accumulated so far.
    pub(super) fn counters(&self) -> RerankCounters {
        self.counters
    }
}

impl RetrievalEvaluator {
    /// Run evaluation for a specific strategy with the semantic rerank stage enabled.
    ///
    /// This is the comparison arm of issue #1031: the same per-query loop as
    /// [`RetrievalEvaluator::evaluate_strategy`], with `judge` attached to the
    /// retriever and `rerank_config` validated and installed by
    /// `CascadeRetriever::with_semantic_rerank`.
    ///
    /// The judge is taken as `Arc<dyn RetrievalJudge>` because the cascade owns
    /// judges behind `Arc` (`CascadeRetriever::with_judge`); the comparison
    /// runner wraps that handle to count judge work.
    ///
    /// # Metric derivation
    ///
    /// Retrieval runs **once per query**; the rerank counters are captured from
    /// the judge itself rather than from a second execution or from
    /// process-global telemetry:
    ///
    /// - `judge_calls_per_query` and `rerank_candidates_per_query` are counted
    ///   by the instrumentation wrapper around `judge`;
    /// - `top1_changed_rate` compares the returned top-1 against the candidate
    ///   the judge saw first, which is the pre-rerank top-1 because
    ///   `semantic_rerank` submits the shortlist in local descending order.
    ///
    /// # Errors
    ///
    /// Fails only when the corpus has no queries, when the corpus cannot be
    /// indexed, or when `rerank_config` is invalid — never because of a judge
    /// failure, which the rerank stage absorbs into local ordering.
    pub fn evaluate_strategy_with_rerank(
        &self,
        strategy: RetrievalStrategy,
        judge: Arc<dyn RetrievalJudge>,
        rerank_config: &SemanticRerankConfig,
    ) -> anyhow::Result<BenchmarkMetrics> {
        self.run_evaluation(strategy, Some(RerankProbe::new(judge, rerank_config)))
    }
}

#[cfg(test)]
mod tests {
    use crate::retrieval::judgment::{
        AtomicScore, CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
    };
    use crate::retrieval::rerank::SemanticRerankConfig;
    use std::sync::Arc;

    use super::RerankProbe;

    /// Failing judge: the call still counts, and the local order is preserved.
    struct FailingJudge;

    impl RetrievalJudge for FailingJudge {
        fn judge_candidates(
            &self,
            _query: &str,
            _candidates: &[JudgmentCandidate<'_>],
        ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
            Err(JudgmentError::Unavailable)
        }
    }

    #[test]
    fn test_probe_counts_calls_and_candidates() {
        let mut probe = RerankProbe::new(
            Arc::new(FailingJudge),
            &SemanticRerankConfig {
                enabled: true,
                ..SemanticRerankConfig::default()
            },
        );

        let start = probe.begin_query();
        let shortlist = vec![
            JudgmentCandidate::new("top", "text", 0.9),
            JudgmentCandidate::new("runner-up", "text", 0.4),
        ];
        assert!(
            probe
                .judge_handle()
                .judge_candidates("q", &shortlist)
                .is_err()
        );

        // Provider failed, so the local order stands: top-1 is unchanged.
        probe.finish_query(start, Some("top"));

        let (calls, candidates, changed) = probe.counters().per_query(1);
        assert_eq!((calls, candidates, changed), (1.0, 2.0, 0.0));
    }

    #[test]
    fn test_probe_records_top1_change_and_empty_results() {
        struct EchoJudge;

        impl RetrievalJudge for EchoJudge {
            fn judge_candidates(
                &self,
                _query: &str,
                candidates: &[JudgmentCandidate<'_>],
            ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
                Ok(candidates
                    .iter()
                    .map(|candidate| CandidateJudgment {
                        id: candidate.id.to_string(),
                        relevance: AtomicScore::new(1.0, 1.0),
                        useful_evidence: AtomicScore::new(0.0, 0.0),
                        contradiction: AtomicScore::new(0.0, 0.0),
                        instruction_like: AtomicScore::new(0.0, 0.0),
                    })
                    .collect())
            }
        }

        let mut probe = RerankProbe::new(
            Arc::new(EchoJudge),
            &SemanticRerankConfig {
                enabled: true,
                ..SemanticRerankConfig::default()
            },
        );
        let shortlist = vec![
            JudgmentCandidate::new("top", "text", 0.9),
            JudgmentCandidate::new("other", "text", 0.4),
        ];

        let start = probe.begin_query();
        probe
            .judge_handle()
            .judge_candidates("q", &shortlist)
            .unwrap();
        probe.finish_query(start, Some("other"));

        // A query that never reached the judge, and one that returned nothing,
        // both count as unchanged.
        let start = probe.begin_query();
        probe.finish_query(start, Some("top"));
        let start = probe.begin_query();
        probe.finish_query(start, None);

        let (calls, candidates, changed) = probe.counters().per_query(3);
        // One judged query out of three: 1 call, 2 candidates, 1 changed top-1.
        assert_eq!(
            (calls, candidates, changed),
            (1.0 / 3.0, 2.0 / 3.0, 1.0 / 3.0)
        );
    }

    #[test]
    fn test_unreranked_probe_stays_at_zero() {
        let probe = RerankProbe::new(Arc::new(FailingJudge), &SemanticRerankConfig::default());

        assert!(!probe.config().enabled);
        let (calls, candidates, changed) = probe.counters().per_query(4);
        assert_eq!((calls, candidates, changed), (0.0, 0.0, 0.0));
    }
}
