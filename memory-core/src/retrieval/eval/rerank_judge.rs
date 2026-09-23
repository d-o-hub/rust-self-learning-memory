//! Deterministic offline judge for the semantic rerank comparison benchmark.
//!
//! The harness needs a rerank comparison arm that runs without credentials,
//! without network access, and with identical results on every machine. This
//! judge is a *mechanism* probe for the semantic rerank stage: it exercises the
//! typed judgment path end to end, but its scores are lexical token overlap, not
//! semantic relevance.
//!
//! It MUST NOT be read as evidence that semantic reranking improves retrieval
//! quality; only a real provider judge can support a quality claim. See
//! `docs/eval_benchmark_guide.md`.

use std::collections::HashSet;

use crate::retrieval::judgment::{
    AtomicScore, CandidateJudgment, JudgmentCandidate, JudgmentError, RetrievalJudge,
};

/// Fixed confidence attached to every [`LocalOverlapJudge`] relevance score.
///
/// The overlap ratio is an exact, reproducible statistic rather than a sampled
/// model output, so the value is a documented constant and not a measured
/// probability. It sits above the default
/// [`crate::retrieval::rerank::SemanticRerankConfig::min_judgment_confidence`]
/// floor of `0.70` so every candidate in the benchmark shortlist takes the
/// judged path; a lower value would silently turn the comparison arm into a
/// plain local run.
pub const LOCAL_OVERLAP_JUDGE_CONFIDENCE: f32 = 0.90;

/// Offline [`RetrievalJudge`] scoring relevance by query/candidate token overlap.
///
/// For each candidate the relevance score is the Jaccard ratio of the query and
/// candidate token sets, clamped to `[0.0, 1.0]`:
///
/// ```text
/// relevance = |query_tokens ∩ candidate_tokens| / |query_tokens ∪ candidate_tokens|
/// ```
///
/// Tokens are produced by lowercasing and splitting on any non-alphanumeric
/// character; duplicates collapse, and a pair with no tokens on either side
/// scores `0.0`. Judgments are returned in input order, one per candidate, and
/// candidates are never reordered or dropped.
#[derive(Debug, Clone, Copy)]
pub struct LocalOverlapJudge {
    confidence: f32,
}

impl LocalOverlapJudge {
    /// Create a judge with [`LOCAL_OVERLAP_JUDGE_CONFIDENCE`] confidence.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            confidence: LOCAL_OVERLAP_JUDGE_CONFIDENCE,
        }
    }

    /// Confidence reported alongside every relevance score.
    #[must_use]
    pub const fn confidence(&self) -> f32 {
        self.confidence
    }
}

impl Default for LocalOverlapJudge {
    fn default() -> Self {
        Self::new()
    }
}

impl RetrievalJudge for LocalOverlapJudge {
    fn judge_candidates(
        &self,
        query: &str,
        candidates: &[JudgmentCandidate<'_>],
    ) -> Result<Vec<CandidateJudgment>, JudgmentError> {
        let query_tokens = tokenize(query);

        Ok(candidates
            .iter()
            .map(|candidate| {
                let relevance = overlap_ratio(&query_tokens, &tokenize(candidate.text));
                CandidateJudgment {
                    id: candidate.id.to_string(),
                    relevance: AtomicScore::new(relevance, self.confidence),
                    // A lexical judge assesses relevance only. `useful_evidence`,
                    // `contradiction`, and `instruction_like` are reported as
                    // "not assessed" (`0.0` value, `0.0` confidence) rather than
                    // as confident negative claims.
                    useful_evidence: AtomicScore::new(0.0, 0.0),
                    contradiction: AtomicScore::new(0.0, 0.0),
                    instruction_like: AtomicScore::new(0.0, 0.0),
                }
            })
            .collect())
    }
}

fn tokenize(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn overlap_ratio(query_tokens: &HashSet<String>, candidate_tokens: &HashSet<String>) -> f32 {
    if query_tokens.is_empty() || candidate_tokens.is_empty() {
        return 0.0;
    }

    let intersection = query_tokens.intersection(candidate_tokens).count();
    let union = query_tokens.len() + candidate_tokens.len() - intersection;
    (intersection as f32 / union as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate<'a>(id: &'a str, text: &'a str) -> JudgmentCandidate<'a> {
        JudgmentCandidate::new(id, text, 0.5)
    }

    #[test]
    fn test_overlap_relevance_is_lexical_and_ordered() {
        let judge = LocalOverlapJudge::new();
        let query = "OAuth2 authentication JWT tokens in Rust";

        let candidates = vec![
            candidate("exact", "OAuth2 authentication JWT tokens in Rust"),
            candidate("partial", "OAuth2 JWT refresh flow"),
            candidate("unrelated", "PostgreSQL query optimization"),
            candidate("empty", ""),
        ];

        let judgments = judge
            .judge_candidates(query, &candidates)
            .expect("offline judge cannot fail");

        assert_eq!(
            judgments.iter().map(|j| j.id.as_str()).collect::<Vec<_>>(),
            vec!["exact", "partial", "unrelated", "empty"],
            "judgments must follow input candidate order"
        );

        let relevance: Vec<f32> = judgments.iter().map(|j| j.relevance.value).collect();
        assert_eq!(relevance[0], 1.0, "identical text is full overlap");
        assert!(
            relevance[0] > relevance[1],
            "a lexically closer candidate must score higher: {relevance:?}"
        );
        assert!(relevance[1] > relevance[2]);
        assert_eq!(relevance[2], 0.0, "disjoint token sets share nothing");
        assert_eq!(relevance[3], 0.0, "an empty candidate has no overlap");

        for judgment in &judgments {
            assert!(
                judgment.is_valid(),
                "judgment must satisfy the trust boundary"
            );
            assert_eq!(
                judgment.relevance.confidence,
                LOCAL_OVERLAP_JUDGE_CONFIDENCE
            );
        }
    }

    #[test]
    fn test_relevance_stays_in_unit_range_and_is_deterministic() {
        let judge = LocalOverlapJudge::new();

        // Multibyte text, punctuation-only queries, and repeated tokens are the
        // cases that can push a naive ratio outside `[0.0, 1.0]`.
        let cases = [
            ("Ünïcödé OAuth2 tökens", "ünïcödé oauth2 tökens and more"),
            ("...!!!", "anything at all"),
            ("a a a a", "a b"),
        ];

        for (query, text) in cases {
            let candidates = vec![candidate("c", text)];
            let first = judge.judge_candidates(query, &candidates).unwrap();
            let second = judge.judge_candidates(query, &candidates).unwrap();

            assert_eq!(first, second, "judge must be deterministic");
            let value = first[0].relevance.value;
            assert!(
                (0.0..=1.0).contains(&value),
                "relevance {value} out of range for {query:?} / {text:?}"
            );
        }

        assert!(
            judge.judge_candidates("anything", &[]).unwrap().is_empty(),
            "an empty batch yields no judgments"
        );
    }

    #[test]
    fn test_unassessed_dimensions_report_zero_confidence() {
        let judge = LocalOverlapJudge::new();
        let candidates = vec![candidate("c", "some text")];

        let judgments = judge.judge_candidates("some text", &candidates).unwrap();

        assert_eq!(judgments[0].relevance.value, 1.0);
        assert_eq!(judgments[0].useful_evidence, AtomicScore::new(0.0, 0.0));
        assert_eq!(judgments[0].contradiction, AtomicScore::new(0.0, 0.0));
        assert_eq!(judgments[0].instruction_like, AtomicScore::new(0.0, 0.0));
    }
}
