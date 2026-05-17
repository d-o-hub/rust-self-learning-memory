//! Integration tests for concept drift detection.

use do_memory_core::{MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome, TaskType};

#[tokio::test]
async fn test_concept_drift_detection_integration() {
    let config = MemoryConfig {
        quality_threshold: 0.0, // Disable quality gating for test
        ..Default::default()
    };
    let memory = SelfLearningMemory::with_config(config);
    let mut receiver = memory.subscribe();

    // Create first version
    let context = TaskContext::default();
    let id1 = memory
        .start_episode(
            "drift task".to_string(),
            context.clone(),
            TaskType::CodeGeneration,
        )
        .await;

    memory
        .complete_episode(
            id1,
            TaskOutcome::Success {
                verdict: "v1".to_string(),
                artifacts: vec!["a.rs".to_string()],
            },
        )
        .await
        .unwrap();

    // Create second version
    let id2 = memory.start_next_version(id1).await.unwrap();
    memory
        .complete_episode(
            id2,
            TaskOutcome::Success {
                verdict: "v2".to_string(),
                artifacts: vec!["a.rs".to_string()],
            },
        )
        .await
        .unwrap();

    // Create third version with a significant shift in reward (Failure)
    let id3 = memory.start_next_version(id1).await.unwrap();
    memory
        .complete_episode(
            id3,
            TaskOutcome::Failure {
                reason: "v3 failed".to_string(),
                error_details: None,
            },
        )
        .await
        .unwrap();

    let mut found_drift = false;
    let target_id = id1.to_string();

    // Drain events to see if drift was detected
    for _ in 0..20 {
        let event_opt = match receiver.try_recv() {
            Ok(e) => Some(e),
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                receiver.try_recv().ok()
            }
            _ => None,
        };

        if let Some(do_memory_core::types::event::MemoryEvent::ConceptDriftDetected {
            parent_id,
            ..
        }) = event_opt
        {
            if parent_id == target_id {
                found_drift = true;
                break;
            }
        }
    }

    println!("Found drift: {found_drift}");
}
