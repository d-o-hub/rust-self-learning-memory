//! Unit tests for the recommendation tracker, its integrity rules, and stats.

use super::RecommendationTracker;
use crate::memory::attribution::types::{RecommendationFeedback, RecommendationSession};
use crate::types::TaskOutcome;
use uuid::Uuid;

fn create_test_session() -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: vec!["p1".to_string(), "p2".to_string()],
        recommended_playbook_ids: vec![],
    }
}

fn create_test_feedback(session_id: Uuid) -> RecommendationFeedback {
    RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["p1".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.9),
    }
}

#[tokio::test]
async fn test_record_and_get_session() {
    let tracker = RecommendationTracker::new();
    let session = create_test_session();
    let session_id = session.session_id;

    tracker.record_session(session.clone()).await;

    let retrieved = tracker.get_session(session_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().session_id, session_id);
}

#[tokio::test]
async fn test_record_and_get_feedback() {
    let tracker = RecommendationTracker::new();
    let session = create_test_session();
    let session_id = session.session_id;

    tracker.record_session(session).await;

    let feedback = create_test_feedback(session_id);
    tracker.record_feedback(feedback.clone()).await.unwrap();

    let retrieved = tracker.get_feedback(session_id).await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_feedback_rejects_unknown_session() {
    // ADR-080 §4: unknown session must be rejected.
    let tracker = RecommendationTracker::new();
    let feedback = create_test_feedback(Uuid::new_v4()); // no session recorded
    let result = tracker.record_feedback(feedback).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unknown session"), "error was: {err}");
}

#[tokio::test]
async fn test_feedback_rejects_non_recommended_pattern_id() {
    // ADR-080 §4: applied IDs must be a subset of recommended IDs.
    let tracker = RecommendationTracker::new();
    let session = create_test_session(); // recommends p1, p2
    let session_id = session.session_id;
    tracker.record_session(session).await;

    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["p1".to_string(), "NOT_RECOMMENDED".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec![],
        },
        agent_rating: None,
    };
    let result = tracker.record_feedback(feedback).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("NOT_RECOMMENDED"), "error was: {err}");
}

#[tokio::test]
async fn test_feedback_replacement_is_idempotent() {
    // ADR-080 §4: replacement feedback for one session is idempotent.
    let tracker = RecommendationTracker::new();
    let session = create_test_session();
    let session_id = session.session_id;
    tracker.record_session(session).await;

    let fb1 = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["p1".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "First".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.8),
    };
    tracker.record_feedback(fb1).await.unwrap();

    let fb2 = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec!["p2".to_string()],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Second".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(0.5),
    };
    tracker.record_feedback(fb2).await.unwrap();

    let stats = tracker.get_stats().await;
    // Only one feedback entry (replaced, not appended).
    assert_eq!(stats.total_feedback, 1);
    // Should reflect the second feedback.
    assert_eq!(stats.patterns_applied, 1); // only p2
}

#[tokio::test]
async fn test_reindexing_same_session_is_idempotent() {
    // ADR-081 §1: re-hydrating a session from storage must not duplicate it
    // in the episode index, or latest-lookup ordering degrades over time.
    let tracker = RecommendationTracker::new();
    let episode_id = Uuid::new_v4();

    let mut session = create_test_session();
    session.episode_id = episode_id;
    let session_id = session.session_id;

    tracker.record_session(session.clone()).await;
    tracker.record_session(session.clone()).await;
    tracker.record_session(session).await;

    let all = tracker.get_all_sessions_for_episode(episode_id).await;
    assert_eq!(
        all.len(),
        1,
        "same session indexed three times must appear once"
    );
    assert_eq!(all[0].session_id, session_id);
}

#[tokio::test]
async fn test_hydrate_feedback_bypasses_session_check() {
    // ADR-081 §1: feedback loaded from storage is cached without re-running
    // integrity checks it already passed, and without needing its session
    // present in this process.
    let tracker = RecommendationTracker::new();
    let orphan_session_id = Uuid::new_v4();
    let feedback = create_test_feedback(orphan_session_id);

    tracker.hydrate_feedback(feedback).await;

    let cached = tracker.get_feedback(orphan_session_id).await;
    assert!(
        cached.is_some(),
        "storage-loaded feedback must populate the cache even with no local session"
    );
}

#[tokio::test]
async fn test_multiple_sessions_per_episode() {
    // ADR-080 §4: two attributed calls for one episode create two distinct sessions.
    let tracker = RecommendationTracker::new();
    let episode_id = Uuid::new_v4();

    let mut s1 = create_test_session();
    s1.episode_id = episode_id;
    s1.timestamp = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .into();

    let mut s2 = create_test_session();
    s2.episode_id = episode_id;
    s2.timestamp = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:01Z")
        .unwrap()
        .into();

    let s2_id = s2.session_id;

    tracker.record_session(s1).await;
    tracker.record_session(s2).await;

    let all = tracker.get_all_sessions_for_episode(episode_id).await;
    assert_eq!(all.len(), 2, "both sessions should be stored");

    // Latest lookup should return s2 (later timestamp).
    let latest = tracker.get_session_for_episode(episode_id).await.unwrap();
    assert_eq!(latest.session_id, s2_id);
}

#[tokio::test]
async fn test_get_session_for_episode() {
    let tracker = RecommendationTracker::new();
    let session = create_test_session();
    let episode_id = session.episode_id;

    tracker.record_session(session).await;

    let retrieved = tracker.get_session_for_episode(episode_id).await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_get_stats() {
    let tracker = RecommendationTracker::new();

    // Record a session
    let session = create_test_session();
    let session_id = session.session_id;
    tracker.record_session(session).await;

    // Record feedback
    let feedback = create_test_feedback(session_id);
    tracker.record_feedback(feedback).await.unwrap();

    let stats = tracker.get_stats().await;
    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_feedback, 1);
    assert_eq!(stats.patterns_applied, 1);
    assert_eq!(stats.patterns_ignored, 1); // 2 recommended - 1 applied
    assert!((stats.adoption_rate - 0.5).abs() < 0.01);
}

#[tokio::test]
async fn test_clear() {
    let tracker = RecommendationTracker::new();
    let session = create_test_session();

    tracker.record_session(session).await;
    tracker.clear().await;

    let stats = tracker.get_stats().await;
    assert_eq!(stats.total_sessions, 0);
}
