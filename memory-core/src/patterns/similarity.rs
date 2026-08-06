//! Similarity calculation utilities for patterns

use crate::types::TaskContext;

/// Calculate similarity between two sequences using normalized edit distance
pub(super) fn sequence_similarity(seq1: &[String], seq2: &[String]) -> f32 {
    if seq1.is_empty() && seq2.is_empty() {
        return 1.0;
    }
    if seq1.is_empty() || seq2.is_empty() {
        return 0.0;
    }

    let distance = edit_distance(seq1, seq2);
    let max_len = seq1.len().max(seq2.len());

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate edit distance (Levenshtein) between two sequences.
///
/// # Implementation notes
/// Uses a single-row DP buffer sized to the shorter sequence. Space is
/// O(min(N, M)) — the same asymptotic bound as the previous two-row rolling
/// buffer — but with one fewer `Vec` allocation and no per-row buffer swap,
/// which improves cache locality.
///
/// An empty shorter side needs no special case: with `len1 == 0` the buffer is
/// `[0]`, `dp[0]` accumulates `len2` across rows, the inner loop is skipped,
/// and `dp[len1] == len2` falls out of the loop naturally.
fn edit_distance(seq1: &[String], seq2: &[String]) -> usize {
    // Ensure s1 is the shorter sequence for O(min(N, M)) space
    let (s1, s2) = if seq1.len() < seq2.len() {
        (seq1, seq2)
    } else {
        (seq2, seq1)
    };

    let len1 = s1.len();
    let len2 = s2.len();

    let mut dp: Vec<usize> = (0..=len1).collect();

    for j in 1..=len2 {
        // `pre_dp` holds dp[i - 1] from the previous row (the diagonal).
        let mut pre_dp = dp[0];
        dp[0] = j;
        for i in 1..=len1 {
            let temp = dp[i];
            let cost = usize::from(s1[i - 1] != s2[j - 1]);
            dp[i] = (dp[i] + 1).min(dp[i - 1] + 1).min(pre_dp + cost);
            pre_dp = temp;
        }
    }

    dp[len1]
}

/// Calculate similarity between two strings using normalized edit distance.
///
/// # Implementation notes
/// Only the *shorter* string — by character count, not byte length — is
/// collected into a `Vec<char>`; the longer string is streamed via `.chars()`
/// and never materialized. This bounds character storage to O(min(N, M)) even
/// for multibyte UTF-8, where byte length would be a misleading proxy for the
/// number of characters.
pub(super) fn string_similarity(s1: &str, s2: &str) -> f32 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    // Compare by char count (`chars().count()` is O(N) with no allocation) so
    // the collected side is truly the shorter in characters.
    let (short, long) = if s1.chars().count() <= s2.chars().count() {
        (s1, s2)
    } else {
        (s2, s1)
    };

    let short_chars: Vec<char> = short.chars().collect();
    let (distance, long_len) = char_edit_distance_streamed(&short_chars, long);
    let max_len = short_chars.len().max(long_len);

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate edit distance (Levenshtein) of a character slice against a
/// streamed string, returning `(distance, len2)`.
///
/// # Implementation notes
/// - Uses a single-row DP buffer of size `len1 + 1`, so the only allocations
///   are the `dp` row and the caller-provided `s1` buffer.
/// - Streams the characters of `s2` via `.chars()` without collecting them,
///   avoiding an O(M) character allocation.
/// - The caller should pass the shorter character sequence as `s1` to keep the
///   DP row at O(min(N, M)). An empty `s1` needs no special case: `dp` is
///   `[0]`, `dp[0]` tracks `len2`, and the inner loop is skipped.
fn char_edit_distance_streamed(s1: &[char], s2: &str) -> (usize, usize) {
    let len1 = s1.len();
    let mut dp: Vec<usize> = (0..=len1).collect();
    let mut len2 = 0;

    for c2 in s2.chars() {
        len2 += 1;
        // `pre_dp` holds dp[i - 1] from the previous row (the diagonal).
        let mut pre_dp = dp[0];
        dp[0] = len2;

        for i in 1..=len1 {
            let temp = dp[i];
            let cost = usize::from(s1[i - 1] != c2);
            dp[i] = (dp[i] + 1).min(dp[i - 1] + 1).min(pre_dp + cost);
            pre_dp = temp;
        }
    }

    (dp[len1], len2)
}

/// Calculate similarity between two ToolSequence patterns
pub(super) fn tool_sequence_similarity(
    tools1: &[String],
    ctx1: &TaskContext,
    tools2: &[String],
    ctx2: &TaskContext,
) -> f32 {
    let sequence_similarity = sequence_similarity(tools1, tools2);
    let context_similarity = context_similarity(ctx1, ctx2);
    sequence_similarity * 0.7 + context_similarity * 0.3
}

/// Calculate similarity between two DecisionPoint patterns
pub(super) fn decision_point_similarity(
    cond1: &str,
    act1: &str,
    ctx1: &TaskContext,
    cond2: &str,
    act2: &str,
    ctx2: &TaskContext,
) -> f32 {
    let condition_sim = string_similarity(cond1, cond2);
    let action_sim = string_similarity(act1, act2);
    let context_sim = context_similarity(ctx1, ctx2);
    condition_sim * 0.4 + action_sim * 0.4 + context_sim * 0.2
}

/// Calculate similarity between two ErrorRecovery patterns
pub(super) fn error_recovery_similarity(
    err1: &str,
    steps1: &[String],
    ctx1: &TaskContext,
    err2: &str,
    steps2: &[String],
    ctx2: &TaskContext,
) -> f32 {
    let error_sim = string_similarity(err1, err2);
    let steps_sim = sequence_similarity(steps1, steps2);
    let context_sim = context_similarity(ctx1, ctx2);
    error_sim * 0.4 + steps_sim * 0.4 + context_sim * 0.2
}

/// Calculate similarity between two ContextPattern patterns
pub(super) fn context_pattern_similarity(
    feat1: &[String],
    rec1: &str,
    feat2: &[String],
    rec2: &str,
) -> f32 {
    let features_sim = sequence_similarity(feat1, feat2);
    let approach_sim = string_similarity(rec1, rec2);
    features_sim * 0.6 + approach_sim * 0.4
}

/// Calculate context similarity between two task contexts
pub(super) fn context_similarity(ctx1: &TaskContext, ctx2: &TaskContext) -> f32 {
    let mut score = 0.0;
    let mut weight_sum = 0.0;

    // Domain match (weight: 0.4)
    if ctx1.domain == ctx2.domain {
        score += 0.4;
    }
    weight_sum += 0.4;

    // Language match (weight: 0.3)
    match (&ctx1.language, &ctx2.language) {
        (Some(l1), Some(l2)) if l1 == l2 => score += 0.3,
        (None, None) => score += 0.15, // Partial credit for both being None
        _ => {}
    }
    weight_sum += 0.3;

    // Tags overlap (weight: 0.3)
    if !ctx1.tags.is_empty() || !ctx2.tags.is_empty() {
        let common_tags: Vec<_> = ctx1.tags.iter().filter(|t| ctx2.tags.contains(t)).collect();

        let total_unique_tags = ctx1
            .tags
            .iter()
            .chain(ctx2.tags.iter())
            .collect::<std::collections::HashSet<_>>()
            .len();

        if total_unique_tags > 0 {
            let jaccard = common_tags.len() as f32 / total_unique_tags as f32;
            score += jaccard * 0.3;
        }
    }
    weight_sum += 0.3;

    if weight_sum > 0.0 {
        score / weight_sum
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference: naive full-matrix Levenshtein over char slices.
    /// Independent of the optimized single-row DP, used for cross-checking.
    fn reference_levenshtein(a: &[char], b: &[char]) -> usize {
        reference_levenshtein_slice(a, b)
    }

    /// Reference: naive full-matrix Levenshtein over arbitrary element slices.
    fn reference_levenshtein_slice<T: PartialEq>(a: &[T], b: &[T]) -> usize {
        let (rows, cols) = (a.len() + 1, b.len() + 1);
        let mut matrix = vec![vec![0usize; cols]; rows];
        for (i, row) in matrix.iter_mut().enumerate() {
            row[0] = i;
        }
        for (j, cell) in matrix[0].iter_mut().enumerate() {
            *cell = j;
        }
        for i in 1..rows {
            for j in 1..cols {
                let cost = usize::from(a[i - 1] != b[j - 1]);
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        matrix[a.len()][b.len()]
    }

    #[test]
    fn test_sequence_similarity() {
        let seq1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let seq2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        assert_eq!(sequence_similarity(&seq1, &seq2), 1.0);

        let seq3 = vec!["a".to_string(), "b".to_string(), "d".to_string()];
        let sim = sequence_similarity(&seq1, &seq3);
        // 2 out of 3 match
        assert!(sim > 0.6 && sim < 0.7);
    }

    #[test]
    fn test_string_similarity() {
        assert_eq!(string_similarity("hello", "hello"), 1.0);
        assert_eq!(string_similarity("", ""), 1.0);
        assert_eq!(string_similarity("abc", ""), 0.0);

        // "hello" vs "hallo" - one character different
        let sim = string_similarity("hello", "hallo");
        assert!(sim > 0.7 && sim < 0.9);
    }

    #[test]
    fn test_context_similarity() {
        let ctx1 = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["async".to_string(), "http".to_string()],
            ..Default::default()
        };

        let ctx2 = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["async".to_string(), "rest".to_string()],
            ..Default::default()
        };

        let similarity = context_similarity(&ctx1, &ctx2);

        // Same domain, same language, some tag overlap
        assert!(similarity > 0.7);
    }

    #[test]
    fn test_edit_distance_matches_reference() {
        // Cases include empty sides, exact matches, substitutions, insertions,
        // deletions, asymmetry, and multibyte elements.
        let cases: &[(&[&str], &[&str])] = &[
            (&[], &[]),
            (&[], &["a"]),
            (&["a"], &[]),
            (&["a"], &["a"]),
            (&["a", "b", "c"], &["a", "b", "c"]),
            (&["a", "b", "c"], &["a", "b", "d"]),
            (&["x", "y"], &["a", "b", "c"]),
            (&["tool_a", "tool_b", "tool_c"], &["tool_a", "tool_b"]),
            (&["café"], &["cafe"]), // multibyte elements
        ];

        for (s1, s2) in cases {
            let v1: Vec<String> = s1.iter().map(|s| (*s).to_string()).collect();
            let v2: Vec<String> = s2.iter().map(|s| (*s).to_string()).collect();
            let expected = reference_levenshtein_slice(s1, s2);
            assert_eq!(
                edit_distance(&v1, &v2),
                expected,
                "distance({s1:?}, {s2:?})"
            );
            // Levenshtein is symmetric; also exercises both arg orderings.
            assert_eq!(
                edit_distance(&v2, &v1),
                expected,
                "distance({s2:?}, {s1:?})"
            );
        }
    }

    #[test]
    fn test_char_edit_distance_streamed_matches_reference() {
        // Includes empty sides, classic cases, and multibyte strings where byte
        // length and char count disagree.
        let cases: &[(&str, &str)] = &[
            ("", ""),
            ("", "hello"),
            ("hello", ""),
            ("hello", "hello"),
            ("hello", "hallo"),
            ("kitten", "sitting"),
            ("goodbye", "hi"),
            ("héllo", "hello"),
            ("ééé", "xyzw"),
            ("x", "yyyy"),
            ("yyyy", "x"),
        ];

        for (s1, s2) in cases {
            let s1_chars: Vec<char> = s1.chars().collect();
            let s2_chars: Vec<char> = s2.chars().collect();
            let expected = reference_levenshtein(&s1_chars, &s2_chars);
            let (distance, len2) = char_edit_distance_streamed(&s1_chars, s2);
            assert_eq!(distance, expected, "distance({s1:?}, {s2:?})");
            assert_eq!(len2, s2_chars.len(), "len2({s1:?}, {s2:?})");
        }
    }

    #[test]
    fn test_string_similarity_longer_first_and_unicode() {
        // First argument strictly longer in characters -> exercises the
        // short/long selection `else` branch (s1 longer than s2).
        assert_eq!(string_similarity("goodbye", "hi"), 0.0);

        // One substitution; max length 5 chars.
        assert_eq!(string_similarity("héllo", "hello"), 0.8);

        // No overlap, multibyte; char-count selection keeps the DP row minimal.
        assert_eq!(string_similarity("ééé", "xyzw"), 0.0);

        // Symmetry: swapping arguments must not change the result.
        for (a, b) in [("goodbye", "hi"), ("héllo", "hello"), ("ééé", "xyzw")] {
            assert_eq!(string_similarity(a, b), string_similarity(b, a));
        }
    }

    #[test]
    fn test_char_edit_distance_streamed_empty() {
        let s1: Vec<char> = vec![];
        let (dist, len2) = char_edit_distance_streamed(&s1, "hello");
        assert_eq!(dist, 5);
        assert_eq!(len2, 5);
    }

    #[test]
    fn test_edit_distance_empty() {
        let seq1: Vec<String> = vec![];
        let seq2: Vec<String> = vec![];
        assert_eq!(edit_distance(&seq1, &seq2), 0);

        let seq3 = vec!["a".to_string()];
        assert_eq!(edit_distance(&seq1, &seq3), 1);
    }
}
