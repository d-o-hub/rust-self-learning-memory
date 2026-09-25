//! Evidence-classification comparison arm for the retrieval benchmark (issue #1032).
//!
//! [`RetrievalEvaluator::evaluate_with_evidence`] runs the same per-query loop as
//! the baseline arms, with the deterministic [`EvidenceFixtureJudge`] attached
//! and an [`EvidencePolicy`] installed on the cascade, and reports what the
//! evidence stage actually did: per-dimension precision/recall against the
//! corpus labels, the disposition distribution, the release-blocking false-drop
//! count, judge calls per query, and the judge time the stage added.
//!
//! Retrieval executes **once per query**. The counters and the added latency are
//! captured from the judge call itself, never from a second retrieval and never
//! from process-global telemetry that concurrent retrieval elsewhere in the
//! process could pollute.
//!
//! # Mechanism probe, not a quality claim
//!
//! [`EvidenceFixtureJudge`] replays the corpus's own `evidence_labels`, so the
//! arm verifies the classification path end to end — thresholds, disposition
//! policy, telemetry, and reporting — and cannot be read as evidence that
//! evidence-aware classification improves retrieval quality. Only a real
//! provider judge against a labelled corpus can support that claim. See
//! `docs/eval_benchmark_guide.md`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::retrieval::cascade::{CascadeError, CascadeResult};
use crate::retrieval::evidence::{
    CandidateEvidence, EvidenceDisposition, EvidencePolicy, EvidenceRetrievalResult,
};
use crate::retrieval::judgment::{
    AtomicScore, CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
};

use super::runner::{RetrievalEvaluator, compute_percentiles};
use super::types::{
    BenchmarkMetrics, ClassificationMetrics, DispositionDistribution, EvidenceFixtureLabel,
    EvidenceMetrics, FixtureCorpus, RetrievalStrategy, count_ratio,
};

/// Fixed confidence attached to every [`EvidenceFixtureJudge`] dimension score.
///
/// Fixture labels are ground truth for this mechanism probe rather than a
/// sampled model output, so the judge reports full confidence and every judgment
/// clears [`EvidencePolicy::min_confidence`]; the policy therefore always acts
/// on the labelled values.
pub const EVIDENCE_FIXTURE_JUDGE_CONFIDENCE: f32 = 1.0;

/// Offline [`RetrievalJudge`] replaying a corpus's own evidence labels.
///
/// For every submitted candidate the judge returns the labelled boolean of each
/// dimension as `1.0`/`0.0` at [`EVIDENCE_FIXTURE_JUDGE_CONFIDENCE`]. Ids and
/// candidate order are preserved exactly, one judgment per candidate, and no
/// candidate is ever dropped or reordered.
///
/// Labels are keyed by query text — the only query identifier a judge receives —
/// so two queries sharing text share labels.
///
/// A candidate with no label for that query is reported **all-false** (all four
/// dimensions `0.0`) at the same full confidence. That is deliberate: the
/// unlabelled candidate is still judged, and under the default
/// [`EvidencePolicy`] the trusted all-false judgment yields
/// [`EvidenceDisposition::Demote`] (`relevance` below `min_relevance`, and
/// `allow_drop` unset). Demotion is recoverable and never removes a candidate,
/// so an unlabelled id can never masquerade as a false drop.
#[derive(Debug, Clone)]
pub struct EvidenceFixtureJudge {
    labels: HashMap<String, HashMap<String, EvidenceFixtureLabel>>,
}

impl EvidenceFixtureJudge {
    /// Capture every query's `evidence_labels`, keyed by query text.
    #[must_use]
    pub fn from_corpus(corpus: &FixtureCorpus) -> Self {
        Self::new(
            corpus
                .queries
                .iter()
                .map(|query| (query.query.clone(), query.evidence_labels.clone()))
                .collect(),
        )
    }

    /// Build from an explicit `query text -> (candidate id -> label)` map.
    #[must_use]
    pub fn new(labels: HashMap<String, HashMap<String, EvidenceFixtureLabel>>) -> Self {
        Self { labels }
    }

    /// Label for one candidate of one query, when both are labelled.
    fn label(&self, query: &str, candidate_id: &str) -> Option<&EvidenceFixtureLabel> {
        self.labels.get(query)?.get(candidate_id)
    }
}

impl RetrievalJudge for EvidenceFixtureJudge {
    fn judge_candidates(
        &self,
        query: &str,
        candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
        Ok(candidates
            .iter()
            .map(|candidate| {
                let label = self.label(query, candidate.id);
                let score = |expected: bool| {
                    let value = if expected { 1.0 } else { 0.0 };
                    AtomicScore::new(value, EVIDENCE_FIXTURE_JUDGE_CONFIDENCE)
                };
                CandidateJudgment {
                    id: candidate.id.to_string(),
                    relevance: score(label.is_some_and(|label| label.relevance)),
                    useful_evidence: score(label.is_some_and(|label| label.useful_evidence)),
                    contradiction: score(label.is_some_and(|label| label.contradiction)),
                    instruction_like: score(label.is_some_and(|label| label.instruction_like)),
                }
            })
            .collect())
    }
}

/// Judge instrumentation for one evidence comparison run.
///
/// Wraps the caller's judge to count invocations and to time the provider work
/// each invocation added. The judge is owned (`Arc<dyn RetrievalJudge>`),
/// mirroring `CascadeRetriever::with_judge`, because the cascade stores judges
/// behind `Arc`.
struct TimedJudge {
    inner: Arc<dyn RetrievalJudge>,
    stats: Mutex<JudgeStats>,
}

#[derive(Default)]
struct JudgeStats {
    /// Number of judge invocations so far.
    calls: usize,
    /// Microseconds spent inside the judge so far.
    elapsed_us: u64,
}

impl TimedJudge {
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

    /// `(judge calls, judge microseconds)` observed so far.
    fn totals(&self) -> (usize, u64) {
        let stats = self.stats();
        (stats.calls, stats.elapsed_us)
    }
}

impl RetrievalJudge for TimedJudge {
    fn judge_candidates(
        &self,
        query: &str,
        candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
        let start = Instant::now();
        let result = self.inner.judge_candidates(query, candidates);
        let elapsed_us = start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;

        // Recorded even when the provider failed: the attempt consumed the
        // shortlist, and `judge_calls_per_query` must not hide it.
        {
            let mut stats = self.stats();
            stats.calls += 1;
            stats.elapsed_us += elapsed_us;
        }

        result
    }
}

/// Judge totals captured before one evidence retrieval execution.
pub(super) struct EvidenceStart {
    calls: usize,
    elapsed_us: u64,
}

/// Accumulated classification outcomes for one evidence comparison run.
#[derive(Default)]
struct EvidenceConfusion {
    /// True/false positives/negatives per dimension, in `EVIDENCE_DIMENSIONS` order.
    tp: [u64; 4],
    fp: [u64; 4],
    fn_: [u64; 4],
    tn: [u64; 4],
    /// Classified candidates per disposition, in `EvidenceDisposition` rank order.
    dispositions: [u64; 4],
    /// Labelled candidates expected to be kept that were dropped.
    false_drops: u64,
    /// Classified candidates that carry a label for that query.
    labelled: u64,
}

impl EvidenceConfusion {
    /// Fold one query's classified hits against that query's labels.
    ///
    /// Only hits that carry evidence are counted: a candidate the evidence stage
    /// never judged (or a cascade that never ran) contributes nothing. A hit
    /// without a label for its query counts towards the disposition distribution
    /// but not towards the per-dimension metrics, the labelled denominator, or
    /// the false-drop count.
    fn record(
        &mut self,
        labels: &HashMap<String, EvidenceFixtureLabel>,
        result: Option<&EvidenceRetrievalResult>,
        policy: &EvidencePolicy,
    ) {
        let Some(result) = result else {
            return;
        };

        for hit in &result.hits {
            let Some(evidence) = hit.evidence.as_ref() else {
                continue;
            };
            self.dispositions[evidence.disposition.rank() as usize] += 1;

            let Some(label) = labels.get(&hit.episode_id) else {
                continue;
            };
            self.labelled += 1;

            let predicted = predicted_dimensions(evidence, policy);
            let expected = [
                label.relevance,
                label.useful_evidence,
                label.contradiction,
                label.instruction_like,
            ];
            for (dimension, predicted) in predicted.iter().enumerate() {
                match (*predicted, expected[dimension]) {
                    (true, true) => self.tp[dimension] += 1,
                    (true, false) => self.fp[dimension] += 1,
                    (false, true) => self.fn_[dimension] += 1,
                    (false, false) => self.tn[dimension] += 1,
                }
            }

            let expected_keep = label.expected_disposition == "keep";
            if expected_keep && evidence.disposition == EvidenceDisposition::Drop {
                self.false_drops += 1;
            }
        }
    }

    /// Turn the accumulated counts into the reported metrics block.
    fn metrics(
        &self,
        judge_calls: usize,
        total_queries: usize,
        added_latency_p50_us: u64,
        added_latency_p95_us: u64,
    ) -> EvidenceMetrics {
        EvidenceMetrics {
            per_dimension: std::array::from_fn(|dimension| {
                ClassificationMetrics::from_counts(
                    self.tp[dimension],
                    self.fp[dimension],
                    self.fn_[dimension],
                    self.tn[dimension],
                )
            }),
            dispositions: DispositionDistribution::from_counts(self.dispositions),
            false_drop_count: self.false_drops,
            false_drop_rate: count_ratio(self.false_drops, self.labelled),
            judge_calls_per_query: count_ratio(judge_calls as u64, total_queries as u64),
            added_latency_p50_us,
            added_latency_p95_us,
        }
    }
}

/// Judged dimensions binarized at their policy thresholds.
fn predicted_dimensions(evidence: &CandidateEvidence, policy: &EvidencePolicy) -> [bool; 4] {
    [
        evidence.relevance.value >= policy.min_relevance,
        evidence.useful_evidence.value >= policy.min_useful_evidence,
        evidence.contradiction.value >= policy.contradiction_flag_threshold,
        evidence.instruction_like.value >= policy.instruction_flag_threshold,
    ]
}

/// Judge plus policy for one evidence comparison run.
pub(super) struct EvidenceProbe {
    /// Handle installed on the cascade (`Arc<dyn RetrievalJudge>`, as
    /// `CascadeRetriever::with_judge` stores).
    judge: Arc<dyn RetrievalJudge>,
    /// Instrumented view of the same wrapper, used for calls and latency.
    timed: Arc<TimedJudge>,
    /// Policy handed to `CascadeRetriever::with_evidence_policy`.
    policy: EvidencePolicy,
    /// Classification outcomes accumulated over the run.
    confusion: EvidenceConfusion,
    /// Per-query judge microseconds, one entry per evaluated query.
    added_latency_us: Vec<u64>,
    /// Judge invocations over the run.
    judge_calls: usize,
}

impl EvidenceProbe {
    pub(super) fn new(judge: Arc<dyn RetrievalJudge>, policy: &EvidencePolicy) -> Self {
        let timed: Arc<TimedJudge> = Arc::new(TimedJudge::new(judge));
        // Bind the concrete handle before coercing: annotating `Arc::clone`
        // with the trait-object type makes the clone infer `dyn RetrievalJudge`
        // as its parameter and reject the concrete argument.
        let timed_handle = Arc::clone(&timed);
        let judge_handle: Arc<dyn RetrievalJudge> = timed_handle;
        Self {
            judge: judge_handle,
            timed,
            policy: policy.clone(),
            confusion: EvidenceConfusion::default(),
            added_latency_us: Vec::new(),
            judge_calls: 0,
        }
    }

    /// Judge handle to install on the cascade.
    pub(super) fn judge_handle(&self) -> Arc<dyn RetrievalJudge> {
        Arc::clone(&self.judge)
    }

    /// Policy to validate and install on the cascade.
    pub(super) fn policy(&self) -> &EvidencePolicy {
        &self.policy
    }

    /// Capture judge totals before one retrieval execution.
    pub(super) fn begin_query(&self) -> EvidenceStart {
        let (calls, elapsed_us) = self.timed.totals();
        EvidenceStart { calls, elapsed_us }
    }

    /// Fold one retrieval execution's judge work and classification into the run.
    ///
    /// `result` is `None` when the cascade failed, which records a query that
    /// reached no judge and classified nothing instead of skipping the query.
    pub(super) fn record(
        &mut self,
        start: EvidenceStart,
        labels: &HashMap<String, EvidenceFixtureLabel>,
        result: Option<&EvidenceRetrievalResult>,
    ) {
        let (calls, elapsed_us) = self.timed.totals();
        self.judge_calls += calls.saturating_sub(start.calls);
        self.added_latency_us
            .push(elapsed_us.saturating_sub(start.elapsed_us));
        self.confusion.record(labels, result, &self.policy);
    }

    /// Metrics block reported for this run.
    ///
    /// `total_queries` is the number of evaluated queries; the per-query rates
    /// are zero-safe, so an empty corpus yields zeros rather than a panic.
    pub(super) fn metrics(&self, total_queries: usize) -> EvidenceMetrics {
        let latency = compute_percentiles(&self.added_latency_us);
        self.confusion.metrics(
            self.judge_calls,
            total_queries,
            latency.p50_us,
            latency.p95_us,
        )
    }
}

/// One query's retrieval outcome for the two execution modes the loop drives.
pub(super) enum QueryOutcome {
    /// Plain retrieval: the evidence stage never ran.
    Plain(Result<CascadeResult, CascadeError>),
    /// Evidence-aware retrieval: classification ran alongside the cascade.
    Evidence(Result<EvidenceRetrievalResult, CascadeError>),
}

impl QueryOutcome {
    /// Cascade view: the plain result, or the evidence result's unchanged base.
    pub(super) fn cascade(&self) -> Result<&CascadeResult, &CascadeError> {
        match self {
            Self::Plain(result) => result.as_ref(),
            Self::Evidence(result) => result.as_ref().map(|evidence| &evidence.base),
        }
    }

    /// Classified evidence result, `None` for plain retrieval or a failed cascade.
    pub(super) fn evidence(&self) -> Option<&EvidenceRetrievalResult> {
        match self {
            Self::Plain(_) => None,
            Self::Evidence(result) => result.as_ref().ok(),
        }
    }
}

impl RetrievalEvaluator {
    /// Run evaluation for a strategy with the evidence-classification stage enabled.
    ///
    /// This is the comparison arm of issue #1032: the same per-query loop as
    /// [`RetrievalEvaluator::evaluate_strategy`], with `judge` attached to the
    /// retriever and `policy` validated and installed by
    /// `CascadeRetriever::with_evidence_policy`. The arm reuses
    /// [`RetrievalStrategy::LocalOnly`] in the CLI, so it stays CPU-local and
    /// needs no credentials.
    ///
    /// Retrieval runs **once per query**; the metrics are captured from that one
    /// execution:
    ///
    /// - judge calls and added latency are counted and timed by the
    ///   instrumentation wrapper around `judge`, so
    ///   [`EvidenceMetrics::judge_calls_per_query`] is `1.0` for every query the
    ///   stage judged and [`EvidenceMetrics::added_latency_p50_us`] is the
    ///   median per-query provider time the stage added;
    /// - per-dimension counts compare the judged values binarized at the policy
    ///   thresholds against the query's labels;
    /// - disposition counts cover every classified candidate, while the
    ///   per-dimension metrics and [`EvidenceMetrics::false_drop_count`] cover
    ///   only labelled candidates.
    ///
    /// The default policy sets `allow_drop = false`, so the arm cannot remove a
    /// local candidate by accident and any false drop it reports is a real
    /// classification defect rather than a policy choice.
    ///
    /// # Errors
    ///
    /// Fails only when the corpus has no queries, when the corpus cannot be
    /// indexed, or when `policy` is invalid — never because of a judge failure,
    /// which the evidence stage absorbs into the unjudged fallback.
    pub fn evaluate_with_evidence(
        &self,
        strategy: RetrievalStrategy,
        judge: Arc<dyn RetrievalJudge>,
        policy: &EvidencePolicy,
    ) -> anyhow::Result<BenchmarkMetrics> {
        self.run_evaluation(strategy, None, Some(EvidenceProbe::new(judge, policy)))
    }
}

#[cfg(test)]
mod tests;
