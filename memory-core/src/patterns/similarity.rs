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
/// Calculate edit distance (Levenshtein) between two slices of equal element types.
///
/// # Implementation notes
/// Uses a single-row DP buffer sized to the shorter sequence ($O(\min(N, M))$ space).
/// Employs a stack-allocated buffer (`[usize; 129]`) for lengths up to 128 elements,
/// eliminating heap allocations completely for typical short inputs (e.g. byte slices
/// or string sequence slices).
fn slice_edit_distance<T: PartialEq>(seq1: &[T], seq2: &[T]) -> usize {
    let (s1, s2) = if seq1.len() < seq2.len() {
        (seq1, seq2)
    } else {
        (seq2, seq1)
    };

    let len1 = s1.len();
    let len2 = s2.len();

    let mut stack_dp = [0usize; 129];
    let mut heap_dp;
    let dp: &mut [usize] = if len1 < 129 {
        for (i, elem) in stack_dp[..=len1].iter_mut().enumerate() {
            *elem = i;
        }
        &mut stack_dp[..=len1]
    } else {
        heap_dp = (0..=len1).collect::<Vec<_>>();
        &mut heap_dp
    };

    for j in 1..=len2 {
        let mut pre_dp = dp[0];
        dp[0] = j;
        let c2 = &s2[j - 1];
        for i in 1..=len1 {
            let temp = dp[i];
            let cost = usize::from(&s1[i - 1] != c2);
            dp[i] = (dp[i] + 1).min(dp[i - 1] + 1).min(pre_dp + cost);
            pre_dp = temp;
        }
    }

    dp[len1]
}

fn edit_distance(seq1: &[String], seq2: &[String]) -> usize {
    slice_edit_distance(seq1, seq2)
}

/// Calculate similarity between two strings using normalized edit distance.
///
/// # Implementation notes
/// For ASCII inputs (the common case for system actions, tools, and tags),
/// an ASCII fast path processes raw byte slices via `slice_edit_distance`,
/// bypassing `Vec<char>` allocations and using stack buffers.
///
/// For multibyte UTF-8 inputs, only the *shorter* string — by character count — is
/// collected into a `Vec<char>`; the longer string is streamed via `.chars()`.
pub(super) fn string_similarity(s1: &str, s2: &str) -> f32 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    // ASCII fast path: avoids `Vec<char>` collection and uses stack-buffered slice edit distance.
    if s1.is_ascii() && s2.is_ascii() {
        let b1 = s1.as_bytes();
        let b2 = s2.as_bytes();
        let distance = slice_edit_distance(b1, b2);
        let max_len = b1.len().max(b2.len());
        return 1.0 - (distance as f32 / max_len as f32);
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

    // Tags overlap (weight: 0.3). At least one side is non-empty here, so the
    // union is non-empty and the Jaccard ratio is well-defined.
    if !ctx1.tags.is_empty() || !ctx2.tags.is_empty() {
        let (intersection_count, union_count) =
            calculate_tag_jaccard_counts(&ctx1.tags, &ctx2.tags);
        let jaccard = intersection_count as f32 / union_count as f32;
        score += jaccard * 0.3;
    }
    weight_sum += 0.3;

    if weight_sum > 0.0 {
        score / weight_sum
    } else {
        0.0
    }
}

/// Calculate set intersection and set union sizes for tag lists.
///
/// # Optimization
/// Avoids the intermediate `Vec` and `HashSet` heap allocations of the
/// previous implementation for typical small tag sets (N, M <= 16) using
/// allocation-free linear scans: O(N*M) time with O(1) extra space (at most
/// 256 string comparisons at the threshold). Larger sets fall back to
/// `HashSet<&str>` lookup at O(N+M) time, matching the previous
/// implementation's asymptotics.
///
/// Duplicate tags are counted once on each side (true Jaccard semantics);
/// the previous implementation counted duplicate occurrences in the first
/// list against a deduplicated union, which could yield ratios above 1.0.
fn calculate_tag_jaccard_counts(tags1: &[String], tags2: &[String]) -> (usize, usize) {
    if tags1.len() <= 16 && tags2.len() <= 16 {
        let mut common = 0;
        let mut unique1 = 0;

        for (idx, t1) in tags1.iter().enumerate() {
            if tags1[..idx].iter().any(|prev| prev == t1) {
                continue;
            }
            unique1 += 1;
            if tags2.iter().any(|t2| t2 == t1) {
                common += 1;
            }
        }

        let mut unique2_only = 0;
        for (idx, t2) in tags2.iter().enumerate() {
            if tags2[..idx].iter().any(|prev| prev == t2) {
                continue;
            }
            if !tags1.iter().any(|t1| t1 == t2) {
                unique2_only += 1;
            }
        }

        (common, unique1 + unique2_only)
    } else {
        let set1: std::collections::HashSet<&str> = tags1.iter().map(String::as_str).collect();
        let set2: std::collections::HashSet<&str> = tags2.iter().map(String::as_str).collect();
        let common = set1.intersection(&set2).count();
        let union_size = set1.union(&set2).count();
        (common, union_size)
    }
}

#[cfg(test)]
mod tests;
