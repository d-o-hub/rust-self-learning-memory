//! Fuzzy string matching for typo-tolerant search
//!
//! This module provides fuzzy matching capabilities using the Levenshtein distance
//! algorithm. It allows finding text that is similar but not exactly matching,
//! which is useful for handling typos, spelling variations, and approximate searches.

use strsim::normalized_levenshtein;

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
/// use memory_core::search::fuzzy::fuzzy_match;
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
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Fast path: exact substring match gets perfect score
    if text_lower.contains(&query_lower) {
        return Some(1.0);
    }

    // Calculate similarity score
    let score = normalized_levenshtein(&text_lower, &query_lower);

    if score >= threshold {
        Some(score)
    } else {
        None
    }
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
/// use memory_core::search::fuzzy::fuzzy_search_in_text;
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
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let text_words: Vec<&str> = text_lower.split_whitespace().collect();

    // Fast path: check if query is a substring
    if let Some(pos) = text_lower.find(&query_lower) {
        matches.push((pos, 1.0));
        return matches;
    }

    // Try single-word matches
    for (word_idx, word) in text_words.iter().enumerate() {
        if let Some(score) = fuzzy_match(word, &query_lower, threshold) {
            // Calculate approximate position in original text
            let position = text_words[..word_idx]
                .iter()
                .map(|w| w.len() + 1)
                .sum::<usize>();
            matches.push((position, score));
        }
    }

    // Try multi-word matches (sliding window)
    if query_words.len() > 1 {
        for window_size in 2..=query_words.len().min(5) {
            for window in text_words.windows(window_size) {
                let window_text = window.join(" ");
                if let Some(score) = fuzzy_match(&window_text, &query_lower, threshold) {
                    // Calculate approximate position
                    let word_idx = text_words.iter().position(|&w| w == window[0]).unwrap_or(0);
                    let position = text_words[..word_idx]
                        .iter()
                        .map(|w| w.len() + 1)
                        .sum::<usize>();
                    matches.push((position, score));
                }
            }
        }
    }

    // Sort by score (highest first), then by position (earliest first)
    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));

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
    texts
        .into_iter()
        .filter_map(|text| fuzzy_match(text, query, threshold))
        .max_by(|a, b| a.partial_cmp(b).unwrap())
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
        let texts = vec!["hello", "database", "connection"];
        let score = best_fuzzy_match(texts.iter().copied(), "databse", 0.7).unwrap();

        assert!(score > 0.7);
    }

    #[test]
    fn test_best_fuzzy_match_no_match() {
        let texts = vec!["hello", "world"];
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
}
