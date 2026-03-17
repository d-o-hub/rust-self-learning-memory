//! Snapshot tests for ADR-044 features (Playbooks, Attribution)

use chrono::{DateTime, TimeZone, Utc};
use insta::assert_debug_snapshot;
use memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use memory_core::memory::playbook::{PlaybookPitfall, PlaybookStep, RecommendedPlaybook};
use memory_core::types::TaskOutcome;
use uuid::Uuid;

fn fixed_timestamp() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 16, 12, 0, 0).unwrap()
}

#[test]
fn test_recommended_playbook_snapshot() {
    let playbook_id = Uuid::from_u128(12345);
    let playbook = RecommendedPlaybook {
        playbook_id,
        task_match_score: 0.85,
        why_relevant: "Based on 3 successful patterns".to_string(),
        when_to_apply: vec!["When using Axum".to_string()],
        when_not_to_apply: vec!["When using Actix".to_string()],
        ordered_steps: vec![
            PlaybookStep::new(1, "Install dependencies".to_string())
                .with_tool_hint("cargo")
                .with_expected_result("Dependencies installed"),
        ],
        pitfalls: vec![
            PlaybookPitfall::new("Version mismatch", "Incompatible with old Rust")
                .with_mitigation("Upgrade to Rust 1.80+"),
        ],
        expected_outcome: "A working API endpoint".to_string(),
        confidence: 0.9,
        supporting_pattern_ids: vec![Uuid::from_u128(1)],
        supporting_episode_ids: vec![Uuid::from_u128(2)],
        created_at: fixed_timestamp(),
    };

    assert_debug_snapshot!(playbook);
}

#[test]
fn test_recommendation_session_snapshot() {
    let session = RecommendationSession {
        session_id: Uuid::from_u128(100),
        episode_id: Uuid::from_u128(200),
        timestamp: fixed_timestamp(),
        recommended_pattern_ids: vec!["p-1".to_string()],
        recommended_playbook_ids: vec![Uuid::from_u128(300)],
    };

    assert_debug_snapshot!(session);
}

#[test]
fn test_recommendation_feedback_snapshot() {
    let feedback = RecommendationFeedback {
        session_id: Uuid::from_u128(100),
        applied_pattern_ids: vec!["p-1".to_string()],
        consulted_episode_ids: vec![Uuid::from_u128(400)],
        outcome: TaskOutcome::Success {
            verdict: "Task completed successfully".to_string(),
            artifacts: vec!["app.rs".to_string()],
        },
        agent_rating: Some(0.95),
    };

    assert_debug_snapshot!(feedback);
}
