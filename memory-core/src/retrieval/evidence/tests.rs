//! Deterministic unit tests for evidence classification.
//!
//! Every test builds judgments directly (no provider, no I/O), so each policy
//! branch is pinned to exact inputs and the `allow_drop` invariant is asserted
//! over a table rather than a single example.

use super::*;

/// An atomic score at full confidence.
const fn full(value: f32) -> AtomicScore {
    AtomicScore::new(value, 1.0)
}

/// A judgment whose four dimensions are all fully confident.
fn judgment(
    relevance: f32,
    useful_evidence: f32,
    contradiction: f32,
    instruction_like: f32,
) -> CandidateJudgment {
    CandidateJudgment {
        id: "ep-1".to_string(),
        relevance: full(relevance),
        useful_evidence: full(useful_evidence),
        contradiction: full(contradiction),
        instruction_like: full(instruction_like),
    }
}

/// A judgment with one shared confidence applied to all four dimensions.
fn judgment_with_confidence(
    confidence: f32,
    relevance: f32,
    useful_evidence: f32,
    contradiction: f32,
    instruction_like: f32,
) -> CandidateJudgment {
    let score = |value| AtomicScore::new(value, confidence);
    CandidateJudgment {
        id: "ep-1".to_string(),
        relevance: score(relevance),
        useful_evidence: score(useful_evidence),
        contradiction: score(contradiction),
        instruction_like: score(instruction_like),
    }
}

/// A copy of the default policy with one field mutated.
fn policy_with(mutate: impl FnOnce(&mut EvidencePolicy)) -> EvidencePolicy {
    let mut policy = EvidencePolicy::default();
    mutate(&mut policy);
    policy
}

#[test]
fn default_policy_matches_frozen_contract() {
    let policy = EvidencePolicy::default();

    assert_eq!(policy.min_confidence, 0.70);
    assert_eq!(policy.min_relevance, 0.50);
    assert_eq!(policy.min_useful_evidence, 0.50);
    assert_eq!(policy.contradiction_flag_threshold, 0.70);
    assert_eq!(policy.instruction_flag_threshold, 0.70);
    assert!(!policy.allow_drop);
    assert_eq!(policy.candidate_limit, 20);
    assert_eq!(policy.validated(), Ok(policy));
}

#[test]
fn healthy_judgment_is_kept() {
    let policy = EvidencePolicy::default();

    assert_eq!(
        classify_disposition(&judgment(0.90, 0.90, 0.10, 0.10), &policy),
        EvidenceDisposition::Keep
    );
}

#[test]
fn instruction_like_judgment_is_flagged() {
    let policy = EvidencePolicy::default();

    assert_eq!(
        classify_disposition(&judgment(0.95, 0.95, 0.0, 0.90), &policy),
        EvidenceDisposition::Flag
    );
}

#[test]
fn contradictory_judgment_is_flagged() {
    let policy = EvidencePolicy::default();

    assert_eq!(
        classify_disposition(&judgment(0.95, 0.95, 0.85, 0.0), &policy),
        EvidenceDisposition::Flag
    );
}

#[test]
fn instruction_flag_precedes_contradiction_and_drop() {
    let policy = policy_with(|p| p.allow_drop = true);
    // Low relevance would drop, and contradiction would flag, but the
    // instruction test is first: injected text is never dropped.
    let judgment = judgment(0.05, 0.05, 0.99, 0.99);

    assert_eq!(
        classify_disposition(&judgment, &policy),
        EvidenceDisposition::Flag
    );
}

#[test]
fn low_relevance_is_demoted_by_default() {
    let policy = EvidencePolicy::default();

    assert_eq!(
        classify_disposition(&judgment(0.20, 0.90, 0.10, 0.10), &policy),
        EvidenceDisposition::Demote
    );
}

#[test]
fn low_useful_evidence_is_demoted() {
    let policy = EvidencePolicy::default();

    assert_eq!(
        classify_disposition(&judgment(0.90, 0.20, 0.10, 0.10), &policy),
        EvidenceDisposition::Demote
    );
}

#[test]
fn low_relevance_drops_only_when_allowed() {
    let allowed = policy_with(|p| p.allow_drop = true);
    let judgment = judgment(0.20, 0.90, 0.10, 0.10);

    assert_eq!(
        classify_disposition(&judgment, &allowed),
        EvidenceDisposition::Drop
    );
}

#[test]
fn low_useful_evidence_is_demoted_even_when_drops_are_allowed() {
    // Only low relevance is decisive enough to drop; low usefulness stays a demotion.
    let allowed = policy_with(|p| p.allow_drop = true);

    assert_eq!(
        classify_disposition(&judgment(0.90, 0.20, 0.10, 0.10), &allowed),
        EvidenceDisposition::Demote
    );
}

#[test]
fn low_confidence_keeps_with_evidence_attached() {
    let policy = EvidencePolicy::default();
    // Every dimension is below the demote/drop thresholds, but confidence is
    // too low to act on, so the candidate is kept and the judgment is attached.
    let judgment = judgment_with_confidence(0.10, 0.05, 0.05, 0.05, 0.05);

    let disposition = classify_disposition(&judgment, &policy);
    assert_eq!(disposition, EvidenceDisposition::Keep);

    let evidence = CandidateEvidence::from_judgment(&judgment, disposition);
    assert_eq!(evidence.disposition, EvidenceDisposition::Keep);
    assert_eq!(evidence.relevance, judgment.relevance);
    assert_eq!(evidence.useful_evidence, judgment.useful_evidence);
    assert_eq!(evidence.contradiction, judgment.contradiction);
    assert_eq!(evidence.instruction_like, judgment.instruction_like);
}

#[test]
fn drop_never_returned_when_drop_disallowed() {
    let policies = [
        EvidencePolicy::default(),
        policy_with(|p| {
            p.allow_drop = false;
            p.min_confidence = 0.0;
            p.min_relevance = 1.0;
            p.min_useful_evidence = 1.0;
        }),
    ];
    let table = [
        judgment(0.0, 0.0, 1.0, 1.0),
        judgment(0.0, 0.0, 0.0, 0.0),
        judgment(0.99, 0.99, 0.99, 0.0),
        judgment(0.49, 0.49, 0.0, 0.0),
        judgment(0.50, 1.0, 0.0, 0.0),
        judgment_with_confidence(0.0, 0.0, 0.0, 0.0, 0.0),
        judgment_with_confidence(1.0, 0.0, 0.0, 0.0, 0.0),
    ];

    for policy in &policies {
        assert!(!policy.allow_drop);
        for judgment in &table {
            let disposition = classify_disposition(judgment, policy);
            assert_ne!(disposition, EvidenceDisposition::Drop);
            assert!(disposition.rank() < EvidenceDisposition::Drop.rank());
        }
    }
}

#[test]
fn rank_orders_dispositions_by_policy_severity() {
    assert_eq!(EvidenceDisposition::Keep.rank(), 0);
    assert_eq!(EvidenceDisposition::Flag.rank(), 1);
    assert_eq!(EvidenceDisposition::Demote.rank(), 2);
    assert_eq!(EvidenceDisposition::Drop.rank(), 3);

    assert!(EvidenceDisposition::Keep < EvidenceDisposition::Flag);
    assert!(EvidenceDisposition::Flag < EvidenceDisposition::Demote);
    assert!(EvidenceDisposition::Demote < EvidenceDisposition::Drop);

    let mut sorted = vec![
        EvidenceDisposition::Drop,
        EvidenceDisposition::Keep,
        EvidenceDisposition::Demote,
        EvidenceDisposition::Flag,
    ];
    sorted.sort();
    assert_eq!(
        sorted,
        vec![
            EvidenceDisposition::Keep,
            EvidenceDisposition::Flag,
            EvidenceDisposition::Demote,
            EvidenceDisposition::Drop,
        ]
    );
}

#[test]
fn validated_accepts_boundary_values() {
    let policy = policy_with(|p| {
        p.min_confidence = 0.0;
        p.min_relevance = 1.0;
        p.min_useful_evidence = 0.0;
        p.contradiction_flag_threshold = 1.0;
        p.instruction_flag_threshold = 0.0;
        p.candidate_limit = 1;
    });

    assert_eq!(policy.validated(), Ok(policy));
}

#[test]
fn validated_rejects_each_invalid_field() {
    let cases: [(EvidencePolicy, EvidencePolicyError); 8] = [
        (
            policy_with(|p| p.candidate_limit = 0),
            EvidencePolicyError::ZeroCandidateLimit,
        ),
        (
            policy_with(|p| p.min_confidence = 1.5),
            EvidencePolicyError::InvalidMinConfidence,
        ),
        (
            policy_with(|p| p.min_confidence = f32::NAN),
            EvidencePolicyError::InvalidMinConfidence,
        ),
        (
            policy_with(|p| p.min_relevance = -0.1),
            EvidencePolicyError::InvalidMinRelevance,
        ),
        (
            policy_with(|p| p.min_useful_evidence = 2.0),
            EvidencePolicyError::InvalidMinUsefulEvidence,
        ),
        (
            policy_with(|p| p.contradiction_flag_threshold = f32::INFINITY),
            EvidencePolicyError::InvalidContradictionThreshold,
        ),
        (
            policy_with(|p| p.instruction_flag_threshold = -1.0),
            EvidencePolicyError::InvalidInstructionThreshold,
        ),
        (
            policy_with(|p| p.instruction_flag_threshold = f32::NEG_INFINITY),
            EvidencePolicyError::InvalidInstructionThreshold,
        ),
    ];

    for (policy, expected) in cases {
        assert_eq!(policy.validated(), Err(expected));
    }
}
