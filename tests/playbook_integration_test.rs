use memory_core::ExecutionStep;
use memory_core::memory::SelfLearningMemory;
use memory_core::pattern::{Pattern, PatternEffectiveness};
use memory_core::types::{MemoryConfig, TaskContext, TaskOutcome, TaskType};
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
    assert!(!playbooks.is_empty());
    let playbook = &playbooks[0];
    assert!(playbook.confidence > 0.0);
    assert!(!playbook.ordered_steps.is_empty());

    // Check that it contains steps derived from our pattern
    let has_tool1 = playbook
        .ordered_steps
        .iter()
        .any(|s| s.action.contains("tool1"));
    assert!(has_tool1);
}
