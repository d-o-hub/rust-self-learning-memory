//! Input validation and resource limit tests for memory-core
//!
//! Tests verify the system handles various edge cases gracefully:
//! - Large inputs
//! - Many metadata fields
//! - Many execution steps
//! - Special characters
//! - Nested JSON
//! - Empty inputs

use memory_core::memory::SelfLearningMemory;
use memory_core::{
    ComplexityLevel, ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType,
};
use uuid::Uuid;

#[tokio::test]
async fn test_handles_large_episode_description() {
    // Test with 1MB description
    let memory = SelfLearningMemory::new();

    let large_description = "x".repeat(1_000_000); // 1MB

    let result_id = memory
        .start_episode(
            large_description.clone(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Should accept large input - verify it stores correctly
    let episode = memory.get_episode(result_id).await.unwrap();
    assert_eq!(
        episode.task_description.len(),
        1_000_000,
        "Should store full 1MB description"
    );
    assert_eq!(episode.task_description, large_description);
}

#[tokio::test]
async fn test_handles_excessive_metadata_fields() {
    let memory = SelfLearningMemory::new();

    let id = memory
        .start_episode("Test".to_string(), TaskContext::default(), TaskType::Other)
        .await;

    let mut episode = memory.get_episode(id).await.unwrap();

    // Try adding 1000 metadata fields
    let mut metadata = episode.metadata.clone();
    for i in 0..1000 {
        metadata.insert(format!("field_{}", i), format!("value_{}", i));
    }

    // System should handle this gracefully
    episode.metadata = metadata;

    // Verify we can serialize with many fields
    let json_result = serde_json::to_string(&episode);
    assert!(
        json_result.is_ok(),
        "Should be able to serialize episode with 1000 metadata fields"
    );

    let json = json_result.unwrap();
    assert!(
        json.contains("field_0"),
        "Should contain first metadata field"
    );
    assert!(
        json.contains("field_999"),
        "Should contain last metadata field"
    );
}

#[tokio::test]
async fn test_handles_many_execution_steps() {
    let memory = SelfLearningMemory::new();

    let id = memory
        .start_episode("Test".to_string(), TaskContext::default(), TaskType::Other)
        .await;

    // Add 100 steps
    for i in 0..100 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), format!("action_{}", i));
        step.parameters = serde_json::json!({"step": i});
        step.result = Some(ExecutionResult::Success {
            output: format!("output_{}", i),
        });
        step.latency_ms = 10;
        step.tokens_used = Some(50);

        memory.log_step(id, step).await;
    }

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(episode.steps.len(), 100, "Should store all 100 steps");

    // Verify first and last steps
    assert_eq!(episode.steps[0].step_number, 1);
    assert_eq!(episode.steps[0].tool, "tool_0");
    assert_eq!(episode.steps[99].step_number, 100);
    assert_eq!(episode.steps[99].tool, "tool_99");
}

#[tokio::test]
async fn test_uuid_type_safety() {
    // Document that UUIDs provide compile-time safety
    let memory = SelfLearningMemory::new();

    let valid_id = Uuid::new_v4();

    // This is type-safe - can't pass invalid UUID
    let result = memory.get_episode(valid_id).await;
    assert!(
        result.is_err(),
        "Non-existent UUID should return NotFound error"
    );

    // Verify the error is NotFound
    match result {
        Err(memory_core::Error::NotFound(id)) => {
            assert_eq!(id, valid_id, "Error should contain the UUID we queried");
        }
        _ => panic!("Expected NotFound error"),
    }

    // Create a real episode and verify UUID works
    let real_id = memory
        .start_episode(
            "Real task".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    let episode = memory.get_episode(real_id).await;
    assert!(
        episode.is_ok(),
        "Valid UUID for existing episode should work"
    );
}

#[tokio::test]
async fn test_empty_task_description() {
    let memory = SelfLearningMemory::new();

    // Empty description should be allowed (might be valid use case)
    let id = memory
        .start_episode("".to_string(), TaskContext::default(), TaskType::Other)
        .await;

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(
        episode.task_description, "",
        "Empty description should be stored as-is"
    );

    // Can still complete the episode
    memory
        .complete_episode(
            id,
            TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let completed = memory.get_episode(id).await.unwrap();
    assert!(completed.is_complete());
}

#[tokio::test]
async fn test_special_characters_in_description() {
    let memory = SelfLearningMemory::new();

    // Unicode, emojis, symbols
    let special_chars = "Task with 中文 émojis 🎉 and symbols @#$%^&*()";
    let id = memory
        .start_episode(
            special_chars.to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(
        episode.task_description, special_chars,
        "Should preserve special characters correctly"
    );

    // Test in execution steps too
    let mut step = ExecutionStep::new(1, "tool_🔧".to_string(), "Action with 特殊文字".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "Output with émojis 🎉".to_string(),
    });

    memory.log_step(id, step).await;

    let updated = memory.get_episode(id).await.unwrap();
    assert_eq!(updated.steps[0].tool, "tool_🔧");
    assert_eq!(updated.steps[0].action, "Action with 特殊文字");
}

#[tokio::test]
async fn test_deeply_nested_json_in_metadata() {
    let memory = SelfLearningMemory::new();

    // Create deeply nested JSON (50 levels)
    let mut nested = serde_json::json!("deepest");
    for _ in 0..50 {
        nested = serde_json::json!({"nested": nested});
    }

    let id = memory
        .start_episode("Test".to_string(), TaskContext::default(), TaskType::Other)
        .await;

    // Add nested JSON as a parameter in an execution step
    let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    step.parameters = nested.clone();
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });

    memory.log_step(id, step).await;

    // Should handle nested JSON (serde has recursion limits but 50 is usually OK)
    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(
        episode.steps[0].parameters, nested,
        "Should preserve deeply nested JSON"
    );

    // Also test serialization
    let json_result = serde_json::to_string(&episode);
    assert!(json_result.is_ok(), "Should serialize nested JSON");
}

#[tokio::test]
async fn test_null_bytes_in_strings() {
    let memory = SelfLearningMemory::new();

    // Strings with null bytes
    let description_with_null = format!("Task{}\0with null", 0 as char);
    let id = memory
        .start_episode(
            description_with_null.clone(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Should handle gracefully
    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(
        episode.task_description, description_with_null,
        "Should preserve null bytes in strings"
    );

    // Test in metadata too
    let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    step.metadata
        .insert("key\0with_null".to_string(), "value\0too".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });

    memory.log_step(id, step).await;

    let updated = memory.get_episode(id).await.unwrap();
    assert!(updated.steps[0].metadata.contains_key("key\0with_null"));
}

#[tokio::test]
async fn test_very_long_tool_sequence() {
    let memory = SelfLearningMemory::new();

    let id = memory
        .start_episode(
            "Long sequence test".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Add 500 steps to test really long sequences
    for i in 0..500 {
        let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i % 10), "action".to_string());
        step.result = Some(if i % 5 == 0 {
            ExecutionResult::Error {
                message: "Simulated error".to_string(),
            }
        } else {
            ExecutionResult::Success {
                output: "OK".to_string(),
            }
        });

        memory.log_step(id, step).await;
    }

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(episode.steps.len(), 500, "Should store 500 steps");

    // Complete and verify patterns can be extracted
    memory
        .complete_episode(
            id,
            TaskOutcome::Success {
                verdict: "Long sequence completed".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    let completed = memory.get_episode(id).await.unwrap();
    assert!(completed.is_complete());
    assert!(completed.reward.is_some());
    assert!(completed.reflection.is_some());
}

#[tokio::test]
async fn test_large_json_parameters() {
    let memory = SelfLearningMemory::new();

    let id = memory
        .start_episode(
            "Large params test".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Create a large JSON object (1000 key-value pairs)
    let mut large_params = serde_json::Map::new();
    for i in 0..1000 {
        large_params.insert(
            format!("param_{}", i),
            serde_json::json!({
                "index": i,
                "data": format!("value_{}", i),
                "nested": {
                    "level1": {
                        "level2": {
                            "value": i * 2
                        }
                    }
                }
            }),
        );
    }

    let mut step = ExecutionStep::new(1, "tool".to_string(), "action".to_string());
    step.parameters = serde_json::Value::Object(large_params);
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });

    memory.log_step(id, step).await;

    let episode = memory.get_episode(id).await.unwrap();
    let params_obj = episode.steps[0].parameters.as_object().unwrap();
    assert_eq!(
        params_obj.len(),
        1000,
        "Should store all 1000 parameter keys"
    );
    assert!(params_obj.contains_key("param_0"));
    assert!(params_obj.contains_key("param_999"));
}

#[tokio::test]
async fn test_task_context_with_many_tags() {
    let memory = SelfLearningMemory::new();

    // Create context with 100 tags
    let many_tags: Vec<String> = (0..100).map(|i| format!("tag_{}", i)).collect();

    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Complex,
        domain: "test-domain".to_string(),
        tags: many_tags.clone(),
    };

    let id = memory
        .start_episode("Test".to_string(), context.clone(), TaskType::Other)
        .await;

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(episode.context.tags.len(), 100, "Should store all 100 tags");
    assert_eq!(episode.context.tags, many_tags);

    // Complete and test retrieval with tag matching
    memory
        .complete_episode(
            id,
            TaskOutcome::Success {
                verdict: "Success".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    // Query with one of the tags
    let query_context = TaskContext {
        tags: vec!["tag_50".to_string()],
        domain: "different-domain".to_string(),
        ..Default::default()
    };

    let relevant = memory
        .retrieve_relevant_context("Test".to_string(), query_context, 10)
        .await;

    assert!(
        !relevant.is_empty(),
        "Should find episode with matching tag"
    );
}

#[tokio::test]
async fn test_whitespace_only_fields() {
    let memory = SelfLearningMemory::new();

    // Test with whitespace-only description
    let whitespace_desc = "   \n\t\r\n   ".to_string();
    let id = memory
        .start_episode(
            whitespace_desc.clone(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(
        episode.task_description, whitespace_desc,
        "Should preserve whitespace"
    );

    // Test with whitespace-only tool and action
    let mut step = ExecutionStep::new(1, "  \t  ".to_string(), "  \n  ".to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });

    memory.log_step(id, step).await;

    let updated = memory.get_episode(id).await.unwrap();
    assert_eq!(updated.steps[0].tool, "  \t  ");
    assert_eq!(updated.steps[0].action, "  \n  ");
}

#[tokio::test]
async fn test_concurrent_step_logging() {
    let memory = SelfLearningMemory::new();

    let id = memory
        .start_episode(
            "Concurrent steps".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    // Log steps concurrently (simulating potential race conditions)
    let mut handles = vec![];
    for i in 0..20 {
        let mem = memory.clone();
        let handle = tokio::spawn(async move {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            mem.log_step(id, step).await;
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(episode.steps.len(), 20, "Should have logged all 20 steps");
}
