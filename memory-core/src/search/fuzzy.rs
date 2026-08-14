//! Fuzzy string matching for typo-tolerant search
//!
//! This module provides fuzzy matching capabilities using the Levenshtein distance
//! algorithm. It allows finding text that is similar but not exactly matching,
//! which is useful for handling typos, spelling variations, and approximate searches.

use std::borrow::Cow;
use strsim::normalized_levenshtein;

/// Avoid memory allocation if the string is already lowercase.
#[inline]
fn to_lowercase_cow(s: &str) -> Cow<'_, str> {
    if s.chars().any(|c| c.is_uppercase()) {
        Cow::Owned(s.to_lowercase())
    } else {
        Cow::Borrowed(s)
    }
}

/// Internal helper for fuzzy matching with pre-lowercased strings.
fn fuzzy_match_lowercased(text_lower: &str, query_lower: &str, threshold: f64) -> Option<f64> {
    // Fast path: exact substring match gets perfect score
    if text_lower.contains(query_lower) {
        return Some(1.0);
    }

    // Calculate similarity score
    let score = normalized_levenshtein(text_lower, query_lower);

    if score >= threshold {
        Some(score)
    } else {
        None
    }
}

/// Perform fuzzy matching between a query and text
///
/// Returns the similarity score (0.0 to 1.0) if the text matches the query
/// above the given threshold. Returns `None` if below threshold.
///
/// # Arguments
///
/// * `text` - The text to search in
/// * `query` - The search query
/// * `threshold` - Minimum similarity score (0.0 to 1.0)
///
/// # Returns
///
/// `Some(score)` if match quality >= threshold, `None` otherwise
///
/// # Examples
///
/// ```
/// use do_memory_core::search::fuzzy::fuzzy_match;
///
/// // Exact match
/// assert_eq!(fuzzy_match("database", "database", 0.8), Some(1.0));
///
/// // Close match (typo)
/// let score = fuzzy_match("database", "databse", 0.8).unwrap();
/// assert!(score > 0.8);
///
/// // Too different
/// assert_eq!(fuzzy_match("database", "xyz", 0.8), None);
/// ```
#[must_use]
pub fn fuzzy_match(text: &str, query: &str, threshold: f64) -> Option<f64> {
    let text_lower = to_lowercase_cow(text);
    let query_lower = to_lowercase_cow(query);

    fuzzy_match_lowercased(&text_lower, &query_lower, threshold)
}

/// Search for fuzzy matches within a text body
///
/// This function searches for the query within a larger text body,
/// checking each word and word combination for fuzzy matches.
///
/// # Arguments
///
/// * `text` - The text to search in
/// * `query` - The search query
/// * `threshold` - Minimum similarity score (0.0 to 1.0)
///
/// # Returns
///
/// A vector of tuples containing (position, `similarity_score`) for each match
///
/// # Examples
///
/// ```
/// use do_memory_core::search::fuzzy::fuzzy_search_in_text;
///
/// let text = "This is a database connection example";
/// let matches = fuzzy_search_in_text(text, "databse", 0.8);
///
/// assert!(!matches.is_empty());
/// assert!(matches[0].1 > 0.8); // similarity score
/// ```
#[must_use]
pub fn fuzzy_search_in_text(text: &str, query: &str, threshold: f64) -> Vec<(usize, f64)> {
    let mut matches = Vec::new();
    let text_lower = to_lowercase_cow(text);
    let query_lower = to_lowercase_cow(query);
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let text_words: Vec<&str> = text_lower.split_whitespace().collect();

    // Fast path: check if query is a substring
    if let Some(pos) = text_lower.find(query_lower.as_ref()) {
        matches.push((pos, 1.0));
        return matches;
    }

    // Cache the base pointer to avoid pointer arithmetic overhead in the loop
    let base_ptr = text_lower.as_ptr() as usize;

    // Try single-word matches
    for word in &text_words {
        if let Some(score) = fuzzy_match_lowercased(word, &query_lower, threshold) {
            // Calculate exact position in the lowercased string in O(1)
            let position = word.as_ptr() as usize - base_ptr;
            matches.push((position, score));
        }
    }

    // Try multi-word matches (sliding window)
    if query_words.len() > 1 {
        for window_size in 2..=query_words.len().min(5) {
            for window in text_words.windows(window_size) {
                // Normalize whitespace: split_whitespace collapses irregular spacing,
                // so join with single spaces to match query normalization.
                let window_text = window.join(" ");
                // O(1) position via pointer subtraction into the lowercased string
                let start = window[0].as_ptr() as usize - base_ptr;

                if let Some(score) = fuzzy_match_lowercased(&window_text, &query_lower, threshold)
                {
                    matches.push((start, score));
                }
            }
        }
    }

    // Sort by score (highest first), then by position (earliest first)
    // Use unwrap_or(Ordering::Equal) to handle NaN safely (treat NaN as equal)
    matches.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });

    // Deduplicate matches that are too close together
    let mut deduped = Vec::new();
    for (pos, score) in matches {
        if deduped.is_empty()
            || deduped
                .iter()
                .all(|(p, _)| (*p as i64 - pos as i64).abs() > 5)
        {
            deduped.push((pos, score));
        }
    }

    deduped
}

/// Calculate the best fuzzy match score for a query across multiple text fields
///
/// This is a helper function for multi-field search that returns the highest
/// similarity score found across all provided text fields.
///
/// # Arguments
///
/// * `texts` - Iterator of text strings to search in
/// * `query` - The search query
/// * `threshold` - Minimum similarity score (0.0 to 1.0)
///
/// # Returns
///
/// The best (highest) similarity score found, or `None` if no matches
#[must_use]
pub fn best_fuzzy_match<'a, I>(texts: I, query: &str, threshold: f64) -> Option<f64>
where
    I: IntoIterator<Item = &'a str>,
{
    let query_lower = to_lowercase_cow(query);
    texts
        .into_iter()
        .filter_map(|text| {
            let text_lower = to_lowercase_cow(text);
            fuzzy_match_lowercased(&text_lower, &query_lower, threshold)
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert_eq!(fuzzy_match("database", "database", 0.8), Some(1.0));
        assert_eq!(fuzzy_match("hello world", "hello", 0.8), Some(1.0));
    }

    #[test]
    fn test_fuzzy_match_typo() {
        // Common typos should match
        let score = fuzzy_match("database", "databse", 0.7).unwrap();
        assert!(score > 0.7);

        let score = fuzzy_match("connection", "conection", 0.7).unwrap();
        assert!(score > 0.7);
    }

    #[test]
    fn test_fuzzy_match_below_threshold() {
        // Very different strings should not match
        assert_eq!(fuzzy_match("database", "xyz", 0.8), None);
        assert_eq!(fuzzy_match("hello", "goodbye", 0.8), None);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(fuzzy_match("Database", "DATABASE", 0.8), Some(1.0));
        assert_eq!(fuzzy_match("Hello World", "hello world", 0.8), Some(1.0));
    }

    #[test]
    fn test_fuzzy_search_in_text() {
        let text = "This is a database connection example";
        let matches = fuzzy_search_in_text(text, "databse", 0.7);

        assert!(!matches.is_empty());
        assert!(matches[0].1 > 0.7);
    }

    #[test]
    fn test_fuzzy_search_exact_substring() {
        let text = "This is a database connection example";
        let matches = fuzzy_search_in_text(text, "database", 0.8);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1, 1.0);
    }

    #[test]
    fn test_fuzzy_search_multi_word() {
        let text = "This is a database connection example";
        let matches = fuzzy_search_in_text(text, "databse conection", 0.7);

        assert!(!matches.is_empty());
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let text = "This is a database connection example";
        let matches = fuzzy_search_in_text(text, "xyz", 0.8);

        assert!(matches.is_empty());
    }

    #[test]
    fn test_best_fuzzy_match() {
        let texts = ["hello", "database", "connection"];
        let score = best_fuzzy_match(texts.iter().copied(), "databse", 0.7).unwrap();

        assert!(score > 0.7);
    }

    #[test]
    fn test_best_fuzzy_match_no_match() {
        let texts = ["hello", "world"];
        let score = best_fuzzy_match(texts.iter().copied(), "xyz", 0.8);

        assert_eq!(score, None);
    }

    #[test]
    fn test_fuzzy_match_empty_strings() {
        // Empty strings match perfectly
        assert_eq!(fuzzy_match("", "", 0.8), Some(1.0));
        // Text searching for empty query should return 1.0 (substring match)
        assert_eq!(fuzzy_match("text", "", 0.8), Some(1.0));
        // Empty text can't contain non-empty query
        assert_eq!(fuzzy_match("", "text", 0.8), None);
    }

    #[test]
    fn test_fuzzy_search_special_characters() {
        let text = "error: database-connection failed!";
        // The word "database-connection" should fuzzy match "databse"
        // We need to adjust the threshold since the hyphenated word is longer
        let matches = fuzzy_search_in_text(text, "database", 0.7);

        // Should find exact substring match
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_fuzzy_search_exact_position_preservation() {
        // Verify that exact position matching works with multi-space text.
        let text = "this    is  a   database  connection";
        let matches = fuzzy_search_in_text(text, "database", 0.8);
        assert_eq!(matches.len(), 1);
        // "database" starts exactly at index 16 in the original string
        assert_eq!(matches[0].0, 16);
    }

    #[test]
    fn test_fuzzy_search_multi_word_irregular_whitespace() {
        // Multi-word query against text with irregular spacing.
        // The window join must normalize whitespace so the fuzzy match sees
        // "database connection" (single spaces) regardless of original spacing.
        let text = "this    is  a   database  connection  example";
        let matches = fuzzy_search_in_text(text, "database connection", 0.7);
        assert!(
            !matches.is_empty(),
            "multi-word query must match across irregular whitespace"
        );
        assert!(matches[0].1 > 0.7);
    }

    #[test]
    fn test_fuzzy_search_multi_word_window_position() {
        // Verify that multi-word window matching reports correct byte positions.
        let text = "alpha beta gamma delta epsilon";
        let matches = fuzzy_search_in_text(text, "gamma delta", 0.8);
        assert!(!matches.is_empty());
        // "gamma" starts at byte 11 in "alpha beta gamma delta epsilon" (6+5)
        assert_eq!(matches[0].0, 11);
    }

    #[test]
    fn test_fuzzy_search_single_word_text() {
        let matches = fuzzy_search_in_text("database", "database", 0.8);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, 0);
        assert_eq!(matches[0].1, 1.0);
    }

    #[test]
    fn test_fuzzy_search_empty_text() {
        let matches = fuzzy_search_in_text("", "query", 0.8);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        // Empty query is a substring of any text
        let matches = fuzzy_search_in_text("hello world", "", 0.8);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, 0);
        assert_eq!(matches[0].1, 1.0);
    }

    #[test]
    fn test_fuzzy_search_case_insensitive_positions() {
        // Cow lowercasing should not affect position accuracy
        let text = "Hello World Database Connection";
        let matches = fuzzy_search_in_text(text, "database", 0.8);
        assert_eq!(matches.len(), 1);
        // "database" starts at byte 12 in the lowercased string
        assert_eq!(matches[0].0, 12);
    }

    #[test]
    fn test_fuzzy_search_multi_word_high_threshold() {
        // At high thresholds, normalized whitespace is critical for matching.
        // Without whitespace normalization, "database  connection" vs
        // "database connection" would score lower due to extra space chars.
        let text = "found  database  connection  here";
        let matches = fuzzy_search_in_text(text, "database connection", 0.9);
        assert!(
            !matches.is_empty(),
            "must match at high threshold with normalized whitespace"
        );
    }
}
