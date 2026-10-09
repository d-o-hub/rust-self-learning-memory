//! Lexical fallback scoring for pattern search.
//!
//! The semantic component of [`super::scoring::ScoreBreakdown`] normally comes from embedding
//! cosine similarity. When a query has no usable embedding — no service configured, embedding
//! generation failed, or the caller passed an empty vector — this module supplies a bounded,
//! deterministic stand-in so the component keeps discriminating between patterns instead of
//! collapsing to a constant.

use crate::embeddings::semantic_text::pattern_to_text;
use crate::patterns::Pattern;
#[cfg(test)]
use crate::types::TaskContext;
use std::collections::HashSet;

/// Neutral score returned when there is nothing to compare.
///
/// Preserves the historical behaviour for degenerate input (blank query) so the pattern keeps
/// its previous ranking instead of dropping out.
pub(crate) const NEUTRAL_SIMILARITY: f32 = 0.5;

/// Words the text builder emits as field labels rather than as content.
///
/// [`pattern_to_text`] writes `domain: …`, `language: …`, `framework: …`, `tags: …`,
/// `complexity: …`. Counting those label tokens as matches would credit every pattern for
/// text the caller never asked about.
const STRUCTURAL_LABELS: [&str; 5] = ["domain", "language", "framework", "tags", "complexity"];

/// Lowercase, split on any non-alphanumeric character, and drop the noise.
///
/// Splitting on punctuation (not just whitespace) is what lets `web-api` match a query
/// saying `Web API`, and matches the episode-tag convention of trimming and lowercasing.
fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.len() >= 2)
        .filter(|token| !STRUCTURAL_LABELS.contains(token))
        .map(str::to_string)
        .collect()
}

/// Fallback keyword-based similarity used when no usable query embedding exists.
///
/// Scores the query the provider path embeds (the raw `query`, as
/// `search_patterns_semantic` hands to `embed_text`) against [`pattern_to_text`], the text
/// `embed_pattern` embeds for the cosine path — so a pattern mentioning the query's terms
/// outranks one that does not, and both paths measure the same two strings.
///
/// Bounded and local: the fraction of the query's distinct terms the pattern mentions,
/// `0.0..=1.0`. No corpus statistics are available per call, so this makes no IDF claim
/// and deliberately does NOT consult the task context — context agreement stays the job of
/// [`super::scoring::calculate_context_match`] and must not be double-counted here.
pub fn calculate_keyword_similarity(query: &str, pattern: &Pattern) -> f32 {
    let query_tokens = tokenize(query);
    if query_tokens.is_empty() {
        // No typed terms to match on: keep the historical neutral value.
        return NEUTRAL_SIMILARITY;
    }

    let pattern_tokens = tokenize(&pattern_to_text(pattern));
    let common = query_tokens.intersection(&pattern_tokens).count();

    common as f32 / query_tokens.len() as f32
}

#[cfg(test)]
pub(crate) fn pattern_with(tools: &[&str], tags: &[&str]) -> Pattern {
    use crate::patterns::PatternEffectiveness;
    use chrono::Duration;
    use uuid::Uuid;

    Pattern::ToolSequence {
        id: Uuid::new_v4(),
        tools: tools.iter().map(|t| (*t).to_string()).collect(),
        context: TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            framework: None,
            complexity: crate::types::ComplexityLevel::Moderate,
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
        },
        success_rate: 0.9,
        avg_latency: Duration::milliseconds(100),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness::new(),
    }
}

#[cfg(test)]
pub(crate) fn query_context(tags: &[&str]) -> TaskContext {
    TaskContext {
        domain: "web-api".to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: crate::types::ComplexityLevel::Moderate,
        tags: tags.iter().map(|t| (*t).to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The defect this module exists to prevent: both arguments were ignored, so no query
    /// could distinguish anything.
    #[test]
    fn keyword_similarity_prefers_the_pattern_matching_the_query() {
        let matching = pattern_with(&["axum", "tokio", "postgres"], &["async", "rest"]);
        let unrelated = pattern_with(&["ffmpeg", "blender"], &["media"]);

        // Three query tokens; the matching pattern carries two of them (`async`, `rest`).
        let matched = calculate_keyword_similarity("async rest server", &matching);
        let unmatched = calculate_keyword_similarity("async rest server", &unrelated);

        assert!(
            matched > unmatched,
            "a pattern carrying the query's terms must outrank an unrelated one, \
             got {matched} vs {unmatched}"
        );
        assert!(
            matched - unmatched >= 0.2,
            "the gap has to be wide enough to reorder a result set, got {matched} vs {unmatched}"
        );
        assert!(
            unmatched < NEUTRAL_SIMILARITY,
            "a query-agnostic pattern must not earn the neutral score, got {unmatched}"
        );
        assert_ne!(
            matched, NEUTRAL_SIMILARITY,
            "a matching pattern must not land on the historical constant"
        );
    }

    /// Regression (PR #1138 roast): the first implementation folded `create_query_text`
    /// (query + context fields) into the lexical numerator, so a pattern sharing only the
    /// task context could outrank one carrying the query's own terms.
    #[test]
    fn keyword_similarity_ignores_the_task_context() {
        let context_only = pattern_with(&["ffmpeg"], &["async"]);
        let query_term = pattern_with(&["axum"], &[]);

        let with_query_term = calculate_keyword_similarity("axum server", &query_term);
        let with_context_only = calculate_keyword_similarity("axum server", &context_only);

        assert_eq!(
            with_context_only, 0.0,
            "context overlap must not earn lexical credit"
        );
        assert!(
            with_query_term > with_context_only,
            "the pattern naming the query term must win: {with_query_term} vs {with_context_only}"
        );
    }

    #[test]
    fn keyword_similarity_is_case_and_punctuation_insensitive() {
        let pattern = pattern_with(&["axum", "tokio"], &["async"]);

        assert_eq!(
            calculate_keyword_similarity("Async REST server", &pattern),
            calculate_keyword_similarity("async, rest. server!", &pattern),
            "case and punctuation must not change the score"
        );
    }

    #[test]
    fn keyword_similarity_deduplicates_repeated_terms() {
        let pattern = pattern_with(&["axum", "tokio"], &["async"]);

        assert_eq!(
            calculate_keyword_similarity("async async async axum", &pattern),
            calculate_keyword_similarity("async axum", &pattern),
            "repeating a term must not inflate the score"
        );
    }

    #[test]
    fn keyword_similarity_is_bounded_and_deterministic() {
        let pattern = pattern_with(&["axum", "tokio", "postgres"], &["async", "rest"]);

        let first = calculate_keyword_similarity("async rest axum postgres server", &pattern);
        let second = calculate_keyword_similarity("async rest axum postgres server", &pattern);

        assert_eq!(first, second, "same inputs must give the same score");
        // Four of the five query terms (`async`, `rest`, `axum`, `postgres`) appear in the
        // pattern text; `server` does not.
        assert!(
            (first - 0.8).abs() < 1e-6,
            "expected exactly 4/5 query terms to match, got {first}"
        );
        assert!(
            (0.0..=1.0).contains(&first),
            "score {first} escaped the bounded 0.0..=1.0 range"
        );
    }

    /// Degenerate input keeps the historical neutral value instead of dropping the pattern.
    #[test]
    fn empty_inputs_fall_back_to_the_neutral_constant() {
        let pattern = pattern_with(&["axum"], &["async"]);

        assert_eq!(
            calculate_keyword_similarity("", &pattern),
            NEUTRAL_SIMILARITY,
            "a blank query has no typed terms to match on"
        );
        assert_eq!(
            calculate_keyword_similarity("...  !!!", &pattern),
            NEUTRAL_SIMILARITY,
            "punctuation-only input tokenises to nothing"
        );
    }
}
