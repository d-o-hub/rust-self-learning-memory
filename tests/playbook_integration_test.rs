#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Playbook Integration Tests

#![allow(missing_docs)]

use do_memory_core::ExecutionStep;
use do_memory_core::memory::SelfLearningMemory;
use do_memory_core::patterns::{Pattern, PatternEffectiveness};
use do_memory_core::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
use do_memory_core::{AttributedPlaybookRequest, RecommendationFeedback};
use uuid::Uuid;

#[tokio::test]
async fn test_playbook_generation_flow() {
    // Set low quality threshold for test
    let config = MemoryConfig {
        quality_threshold: 0.0,
        ..MemoryConfig::default()
    };
    let memory = SelfLearningMemory::with_config(config);

    // 1. Create some historical data (episodes and patterns)
    let context = TaskContext {
        domain: "test-domain".to_string(),
        ..TaskContext::default()
    };

    let episode_id = memory
        .start_episode(
            "Historical success task".to_string(),
            context.clone(),
            TaskType::Testing,
        )
        .await;

    memory
        .log_step(
            episode_id,
            ExecutionStep::new(1, "tool1".to_string(), "action1".to_string()),
        )
        .await;

    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Worked".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Manually add a pattern for the domain
    {
        let mut patterns = memory.patterns_fallback().write().await;
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: context.clone(),
            success_rate: 0.9,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        };
        patterns.insert(pattern.id(), pattern);
    }

    // 2. Retrieve playbook for a similar task
    let playbooks = memory
        .retrieve_playbooks(
            "New task in same domain",
            "test-domain",
            TaskType::Testing,
            context,
            1,
            5,
        )
        .await;

    // 3. Verify playbook content
    assert_ne!(playbooks.len(), 0);
    let playbook = &playbooks[0];
    assert!(playbook.confidence > 0.0);
    assert_ne!(playbook.ordered_steps.len(), 0);

    // Check that it contains steps derived from our pattern
    let has_tool1 = playbook
        .ordered_steps
        .iter()
        .any(|s| s.action.contains("tool1"));
    assert!(has_tool1);
}

#[tokio::test]
async fn feedback_accepts_the_pattern_cited_for_a_step() {
    let config = MemoryConfig {
        quality_threshold: 0.0,
        ..MemoryConfig::default()
    };
    let memory = SelfLearningMemory::with_config(config);

    let context = TaskContext {
        domain: "networking".to_string(),
        ..TaskContext::default()
    };
    let episode_id = memory
        .start_episode(
            "Historical retry task".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;
    memory
        .complete_episode(
            episode_id,
            TaskOutcome::Success {
                verdict: "Worked".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let pattern = Pattern::ErrorRecovery {
        id: Uuid::new_v4(),
        error_type: "timeout".to_string(),
        recovery_steps: vec![
            "retry_with_backoff".to_string(),
            "use_cached_data".to_string(),
        ],
        success_rate: 0.9,
        context: context.clone(),
        effectiveness: PatternEffectiveness::default(),
    };
    let pattern_id = pattern.id();
    {
        let mut patterns = memory.patterns_fallback().write().await;
        patterns.insert(pattern.id(), pattern);
    }

    let attributed = memory
        .retrieve_playbooks_attributed(AttributedPlaybookRequest {
            episode_id,
            task_description: "Add retry to the HTTP client".to_string(),
            domain: "networking".to_string(),
            task_type: TaskType::CodeGeneration,
            context,
            max_playbooks: 1,
            max_steps_per_playbook: 5,
        })
        .await
        .unwrap();

    let playbook = attributed
        .playbooks
        .first()
        .expect("a playbook should be generated");
    let cited = playbook
        .step_source(1)
        .expect("the first step should be attributed")
        .pattern_id;
    assert_eq!(cited, pattern_id);

    // The cited ID is what the consumer followed, so the existing feedback call
    // must accept it without the session carrying a per-step field.
    memory
        .record_recommendation_feedback(RecommendationFeedback {
            session_id: attributed.session.session_id,
            applied_pattern_ids: vec![cited.to_string()],
            consulted_episode_ids: Vec::new(),
            outcome: TaskOutcome::Success {
                verdict: "Followed step 1".to_string(),
                artifacts: Vec::new(),
            },
            agent_rating: Some(0.9),
        })
        .await
        .expect("a cited step pattern must be accepted");
}
