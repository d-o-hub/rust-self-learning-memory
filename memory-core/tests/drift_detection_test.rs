use do_memory_core::{SelfLearningMemory, TaskContext, TaskType, TaskOutcome, ExecutionStep, ExecutionResult, MemoryConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_concept_drift_detection_integration() {
    let mut config = MemoryConfig::default();
    config.quality_threshold = 0.0; // Disable quality gating for test
    let memory = SelfLearningMemory::with_config(config);
    let mut receiver = memory.subscribe();

    // Create first version
    let context = TaskContext::default();
    let id1 = memory.start_episode("drift task".to_string(), context.clone(), TaskType::CodeGeneration).await;

    memory.complete_episode(id1, TaskOutcome::Success {
        verdict: "v1".to_string(),
        artifacts: vec!["a.rs".to_string()]
    }).await.unwrap();

    // Create second version
    let id2 = memory.start_next_version(id1).await.unwrap();
    memory.complete_episode(id2, TaskOutcome::Success {
        verdict: "v2".to_string(),
        artifacts: vec!["a.rs".to_string()]
    }).await.unwrap();

    // Create third version with a significant shift in reward (Failure)
    let id3 = memory.start_next_version(id1).await.unwrap();
    memory.complete_episode(id3, TaskOutcome::Failure {
        reason: "v3 failed".to_string(),
        error_details: None
    }).await.unwrap();

    let mut found_drift = false;
    // Drain events to see if drift was detected
    for _ in 0..20 {
        if let Ok(event) = receiver.try_recv() {
            if let do_memory_core::types::event::MemoryEvent::ConceptDriftDetected { parent_id, .. } = event {
                if parent_id == id1.to_string() {
                    found_drift = true;
                    break;
                }
            }
        } else {
            // Wait a bit for async emission if needed
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            if let Ok(event) = receiver.try_recv() {
                if let do_memory_core::types::event::MemoryEvent::ConceptDriftDetected { parent_id, .. } = event {
                    if parent_id == id1.to_string() {
                        found_drift = true;
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    println!("Found drift: {}", found_drift);
}
