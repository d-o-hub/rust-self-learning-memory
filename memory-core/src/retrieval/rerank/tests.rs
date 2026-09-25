//! Deterministic tests for the optional semantic rerank stage.
//!
//! Every test uses a fake judge that records the batch it observed, so provider
//! interactions (call count, batch contents, batch order) are asserted directly
//! instead of inferred from the outcome.

use super::*;
use crate::retrieval::judgment::{AtomicScore, CandidateJudgment, JudgmentError};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Relevance score factory used to build canned provider judgments per candidate id.
type RelevanceFn = dyn Fn(&str) -> AtomicScore + Send + Sync;

/// Canned provider response.
enum FakeResponse {
    /// One relevance score per observed candidate id.
    Scores(Box<RelevanceFn>),
    /// Raw judgments returned verbatim (used to emulate malformed output).
    Raw(Vec<CandidateJudgment>),
    /// Provider failure (timeout, unavailable, invalid, provider error).
    Fail(JudgmentError),
}

/// Judge stub that counts calls and records every observed batch.
struct FakeJudge {
    calls: AtomicUsize,
    seen: Mutex<Vec<Vec<String>>>,
    response: FakeResponse,
    reverse_output: bool,
}

impl FakeJudge {
    /// Judge that answers with `relevance(id)` for each observed candidate.
    fn scores<F>(relevance: F) -> Self
    where
        F: Fn(&str) -> AtomicScore + Send + Sync + 'static,
    {
        Self {
            calls: AtomicUsize::new(0),
            seen: Mutex::new(Vec::new()),
            response: FakeResponse::Scores(Box::new(relevance)),
            reverse_output: false,
        }
    }

    /// Judge that answers every candidate with the same relevance score.
    fn fixed(relevance: AtomicScore) -> Self {
        Self::scores(move |_| relevance)
    }

    /// Judge that returns raw (possibly malformed) judgments in the given order.
    fn raw(judgments: Vec<CandidateJudgment>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            seen: Mutex::new(Vec::new()),
            response: FakeResponse::Raw(judgments),
            reverse_output: false,
        }
    }

    /// Judge that always fails with `err`.
    fn failing(err: JudgmentError) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            seen: Mutex::new(Vec::new()),
            response: FakeResponse::Fail(err),
            reverse_output: false,
        }
    }

    /// Judge whose provider returns its batch in reverse order.
    fn scores_reversed<F>(relevance: F) -> Self
    where
        F: Fn(&str) -> AtomicScore + Send + Sync + 'static,
    {
        let mut judge = Self::scores(relevance);
        judge.reverse_output = true;
        judge
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }

    /// Candidate ids observed in the most recent provider call, in input order.
    fn shortlisted(&self) -> Vec<String> {
        self.seen
            .lock()
            .expect("fake judge call log poisoned")
            .last()
            .cloned()
            .unwrap_or_default()
    }
}

impl RetrievalJudge for FakeJudge {
    fn judge_candidates(
        &self,
        _query: &str,
        candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.seen
            .lock()
            .expect("fake judge call log poisoned")
            .push(candidates.iter().map(|c| c.id.to_string()).collect());

        match &self.response {
            FakeResponse::Fail(err) => Err(clone_error(err)),
            FakeResponse::Raw(judgments) => Ok(judgments.clone()),
            FakeResponse::Scores(relevance) => {
                let mut judgments: Vec<CandidateJudgment> = candidates
                    .iter()
                    .map(|candidate| judgment(candidate.id, relevance(candidate.id)))
                    .collect();
                if self.reverse_output {
                    judgments.reverse();
                }
                Ok(judgments)
            }
        }
    }
}

fn clone_error(err: &JudgmentError) -> JudgmentError {
    match err {
        JudgmentError::Unavailable => JudgmentError::Unavailable,
        JudgmentError::Timeout => JudgmentError::Timeout,
        JudgmentError::Invalid(msg) => JudgmentError::Invalid(msg.clone()),
        JudgmentError::Provider(msg) => JudgmentError::Provider(msg.clone()),
    }
}

/// Builds a candidate judgment with a relevant value/confidence pair.
fn judgment(id: &str, relevance: AtomicScore) -> CandidateJudgment {
    CandidateJudgment {
        id: id.to_string(),
        relevance,
        useful_evidence: AtomicScore::new(0.5, relevance.confidence),
        contradiction: AtomicScore::new(0.0, relevance.confidence),
        instruction_like: AtomicScore::new(0.0, relevance.confidence),
    }
}

fn candidates(items: &[(&str, f32)]) -> Vec<(String, f32)> {
    items
        .iter()
        .map(|(id, score)| ((*id).to_string(), *score))
        .collect()
}

fn texts<'a>(items: &[(&'a str, &'a str)]) -> HashMap<&'a str, &'a str> {
    items.iter().copied().collect()
}

fn ids(items: &[(String, f32)]) -> Vec<String> {
    items.iter().map(|(id, _)| id.clone()).collect()
}

fn scores(items: &[(String, f32)]) -> Vec<f32> {
    items.iter().map(|(_, score)| *score).collect()
}

/// Enabled rerank using the default weights/floor.
fn enabled() -> SemanticRerankConfig {
    SemanticRerankConfig {
        enabled: true,
        ..SemanticRerankConfig::default()
    }
}

fn approx(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 1e-6
}

/// Candidates used by the fusion tests: local scores 0.9 / 0.6 / 0.5.
fn fusion_candidates() -> Vec<(String, f32)> {
    candidates(&[("a", 0.9), ("b", 0.6), ("c", 0.5)])
}

fn fusion_texts() -> HashMap<&'static str, &'static str> {
    texts(&[("a", "alpha"), ("b", "bravo"), ("c", "charlie")])
}

/// Passage text for candidates "a".."e".
fn wide_texts() -> HashMap<&'static str, &'static str> {
    texts(&[
        ("a", "alpha"),
        ("b", "bravo"),
        ("c", "charlie"),
        ("d", "delta"),
        ("e", "echo"),
    ])
}

#[test]
fn test_default_config_is_disabled_with_frozen_defaults() {
    let config = SemanticRerankConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.shortlist_k, 20);
    assert_eq!(config.output_k, 10);
    assert_eq!(config.local_weight, 0.4);
    assert_eq!(config.semantic_weight, 0.6);
    assert_eq!(config.min_judgment_confidence, 0.70);
    assert!(config.validated().is_ok());
}

#[test]
fn test_disabled_config_returns_candidates_without_provider_call() {
    // Also proves precedence: a disabled stage never validates its other fields.
    let config = SemanticRerankConfig {
        enabled: false,
        shortlist_k: 0,
        ..SemanticRerankConfig::default()
    };
    let judge = FakeJudge::fixed(AtomicScore::new(1.0, 1.0));
    let cands = fusion_candidates();

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &config);

    assert_eq!(outcome.status, RerankStatus::Disabled);
    assert_eq!(outcome.ids, ids(&cands));
    assert_eq!(outcome.scores, scores(&cands));
    assert_eq!(outcome.shortlist_len, 0);
    assert_eq!(outcome.output_len, cands.len());
    assert!(!outcome.top1_changed);
    assert_eq!(
        judge.calls(),
        0,
        "disabled rerank must not call the provider"
    );
}

#[test]
fn test_missing_judge_returns_candidates_unchanged() {
    let cands = fusion_candidates();

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), None, &enabled());

    assert_eq!(outcome.status, RerankStatus::NotConfigured);
    assert_eq!(outcome.ids, ids(&cands));
    assert_eq!(outcome.scores, scores(&cands));
    assert_eq!(outcome.shortlist_len, 0);
    assert_eq!(outcome.output_len, cands.len());
    assert!(!outcome.top1_changed);
}

#[test]
fn test_invalid_config_returns_candidates_without_provider_call() {
    let judge = FakeJudge::fixed(AtomicScore::new(1.0, 1.0));
    let cands = fusion_candidates();

    for config in [
        SemanticRerankConfig {
            enabled: true,
            shortlist_k: 0,
            ..SemanticRerankConfig::default()
        },
        SemanticRerankConfig {
            enabled: true,
            shortlist_k: 2,
            output_k: 3,
            ..SemanticRerankConfig::default()
        },
        SemanticRerankConfig {
            enabled: true,
            local_weight: f32::NAN,
            ..SemanticRerankConfig::default()
        },
    ] {
        let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &config);
        assert_eq!(outcome.status, RerankStatus::Invalid);
        assert_eq!(outcome.ids, ids(&cands));
        assert_eq!(outcome.scores, scores(&cands));
        assert_eq!(outcome.shortlist_len, 0);
    }

    assert_eq!(
        judge.calls(),
        0,
        "invalid config must not call the provider"
    );
}

#[test]
fn test_empty_candidates_applied_without_provider_call() {
    let judge = FakeJudge::fixed(AtomicScore::new(1.0, 1.0));

    let outcome = semantic_rerank("query", &[], &HashMap::new(), Some(&judge), &enabled());

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert!(outcome.ids.is_empty());
    assert!(outcome.scores.is_empty());
    assert_eq!(outcome.shortlist_len, 0);
    assert_eq!(outcome.output_len, 0);
    assert!(!outcome.top1_changed);
    assert_eq!(judge.calls(), 0);
}

#[test]
fn test_text_lookup_mismatch_returns_candidates_without_provider_call() {
    let judge = FakeJudge::fixed(AtomicScore::new(1.0, 1.0));
    let cands = fusion_candidates();
    // "c" has no passage text.
    let partial_texts = texts(&[("a", "alpha"), ("b", "bravo")]);

    let outcome = semantic_rerank("query", &cands, &partial_texts, Some(&judge), &enabled());

    assert_eq!(outcome.status, RerankStatus::Invalid);
    assert_eq!(outcome.ids, ids(&cands));
    assert_eq!(outcome.scores, scores(&cands));
    assert_eq!(outcome.shortlist_len, cands.len());
    assert_eq!(outcome.output_len, cands.len());
    assert_eq!(
        judge.calls(),
        0,
        "missing text lookup must not call the provider"
    );
}

#[test]
fn test_shortlist_of_one_skips_provider_and_caps_local_order() {
    let judge = FakeJudge::fixed(AtomicScore::new(0.0, 0.1));
    let cands = fusion_candidates();
    let config = SemanticRerankConfig {
        enabled: true,
        shortlist_k: 1,
        output_k: 1,
        ..SemanticRerankConfig::default()
    };

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &config);

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids, vec!["a"]);
    assert_eq!(outcome.scores, vec![0.9]);
    assert_eq!(outcome.shortlist_len, 1);
    assert_eq!(outcome.output_len, 1);
    assert!(!outcome.top1_changed);
    assert_eq!(
        judge.calls(),
        0,
        "a single shortlisted candidate needs no judgment"
    );
}

#[test]
fn test_shortlist_is_capped_to_shortlist_k_in_local_order() {
    let cands = candidates(&[("a", 0.9), ("b", 0.8), ("c", 0.7), ("d", 0.6), ("e", 0.5)]);
    let judge = FakeJudge::fixed(AtomicScore::new(0.5, 0.9));
    let config = SemanticRerankConfig {
        enabled: true,
        shortlist_k: 3,
        output_k: 3,
        ..SemanticRerankConfig::default()
    };

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &config);

    assert_eq!(
        judge.shortlisted(),
        vec!["a", "b", "c"],
        "only the leading shortlist_k candidates may be sent to the provider, in local order"
    );
    assert_eq!(judge.calls(), 1);
    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.shortlist_len, 3);
    assert_eq!(outcome.output_len, 3);
    assert_eq!(outcome.ids, vec!["a", "b", "c"]);
}

#[test]
fn test_semantically_stronger_candidate_is_promoted_with_fused_scores() {
    // Local scores 0.9/0.6/0.5 normalize to 1.0/0.25/0.0.
    let cands = fusion_candidates();
    // b is the semantically strong candidate; a is judged irrelevant.
    let judge = FakeJudge::scores(|id| match id {
        "b" => AtomicScore::new(1.0, 0.9),
        "c" => AtomicScore::new(0.5, 0.9),
        _ => AtomicScore::new(0.1, 0.9),
    });

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &enabled());

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids, vec!["b", "a", "c"]);
    assert!(outcome.top1_changed, "promoted candidate must flip top-1");
    assert_eq!(outcome.shortlist_len, 3);
    assert_eq!(outcome.output_len, 3);
    assert_eq!(judge.calls(), 1, "one batched call per rerank invocation");
    assert!(approx(outcome.scores[0], 0.4 * 0.25 + 0.6 * 0.9));
    assert!(approx(outcome.scores[1], 0.4 * 1.0 + 0.6 * 0.09));
    assert!(approx(outcome.scores[2], 0.4 * 0.0 + 0.6 * 0.45));
}

#[test]
fn test_provider_output_order_is_joined_by_id() {
    let relevance = |id: &str| match id {
        "b" => AtomicScore::new(1.0, 0.9),
        "c" => AtomicScore::new(0.5, 0.9),
        _ => AtomicScore::new(0.1, 0.9),
    };

    let in_order = FakeJudge::scores(relevance);
    let reversed = FakeJudge::scores_reversed(relevance);
    let cands = fusion_candidates();

    let ordered_outcome = semantic_rerank(
        "query",
        &cands,
        &fusion_texts(),
        Some(&in_order),
        &enabled(),
    );
    let reversed_outcome = semantic_rerank(
        "query",
        &cands,
        &fusion_texts(),
        Some(&reversed),
        &enabled(),
    );

    assert_eq!(
        ordered_outcome, reversed_outcome,
        "provider batch order must not influence the fused ranking"
    );
    assert_eq!(reversed_outcome.ids, vec!["b", "a", "c"]);
}

#[test]
fn test_low_confidence_candidate_keeps_its_local_score() {
    let cands = candidates(&[("a", 0.9), ("b", 0.6)]);
    // "a" claims irrelevance with a confidence below the 0.70 floor: if that
    // judgment were honored, a (0.4) would drop below b (0.54).
    let judge = FakeJudge::scores(|id| match id {
        "a" => AtomicScore::new(0.0, 0.69),
        _ => AtomicScore::new(1.0, 0.9),
    });

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &enabled());

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids, vec!["a", "b"]);
    assert!(!outcome.top1_changed);
    assert!(
        approx(outcome.scores[0], 1.0),
        "below-floor candidate keeps normalized local score"
    );
    assert!(approx(outcome.scores[1], 0.4 * 0.0 + 0.6 * 0.9));
}

#[test]
fn test_all_low_confidence_returns_original_list_exactly() {
    let cands = fusion_candidates();
    let judge = FakeJudge::fixed(AtomicScore::new(0.9, 0.2));
    let config = SemanticRerankConfig {
        enabled: true,
        output_k: 1,
        ..SemanticRerankConfig::default()
    };

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &config);

    assert_eq!(outcome.status, RerankStatus::LowConfidence);
    assert_eq!(
        outcome.ids,
        ids(&cands),
        "fallback must keep every local candidate"
    );
    assert_eq!(
        outcome.scores,
        scores(&cands),
        "fallback must keep original local scores"
    );
    assert_eq!(outcome.shortlist_len, 3);
    assert_eq!(outcome.output_len, cands.len());
    assert!(!outcome.top1_changed);
    assert_eq!(judge.calls(), 1);
}

#[test]
fn test_provider_failures_return_original_list_with_provider_error_status() {
    let cands = fusion_candidates();

    for err in [
        JudgmentError::Unavailable,
        JudgmentError::Timeout,
        JudgmentError::Provider("provider exploded".to_string()),
    ] {
        let judge = FakeJudge::failing(err);
        let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &enabled());

        assert_eq!(outcome.status, RerankStatus::ProviderError);
        assert_eq!(outcome.ids, ids(&cands));
        assert_eq!(outcome.scores, scores(&cands));
        assert_eq!(outcome.shortlist_len, 3);
        assert_eq!(outcome.output_len, cands.len());
        assert!(!outcome.top1_changed);
        assert_eq!(judge.calls(), 1);
    }
}

#[test]
fn test_invalid_provider_output_returns_original_list() {
    let cands = fusion_candidates();

    let malformed = [
        // Unknown and missing candidate ids.
        vec![
            judgment("zzz", AtomicScore::new(1.0, 0.9)),
            judgment("a", AtomicScore::new(1.0, 0.9)),
            judgment("b", AtomicScore::new(1.0, 0.9)),
        ],
        // Out-of-bound atomic score.
        vec![
            judgment("a", AtomicScore::new(1.5, 0.9)),
            judgment("b", AtomicScore::new(0.5, 0.9)),
            judgment("c", AtomicScore::new(0.5, 0.9)),
        ],
    ];

    for raw in malformed {
        let judge = FakeJudge::raw(raw);
        let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &enabled());

        assert_eq!(outcome.status, RerankStatus::Invalid);
        assert_eq!(outcome.ids, ids(&cands));
        assert_eq!(outcome.scores, scores(&cands));
        assert_eq!(outcome.output_len, cands.len());
        assert_eq!(judge.calls(), 1);
    }

    // A provider-reported validation failure maps to the same status.
    let judge = FakeJudge::failing(JudgmentError::Invalid("bad payload".to_string()));
    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &enabled());
    assert_eq!(outcome.status, RerankStatus::Invalid);
    assert_eq!(outcome.ids, ids(&cands));
    assert_eq!(outcome.scores, scores(&cands));
}

#[test]
fn test_output_k_is_respected_after_fusion() {
    let cands = candidates(&[("a", 0.9), ("b", 0.8), ("c", 0.7), ("d", 0.6), ("e", 0.5)]);
    let judge = FakeJudge::scores(|id| match id {
        "a" => AtomicScore::new(0.2, 0.9),
        "b" => AtomicScore::new(1.0, 0.9),
        _ => AtomicScore::new(0.5, 0.9),
    });
    let config = SemanticRerankConfig {
        enabled: true,
        output_k: 2,
        ..SemanticRerankConfig::default()
    };

    let outcome = semantic_rerank("query", &cands, &wide_texts(), Some(&judge), &config);

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids, vec!["b", "a"]);
    assert_eq!(outcome.scores.len(), 2);
    assert!(approx(outcome.scores[0], 0.4 * 0.75 + 0.6 * 0.9));
    assert!(approx(outcome.scores[1], 0.4 * 1.0 + 0.6 * 0.18));
    assert_eq!(outcome.shortlist_len, 5);
    assert_eq!(outcome.output_len, 2);
    assert!(outcome.top1_changed);
}

#[test]
fn test_ties_keep_local_rank_order_deterministically() {
    // Equal local scores normalize to 1.0, and equal relevance scores fuse to
    // equal final scores: the tie must resolve to the local rank, never to id
    // order ("z" precedes "a" here) and never to an unstable ordering.
    let cands = candidates(&[("z", 0.5), ("a", 0.5), ("m", 0.5)]);
    let passages = texts(&[("z", "zulu"), ("a", "alpha"), ("m", "mike")]);
    let judge = FakeJudge::scores(|id| match id {
        "m" => AtomicScore::new(1.0, 0.9),
        _ => AtomicScore::new(0.0, 0.9),
    });

    let first = semantic_rerank("query", &cands, &passages, Some(&judge), &enabled());
    let second = semantic_rerank("query", &cands, &passages, Some(&judge), &enabled());

    assert_eq!(first.ids, vec!["m", "z", "a"]);
    assert!(
        approx(first.scores[1], first.scores[2]),
        "tied final scores"
    );
    assert_eq!(first, second, "repeated runs must be byte-identical");
}

#[test]
fn test_configured_weights_are_normalized_before_fusion() {
    // Weights 1.0/1.0 must behave as 0.5/0.5, so the fused scores stay a
    // convex combination instead of scaling past 1.0.
    let cands = candidates(&[("a", 0.9), ("b", 0.6)]);
    let judge = FakeJudge::scores(|id| match id {
        "a" => AtomicScore::new(0.1, 0.9),
        _ => AtomicScore::new(1.0, 0.9),
    });
    let config = SemanticRerankConfig {
        enabled: true,
        local_weight: 1.0,
        semantic_weight: 1.0,
        ..SemanticRerankConfig::default()
    };

    let outcome = semantic_rerank("query", &cands, &fusion_texts(), Some(&judge), &config);

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids, vec!["a", "b"]);
    assert!(approx(outcome.scores[0], 0.5 * 1.0 + 0.5 * 0.09));
    assert!(approx(outcome.scores[1], 0.5 * 0.0 + 0.5 * 0.9));
}

#[test]
fn test_judge_is_called_at_most_once_per_invocation() {
    let cands = candidates(&[
        ("c0", 1.00),
        ("c1", 0.95),
        ("c2", 0.90),
        ("c3", 0.85),
        ("c4", 0.80),
        ("c5", 0.75),
        ("c6", 0.70),
        ("c7", 0.65),
        ("c8", 0.60),
        ("c9", 0.55),
        ("c10", 0.50),
        ("c11", 0.45),
    ]);
    let passages: Vec<(&str, &str)> = cands
        .iter()
        .map(|(id, _)| (id.as_str(), "passage"))
        .collect();
    let judge = FakeJudge::fixed(AtomicScore::new(0.8, 0.9));

    let outcome = semantic_rerank("query", &cands, &texts(&passages), Some(&judge), &enabled());

    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(
        judge.calls(),
        1,
        "no per-candidate provider call is permitted"
    );
    assert_eq!(judge.shortlisted().len(), 12);
}

#[test]
fn test_validated_rejects_invalid_configurations() {
    let base = enabled();

    let rejected = [
        (
            SemanticRerankConfig {
                shortlist_k: 0,
                ..base.clone()
            },
            RerankConfigError::ZeroShortlist,
        ),
        (
            SemanticRerankConfig {
                output_k: 0,
                ..base.clone()
            },
            RerankConfigError::InvalidOutputK {
                output_k: 0,
                shortlist_k: 20,
            },
        ),
        (
            SemanticRerankConfig {
                shortlist_k: 3,
                output_k: 4,
                ..base.clone()
            },
            RerankConfigError::InvalidOutputK {
                output_k: 4,
                shortlist_k: 3,
            },
        ),
        (
            SemanticRerankConfig {
                local_weight: f32::NAN,
                ..base.clone()
            },
            RerankConfigError::NonFiniteWeight,
        ),
        (
            SemanticRerankConfig {
                semantic_weight: f32::INFINITY,
                ..base.clone()
            },
            RerankConfigError::NonFiniteWeight,
        ),
        (
            SemanticRerankConfig {
                local_weight: -0.1,
                ..base.clone()
            },
            RerankConfigError::NegativeWeight,
        ),
        (
            SemanticRerankConfig {
                local_weight: 0.0,
                semantic_weight: 0.0,
                ..base.clone()
            },
            RerankConfigError::ZeroWeightSum,
        ),
        (
            SemanticRerankConfig {
                min_judgment_confidence: -0.01,
                ..base.clone()
            },
            RerankConfigError::InvalidConfidence,
        ),
        (
            SemanticRerankConfig {
                min_judgment_confidence: 1.5,
                ..base.clone()
            },
            RerankConfigError::InvalidConfidence,
        ),
        (
            SemanticRerankConfig {
                min_judgment_confidence: f32::NAN,
                ..base.clone()
            },
            RerankConfigError::InvalidConfidence,
        ),
    ];

    for (config, expected) in rejected {
        assert_eq!(config.validated(), Err(expected));
    }
}

#[test]
fn test_validated_normalizes_weights_without_mutating_the_original() {
    let config = SemanticRerankConfig {
        local_weight: 2.0,
        semantic_weight: 6.0,
        ..SemanticRerankConfig::default()
    };

    let normalized = config.validated().expect("weights are normalizable");

    assert!(approx(normalized.local_weight, 0.25));
    assert!(approx(normalized.semantic_weight, 0.75));
    assert!(approx(
        normalized.local_weight + normalized.semantic_weight,
        1.0
    ));
    assert!(
        approx(config.local_weight, 2.0),
        "original config is untouched"
    );
    assert!(approx(config.semantic_weight, 6.0));

    let default = SemanticRerankConfig::default()
        .validated()
        .expect("default config is valid");
    assert!(approx(default.local_weight, 0.4));
    assert!(approx(default.semantic_weight, 0.6));
}

#[test]
fn test_min_max_normalization_is_bounded_and_deterministic() {
    assert!(approx(normalize_min_max(&[0.9])[0], 1.0));
    assert!(approx(normalize_min_max(&[0.5, 0.5])[0], 1.0));
    assert!(approx(normalize_min_max(&[0.5, 0.5])[1], 1.0));
    assert!(approx(normalize_min_max(&[0.9, 0.6, 0.5])[0], 1.0));
    assert!(approx(normalize_min_max(&[0.9, 0.6, 0.5])[1], 0.25));
    assert!(approx(normalize_min_max(&[0.9, 0.6, 0.5])[2], 0.0));
    // Degenerate inputs must not produce NaN or out-of-range values.
    assert!(
        normalize_min_max(&[f32::NAN, 0.9, 0.5])
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    );
    assert!(
        normalize_min_max(&[f32::INFINITY, 0.5])
            .iter()
            .all(|v| (0.0..=1.0).contains(v))
    );
    assert!(normalize_min_max(&[]).is_empty());
}

/// A shared judgment batch fuses exactly like the internal provider path: the
/// supplied shortlist and its judgments produce the same order and scores as a
/// provider call answering with the same values.
#[test]
fn test_shared_judgment_batch_fuses_identically_to_the_internal_provider_call() {
    let relevance = |id: &str| match id {
        "b" => AtomicScore::new(1.0, 0.9),
        "c" => AtomicScore::new(0.5, 0.9),
        _ => AtomicScore::new(0.1, 0.9),
    };
    let cands = fusion_candidates();
    let passages = fusion_texts();
    let config = enabled();

    // Shared batch: one provider call, then fusion of the supplied judgments.
    let batch_judge = FakeJudge::scores(relevance);
    let batch = judge_shortlist_once(
        "query",
        &cands,
        &passages,
        Some(&batch_judge),
        config.shortlist_k,
    );
    assert_eq!(batch_judge.calls(), 1);
    assert_eq!(batch.status, RerankStatus::Applied);
    assert_eq!(batch.shortlist_len, cands.len());

    // Fusion reads only the id and the local score of each shortlisted entry.
    let mut shortlist: Vec<JudgmentCandidate<'_>> = Vec::with_capacity(cands.len());
    for (id, score) in cands.iter().take(batch.shortlist_len) {
        shortlist.push(JudgmentCandidate::new(id, "", *score));
    }
    let (fused, avg_confidence, confident_count) =
        fuse_with_judgments(&shortlist, &batch.judgments, &config, config.output_k);
    assert_eq!(confident_count, fused.len());

    // The stage itself, answering with the same judgments.
    let stage_judge = FakeJudge::scores(relevance);
    let outcome = semantic_rerank("query", &cands, &passages, Some(&stage_judge), &config);

    assert_eq!(stage_judge.calls(), 1);
    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids, ids(&fused));
    assert_eq!(outcome.scores, scores(&fused));
    assert!(approx(avg_confidence, 0.9));
}

/// The shared provider seam makes no provider call without a judge and no call
/// when the shortlist holds at most one candidate.
#[test]
fn test_judge_shortlist_once_skips_the_provider_without_a_judge_or_a_shortlist() {
    let judge = FakeJudge::fixed(AtomicScore::new(0.9, 0.9));
    let cands = fusion_candidates();
    let passages = fusion_texts();

    let unconfigured = judge_shortlist_once("query", &cands, &passages, None, 20);
    assert_eq!(unconfigured.status, RerankStatus::NotConfigured);
    assert_eq!(unconfigured.shortlist_len, 0);
    assert!(unconfigured.judgments.is_empty());
    assert_eq!(unconfigured.provider_ms, 0);

    // Nothing to shortlist, so nothing to call.
    let empty = judge_shortlist_once("query", &[], &HashMap::new(), Some(&judge), 20);
    assert_eq!(empty.status, RerankStatus::Applied);
    assert_eq!(empty.shortlist_len, 0);
    assert!(empty.judgments.is_empty());
    assert_eq!(empty.provider_ms, 0);

    // One candidate, because `shortlist_k` caps the batch to one.
    let capped = judge_shortlist_once("query", &cands, &passages, Some(&judge), 1);
    assert_eq!(capped.status, RerankStatus::Applied);
    assert_eq!(capped.shortlist_len, 1);
    assert!(capped.judgments.is_empty());
    assert_eq!(capped.provider_ms, 0);

    // One candidate, because the candidate list itself holds a single one.
    let single = judge_shortlist_once("query", &cands[..1], &passages, Some(&judge), 20);
    assert_eq!(single.status, RerankStatus::Applied);
    assert_eq!(single.shortlist_len, 1);
    assert!(single.judgments.is_empty());
    assert_eq!(single.provider_ms, 0);

    // No path above may reach the provider.
    assert_eq!(judge.calls(), 0);
}

/// `judge_shortlist_once` must map provider contract violations to bounded
/// statuses without inventing a ranking: a wrong judgment count is `Invalid`,
/// provider failures keep their class, and a missing judge never calls out.
#[test]
fn test_judge_shortlist_once_maps_count_mismatch_and_provider_errors() {
    let candidates = [("ep-1".to_string(), 0.9_f32), ("ep-2".to_string(), 0.5_f32)];
    let texts: HashMap<&str, &str> = HashMap::from([("ep-1", "one"), ("ep-2", "two")]);

    // Wrong count: one judgment for a two-candidate shortlist.
    let mismatched = FakeJudge::raw(vec![judgment("ep-1", AtomicScore::new(0.9, 0.9))]);
    let batch = judge_shortlist_once("q", &candidates, &texts, Some(&mismatched), 2);
    assert_eq!(batch.status, RerankStatus::Invalid);
    assert_eq!(batch.shortlist_len, 2);

    // Provider failure keeps its class.
    let failing = FakeJudge::failing(JudgmentError::Unavailable);
    let batch = judge_shortlist_once("q", &candidates, &texts, Some(&failing), 2);
    assert_eq!(batch.status, RerankStatus::ProviderError);

    // No judge: no provider call, no shortlist built.
    let batch = judge_shortlist_once("q", &candidates, &texts, None, 2);
    assert_eq!(batch.status, RerankStatus::NotConfigured);
    assert_eq!(batch.shortlist_len, 0);
}

/// The provider path behind `semantic_rerank` must surface malformed output and
/// provider failures as bounded statuses, and must truncate the fused ranking
/// to `output_k`.
#[test]
fn test_semantic_rerank_surfaces_provider_contract_and_truncates_output() {
    let candidates: Vec<(String, f32)> = vec![
        ("ep-1".to_string(), 0.9),
        ("ep-2".to_string(), 0.5),
        ("ep-3".to_string(), 0.2),
    ];
    let texts: HashMap<&str, &str> =
        HashMap::from([("ep-1", "one"), ("ep-2", "two"), ("ep-3", "three")]);
    let config = SemanticRerankConfig {
        enabled: true,
        output_k: 2,
        ..SemanticRerankConfig::default()
    };

    // Wrong judgment count violates the provider contract -> Invalid.
    let mismatched = FakeJudge::raw(vec![judgment("ep-1", AtomicScore::new(0.9, 0.9))]);
    let outcome = semantic_rerank("q", &candidates, &texts, Some(&mismatched), &config);
    assert_eq!(outcome.status, RerankStatus::Invalid);
    assert_eq!(outcome.ids.len(), candidates.len());

    // Provider failure -> ProviderError with the local order preserved.
    let failing = FakeJudge::failing(JudgmentError::Timeout);
    let outcome = semantic_rerank("q", &candidates, &texts, Some(&failing), &config);
    assert_eq!(outcome.status, RerankStatus::ProviderError);
    assert_eq!(
        outcome.ids,
        vec!["ep-1".to_string(), "ep-2".to_string(), "ep-3".to_string()]
    );

    // Successful fusion truncates to `output_k`.
    let judge = FakeJudge::fixed(AtomicScore::new(0.8, 0.9));
    let outcome = semantic_rerank("q", &candidates, &texts, Some(&judge), &config);
    assert_eq!(outcome.status, RerankStatus::Applied);
    assert_eq!(outcome.ids.len(), 2, "output_k caps the returned ranking");
    assert_eq!(outcome.output_len, 2);
    assert_eq!(outcome.shortlist_len, 3);
}
