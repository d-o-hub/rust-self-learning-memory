//! Lexical fallback scoring for pattern search.
//!
//! The semantic component of [`super::scoring::ScoreBreakdown`] normally comes from embedding
//! cosine similarity. When a query has no usable embedding — no service configured, embedding
//! generation failed, or the caller passed an empty vector — this module supplies a bounded,
//! deterministic stand-in so the component keeps discriminating between patterns instead of
//! collapsing to a constant.

use crate::embeddings::semantic_text::{create_query_text, pattern_to_text};
use crate::patterns::Pattern;
use crate::types::TaskContext;
use std::collections::HashSet;

/// Neutral score returned when there is nothing to compare.
///
/// Preserves the historical behaviour for degenerate input (blank query, context-free
/// pattern) so such a pattern keeps its previous ranking instead of dropping out.
pub(crate) const NEUTRAL_SIMILARITY: f32 = 0.5;

/// Words the text builders emit as field labels rather than as content.
///
/// [`pattern_to_text`] and [`create_query_text`] write `domain: …`, `language: …`,
/// `framework: …`, `tags: …`, `complexity: …`. Counting those label tokens as matches
/// would credit every pattern for text the caller never asked about.
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
/// Scores the same two strings the provider would have been handed — `create_query_text`
/// and `pattern_to_text` — so a pattern mentioning the query's terms outranks one that does
/// not. Bounded and local: the fraction of the query's distinct terms the pattern mentions,
/// `0.0..=1.0`. No corpus statistics are available per call, so this makes no IDF claim;
/// context agreement stays the job of [`super::scoring::calculate_context_match`].
pub fn calculate_keyword_similarity(query: &str, pattern: &Pattern, context: &TaskContext) -> f32 {
    let query_tokens = tokenize(&create_query_text(query, context));
    let pattern_tokens = tokenize(&pattern_to_text(pattern));

    // Only terms the caller typed count as lexical evidence. `create_query_text` appends the
    // context fields, so a blank query would otherwise credit every pattern sharing that
    // context with a full match.
    if tokenize(query).is_empty() || pattern_tokens.is_empty() {
        return NEUTRAL_SIMILARITY;
    }

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
        let context = query_context(&["async", "rest"]);

        let matched =
            calculate_keyword_similarity("wire up an async rest server", &matching, &context);
        let unmatched =
            calculate_keyword_similarity("wire up an async rest server", &unrelated, &context);

        assert!(
            matched > unmatched,
            "a pattern carrying the query's terms must outrank an unrelated one, \
             got {matched} vs {unmatched}"
        );
        assert!(
            matched >= 0.5,
            "the pattern naming most of the query's terms should cover at least half of them, \
             got {matched}"
        );
        assert!(
            matched - unmatched >= 0.2,
            "the gap has to be wide enough to reorder a result set, got {matched} vs {unmatched}"
        );
        // The unrelated pattern still shares the context's domain and language, which is real
        // but weak evidence — it must not land at the neutral constant the stub returned.
        assert!(
            unmatched < NEUTRAL_SIMILARITY,
            "context-only overlap must not earn a neutral score, got {unmatched}"
        );
    }

    #[test]
    fn keyword_similarity_is_case_and_punctuation_insensitive() {
        let pattern = pattern_with(&["axum", "tokio"], &["async"]);
        let context = query_context(&["async"]);

        assert_eq!(
            calculate_keyword_similarity("Async REST server", &pattern, &context),
            calculate_keyword_similarity("async, rest. server!", &pattern, &context),
            "case and punctuation must not change the score"
        );
    }

    #[test]
    fn keyword_similarity_deduplicates_repeated_terms() {
        let pattern = pattern_with(&["axum", "tokio"], &["async"]);
        let context = query_context(&["async"]);

        assert_eq!(
            calculate_keyword_similarity("async async async axum", &pattern, &context),
            calculate_keyword_similarity("async axum", &pattern, &context),
            "repeating a term must not inflate the score"
        );
    }

    #[test]
    fn keyword_similarity_is_bounded_and_deterministic() {
        let pattern = pattern_with(&["axum", "tokio", "postgres"], &["async", "rest"]);
        let context = query_context(&["async"]);

        let first =
            calculate_keyword_similarity("async rest axum postgres server", &pattern, &context);
        let second =
            calculate_keyword_similarity("async rest axum postgres server", &pattern, &context);

        assert_eq!(first, second, "same inputs must give the same score");
        assert!(
            (0.0..=1.0).contains(&first),
            "score {first} escaped the bounded 0.0..=1.0 range"
        );
    }

    /// Degenerate input keeps the historical neutral value instead of dropping the pattern.
    #[test]
    fn empty_inputs_fall_back_to_the_neutral_constant() {
        let pattern = pattern_with(&["axum"], &["async"]);
        let context = query_context(&["async"]);

        assert_eq!(
            calculate_keyword_similarity("", &pattern, &context),
            NEUTRAL_SIMILARITY,
            "a blank query has no typed terms to match on, even with a rich context"
        );
        assert_eq!(
            calculate_keyword_similarity("...  !!!", &pattern, &context),
            NEUTRAL_SIMILARITY,
            "punctuation-only input tokenises to nothing"
        );
    }
}
