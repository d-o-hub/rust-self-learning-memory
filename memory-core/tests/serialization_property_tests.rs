//! Property-based tests for serialization round-trips and calculator invariants
//!
//! Implements ADR-042 ACT-030 Phase 2: Property tests for serialization and calculations.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::field_reassign_with_default
)]

use chrono::{Duration, Utc};
use memory_core::episode::{Episode, ExecutionStep};
use memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use memory_core::memory::playbook::{PlaybookPitfall, PlaybookStep, RecommendedPlaybook};
use memory_core::pattern::PatternEffectiveness;
use memory_core::reward::RewardCalculator;
use memory_core::types::{
    ComplexityLevel, ExecutionResult, OutcomeStats, TaskContext, TaskOutcome, TaskType,
};
use proptest::prelude::*;

// ============================================================================
// Custom Strategies
// ============================================================================

fn task_type_strategy() -> impl Strategy<Value = TaskType> {
    prop_oneof![
        Just(TaskType::CodeGeneration),
        Just(TaskType::Debugging),
        Just(TaskType::Refactoring),
        Just(TaskType::Testing),
        Just(TaskType::Analysis),
        Just(TaskType::Documentation),
        Just(TaskType::Other),
    ]
}

fn complexity_strategy() -> impl Strategy<Value = ComplexityLevel> {
    prop_oneof![
        Just(ComplexityLevel::Simple),
        Just(ComplexityLevel::Moderate),
        Just(ComplexityLevel::Complex),
    ]
}

// ============================================================================
// Serialization Round-trip Tests
// ============================================================================

proptest! {
    #[test]
    fn task_type_postcard_roundtrip(task_type in task_type_strategy()) {
        let encoded = postcard::to_allocvec(&task_type).expect("serialize");
        let decoded: TaskType = postcard::from_bytes(&encoded).expect("deserialize");
        prop_assert_eq!(task_type, decoded);
    }
}

proptest! {
    #[test]
    fn complexity_level_postcard_roundtrip(level in complexity_strategy()) {
        let encoded = postcard::to_allocvec(&level).expect("serialize");
        let decoded: ComplexityLevel = postcard::from_bytes(&encoded).expect("deserialize");
        prop_assert_eq!(level, decoded);
    }
}

// ============================================================================
// Reward Calculator Property Tests
// ============================================================================

proptest! {
    #[test]
    fn reward_calculator_efficiency_bounds(num_steps in 0usize..100usize, duration_secs in 0i64..3600i64) {
        let calculator = RewardCalculator::new();
        let mut episode = Episode::new("Test task".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        for i in 0..num_steps {
            let step = ExecutionStep::new(i + 1, "test_tool".to_string(), "Test action".to_string());
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        episode.start_time = Utc::now() - Duration::seconds(duration_secs);
        episode.end_time = Some(Utc::now());

        let score = calculator.calculate(&episode);

        prop_assert!(score.efficiency >= 0.5, "Efficiency below minimum 0.5");
        prop_assert!(score.efficiency <= 1.5, "Efficiency above maximum 1.5");
    }
}

proptest! {
    #[test]
    fn reward_calculator_base_bounds(outcome_success in proptest::bool::ANY) {
        let calculator = RewardCalculator::new();
        let mut episode = Episode::new("Test task".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        let outcome = if outcome_success {
            TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] }
        } else {
            TaskOutcome::Failure { reason: "Failed".to_string(), error_details: None }
        };
        episode.complete(outcome);

        let score = calculator.calculate(&episode);

        prop_assert!(score.base >= 0.0, "Base below minimum 0.0");
        prop_assert!(score.base <= 1.0, "Base above maximum 1.0");
    }
}

proptest! {
    #[test]
    fn reward_calculator_complexity_bonus_bounds(complexity in complexity_strategy()) {
        let calculator = RewardCalculator::new();
        let mut context = TaskContext::default();
        context.complexity = complexity;

        let mut episode = Episode::new("Test task".to_string(), context, TaskType::CodeGeneration);
        episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });

        let score = calculator.calculate(&episode);

        prop_assert!(score.complexity_bonus >= 1.0);
        prop_assert!(score.complexity_bonus <= 1.2);
    }
}

// ============================================================================
// Pattern Effectiveness Property Tests
// ============================================================================

proptest! {
    #[test]
    fn pattern_effectiveness_usage_rate_bounds(times_retrieved in 0usize..100usize, times_applied in 0usize..100usize) {
        let mut effectiveness = PatternEffectiveness::default();
        effectiveness.times_retrieved = times_retrieved;
        effectiveness.times_applied = times_applied.min(times_retrieved);

        let usage_rate = effectiveness.usage_rate();

        prop_assert!(usage_rate >= 0.0);
        prop_assert!(usage_rate <= 1.0);
    }
}

proptest! {
    #[test]
    fn pattern_effectiveness_success_rate_bounds(success_when_applied in 0usize..50usize, failure_when_applied in 0usize..50usize) {
        let mut effectiveness = PatternEffectiveness::default();
        effectiveness.success_when_applied = success_when_applied;
        effectiveness.failure_when_applied = failure_when_applied;
        effectiveness.times_applied = success_when_applied + failure_when_applied;

        let success_rate = effectiveness.application_success_rate();

        prop_assert!(success_rate >= 0.0);
        prop_assert!(success_rate <= 1.0);
    }
}

proptest! {
    #[test]
    fn pattern_effectiveness_score_non_negative(times_applied in 0usize..100usize, success_rate in 0.0f32..1.0f32, reward_delta in -1.0f32..1.0f32) {
        let mut effectiveness = PatternEffectiveness::default();
        effectiveness.times_applied = times_applied;
        effectiveness.success_when_applied = (times_applied as f32 * success_rate) as usize;
        effectiveness.failure_when_applied = times_applied - effectiveness.success_when_applied;
        effectiveness.avg_reward_delta = reward_delta;

        let score = effectiveness.effectiveness_score();

        prop_assert!(score >= 0.0, "Effectiveness score is negative");
    }
}

// ============================================================================
// Outcome Stats Property Tests
// ============================================================================

proptest! {
    #[test]
    fn outcome_stats_success_rate_bounds(success_count in 0usize..100usize, failure_count in 0usize..100usize) {
        let stats = OutcomeStats {
            success_count,
            failure_count,
            total_count: success_count + failure_count,
            avg_duration_secs: 0.0,
        };

        let success_rate = stats.success_rate();

        prop_assert!(success_rate >= 0.0);
        prop_assert!(success_rate <= 1.0);
    }
}

#[test]
fn outcome_stats_success_rate_zero_total() {
    let stats = OutcomeStats {
        success_count: 0,
        failure_count: 0,
        total_count: 0,
        avg_duration_secs: 0.0,
    };
    let success_rate = stats.success_rate();
    assert!((success_rate - 0.0_f32).abs() < f32::EPSILON);
}

// ============================================================================
// Episode Property Tests
// ============================================================================

proptest! {
    #[test]
    fn episode_step_count_consistency(step_results in proptest::collection::vec(proptest::bool::ANY, 0..50usize)) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::Testing);

        let expected_successes = step_results.iter().filter(|&&r| r).count();
        let expected_failures = step_results.iter().filter(|&&r| !r).count();

        for (i, &is_success) in step_results.iter().enumerate() {
            let mut step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            step.result = if is_success {
                Some(ExecutionResult::Success { output: "ok".to_string() })
            } else {
                Some(ExecutionResult::Error { message: "err".to_string() })
            };
            episode.add_step(step);
        }

        prop_assert_eq!(episode.successful_steps_count(), expected_successes);
        prop_assert_eq!(episode.failed_steps_count(), expected_failures);
    }
}

proptest! {
    #[test]
    fn episode_completion_state(num_steps in 0usize..10usize, should_complete in proptest::bool::ANY) {
        let mut episode = Episode::new("Test".to_string(), TaskContext::default(), TaskType::CodeGeneration);

        for i in 0..num_steps {
            let step = ExecutionStep::new(i + 1, "tool".to_string(), "action".to_string());
            episode.add_step(step);
        }

        if should_complete {
            episode.complete(TaskOutcome::Success { verdict: "Done".to_string(), artifacts: vec![] });
        }

        prop_assert_eq!(episode.is_complete(), should_complete);
    }
}

// ============================================================================
// ADR-044 Playbook & Attribution Property Tests
// ============================================================================

proptest! {
    #[test]
    fn recommended_playbook_postcard_roundtrip(
        task_match_score in 0.0f32..1.0f32,
        confidence in 0.0f32..1.0f32,
    ) {
        let playbook_id = uuid::Uuid::new_v4();
        let playbook = RecommendedPlaybook {
            playbook_id,
            task_match_score,
            why_relevant: "PropTest".to_string(),
            when_to_apply: vec!["Always".to_string()],
            when_not_to_apply: vec!["Never".to_string()],
            ordered_steps: vec![PlaybookStep::new(1, "Step 1".to_string())],
            pitfalls: vec![PlaybookPitfall::new("P", "R")],
            expected_outcome: "Success".to_string(),
            confidence,
            supporting_pattern_ids: vec![uuid::Uuid::new_v4()],
            supporting_episode_ids: vec![uuid::Uuid::new_v4()],
            created_at: Utc::now(),
        };

        let encoded = postcard::to_allocvec(&playbook).expect("serialize");
        let decoded: RecommendedPlaybook = postcard::from_bytes(&encoded).expect("deserialize");
        prop_assert_eq!(playbook.playbook_id, decoded.playbook_id);
        prop_assert!((playbook.task_match_score - decoded.task_match_score).abs() < 0.0001);
        prop_assert_eq!(playbook.why_relevant, decoded.why_relevant);
    }
}

proptest! {
    #[test]
    fn recommendation_session_postcard_roundtrip(
        num_patterns in 0usize..10usize,
        num_playbooks in 0usize..10usize,
    ) {
        let session = RecommendationSession {
            session_id: uuid::Uuid::new_v4(),
            episode_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            recommended_pattern_ids: (0..num_patterns).map(|i| format!("p-{i}")).collect(),
            recommended_playbook_ids: (0..num_playbooks).map(|_| uuid::Uuid::new_v4()).collect(),
        };

        let encoded = postcard::to_allocvec(&session).expect("serialize");
        let decoded: RecommendationSession = postcard::from_bytes(&encoded).expect("deserialize");
        prop_assert_eq!(session.session_id, decoded.session_id);
        prop_assert_eq!(session.episode_id, decoded.episode_id);
        prop_assert_eq!(session.recommended_pattern_ids, decoded.recommended_pattern_ids);
        prop_assert_eq!(session.recommended_playbook_ids, decoded.recommended_playbook_ids);
    }
}

proptest! {
    #[test]
    fn recommendation_feedback_postcard_roundtrip(
        agent_rating in proptest::option::of(0.0f32..1.0f32),
    ) {
        let feedback = RecommendationFeedback {
            session_id: uuid::Uuid::new_v4(),
            applied_pattern_ids: vec!["p-1".to_string()],
            consulted_episode_ids: vec![uuid::Uuid::new_v4()],
            outcome: TaskOutcome::Success { verdict: "ok".to_string(), artifacts: vec![] },
            agent_rating,
        };

        let encoded = postcard::to_allocvec(&feedback).expect("serialize");
        let decoded: RecommendationFeedback = postcard::from_bytes(&encoded).expect("deserialize");
        prop_assert_eq!(feedback.session_id, decoded.session_id);
        prop_assert_eq!(feedback.agent_rating, decoded.agent_rating);
    }
}
