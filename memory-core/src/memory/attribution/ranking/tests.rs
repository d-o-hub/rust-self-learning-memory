//! Unit coverage for the derived feedback ranking index (ADR-082).
//!
//! Split out of `ranking.rs` to keep the source file within the 500-LOC gate,
//! mirroring the `tracker/tests.rs` layout.

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

fn feedback(session_id: Uuid, applied: &[&str], outcome: TaskOutcome) -> RecommendationFeedback {
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

/// Deterministic session fixture: ids derive from `id` so the incremental and
/// reference histories are reproducible.
fn fixed_session(id: u128, recommended: &[&str]) -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::from_u128(id),
        episode_id: Uuid::from_u128(1_000 + id),
        timestamp: Utc::now(),
        recommended_pattern_ids: recommended.iter().map(|s| (*s).to_string()).collect(),
        recommended_playbook_ids: vec![],
    }
}

fn fixed_feedback(session_index: u128, applied: &[&str], positive: bool) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id: Uuid::from_u128(session_index),
        applied_pattern_ids: applied.iter().map(|s| (*s).to_string()).collect(),
        consulted_episode_ids: vec![],
        outcome: if positive {
            TaskOutcome::Success {
                verdict: "ok".to_string(),
                artifacts: vec![],
            }
        } else {
            TaskOutcome::Failure {
                reason: "no".to_string(),
                error_details: None,
            }
        },
        agent_rating: None,
    }
}

/// ADR-082 incremental path: a sequence of replacements (Success→Failure,
/// Failure→Success, widened and shrunk applied sets, duplicate submission) must
/// leave the index equal to a `from_history` rebuild of the same latest-wins
/// history.
#[test]
fn incremental_updates_match_full_rebuild() {
    let sessions: Vec<RecommendationSession> = (0..16)
        .map(|i| fixed_session(i, &["p0", "p1", "p2", "p3"]))
        .collect();
    let initial: Vec<RecommendationFeedback> = (0..16)
        .map(|i| match i % 4 {
            0 => fixed_feedback(i, &["p0"], true),
            1 => fixed_feedback(i, &["p1", "p2"], false),
            2 => fixed_feedback(i, &["p2"], true),
            _ => fixed_feedback(i, &["p3", "p2"], true),
        })
        .collect();

    let mut incremental = RankingIndex::from_history(&sessions, &initial);
    let mut reference = initial.clone();

    let replacements = [
        fixed_feedback(1, &["p1"], true),       // Failure→Success
        fixed_feedback(1, &["p1"], false),      // Success→Failure
        fixed_feedback(5, &["p1", "p2"], true), // widened applied set
        fixed_feedback(5, &["p1", "p2"], true), // duplicate re-submission
        fixed_feedback(12, &["p3"], false),     // shrunk applied set
    ];

    for fb in &replacements {
        assert!(
            incremental.apply_feedback(fb),
            "incremental update must be accepted"
        );
        reference.retain(|prior| prior.session_id != fb.session_id);
        reference.push(fb.clone());
    }

    let rebuilt = RankingIndex::from_history(&sessions, &reference);
    assert_eq!(
        incremental.inner, rebuilt.inner,
        "learned counters must match a full rebuild"
    );
    assert_eq!(incremental.contributions, rebuilt.contributions);
    assert_eq!(incremental.len(), rebuilt.len());
    for pid in ["p0", "p1", "p2", "p3"] {
        assert_eq!(
            incremental.boost(pid),
            rebuilt.boost(pid),
            "{pid} boost must match the rebuild"
        );
    }
}

/// The incremental update must touch only the changed session's patterns: an
/// unrelated pattern's counter is untouched, and a pattern whose evidence drops
/// to zero is removed exactly as a rebuild omits it.
#[test]
fn incremental_update_touches_only_affected_patterns() {
    // One session per pattern, so each pattern's evidence comes from exactly one
    // contribution and a replacement can zero it.
    let sessions: Vec<RecommendationSession> = (0..4)
        .map(|i| fixed_session(i, &["p0", "p1", "p2", "p3"]))
        .collect();
    let history: Vec<RecommendationFeedback> = (0..4)
        .map(|i| match i {
            0 => fixed_feedback(0, &["p0"], true),
            1 => fixed_feedback(1, &["p1"], true),
            2 => fixed_feedback(2, &["p2"], true),
            _ => fixed_feedback(3, &["p3"], true),
        })
        .collect();

    let mut index = RankingIndex::from_history(&sessions, &history);
    let before = index.inner.clone();

    // Session 0 applied p0 (success); replace it with a p2 failure.
    let replacement = fixed_feedback(0, &["p2"], false);
    assert!(index.apply_feedback(&replacement));

    for pid in ["p1", "p3"] {
        assert_eq!(
            index.inner.get(pid),
            before.get(pid),
            "{pid} must be untouched by an unrelated session's replacement"
        );
    }
    assert!(
        !index.inner.contains_key("p0"),
        "p0 evidence dropped to zero and the entry was removed"
    );
    let p2_before = before.get("p2").copied().unwrap();
    let p2_after = index.inner.get("p2").copied().unwrap();
    assert_eq!(
        p2_after,
        PatternRankingState {
            applied: p2_before.applied + 1,
            succeeded: p2_before.succeeded,
        },
        "p2 gains exactly one applied-failure record"
    );
    assert_eq!(
        index.len(),
        before.len() - 1,
        "only the zeroed pattern entry disappeared"
    );
    assert_eq!(index.boost("p0"), 0.0);
}

/// A failed consistency check leaves the index untouched (and does not advance
/// the mutation counter) so the caller can fall back to a full rebuild.
#[test]
fn inconsistent_prior_contribution_reports_failure_without_mutation() {
    let s = session(&["p1"]);
    let f = feedback(
        s.session_id,
        &["p1"],
        TaskOutcome::Success {
            verdict: "ok".to_string(),
            artifacts: vec![],
        },
    );
    let mut index = RankingIndex::from_history(std::slice::from_ref(&s), std::slice::from_ref(&f));
    // Simulate an index corrupted out-of-band: the recorded contribution claims
    // one applied p1, but the counter no longer backs it.
    index.inner.get_mut("p1").unwrap().applied = 0;
    let snapshot = index.inner.clone();
    let revision = index.revision();

    let replacement = feedback(
        s.session_id,
        &["p1"],
        TaskOutcome::Failure {
            reason: "no".to_string(),
            error_details: None,
        },
    );
    assert!(
        !index.apply_feedback(&replacement),
        "underflowing subtraction must report failure"
    );
    assert_eq!(
        index.inner, snapshot,
        "a rejected update must not mutate counters"
    );
    assert_eq!(
        index.revision(),
        revision,
        "a rejected update must not advance the revision"
    );
}

/// The rebuild guard relies on a monotonic mutation counter that survives a
/// `replace_with` overwrite.
#[test]
fn revision_advances_on_mutation_and_survives_rebuild() {
    let mut index = RankingIndex::default();
    let start = index.revision();

    assert!(index.apply_feedback(&fixed_feedback(1, &["p"], true)));
    assert_eq!(index.revision(), start + 1);

    index.replace_with(RankingIndex::default());
    assert_eq!(
        index.revision(),
        start + 2,
        "rebuild must carry the counter forward"
    );
    assert!(
        index.is_empty(),
        "replace_with installs the rebuilt snapshot"
    );
}
