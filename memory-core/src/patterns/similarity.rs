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
/// # Optimization
/// Uses a single-row DP buffer to reduce space complexity to O(min(N, M)).
/// This eliminates one entire vector allocation (from 2 * (min(N, M) + 1) to (min(N, M) + 1))
/// and avoids O(M) std::mem::swap operations, improving CPU cache locality.
fn edit_distance(seq1: &[String], seq2: &[String]) -> usize {
    // Ensure s1 is the shorter sequence for O(min(N, M)) space
    let (s1, s2) = if seq1.len() < seq2.len() {
        (seq1, seq2)
    } else {
        (seq2, seq1)
    };

    let len1 = s1.len();
    let len2 = s2.len();

    // After swapping, len1 <= len2, so len1 == 0 implies len2 == 0 too.
    if len1 == 0 {
        return len2;
    }

    let mut dp: Vec<usize> = (0..=len1).collect();

    for j in 1..=len2 {
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
/// # Optimization
/// 1. We determine the shorter string by byte length (O(1) heuristic) and only collect
///    its characters into a `Vec<char>`. The longer string is streamed via `.chars()`.
///    This cuts the space allocated for character collections from O(N + M) to O(min(N, M)),
///    and eliminates a `Vec<char>` allocation for the longer string entirely.
/// 2. We use a single-row DP buffer for Levenshtein distance calculation to keep
///    space complexity of the algorithm at O(min(N, M)) with a single Vec allocation.
pub(super) fn string_similarity(s1: &str, s2: &str) -> f32 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    // Heuristically assume shorter byte-length corresponds to fewer characters to minimize collection size.
    let (short, long) = if s1.len() <= s2.len() {
        (s1, s2)
    } else {
        (s2, s1)
    };

    let s1_chars: Vec<char> = short.chars().collect();
    let (distance, len2) = char_edit_distance_streamed(&s1_chars, long);
    let max_len = s1_chars.len().max(len2);

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate edit distance (Levenshtein) of a character slice against a streamed string.
///
/// # Optimization
/// 1. Uses a single-row DP buffer `dp` of size `len1 + 1` to reduce auxiliary space complexity
///    to O(min(N, M)), allocating only one `Vec<usize>` instead of two.
/// 2. Streams the characters of the second string `s2` without collecting them to a Vec,
///    which reduces the total allocation to a single vector of characters `s1_chars` plus `dp` row,
///    down from storing M characters.
fn char_edit_distance_streamed(s1: &[char], s2: &str) -> (usize, usize) {
    let len1 = s1.len();
    if len1 == 0 {
        let len2 = s2.chars().count();
        return (len2, len2);
    }

    let mut dp: Vec<usize> = (0..=len1).collect();
    let mut len2 = 0;

    for c2 in s2.chars() {
        len2 += 1;
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
}
