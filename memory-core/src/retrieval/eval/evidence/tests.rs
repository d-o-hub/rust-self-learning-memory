//! Tests for the evidence-classification benchmark arm (issue #1032).

use std::collections::HashMap;
use std::sync::Arc;

use crate::retrieval::eval::*;
use crate::retrieval::evidence::{
    CandidateEvidence, EvidenceDisposition, EvidenceHit, EvidencePolicy, EvidenceRetrievalResult,
    classify_disposition,
};
use crate::retrieval::judgment::{
    AtomicScore, CandidateJudgment, JudgmentCandidate, RetrievalJudge,
};
use crate::retrieval::{CascadeResult, FallbackReason};

/// Query text shared by [`labelled_corpus`]'s only query.
const LABELLED_QUERY: &str = "alpha beta gamma retrieval pipeline";

fn item(id: &str, tail: &str) -> FixtureItem {
    FixtureItem {
        id: id.to_string(),
        text: format!("alpha beta gamma retrieval {tail}"),
        context: None,
        tags: Vec::new(),
        is_successful: Some(true),
        reward_score: Some(1.0),
    }
}

/// Builds a fixture label from the four dimension expectations
/// (`[relevance, useful_evidence, contradiction, instruction_like]`) and the
/// expected disposition.
fn label(dimensions: [bool; 4], expected_disposition: &str) -> EvidenceFixtureLabel {
    let [relevance, useful_evidence, contradiction, instruction_like] = dimensions;
    EvidenceFixtureLabel {
        relevance,
        useful_evidence,
        contradiction,
        instruction_like,
        expected_disposition: expected_disposition.to_string(),
    }
}

/// Every item shares most tokens with the query, so its BM25 shortlist holds all
/// three candidates and the evidence stage always reaches the judge. One
/// candidate is relevant, one contradicts the query, and one reads like an
/// instruction — with the fixture judge replaying exactly those labels.
fn labelled_corpus() -> FixtureCorpus {
    let labels = HashMap::from([
        (
            "item-relevant".to_string(),
            label([true, true, false, false], "keep"),
        ),
        (
            "item-contradictory".to_string(),
            label([false, false, true, false], "flag"),
        ),
        (
            "item-instruction".to_string(),
            label([true, true, false, true], "flag"),
        ),
    ]);

    FixtureCorpus {
        version: "1.0.0".to_string(),
        description: "Evidence classification corpus".to_string(),
        corpus: vec![
            item("item-relevant", "pipeline"),
            item("item-contradictory", "contradiction"),
            item("item-instruction", "instruction"),
        ],
        queries: vec![BenchmarkQuery {
            id: "q-evidence".to_string(),
            query: LABELLED_QUERY.to_string(),
            context: None,
            expected_ids: vec!["item-relevant".to_string()],
            tags: Vec::new(),
            expected_accepted_id: Some("item-relevant".to_string()),
            evidence_labels: labels,
        }],
    }
}

/// The fixture judge replays the corpus's labels verbatim and keeps ids and
/// order; an unlabelled candidate is still judged all-false at full confidence,
/// which the default policy turns into a recoverable demotion.
#[test]
fn test_fixture_judge_replays_labels_and_demotes_unlabelled() {
    let corpus = labelled_corpus();
    let judge = EvidenceFixtureJudge::from_corpus(&corpus);
    let candidates = [
        JudgmentCandidate::new("item-relevant", "text", 0.9),
        JudgmentCandidate::new("item-unlabelled", "text", 0.5),
    ];

    let judgments = judge
        .judge_candidates(LABELLED_QUERY, &candidates)
        .expect("the fixture judge never fails");

    assert_eq!(judgments.len(), candidates.len());
    assert_eq!(judgments[0].id, "item-relevant");
    assert_eq!(judgments[1].id, "item-unlabelled");
    assert!(judgments.iter().all(CandidateJudgment::is_valid));

    let expected = EVIDENCE_FIXTURE_JUDGE_CONFIDENCE;
    assert_eq!(judgments[0].relevance, AtomicScore::new(1.0, expected));
    assert_eq!(
        judgments[0].useful_evidence,
        AtomicScore::new(1.0, expected)
    );
    assert_eq!(
        judgments[0].instruction_like,
        AtomicScore::new(0.0, expected)
    );
    for score in [
        judgments[1].relevance,
        judgments[1].useful_evidence,
        judgments[1].contradiction,
        judgments[1].instruction_like,
    ] {
        assert_eq!(score, AtomicScore::new(0.0, expected));
    }

    let policy = EvidencePolicy::default();
    assert_eq!(
        classify_disposition(&judgments[0], &policy),
        EvidenceDisposition::Keep
    );
    assert_eq!(
        classify_disposition(&judgments[1], &policy),
        EvidenceDisposition::Demote,
        "an unlabelled candidate must be demoted, never dropped"
    );

    // An unknown query yields the same unlabelled default rather than an error.
    let unknown = judge
        .judge_candidates("no such query", &candidates)
        .expect("the fixture judge never fails");
    for judgment in &unknown {
        assert_eq!(judgment.relevance, AtomicScore::new(0.0, expected));
    }
}

/// Shipped fixtures and baselines predate the evidence metrics: both must keep
/// deserializing, with no labels and no metrics block.
#[test]
fn test_shipped_fixtures_and_baseline_load_without_evidence_metrics() {
    let corpus_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../benches/fixtures/retrieval_benchmark_corpus.json"
    );
    let corpus_json = std::fs::read_to_string(corpus_path)
        .unwrap_or_else(|e| panic!("corpus fixture must exist at {corpus_path}: {e}"));
    let corpus: FixtureCorpus =
        serde_json::from_str(&corpus_json).expect("corpus fixture must keep deserializing");
    assert!(!corpus.queries.is_empty());
    assert!(
        corpus.queries.iter().all(|q| q.evidence_labels.is_empty()),
        "shipped queries must stay unlabelled"
    );

    let baseline_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../benches/fixtures/retrieval_baseline.json"
    );
    let baseline_json = std::fs::read_to_string(baseline_path)
        .unwrap_or_else(|e| panic!("baseline fixture must exist at {baseline_path}: {e}"));
    let baseline: BenchmarkReport =
        serde_json::from_str(&baseline_json).expect("baseline JSON must keep deserializing");
    assert!(!baseline.strategies.is_empty());
    for (name, metrics) in &baseline.strategies {
        assert!(
            metrics.evidence_metrics.is_none(),
            "{name} must not gain an evidence block"
        );
    }
}

/// The evidence arm runs offline and its metrics block serializes with every
/// documented key, including the renamed `fn` counter.
#[test]
fn test_evidence_arm_serializes_metrics_block() {
    let corpus = labelled_corpus();
    let judge = EvidenceFixtureJudge::from_corpus(&corpus);
    let evaluator = RetrievalEvaluator::new(corpus);

    let metrics = evaluator
        .evaluate_with_evidence(
            RetrievalStrategy::LocalOnly,
            Arc::new(judge),
            &EvidencePolicy::default(),
        )
        .expect("the evidence arm must run offline");

    assert_eq!(metrics.strategy, RetrievalStrategy::LocalOnly);
    assert_eq!(EVIDENCE_COMPARISON_STRATEGY, "local_only+evidence");
    assert_eq!(metrics.total_queries, 1);

    let evidence = metrics
        .evidence_metrics
        .as_ref()
        .expect("the arm must report an evidence block");
    assert!(
        evidence.judge_calls_per_query.is_finite(),
        "judge calls per query must be a real rate"
    );
    assert_eq!(evidence.false_drop_count, 0);

    let json = serde_json::to_value(&metrics).expect("metrics must serialize");
    let block = json
        .get("evidence_metrics")
        .expect("the JSON report exposes the evidence block");
    for key in [
        "per_dimension",
        "dispositions",
        "false_drop_count",
        "false_drop_rate",
        "judge_calls_per_query",
        "added_latency_p50_us",
        "added_latency_p95_us",
    ] {
        assert!(block.get(key).is_some(), "missing evidence key {key}");
    }

    let dimensions = block["per_dimension"]
        .as_array()
        .expect("per_dimension is an array");
    assert_eq!(dimensions.len(), EVIDENCE_DIMENSIONS.len());
    for (index, dimension) in dimensions.iter().enumerate() {
        for key in ["tp", "fp", "fn", "tn", "precision", "recall"] {
            assert!(
                dimension.get(key).is_some(),
                "missing {key} for dimension {index}"
            );
        }
    }

    for key in ["keep", "flag", "demote", "drop", "total"] {
        assert!(block["dispositions"].get(key).is_some(), "missing {key}");
    }
}

/// Reports without the evidence arm stay unchanged, and the classification
/// section appears as soon as a strategy carries an evidence block.
#[test]
fn test_markdown_renders_evidence_section_only_when_present() {
    let evaluator = RetrievalEvaluator::new(labelled_corpus());
    let mut report = evaluator
        .evaluate_all()
        .expect("baseline evaluation should succeed");

    assert!(
        report
            .strategies
            .values()
            .all(|m| m.evidence_metrics.is_none())
    );
    let markdown = format_markdown_report(&report, None);
    assert!(
        !markdown.contains("Evidence Classification"),
        "a run without the evidence arm must not gain a classification section"
    );

    let evidence = EvidenceMetrics {
        per_dimension: std::array::from_fn(|_| ClassificationMetrics::from_counts(2, 0, 0, 4)),
        dispositions: DispositionDistribution::from_counts([3, 1, 0, 0]),
        false_drop_count: 0,
        false_drop_rate: 0.0,
        judge_calls_per_query: 1.0,
        added_latency_p50_us: 12,
        added_latency_p95_us: 30,
    };
    report
        .strategies
        .get_mut("local_only")
        .expect("local_only present")
        .evidence_metrics = Some(evidence);

    let markdown = format_markdown_report(&report, None);
    assert!(markdown.contains("## Evidence Classification"));
    for dimension in EVIDENCE_DIMENSIONS {
        assert!(
            markdown.contains(dimension),
            "missing dimension {dimension}"
        );
    }
    assert!(markdown.contains("1.000"), "precision is rendered");
    assert!(markdown.contains("False Drop Rate"));
    assert!(markdown.contains("Flag"));
    assert!(markdown.contains("Demote"));
}

/// A single false drop of a labelled keep candidate fails the regression check,
/// while a clean evidence block passes it.
#[test]
fn test_regression_check_is_blocked_by_false_drops() {
    let evaluator = RetrievalEvaluator::new(labelled_corpus());
    let baseline = evaluator.evaluate_all().expect("baseline evaluation");
    let checker = RegressionChecker::new(RegressionThresholds::default());

    let mut clean = baseline.clone();
    let clean_local = clean
        .strategies
        .get_mut("local_only")
        .expect("local_only present");
    clean_local.evidence_metrics = Some(EvidenceMetrics::default());
    assert!(
        checker.check(&clean, &baseline).passed,
        "an evidence block without false drops must not fail the check"
    );

    let mut dropped = baseline.clone();
    let dropped_local = dropped
        .strategies
        .get_mut("local_only")
        .expect("local_only present");
    dropped_local.evidence_metrics = Some(EvidenceMetrics {
        false_drop_count: 1,
        false_drop_rate: 0.25,
        ..EvidenceMetrics::default()
    });

    let result = checker.check(&dropped, &baseline);
    assert!(
        !result.passed,
        "a false drop must fail the regression check"
    );
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.contains("false-dropped")),
        "the violation must name the false drop: {:?}",
        result.violations
    );

    // A comparison-only arm the baseline does not track is exempt from the
    // "missing from baseline" violation but never from the false-drop gate.
    let mut arm = baseline.strategies["local_only"].clone();
    arm.evidence_metrics = Some(EvidenceMetrics {
        false_drop_count: 2,
        false_drop_rate: 0.5,
        ..EvidenceMetrics::default()
    });
    let mut arm_only = baseline.clone();
    arm_only
        .strategies
        .insert(EVIDENCE_COMPARISON_STRATEGY.to_string(), arm);

    let ignoring = checker.check_ignoring(&arm_only, &baseline, &[EVIDENCE_COMPARISON_STRATEGY]);
    assert!(
        !ignoring.passed,
        "a false drop in the comparison-only arm must still fail the check"
    );
    let names_arm = ignoring
        .violations
        .iter()
        .any(|violation| violation.contains(EVIDENCE_COMPARISON_STRATEGY));
    assert!(
        names_arm,
        "the violation must name the arm: {:?}",
        ignoring.violations
    );
}

/// The arm itself must run on default features: without `csm` the cascade
/// reports `CapabilityUnavailable`, the loop falls back to keyword search, no
/// judge is reached, and the metrics block is still produced and bounded.
#[test]
fn test_evidence_arm_runs_without_csm_and_reports_bounded_metrics() {
    let corpus = labelled_corpus();
    let judge = Arc::new(EvidenceFixtureJudge::from_corpus(&corpus));
    let evaluator = RetrievalEvaluator::new(corpus);

    let metrics = evaluator
        .evaluate_with_evidence(
            RetrievalStrategy::LocalOnly,
            judge,
            &EvidencePolicy::default(),
        )
        .expect("the evidence arm must run offline");

    let evidence = metrics
        .evidence_metrics
        .expect("the arm always reports an evidence metrics block");
    assert!(
        (0.0..=1.0).contains(&evidence.judge_calls_per_query),
        "judge calls/query must stay bounded: {}",
        evidence.judge_calls_per_query
    );
    assert_eq!(
        evidence.false_drop_count, 0,
        "the default policy cannot drop a candidate, so a false drop is a defect"
    );
    assert_eq!(evidence.false_drop_rate, 0.0);
    assert!(
        evidence.dispositions.total <= metrics.total_queries as u64 * 20,
        "classified candidates per query must stay bounded by candidate_limit"
    );
}

/// Folding labelled hits into the confusion matrix is pure and must not need
/// `csm`: dimension counts, the disposition distribution, and false drops are
/// asserted exactly, including the unlabelled-hit path.
#[test]
fn test_confusion_counts_dimensions_dispositions_and_false_drops() {
    use super::EvidenceConfusion;

    let keep = label([true, true, false, false], "keep");
    let dropped = label([true, false, false, false], "keep");
    // Labelled negative on every dimension: the judge scores it high, which is
    // a false positive for `relevance` and `useful_evidence`.
    let negative = label([false, false, false, false], "demote");
    let labels = HashMap::from([
        ("hit-dropped".to_string(), dropped.clone()),
        ("hit-labelled".to_string(), keep.clone()),
        ("hit-negative".to_string(), negative.clone()),
    ]);

    let policy = EvidencePolicy {
        allow_drop: true,
        ..EvidencePolicy::default()
    };

    let evidence_for = |relevance: f32, disposition: EvidenceDisposition| {
        let judgment = CandidateJudgment {
            id: "x".to_string(),
            relevance: AtomicScore::new(relevance, 1.0),
            useful_evidence: AtomicScore::new(0.0, 1.0),
            contradiction: AtomicScore::new(0.0, 1.0),
            instruction_like: AtomicScore::new(0.0, 1.0),
        };
        CandidateEvidence::from_judgment(&judgment, disposition)
    };

    let result = EvidenceRetrievalResult {
        base: CascadeResult {
            episode_ids: vec!["hit-labelled".to_string(), "hit-dropped".to_string()],
            scores: vec![1.0, 0.5],
            contributing_tiers: vec!["bm25".to_string()],
            api_calls: 0,
            fallback_reason: FallbackReason::LocalTierSufficient,
            top_score: 1.0,
            score_margin: 0.5,
        },
        hits: vec![
            EvidenceHit {
                episode_id: "hit-labelled".to_string(),
                local_score: 1.0,
                final_score: 1.0,
                evidence: Some(evidence_for(0.9, EvidenceDisposition::Keep)),
            },
            EvidenceHit {
                episode_id: "hit-dropped".to_string(),
                local_score: 0.5,
                final_score: 0.5,
                evidence: Some(evidence_for(0.1, EvidenceDisposition::Drop)),
            },
            EvidenceHit {
                episode_id: "hit-negative".to_string(),
                local_score: 0.45,
                final_score: 0.45,
                evidence: Some(CandidateEvidence::from_judgment(
                    &CandidateJudgment {
                        id: "hit-negative".to_string(),
                        relevance: AtomicScore::new(0.9, 1.0),
                        useful_evidence: AtomicScore::new(0.9, 1.0),
                        contradiction: AtomicScore::new(0.0, 1.0),
                        instruction_like: AtomicScore::new(0.0, 1.0),
                    },
                    EvidenceDisposition::Demote,
                )),
            },
            EvidenceHit {
                episode_id: "hit-unlabelled".to_string(),
                local_score: 0.4,
                final_score: 0.4,
                evidence: Some(evidence_for(0.9, EvidenceDisposition::Flag)),
            },
            EvidenceHit {
                episode_id: "hit-no-evidence".to_string(),
                local_score: 0.3,
                final_score: 0.3,
                evidence: None,
            },
        ],
        status: crate::monitoring::metrics::EvidenceStatus::Applied,
    };

    let mut confusion = EvidenceConfusion::default();
    confusion.record(&labels, Some(&result), &policy);
    let metrics = confusion.metrics(2, 4, 7, 9);

    // One keep + one flag + one demote + one drop; the evidence-less hit is not
    // counted, while the unlabelled flag still counts towards the distribution.
    assert_eq!(metrics.dispositions.keep, 1);
    assert_eq!(metrics.dispositions.flag, 1);
    assert_eq!(metrics.dispositions.demote, 1);
    assert_eq!(metrics.dispositions.drop, 1);
    assert_eq!(metrics.dispositions.total, 4);

    // Only the three labelled hits enter the per-dimension matrix.
    let relevance = metrics.per_dimension[0].clone();
    assert_eq!(
        (relevance.tp, relevance.fp, relevance.fn_, relevance.tn),
        (1, 1, 1, 0),
        "keep hit true positive, negative hit false positive, dropped hit false negative"
    );
    let useful = metrics.per_dimension[1].clone();
    assert_eq!((useful.tp, useful.fp, useful.fn_, useful.tn), (0, 1, 1, 1));

    // `hit-dropped` was labelled "keep" and dropped: a real false drop.
    assert_eq!(metrics.false_drop_count, 1);
    assert_eq!(metrics.false_drop_rate, 1.0 / 3.0);
    assert_eq!(metrics.judge_calls_per_query, 0.5);
    assert_eq!(
        (metrics.added_latency_p50_us, metrics.added_latency_p95_us),
        (7, 9)
    );
}

/// The timing wrapper must record provider attempts, successful or not:
/// `judge_calls_per_query` counts attempts and latency accumulates from them.
#[test]
fn test_timed_judge_records_attempts_and_latency() {
    use super::TimedJudge;

    let labels = HashMap::from([(
        "q".to_string(),
        HashMap::from([("c".to_string(), label([true, true, false, false], "keep"))]),
    )]);
    let judge: Arc<dyn RetrievalJudge> = Arc::new(EvidenceFixtureJudge::new(labels));
    let timed = TimedJudge::new(judge);
    let candidates = vec![JudgmentCandidate::new("c", "alpha", 0.5)];

    assert!(timed.judge_candidates("q", &candidates).is_ok());
    assert_eq!(timed.totals().0, 1, "a judged call is counted once");

    assert!(timed.judge_candidates("q", &candidates).is_ok());
    assert_eq!(timed.totals().0, 2, "every provider attempt accumulates");
}

/// Under the default policy the arm classifies every query exactly once and
/// never drops a labelled keep candidate.
#[cfg(feature = "csm")]
#[test]
fn test_evidence_arm_classifies_once_per_query_without_false_drops() {
    let corpus = labelled_corpus();
    let judge = EvidenceFixtureJudge::from_corpus(&corpus);
    let evaluator = RetrievalEvaluator::new(corpus);

    let metrics = evaluator
        .evaluate_with_evidence(
            RetrievalStrategy::LocalOnly,
            Arc::new(judge),
            &EvidencePolicy::default(),
        )
        .expect("the evidence arm must run against the csm cascade");

    assert_eq!(
        metrics.embedding_calls_per_query, 0.0,
        "the evidence arm stays local: no Tier 4 calls"
    );
    assert!(
        metrics.recall_at_5 > 0.0,
        "classification must not destroy local retrieval quality"
    );

    let evidence = metrics
        .evidence_metrics
        .as_ref()
        .expect("the arm must report an evidence block");
    assert_eq!(
        evidence.judge_calls_per_query, 1.0,
        "the evidence stage judges each query exactly once"
    );
    assert_eq!(
        evidence.false_drop_count, 0,
        "the default policy never drops a labelled keep candidate"
    );
    assert_eq!(evidence.false_drop_rate, 0.0);
    assert_eq!(
        evidence.dispositions.drop, 0,
        "allow_drop is unset, so nothing may be dropped"
    );
    assert!(
        evidence.dispositions.keep >= 1 && evidence.dispositions.flag >= 1,
        "the labelled keep and flag candidates must both be classified: {:?}",
        evidence.dispositions
    );

    for (index, dimension) in evidence.per_dimension.iter().enumerate() {
        assert_eq!(
            (dimension.fp, dimension.fn_),
            (0, 0),
            "dimension {index} must reproduce its labels"
        );
        assert!(
            dimension.tp + dimension.tn > 0,
            "dimension {index} must classify at least one labelled candidate"
        );
    }
}
