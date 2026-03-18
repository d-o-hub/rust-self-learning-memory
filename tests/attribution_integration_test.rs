use chrono::Utc;
use memory_core::memory::SelfLearningMemory;
use memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use memory_core::types::{TaskContext, TaskOutcome, TaskType};
use uuid::Uuid;

#[tokio::test]
async fn test_full_attribution_flow() {
    let memory = SelfLearningMemory::new();

    // 1. Start an episode
    let episode_id = memory
        .start_episode(
            "Integration test task".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    // 2. Simulate recommendations
    let pattern_id = "test-pattern-1".to_string();
    let session = RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id,
        timestamp: Utc::now(),
        recommended_pattern_ids: vec![pattern_id.clone()],
        recommended_playbook_ids: vec![],
    };
    let session_id = session.session_id;
    memory.record_recommendation_session(session).await;

    // 3. Record feedback after task completion
    let feedback = RecommendationFeedback {
        session_id,
        applied_pattern_ids: vec![pattern_id],
        consulted_episode_ids: vec![],
        outcome: TaskOutcome::Success {
            verdict: "Task succeeded".to_string(),
            artifacts: vec![],
        },
        agent_rating: Some(1.0),
    };
    memory
        .record_recommendation_feedback(feedback)
        .await
        .expect("Record feedback");

    // 4. Verify stats
    let stats = memory.get_recommendation_stats().await;
    assert_eq!(stats.total_sessions, 1);
    assert_eq!(stats.total_feedback, 1);
    assert_eq!(stats.patterns_applied, 1);
    assert_eq!(stats.adoption_rate, 1.0);
    assert_eq!(stats.success_after_adoption_rate, 1.0);
}
