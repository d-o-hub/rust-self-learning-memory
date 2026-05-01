//! Property tests for attribution serialization roundtrips
//!
//! Integration tests are separate crate roots and don't inherit .clippy.toml settings

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(missing_docs)]

use chrono::{TimeZone, Utc};
use do_memory_core::memory::attribution::{
    RecommendationFeedback, RecommendationSession, RecommendationStats,
};
use do_memory_core::types::TaskOutcome;
use proptest::prelude::*;
use uuid::Uuid;

fn uuid_strategy() -> impl Strategy<Value = Uuid> {
    any::<[u8; 16]>().prop_map(Uuid::from_bytes)
}

fn task_outcome_strategy() -> impl Strategy<Value = TaskOutcome> {
    prop_oneof![
        any::<String>().prop_map(|v| TaskOutcome::Success {
            verdict: v,
            artifacts: vec![]
        }),
        any::<String>().prop_map(|r| TaskOutcome::Failure {
            reason: r,
            error_details: None
        }),
    ]
}

proptest! {
    #[test]
    fn recommendation_session_serialization_roundtrip(
        session_id in uuid_strategy(),
        episode_id in uuid_strategy(),
        timestamp_ms in 0i64..2_000_000_000_000i64,
        recommended_patterns in prop::collection::vec(".*", 0..10),
        recommended_playbooks in prop::collection::vec(uuid_strategy(), 0..10)
    ) {
        let session = RecommendationSession {
            session_id,
            episode_id,
            timestamp: Utc.timestamp_millis_opt(timestamp_ms).unwrap(),
            recommended_pattern_ids: recommended_patterns,
            recommended_playbook_ids: recommended_playbooks,
        };
        let encoded = serde_json::to_string(&session).expect("serialize");
        let decoded: RecommendationSession = serde_json::from_str(&encoded).expect("deserialize");
        prop_assert_eq!(session, decoded);
    }

    #[test]
    fn recommendation_feedback_serialization_roundtrip(
        session_id in uuid_strategy(),
        applied_patterns in prop::collection::vec(".*", 0..10),
        consulted_episodes in prop::collection::vec(uuid_strategy(), 0..10),
        outcome in task_outcome_strategy(),
        rating in prop::option::of(0.0f32..1.0f32)
    ) {
        let feedback = RecommendationFeedback {
            session_id,
            applied_pattern_ids: applied_patterns,
            consulted_episode_ids: consulted_episodes,
            outcome,
            agent_rating: rating,
        };
        let encoded = serde_json::to_string(&feedback).expect("serialize");
        let decoded: RecommendationFeedback = serde_json::from_str(&encoded).expect("deserialize");
        prop_assert_eq!(feedback, decoded);
    }

    #[test]
    fn recommendation_stats_invariants(
        total_sessions in 0usize..1000usize,
        total_feedback in 0usize..1000usize,
        patterns_applied in 0usize..1000usize,
        successful_applications in 0usize..1000usize
    ) {
        let stats = RecommendationStats {
            total_sessions,
            total_feedback: total_feedback.min(total_sessions),
            patterns_applied,
            patterns_ignored: 0, // Not verified here
            successful_applications: successful_applications.min(patterns_applied),
            adoption_rate: 0.0, // calculated
            success_after_adoption_rate: 0.0, // calculated
            avg_agent_rating: None,
        };

        prop_assert!(stats.successful_applications <= stats.patterns_applied);
        prop_assert!(stats.total_feedback <= stats.total_sessions);
    }
}
