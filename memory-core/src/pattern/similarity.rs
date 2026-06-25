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

/// Calculate edit distance (Levenshtein) between two sequences
///
/// Optimization: Uses O(min(N, M)) space by only keeping track of the current row.
/// This reduces memory allocation from O(N*M) to O(min(N, M)) and allocations from O(N) to O(1).
fn edit_distance(seq1: &[String], seq2: &[String]) -> usize {
    // Ensure seq2 is the shorter sequence to minimize space usage
    let (seq1, seq2) = if seq1.len() < seq2.len() {
        (seq2, seq1)
    } else {
        (seq1, seq2)
    };

    let len1 = seq1.len();
    let len2 = seq2.len();

    if len2 == 0 {
        return len1;
    }

    // Initial row: distances from empty seq1 to prefixes of seq2
    let mut row: Vec<usize> = (0..=len2).collect();

    for i in 1..=len1 {
        let mut prev_diag = row[0]; // Represents matrix[i-1][j-1]
        row[0] = i; // Represents matrix[i][0]
        for j in 1..=len2 {
            let old_row_j = row[j]; // matrix[i-1][j] before update
            let cost = usize::from(seq1[i - 1] != seq2[j - 1]);

            // row[j] is matrix[i-1][j] (deletion)
            // row[j-1] is matrix[i][j-1] (insertion)
            // prev_diag is matrix[i-1][j-1] (substitution)
            row[j] = (row[j] + 1).min(row[j - 1] + 1).min(prev_diag + cost);

            prev_diag = old_row_j;
        }
    }

    row[len2]
}

/// Calculate similarity between two strings using normalized edit distance
pub(super) fn string_similarity(s1: &str, s2: &str) -> f32 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    let distance = char_edit_distance(&chars1, &chars2);
    let max_len = chars1.len().max(chars2.len());

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate edit distance for character sequences
///
/// Optimization: Uses O(min(N, M)) space by only keeping track of the current row.
fn char_edit_distance(chars1: &[char], chars2: &[char]) -> usize {
    let (chars1, chars2) = if chars1.len() < chars2.len() {
        (chars2, chars1)
    } else {
        (chars1, chars2)
    };

    let len1 = chars1.len();
    let len2 = chars2.len();

    if len2 == 0 {
        return len1;
    }

    let mut row: Vec<usize> = (0..=len2).collect();

    for i in 1..=len1 {
        let mut prev_diag = row[0];
        row[0] = i;
        for j in 1..=len2 {
            let old_row_j = row[j];
            let cost = usize::from(chars1[i - 1] != chars2[j - 1]);
            row[j] = (row[j] + 1).min(row[j - 1] + 1).min(prev_diag + cost);
            prev_diag = old_row_j;
        }
    }

    row[len2]
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
