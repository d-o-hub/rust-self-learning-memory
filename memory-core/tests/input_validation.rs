//! BDD-style input validation and resource limit tests for memory-core
//!
//! These tests verify that the system handles various edge cases gracefully:
//! - Large inputs (descriptions, metadata, steps, JSON)
//! - Special characters (unicode, emojis, null bytes, whitespace)
//! - Type safety (UUIDs)
//!
//! All tests follow the Given-When-Then pattern for clarity.

mod common;

use common::{setup_test_memory, ContextBuilder, StepBuilder};
use memory_core::{ExecutionResult, ExecutionStep, TaskContext, TaskOutcome, TaskType};
use uuid::Uuid;

/// Test data for large input scenarios
#[derive(Debug)]
struct LargeInputTest {
    name: &'static str,
    description_size: usize,
    metadata_count: usize,
    step_count: usize,
    json_params_count: usize,
    tags_count: usize,
}

#[tokio::test]
async fn should_handle_large_inputs_without_data_loss() {
    // Given: Various large input scenarios
    let test_cases = vec![
        LargeInputTest {
            name: "Large description (1MB)",
            description_size: 1_000_000,
            metadata_count: 0,
            step_count: 1,
            json_params_count: 0,
            tags_count: 0,
        },
        LargeInputTest {
            name: "Excessive metadata (1000 fields)",
            description_size: 100,
            metadata_count: 1000,
            step_count: 1,
            json_params_count: 0,
            tags_count: 0,
        },
        LargeInputTest {
            name: "Many steps (100)",
            description_size: 100,
            metadata_count: 0,
            step_count: 100,
            json_params_count: 0,
            tags_count: 0,
        },
        LargeInputTest {
            name: "Very long sequence (500 steps)",
            description_size: 100,
            metadata_count: 0,
            step_count: 500,
            json_params_count: 0,
            tags_count: 0,
        },
        LargeInputTest {
            name: "Large JSON parameters (1000 fields)",
            description_size: 100,
            metadata_count: 0,
            step_count: 1,
            json_params_count: 1000,
            tags_count: 0,
        },
        LargeInputTest {
            name: "Many tags (100)",
            description_size: 100,
            metadata_count: 0,
            step_count: 1,
            json_params_count: 0,
            tags_count: 100,
        },
    ];

    // When: We test each scenario
    for test_case in test_cases {
        println!("Testing: {}", test_case.name);
        let memory = setup_test_memory();

        // Given: Appropriate test data size
        let description = if test_case.description_size > 100 {
            "x".repeat(test_case.description_size)
        } else {
            "Test".to_string()
        };

        let mut context_builder = ContextBuilder::new("test-domain").language("rust");
        if test_case.tags_count > 0 {
            for i in 0..test_case.tags_count {
                context_builder = context_builder.tag(format!("tag_{}", i));
            }
        }
        let context = context_builder.build();

        // When: We create an episode with large data
        let id = memory
            .start_episode(description.clone(), context.clone(), TaskType::Other)
            .await;

        // Then: Large description should be stored completely
        if test_case.description_size > 100 {
            let episode = memory.get_episode(id).await.unwrap();
            assert_eq!(
                episode.task_description.len(),
                test_case.description_size,
                "Should store full description"
            );
        }

        // Then: Large metadata should serialize correctly
        if test_case.metadata_count > 0 {
            let mut episode = memory.get_episode(id).await.unwrap();
            for i in 0..test_case.metadata_count {
                episode
                    .metadata
                    .insert(format!("field_{}", i), format!("value_{}", i));
            }
            let json_result = serde_json::to_string(&episode);
            assert!(
                json_result.is_ok(),
                "Should serialize with {} metadata fields",
                test_case.metadata_count
            );
        }

        // When: We add many steps
        for i in 0..test_case.step_count {
            let mut step_builder =
                StepBuilder::new(i + 1, format!("tool_{}", i), format!("action_{}", i));

            // Add large JSON parameters if needed
            if test_case.json_params_count > 0 && i == 0 {
                let mut large_params = serde_json::Map::new();
                for j in 0..test_case.json_params_count {
                    large_params.insert(
                        format!("param_{}", j),
                        serde_json::json!({
                            "index": j,
                            "data": format!("value_{}", j),
                            "nested": {"level1": {"level2": {"value": j * 2}}}
                        }),
                    );
                }
                step_builder = step_builder.parameters(serde_json::Value::Object(large_params));
            }

            let step = step_builder.success("OK").build();
            memory.log_step(id, step).await;
        }

        // Then: All steps should be stored
        let episode = memory.get_episode(id).await.unwrap();
        assert_eq!(
            episode.steps.len(),
            test_case.step_count,
            "Should store all {} steps for test: {}",
            test_case.step_count,
            test_case.name
        );

        // Then: All tags should be stored
        if test_case.tags_count > 0 {
            assert_eq!(
                episode.context.tags.len(),
                test_case.tags_count,
                "Should store all {} tags",
                test_case.tags_count
            );
        }

        // Verify can complete episode even with large data
        if test_case.step_count <= 100 {
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
        }
    }
}

/// Test data for special character and edge case scenarios
#[derive(Debug)]
struct SpecialCharTest {
    name: &'static str,
    description: String,
    tool: String,
    action: String,
    should_test_in_step: bool,
}

#[tokio::test]
async fn should_handle_special_characters_and_edge_cases_gracefully() {
    // Given: Various special character and edge case scenarios
    let test_cases = vec![
        SpecialCharTest {
            name: "Empty description",
            description: "".to_string(),
            tool: "tool".to_string(),
            action: "action".to_string(),
            should_test_in_step: false,
        },
        SpecialCharTest {
            name: "Unicode and emojis",
            description: "Task with 中文 émojis 🎉 and symbols @#$%^&*()".to_string(),
            tool: "tool_🔧".to_string(),
            action: "Action with 特殊文字".to_string(),
            should_test_in_step: true,
        },
        SpecialCharTest {
            name: "Null bytes",
            description: format!("Task{}\0with null", '\0'),
            tool: "tool".to_string(),
            action: "action".to_string(),
            should_test_in_step: true,
        },
        SpecialCharTest {
            name: "Whitespace only",
            description: "   \n\t\r\n   ".to_string(),
            tool: "  \t  ".to_string(),
            action: "  \n  ".to_string(),
            should_test_in_step: true,
        },
    ];

    // When: We test each scenario
    for test_case in test_cases {
        println!("Testing: {}", test_case.name);
        let memory = setup_test_memory();

        // When: We create an episode with special characters
        let id = memory
            .start_episode(
                test_case.description.clone(),
                TaskContext::default(),
                TaskType::Other,
            )
            .await;

        // Then: The description should be preserved exactly
        let episode = memory.get_episode(id).await.unwrap();
        assert_eq!(
            episode.task_description, test_case.description,
            "Should preserve description for: {}",
            test_case.name
        );

        // When: We log steps with special characters (if applicable)
        if test_case.should_test_in_step {
            let mut step = ExecutionStep::new(1, test_case.tool.clone(), test_case.action.clone());
            step.result = Some(ExecutionResult::Success {
                output: format!("Output for {}", test_case.name),
            });

            // Add metadata with null bytes for null byte test
            if test_case.name == "Null bytes" {
                step.metadata
                    .insert("key\0with_null".to_string(), "value\0too".to_string());
            }

            memory.log_step(id, step).await;

            // Then: Special characters should be preserved in steps
            let updated = memory.get_episode(id).await.unwrap();
            assert_eq!(updated.steps[0].tool, test_case.tool);
            assert_eq!(updated.steps[0].action, test_case.action);

            if test_case.name == "Null bytes" {
                assert!(updated.steps[0].metadata.contains_key("key\0with_null"));
            }
        }

        // Then: Episode with special characters should complete successfully
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
}

#[tokio::test]
async fn should_handle_deeply_nested_json_structures() {
    // Given: A memory system and deeply nested JSON (50 levels)
    let memory = setup_test_memory();

    let mut nested = serde_json::json!("deepest");
    for _ in 0..50 {
        nested = serde_json::json!({"nested": nested});
    }

    // When: We create an episode with deeply nested JSON parameters
    let id = memory
        .start_episode("Test".to_string(), TaskContext::default(), TaskType::Other)
        .await;

    let step = StepBuilder::new(1, "tool", "action")
        .parameters(nested.clone())
        .success("OK")
        .build();
    memory.log_step(id, step).await;

    // Then: The nested JSON should be preserved
    let episode = memory.get_episode(id).await.unwrap();
    assert_eq!(
        episode.steps[0].parameters, nested,
        "Should preserve deeply nested JSON"
    );

    // And: The episode should serialize successfully
    let json_result = serde_json::to_string(&episode);
    assert!(json_result.is_ok(), "Should serialize nested JSON");
}

#[tokio::test]
async fn should_provide_type_safe_uuid_handling() {
    // Given: A memory system
    let memory = setup_test_memory();

    // When: We query with a valid but non-existent UUID
    let valid_id = Uuid::new_v4();
    let result = memory.get_episode(valid_id).await;

    // Then: The system should return a NotFound error
    assert!(
        result.is_err(),
        "Non-existent UUID should return NotFound error"
    );

    // And: The error should contain the UUID we queried
    match result {
        Err(memory_core::Error::NotFound(id)) => {
            assert_eq!(id, valid_id, "Error should contain the UUID we queried");
        }
        _ => panic!("Expected NotFound error"),
    }

    // When: We create a real episode and query with its UUID
    let real_id = memory
        .start_episode(
            "Real task".to_string(),
            TaskContext::default(),
            TaskType::Other,
        )
        .await;

    let episode = memory.get_episode(real_id).await;

    // Then: The query should succeed for existing episodes
    assert!(
        episode.is_ok(),
        "Valid UUID for existing episode should work"
    );
}
