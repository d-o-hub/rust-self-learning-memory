//! Tests for the provider-neutral semantic judgment interface.

use super::*;
use proptest::prelude::*;
use serial_test::serial;
use std::sync::atomic::{AtomicUsize, Ordering};

struct FakeJudge {
    call_count: AtomicUsize,
    response: Result<Vec<CandidateJudgment>, JudgmentError>,
}

impl FakeJudge {
    fn new(response: Result<Vec<CandidateJudgment>, JudgmentError>) -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            response,
        }
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }
}

impl RetrievalJudge for FakeJudge {
    fn judge_candidates(
        &self,
        _query: &str,
        _candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
        self.call_count.fetch_add(1, Ordering::Relaxed);
        match &self.response {
            Ok(j) => Ok(j.clone()),
            Err(e) => Err(match e {
                JudgmentError::Unavailable => JudgmentError::Unavailable,
                JudgmentError::Timeout => JudgmentError::Timeout,
                JudgmentError::Invalid(s) => JudgmentError::Invalid(s.clone()),
                JudgmentError::Provider(s) => JudgmentError::Provider(s.clone()),
            }),
        }
    }
}

fn sample_judgment(id: &str, relevance: f32) -> CandidateJudgment {
    CandidateJudgment {
        id: id.to_string(),
        relevance: AtomicScore::new(relevance, 0.9),
        useful_evidence: AtomicScore::new(0.8, 0.9),
        contradiction: AtomicScore::new(0.1, 0.9),
        instruction_like: AtomicScore::new(0.0, 0.9),
    }
}

#[test]
fn test_no_judge_configured_unaffected() {
    let candidates = [
        JudgmentCandidate::new("ep-1", "Authentication with JWT", 0.8),
        JudgmentCandidate::new("ep-2", "Database pool setup", 0.6),
    ];

    let result = evaluate_judgments(None, "auth query", &candidates);
    assert_eq!(result, Ok(None));
}

#[test]
fn test_empty_candidate_batch_zero_provider_work() {
    let judge = FakeJudge::new(Ok(vec![]));
    let result = evaluate_judgments(Some(&judge), "query", &[]);

    assert_eq!(result, Ok(Some(vec![])));
    assert_eq!(
        judge.calls(),
        0,
        "empty batch must perform zero provider calls"
    );
}

#[test]
fn test_valid_batch_shuffled_ids_aligned_correctly() {
    let candidates = [
        JudgmentCandidate::new("ep-1", "Text 1", 0.9),
        JudgmentCandidate::new("ep-2", "Text 2", 0.7),
        JudgmentCandidate::new("ep-3", "Text 3", 0.5),
    ];

    // Provider returns shuffled order: ep-3, ep-1, ep-2
    let raw_judgments = vec![
        sample_judgment("ep-3", 0.3),
        sample_judgment("ep-1", 0.9),
        sample_judgment("ep-2", 0.7),
    ];

    let judge = FakeJudge::new(Ok(raw_judgments));
    let res = evaluate_judgments(Some(&judge), "query", &candidates)
        .expect("evaluation should succeed")
        .expect("should return judgments");

    assert_eq!(res.len(), 3);
    assert_eq!(res[0].id, "ep-1");
    assert_eq!(res[1].id, "ep-2");
    assert_eq!(res[2].id, "ep-3");
}

#[test]
fn test_duplicate_result_id_rejected() {
    let candidates = [
        JudgmentCandidate::new("ep-1", "Text 1", 0.9),
        JudgmentCandidate::new("ep-2", "Text 2", 0.7),
    ];

    let raw_judgments = vec![
        sample_judgment("ep-1", 0.9),
        sample_judgment("ep-1", 0.8), // Duplicate ep-1
    ];

    let judge = FakeJudge::new(Ok(raw_judgments));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_missing_result_id_rejected() {
    let candidates = [
        JudgmentCandidate::new("ep-1", "Text 1", 0.9),
        JudgmentCandidate::new("ep-2", "Text 2", 0.7),
    ];

    // Length matches, but ep-2 is missing (replaced by ep-1 duplicate/unknown)
    let raw_judgments = vec![
        sample_judgment("ep-1", 0.9),
        sample_judgment("ep-3", 0.8), // Unknown ID
    ];

    let judge = FakeJudge::new(Ok(raw_judgments));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_unknown_result_id_rejected() {
    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];

    let raw_judgments = vec![sample_judgment("ep-999", 0.9)];

    let judge = FakeJudge::new(Ok(raw_judgments));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_negative_score_rejected() {
    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];

    let mut judgment = sample_judgment("ep-1", -0.1);
    judgment.relevance.value = -0.1;

    let judge = FakeJudge::new(Ok(vec![judgment]));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_score_greater_than_one_rejected() {
    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];

    let mut judgment = sample_judgment("ep-1", 1.5);
    judgment.relevance.value = 1.5;

    let judge = FakeJudge::new(Ok(vec![judgment]));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_nan_or_infinite_score_rejected() {
    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];

    let mut nan_j = sample_judgment("ep-1", 0.5);
    nan_j.relevance.value = f32::NAN;

    let judge_nan = FakeJudge::new(Ok(vec![nan_j]));
    assert!(matches!(
        evaluate_judgments(Some(&judge_nan), "query", &candidates),
        Err(JudgmentError::Invalid(_))
    ));

    let mut inf_j = sample_judgment("ep-1", 0.5);
    inf_j.relevance.confidence = f32::INFINITY;

    let judge_inf = FakeJudge::new(Ok(vec![inf_j]));
    assert!(matches!(
        evaluate_judgments(Some(&judge_inf), "query", &candidates),
        Err(JudgmentError::Invalid(_))
    ));
}

#[test]
fn test_provider_unavailable_error_propagated() {
    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];

    let judge = FakeJudge::new(Err(JudgmentError::Unavailable));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert_eq!(res, Err(JudgmentError::Unavailable));
}

#[test]
fn test_candidate_count_mismatch_rejected() {
    let candidates = [
        JudgmentCandidate::new("ep-1", "Text 1", 0.9),
        JudgmentCandidate::new("ep-2", "Text 2", 0.7),
    ];

    let judge = FakeJudge::new(Ok(vec![sample_judgment("ep-1", 0.9)]));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_provider_timeout_and_failure_outcomes_propagated() {
    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];

    let judge = FakeJudge::new(Err(JudgmentError::Timeout));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert_eq!(res, Err(JudgmentError::Timeout));

    let judge = FakeJudge::new(Err(JudgmentError::Provider("boom".to_string())));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert_eq!(res, Err(JudgmentError::Provider("boom".to_string())));
}

#[test]
fn test_alignment_impossible_when_candidate_replaced_by_duplicate() {
    // Same length, but ep-2 is missing while ep-1 is duplicated: the ID-set
    // check rejects this before alignment is attempted.
    let candidates = [
        JudgmentCandidate::new("ep-1", "Text 1", 0.9),
        JudgmentCandidate::new("ep-2", "Text 2", 0.7),
    ];

    let raw_judgments = vec![sample_judgment("ep-1", 0.9), sample_judgment("ep-1", 0.5)];

    let judge = FakeJudge::new(Ok(raw_judgments));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Invalid(_))));
}

#[test]
fn test_provider_error_telemetry_outcome_recorded() {
    let metrics = global_retrieval_metrics();
    let before = metrics.snapshot()["judgments"]["judge_configured=true:outcome=provider_error"]
        .as_u64()
        .unwrap_or(0);

    let candidates = [JudgmentCandidate::new("ep-1", "Text 1", 0.9)];
    let judge = FakeJudge::new(Err(JudgmentError::Provider("boom".to_string())));
    let res = evaluate_judgments(Some(&judge), "query", &candidates);
    assert!(matches!(res, Err(JudgmentError::Provider(_))));

    let after = metrics.snapshot()["judgments"]["judge_configured=true:outcome=provider_error"]
        .as_u64()
        .unwrap_or(0);
    assert_eq!(
        after,
        before + 1,
        "provider-error outcome must be recorded exactly once"
    );
}

#[test]
fn test_metrics_no_sensitive_labels() {
    let metrics = global_retrieval_metrics();
    metrics.reset();

    let sensitive_query = "SECRET_USER_QUERY_DATA_12345";
    let sensitive_text = "CONFIDENTIAL_CANDIDATE_TEXT_99999";
    let sensitive_id = "EPISODE_ID_SECRET_XYZ";

    let candidates = [JudgmentCandidate::new(sensitive_id, sensitive_text, 0.9)];
    let judge = FakeJudge::new(Ok(vec![sample_judgment(sensitive_id, 0.95)]));

    let _ = evaluate_judgments(Some(&judge), sensitive_query, &candidates);

    let snapshot_str = metrics.snapshot().to_string();
    let prometheus_str = metrics.export_prometheus();

    assert!(!snapshot_str.contains(sensitive_query));
    assert!(!snapshot_str.contains(sensitive_text));
    assert!(!snapshot_str.contains(sensitive_id));

    assert!(!prometheus_str.contains(sensitive_query));
    assert!(!prometheus_str.contains(sensitive_text));
    assert!(!prometheus_str.contains(sensitive_id));
}

proptest! {
    #[test]
    fn proptest_atomic_score_validation(val in -2.0f32..3.0f32, conf in -2.0f32..3.0f32) {
        let score = AtomicScore::new(val, conf);
        let expected = val.is_finite() && (0.0..=1.0).contains(&val)
            && conf.is_finite() && (0.0..=1.0).contains(&conf);
        prop_assert_eq!(score.is_valid(), expected);
    }
}
