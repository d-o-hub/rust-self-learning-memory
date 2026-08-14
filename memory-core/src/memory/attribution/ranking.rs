//! Feedback-to-ranking adaptation (ADR-082).
//!
//! Attribution feedback is reduced to a per-pattern learned weight that re-ranks
//! pattern recommendations. The weight is a deterministic reduction of the
//! in-process tracker plus capability-gated durable history — the Wilson
//! lower-bound success rate — so the index is idempotent (rebuildable),
//! replacement-safe (the tracker, merged last, is authoritative for
//! latest-feedback-per-session), and rollback-safe (drop-and-rebuild, no
//! destructive journal). After a cold restart the index is a pure function of
//! durable history.

use std::collections::HashMap;

use crate::memory::attribution::types::{RecommendationFeedback, RecommendationSession};
use crate::search::ranking::wilson_lower_bound;
use crate::types::TaskOutcome;

/// Z-score for the Wilson lower bound used as the learned weight (matches episode ranking).
pub const RANKING_WILSON_Z: f64 = 1.96; // == z_scores::CONFIDENCE_95
/// Strength of the learned re-rank term relative to base relevance.
pub const LEARNED_BOOST_SCALE: f32 = 0.25;
/// Candidate-pool overfetch factor used on the recommend path so a boosted
/// pattern can enter the top-N (re-rank runs before truncation).
pub const RECOMMEND_OVERFETCH_FACTOR: usize = 3;

/// Durable per-pattern ranking evidence derived from feedback.
///
/// Evidence is `(applied, succeeded)`: a pattern with no applied feedback
/// carries no learned weight, so exposure alone never boosts ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PatternRankingState {
    /// Times the pattern was applied per feedback (the Wilson sample size).
    pub applied: u64,
    /// Applied with outcome `Success | PartialSuccess`.
    pub succeeded: u64,
}

impl PatternRankingState {
    /// Wilson lower bound of p(success | applied); 0.0 when no applied evidence.
    #[must_use]
    pub fn weight(&self, z: f64) -> f64 {
        wilson_lower_bound(self.succeeded, self.applied, z)
    }
}

/// Derived index of learned pattern weights keyed by pattern id string.
///
/// Keys match `Pattern::id().to_string()` and session `recommended_pattern_ids`.
#[derive(Debug, Clone, Default)]
pub struct RankingIndex {
    inner: HashMap<String, PatternRankingState>,
}

impl RankingIndex {
    /// Deterministic pure function of history.
    ///
    /// Feedback is reduced to the LATEST per session (map overwrite), so
    /// replacement feedback is naturally honored: storage upserts feedback by
    /// `session_id`, and this last-wins reduction mirrors it.
    #[must_use]
    pub fn from_history(
        sessions: &[RecommendationSession],
        feedback: &[RecommendationFeedback],
    ) -> Self {
        let mut sessions_by_id: HashMap<uuid::Uuid, &RecommendationSession> = HashMap::new();
        for s in sessions {
            sessions_by_id.insert(s.session_id, s);
        }

        let mut fb_by_session: HashMap<uuid::Uuid, &RecommendationFeedback> = HashMap::new();
        for f in feedback {
            fb_by_session.insert(f.session_id, f);
        }

        let mut inner: HashMap<String, PatternRankingState> = HashMap::new();
        for f in fb_by_session.values() {
            if !sessions_by_id.contains_key(&f.session_id) {
                continue; // orphan feedback contributes nothing
            }
            let positive = matches!(
                f.outcome,
                TaskOutcome::Success { .. } | TaskOutcome::PartialSuccess { .. }
            );
            for pid in &f.applied_pattern_ids {
                let st = inner.entry(pid.clone()).or_default();
                st.applied += 1;
                if positive {
                    st.succeeded += 1;
                }
            }
        }

        Self { inner }
    }

    /// Learned boost in `[0,1] * LEARNED_BOOST_SCALE` for `pattern_id`; 0.0 when no evidence.
    #[must_use]
    pub fn boost(&self, pattern_id: &str) -> f32 {
        match self.inner.get(pattern_id) {
            Some(st) => (st.weight(RANKING_WILSON_Z) as f32) * LEARNED_BOOST_SCALE,
            None => 0.0,
        }
    }

    /// Number of patterns with derived ranking evidence.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether the index contains no derived evidence.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn session(ids: &[&str]) -> RecommendationSession {
        RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            recommended_pattern_ids: ids.iter().map(|s| (*s).to_string()).collect(),
            recommended_playbook_ids: vec![],
        }
    }

    fn feedback(
        session_id: Uuid,
        applied: &[&str],
        outcome: TaskOutcome,
    ) -> RecommendationFeedback {
        RecommendationFeedback {
            session_id,
            applied_pattern_ids: applied.iter().map(|s| (*s).to_string()).collect(),
            consulted_episode_ids: vec![],
            outcome,
            agent_rating: None,
        }
    }

    #[test]
    fn zero_trials_weight_is_zero() {
        let st = PatternRankingState::default();
        assert_eq!(st.weight(RANKING_WILSON_Z), 0.0);
    }

    #[test]
    fn wilson_is_conservative_at_low_trials() {
        // 1/1 has a wide interval; 10/10 is tighter and bounds upward.
        let one = PatternRankingState {
            applied: 1,
            succeeded: 1,
        };
        let ten = PatternRankingState {
            applied: 10,
            succeeded: 10,
        };
        let w1 = one.weight(RANKING_WILSON_Z);
        let w10 = ten.weight(RANKING_WILSON_Z);
        assert!(
            w10 > w1,
            "more evidence must raise the Wilson lower bound: {w1} vs {w10}"
        );
        assert!(w1 > 0.0 && w1 < 0.5);
        assert!(w10 > 0.6);
    }

    #[test]
    fn more_successes_raise_weight() {
        let mixed = PatternRankingState {
            applied: 3,
            succeeded: 2,
        };
        let none = PatternRankingState {
            applied: 3,
            succeeded: 0,
        };
        assert!(
            mixed.weight(RANKING_WILSON_Z) > none.weight(RANKING_WILSON_Z),
            "2/3 success must outrank 0/3"
        );
    }

    #[test]
    fn from_history_counts_applied_and_succeeded_only() {
        let s = session(&["p1", "p2"]);
        let f = feedback(
            s.session_id,
            &["p1"],
            TaskOutcome::Success {
                verdict: "done".to_string(),
                artifacts: vec![],
            },
        );
        let idx = RankingIndex::from_history(&[s], &[f]);
        let p1 = idx.inner.get("p1").copied().unwrap();
        assert_eq!(
            p1,
            PatternRankingState {
                applied: 1,
                succeeded: 1
            }
        );
        assert_eq!(
            idx.inner.get("p2").copied(),
            None,
            "recommended-but-not-applied must carry no learned evidence"
        );
    }

    #[test]
    fn failure_feedback_does_not_increment_succeeded() {
        let s = session(&["p1"]);
        let f = feedback(
            s.session_id,
            &["p1"],
            TaskOutcome::Failure {
                reason: "nope".to_string(),
                error_details: None,
            },
        );
        let idx = RankingIndex::from_history(&[s], &[f]);
        let p1 = idx.inner.get("p1").copied().unwrap();
        assert_eq!(p1.succeeded, 0);
        assert_eq!(p1.applied, 1);
        assert_eq!(idx.boost("p1"), 0.0);
    }

    #[test]
    fn two_feedbacks_for_one_session_last_wins() {
        let s = session(&["p1"]);
        let first = feedback(
            s.session_id,
            &["p1"],
            TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
        );
        let second = feedback(
            s.session_id,
            &["p1"],
            TaskOutcome::Failure {
                reason: "regressed".to_string(),
                error_details: None,
            },
        );
        let idx = RankingIndex::from_history(std::slice::from_ref(&s), &[first, second]);
        let p1 = idx.inner.get("p1").copied().unwrap();
        assert_eq!(
            p1,
            PatternRankingState {
                applied: 1,
                succeeded: 0
            },
            "replacement feedback must win"
        );
    }

    #[test]
    fn feedback_with_absent_session_is_skipped() {
        let orphans = feedback(
            Uuid::new_v4(),
            &["ghost"],
            TaskOutcome::Success {
                verdict: "orphan".to_string(),
                artifacts: vec![],
            },
        );
        let idx = RankingIndex::from_history(&[], &[orphans]);
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn applied_id_not_in_recommended_counted_defensively() {
        let s = session(&["p1"]);
        let applied: Vec<&str> = vec!["p1", "extra"];
        let mut f = feedback(
            s.session_id,
            &applied,
            TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            },
        );
        // Feedback validation normally rejects non-recommended applied IDs, but the
        // derived index must not crash on them (defensive counting).
        let _ = &mut f;
        let idx = RankingIndex::from_history(&[s], &[f]);
        let extra = idx.inner.get("extra").copied().unwrap();
        assert_eq!(extra.applied, 1);
        assert_eq!(extra.succeeded, 1);
        assert!(idx.boost("extra") > 0.0);
    }

    #[test]
    fn partial_success_counts_as_success() {
        let s = session(&["p1"]);
        let f = feedback(
            s.session_id,
            &["p1"],
            TaskOutcome::PartialSuccess {
                verdict: "partially done".to_string(),
                completed: vec!["core".to_string()],
                failed: vec![],
            },
        );
        let idx = RankingIndex::from_history(&[s], &[f]);
        let p1 = idx.inner.get("p1").copied().unwrap();
        assert_eq!(
            p1,
            PatternRankingState {
                applied: 1,
                succeeded: 1
            },
            "PartialSuccess must count toward the success evidence"
        );
        assert!(idx.boost("p1") > 0.0);
    }

    /// Proportionality guard (calibration 2026-08-13): a single success must
    /// overturn only a near-tie and never leapfrog a clearly-worse candidate.
    /// The realistic base distribution (keyword scoring, 8 patterns) had a
    /// top-2 gap ≈ 0.048 and non-tie gaps ≥ 0.059; at scale=0.25 the boost
    /// (≈ 0.0516) flips the former but not the latter. Pins the envelope so a
    /// change to `LEARNED_BOOST_SCALE` / `RANKING_WILSON_Z` is deliberate.
    #[test]
    fn single_success_boost_stays_in_calibrated_window() {
        let single = PatternRankingState {
            applied: 1,
            succeeded: 1,
        }
        .weight(RANKING_WILSON_Z) as f32;
        let boost = single * LEARNED_BOOST_SCALE;
        // Too weak (<0.04) can't flip a 0.048 near-tie; too hot (>=0.06)
        // leapfrogs a clearly-worse candidate (measured #3 gap ~0.059).
        assert!(
            (0.04..0.06).contains(&boost),
            "single-success boost {boost} outside calibrated envelope"
        );
    }
}
