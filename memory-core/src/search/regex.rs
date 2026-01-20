//! Regex pattern matching for advanced search
//!
//! This module provides regex pattern matching capabilities with safeguards
//! against `ReDoS` (Regular Expression Denial of Service) attacks.

use regex::{Match, Regex};

/// Maximum regex pattern length to prevent memory issues
const MAX_PATTERN_LENGTH: usize = 1000;

/// Maximum number of repetitions in a pattern to prevent catastrophic backtracking
const MAX_REPETITIONS: usize = 100;

/// Check if a captured repetition count exceeds the maximum
fn check_repetition_count(capture: Option<Match<'_>>, max: usize) -> Result<(), String> {
    if let Some(m) = capture {
        if let Ok(count) = m.as_str().parse::<usize>() {
            if count > max {
                return Err(format!(
                    "Repetition count {count} exceeds maximum {max}"
                ));
            }
        }
    }
    Ok(())
}

/// Validate a regex pattern before compilation
///
/// This function checks for potential `ReDoS` patterns and other security issues.
///
/// # Arguments
///
/// * `pattern` - The regex pattern to validate
///
/// # Returns
///
/// `Ok(())` if the pattern is safe, `Err` with a description if not
///
/// # Examples
///
/// ```
/// use memory_core::search::regex::validate_regex_pattern;
///
/// assert!(validate_regex_pattern("^test.*$").is_ok());
/// assert!(validate_regex_pattern("(a+)+b").is_err()); // Catastrophic backtracking
/// ```
pub fn validate_regex_pattern(pattern: &str) -> Result<(), String> {
    // Check pattern length
    if pattern.is_empty() {
        return Err("Regex pattern cannot be empty".to_string());
    }

    if pattern.len() > MAX_PATTERN_LENGTH {
        return Err(format!(
            "Regex pattern too long ({} chars, max {})",
            pattern.len(),
            MAX_PATTERN_LENGTH
        ));
    }

    // Check for nested quantifiers (potential catastrophic backtracking)
    // Patterns like (a+)+, (a*)*, (a+)*, etc.
    let nested_quantifiers = [
        (r"\([^)]*\+[^)]*\)\+", "nested + quantifiers"),
        (r"\([^)]*\*[^)]*\)\*", "nested * quantifiers"),
        (r"\([^)]*\+[^)]*\)\*", "nested +* quantifiers"),
        (r"\([^)]*\*[^)]*\)\+", "nested *+ quantifiers"),
        (r"\([^)]*\{[^}]+\}[^)]*\)\{", "nested {} quantifiers"),
    ];

    for (pattern_regex, description) in &nested_quantifiers {
        if let Ok(re) = Regex::new(pattern_regex) {
            if re.is_match(pattern) {
                return Err(format!(
                    "Pattern contains potentially dangerous {description}: {pattern}"
                ));
            }
        }
    }

    // Check for excessive repetition counts
    if let Ok(re) = Regex::new(r"\{(\d+),?(\d+)?\}") {
        for cap in re.captures_iter(pattern) {
            check_repetition_count(cap.get(1), MAX_REPETITIONS)?;
            check_repetition_count(cap.get(2), MAX_REPETITIONS)?;
        }
    }

    // Try to compile the regex to check for syntax errors
    Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {e}"))?;

    Ok(())
}

/// Search for regex pattern in text
///
/// # Arguments
///
/// * `text` - The text to search in
/// * `pattern` - The regex pattern (must be validated first)
///
/// # Returns
///
/// A vector of tuples containing (position, `matched_text`) for each match
///
/// # Examples
///
/// ```
/// use memory_core::search::regex::regex_search;
///
/// let matches = regex_search("error: database timeout", r"error.*timeout").unwrap();
/// assert_eq!(matches.len(), 1);
/// assert_eq!(matches[0].1, "error: database timeout");
/// ```
pub fn regex_search(text: &str, pattern: &str) -> Result<Vec<(usize, String)>, String> {
    // Validate pattern first
    validate_regex_pattern(pattern)?;

    // Compile regex
    let re = Regex::new(pattern).map_err(|e| format!("Failed to compile regex: {e}"))?;

    // Find all matches
    let matches: Vec<(usize, String)> = re
        .find_iter(text)
        .map(|m| (m.start(), m.as_str().to_string()))
        .collect();

    Ok(matches)
}

/// Search for regex pattern in text with case-insensitive matching
///
/// # Arguments
///
/// * `text` - The text to search in
/// * `pattern` - The regex pattern (must be validated first)
///
/// # Returns
///
/// A vector of tuples containing (position, `matched_text`) for each match
pub fn regex_search_case_insensitive(
    text: &str,
    pattern: &str,
) -> Result<Vec<(usize, String)>, String> {
    // Validate pattern first
    validate_regex_pattern(pattern)?;

    // Add case-insensitive flag if not already present
    let case_insensitive_pattern = if pattern.starts_with("(?i)") {
        pattern.to_string()
    } else {
        format!("(?i){pattern}")
    };

    // Compile regex
    let re = Regex::new(&case_insensitive_pattern)
        .map_err(|e| format!("Failed to compile regex: {e}"))?;

    // Find all matches
    let matches: Vec<(usize, String)> = re
        .find_iter(text)
        .map(|m| (m.start(), m.as_str().to_string()))
        .collect();

    Ok(matches)
}

/// Check if text matches a regex pattern
///
/// # Arguments
///
/// * `text` - The text to check
/// * `pattern` - The regex pattern
///
/// # Returns
///
/// `true` if the pattern matches, `false` otherwise
pub fn regex_matches(text: &str, pattern: &str) -> Result<bool, String> {
    validate_regex_pattern(pattern)?;
    let re = Regex::new(pattern).map_err(|e| format!("Failed to compile regex: {e}"))?;
    Ok(re.is_match(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple_patterns() {
        assert!(validate_regex_pattern("test").is_ok());
        assert!(validate_regex_pattern("^test$").is_ok());
        assert!(validate_regex_pattern(r"\d+").is_ok());
        assert!(validate_regex_pattern("error.*timeout").is_ok());
    }

    #[test]
    fn test_validate_empty_pattern() {
        assert!(validate_regex_pattern("").is_err());
    }

    #[test]
    fn test_validate_too_long_pattern() {
        let long_pattern = "a".repeat(MAX_PATTERN_LENGTH + 1);
        assert!(validate_regex_pattern(&long_pattern).is_err());
    }

    #[test]
    fn test_validate_nested_quantifiers() {
        // These patterns can cause catastrophic backtracking
        assert!(validate_regex_pattern("(a+)+").is_err());
        assert!(validate_regex_pattern("(a*)*").is_err());
        assert!(validate_regex_pattern("(a+)*").is_err());
    }

    #[test]
    fn test_validate_excessive_repetitions() {
        let pattern = format!("a{{{}}}", MAX_REPETITIONS + 1);
        assert!(validate_regex_pattern(&pattern).is_err());
    }

    #[test]
    fn test_validate_invalid_syntax() {
        assert!(validate_regex_pattern("(unclosed").is_err());
        assert!(validate_regex_pattern("[unclosed").is_err());
        assert!(validate_regex_pattern("(?P<incomplete").is_err());
    }

    #[test]
    fn test_regex_search() {
        let text = "error: database connection timeout";
        let matches = regex_search(text, r"error.*timeout").unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, 0);
        assert!(matches[0].1.contains("error"));
        assert!(matches[0].1.contains("timeout"));
    }

    #[test]
    fn test_regex_search_multiple_matches() {
        let text = "error1 and error2 and error3";
        let matches = regex_search(text, r"error\d+").unwrap();

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].1, "error1");
        assert_eq!(matches[1].1, "error2");
        assert_eq!(matches[2].1, "error3");
    }

    #[test]
    fn test_regex_search_no_matches() {
        let text = "success: everything works";
        let matches = regex_search(text, r"error.*timeout").unwrap();

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_regex_search_case_sensitive() {
        let text = "Error and error";
        let matches = regex_search(text, "error").unwrap();

        assert_eq!(matches.len(), 1); // Only lowercase "error"
        assert_eq!(matches[0].1, "error");
    }

    #[test]
    fn test_regex_search_case_insensitive() {
        let text = "Error and error and ERROR";
        let matches = regex_search_case_insensitive(text, "error").unwrap();

        assert_eq!(matches.len(), 3); // All variations
    }

    #[test]
    fn test_regex_matches() {
        assert!(regex_matches("test123", r"\w+\d+").unwrap());
        assert!(!regex_matches("test", r"\d+").unwrap());
    }

    #[test]
    fn test_regex_search_with_anchors() {
        let text = "start test end";
        let matches = regex_search(text, "^start").unwrap();
        assert_eq!(matches.len(), 1);

        let matches = regex_search(text, "end$").unwrap();
        assert_eq!(matches.len(), 1);

        let matches = regex_search(text, "^test").unwrap();
        assert_eq!(matches.len(), 0); // "test" is not at start
    }

    #[test]
    fn test_regex_search_with_groups() {
        let text = "email: test@example.com";
        let matches = regex_search(text, r"\w+@\w+\.\w+").unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].1, "test@example.com");
    }
}
