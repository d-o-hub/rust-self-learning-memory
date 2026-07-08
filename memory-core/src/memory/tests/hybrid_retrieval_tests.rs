use crate::types::RetrievalMode;
use crate::{
    ExecutionResult, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};

#[tokio::test]
async fn test_hybrid_retrieval_flow() {
    let mut config = MemoryConfig::default();
    config.retrieval_mode = RetrievalMode::Hybrid;
    config.enable_embeddings = true;
    config.quality_threshold = 0.0; // Disable quality check for test

    let memory = SelfLearningMemory::with_config(config);

    // 1. Create a few episodes
    let context = TaskContext::default();

    let id1 = memory
        .start_episode(
            "Implement a REST API in Rust".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;
    let mut step1 = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    step1.result = Some(ExecutionResult::Success {
        output: "ok".to_string(),
    });
    memory.log_step(id1, step1).await;
    memory
        .complete_episode(
            id1,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let id2 = memory
        .start_episode(
            "Fix a bug in the database connection".to_string(),
            context.clone(),
            TaskType::Debugging,
        )
        .await;
    let mut step2 = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    step2.result = Some(ExecutionResult::Success {
        output: "ok".to_string(),
    });
    memory.log_step(id2, step2).await;
    memory
        .complete_episode(
            id2,
            TaskOutcome::Success {
                verdict: "Fixed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // 2. Retrieve using hybrid mode
    let results = memory
        .retrieve_relevant_context("REST API implementation".to_string(), context, 5)
        .await;

    assert!(!results.is_empty());
}
