//! Confidence-gated Tier 4 fallback policy (issue #968).
//!
//! Split out of `mod.rs` to keep individual source files under the 500 LOC
//! quality gate. This module is feature-independent: the decision is a pure
//! function of the policy and the local result list, so its unit tests run
//! with and without the `csm` feature.

use super::types::{FallbackPolicy, FallbackReason};

/// Outcome of applying a [`FallbackPolicy`] to one local result list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallbackDecision {
    /// Number of Tier 4 API calls to count (0 or 1).
    pub api_calls: u32,
    /// Why Tier 4 was or was not invoked.
    pub reason: FallbackReason,
    /// Top local score (`0.0` when the result list is empty).
    pub top_score: f32,
    /// Winner margin: top score minus runner-up (absent score counts as
    /// `0.0`, so a lone result reports its full score as the margin).
    pub score_margin: f32,
}

/// Compute confidence inputs for a local result list.
///
/// Returns `(top_score, score_margin, confident)` where `confident` is true
/// only for a non-empty list whose top score reaches `confidence_threshold`
/// and whose winner margin reaches `minimum_margin`. The scan is `O(k)` and
/// makes no ordering assumption about `results`.
#[must_use]
pub fn local_confidence(
    results: &[(String, f32)],
    confidence_threshold: f32,
    minimum_margin: f32,
) -> (f32, f32, bool) {
    let mut top = 0.0_f32;
    let mut runner_up = 0.0_f32;
    for (_, score) in results {
        if *score > top {
            runner_up = top;
            top = *score;
        } else if *score > runner_up {
            runner_up = *score;
        }
    }
    let margin = top - runner_up;
    let confident = !results.is_empty() && top >= confidence_threshold && margin >= minimum_margin;
    (top, margin, confident)
}

/// Apply a [`FallbackPolicy`] to the best local results.
///
/// This is the single decision point for Tier 4 escalation: callers that
/// already hold sufficient local results report
/// [`FallbackReason::LocalTierSufficient`] themselves and only consult this
/// function when the count-based tier rules did not suffice.
#[must_use]
pub fn decide_fallback(
    policy: FallbackPolicy,
    confidence_threshold: f32,
    minimum_margin: f32,
    results: &[(String, f32)],
) -> FallbackDecision {
    let (top_score, score_margin, confident) =
        local_confidence(results, confidence_threshold, minimum_margin);
    match policy {
        FallbackPolicy::AlwaysEmbed => FallbackDecision {
            api_calls: 1,
            reason: FallbackReason::AlwaysEmbedPolicy,
            top_score,
            score_margin,
        },
        FallbackPolicy::LocalOnly => FallbackDecision {
            api_calls: 0,
            reason: FallbackReason::LocalOnlyPolicy,
            top_score,
            score_margin,
        },
        FallbackPolicy::Adaptive => {
            if results.is_empty() {
                FallbackDecision {
                    api_calls: 1,
                    reason: FallbackReason::NoLocalResults,
                    top_score,
                    score_margin,
                }
            } else if confident {
                FallbackDecision {
                    api_calls: 0,
                    reason: FallbackReason::LocalConfident,
                    top_score,
                    score_margin,
                }
            } else {
                FallbackDecision {
                    api_calls: 1,
                    reason: FallbackReason::InsufficientConfidence,
                    top_score,
                    score_margin,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_results_are_never_confident() {
        let (top, margin, confident) = local_confidence(&[], 0.78, 0.08);

        assert!((top - 0.0).abs() < f32::EPSILON);
        assert!((margin - 0.0).abs() < f32::EPSILON);
        assert!(!confident);
    }

    #[test]
    fn lone_strong_result_is_confident_with_full_score_margin() {
        let results = vec![("ep1".to_string(), 0.95)];
        let (top, margin, confident) = local_confidence(&results, 0.78, 0.08);

        assert!((top - 0.95).abs() < f32::EPSILON);
        assert!((margin - 0.95).abs() < f32::EPSILON);
        assert!(confident);
    }

    #[test]
    fn weak_top_score_is_not_confident() {
        let results = vec![("ep1".to_string(), 0.5), ("ep2".to_string(), 0.2)];
        let (_, _, confident) = local_confidence(&results, 0.78, 0.08);

        assert!(!confident);
    }

    #[test]
    fn tied_results_are_not_confident_despite_high_scores() {
        let results = vec![("ep1".to_string(), 0.9), ("ep2".to_string(), 0.9)];
        let (top, margin, confident) = local_confidence(&results, 0.78, 0.08);

        assert!((top - 0.9).abs() < f32::EPSILON);
        assert!((margin - 0.0).abs() < f32::EPSILON);
        assert!(!confident);
    }

    #[test]
    fn clear_winner_is_confident() {
        let results = vec![("ep1".to_string(), 0.9), ("ep2".to_string(), 0.7)];
        let (_, margin, confident) = local_confidence(&results, 0.78, 0.08);

        assert!((margin - 0.2).abs() < 1e-6);
        assert!(confident);
    }

    #[test]
    fn adaptive_rescues_confident_results_and_escalates_the_rest() {
        let confident = vec![("ep1".to_string(), 0.95)];
        let decision = decide_fallback(FallbackPolicy::Adaptive, 0.78, 0.08, &confident);
        assert_eq!(decision.api_calls, 0);
        assert_eq!(decision.reason, FallbackReason::LocalConfident);

        let weak = vec![("ep1".to_string(), 0.5), ("ep2".to_string(), 0.45)];
        let decision = decide_fallback(FallbackPolicy::Adaptive, 0.78, 0.08, &weak);
        assert_eq!(decision.api_calls, 1);
        assert_eq!(decision.reason, FallbackReason::InsufficientConfidence);

        let empty: Vec<(String, f32)> = Vec::new();
        let decision = decide_fallback(FallbackPolicy::Adaptive, 0.78, 0.08, &empty);
        assert_eq!(decision.api_calls, 1);
        assert_eq!(decision.reason, FallbackReason::NoLocalResults);
    }

    #[test]
    fn always_embed_counts_a_call_even_on_confident_local_results() {
        let confident = vec![("ep1".to_string(), 0.95)];
        let decision = decide_fallback(FallbackPolicy::AlwaysEmbed, 0.78, 0.08, &confident);

        assert_eq!(decision.api_calls, 1);
        assert_eq!(decision.reason, FallbackReason::AlwaysEmbedPolicy);
    }

    #[test]
    fn local_only_never_counts_a_call_even_without_results() {
        let empty: Vec<(String, f32)> = Vec::new();
        let decision = decide_fallback(FallbackPolicy::LocalOnly, 0.78, 0.08, &empty);

        assert_eq!(decision.api_calls, 0);
        assert_eq!(decision.reason, FallbackReason::LocalOnlyPolicy);
    }
}
