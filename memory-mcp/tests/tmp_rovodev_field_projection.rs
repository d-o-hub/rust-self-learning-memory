//! Test field selection/projection optimization
//!
//! This test verifies the token optimization achieved through field projection:
//! - Clients can request specific fields to reduce output tokens
//! - 20-60% token reduction on large responses

use memory_mcp::server::tools::field_projection::FieldSelector;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn test_field_selector_basic() {
    // Create a selector for specific fields
    let mut fields = HashSet::new();
    fields.insert("episode.id".to_string());
    fields.insert("episode.task_description".to_string());

    let selector = FieldSelector::new(fields);

    // Apply to a full result
    let full_result = json!({
        "episode": {
            "id": "123",
            "task_description": "Test task",
            "steps": [
                {"action": "step1", "result": "data"},
                {"action": "step2", "result": "more data"}
            ],
            "outcome": "success",
            "reflection": "Long reflection text here..."
        }
    });

    let filtered = selector.apply(&full_result).unwrap();

    // Verify only requested fields are present
    let filtered_str = serde_json::to_string(&filtered).unwrap();
    assert!(filtered_str.contains("\"id\""));
    assert!(filtered_str.contains("\"task_description\""));

    // Verify size reduction
    let full_size = serde_json::to_string(&full_result).unwrap().len();
    let filtered_size = filtered_str.len();
    let reduction_percent = ((full_size - filtered_size) as f64 / full_size as f64) * 100.0;

    println!("Full size: {} bytes", full_size);
    println!("Filtered size: {} bytes", filtered_size);
    println!("Reduction: {:.1}%", reduction_percent);

    assert!(
        reduction_percent > 20.0,
        "Should achieve at least 20% reduction"
    );
}

#[test]
fn test_field_selector_from_request() {
    // Test creating selector from request args
    let args = json!({
        "episode_id": "123",
        "fields": ["episode.id", "episode.task_description"]
    });

    let selector = FieldSelector::from_request(&args);

    let result = json!({
        "episode": {
            "id": "123",
            "task_description": "Test",
            "steps": [],
            "outcome": "success"
        }
    });

    let filtered = selector.apply(&result).unwrap();
    let filtered_str = serde_json::to_string(&filtered).unwrap();

    assert!(filtered_str.len() < serde_json::to_string(&result).unwrap().len());
}

#[test]
fn test_field_selector_no_fields_returns_all() {
    // When no fields specified, should return all (backward compatible)
    let args = json!({
        "episode_id": "123"
    });

    let selector = FieldSelector::from_request(&args);

    let result = json!({
        "episode": {
            "id": "123",
            "task_description": "Test",
            "steps": []
        }
    });

    let filtered = selector.apply(&result).unwrap();

    // Should be identical
    assert_eq!(
        serde_json::to_string(&result).unwrap(),
        serde_json::to_string(&filtered).unwrap()
    );
}

#[test]
fn test_field_selector_nested_fields() {
    // Test deeply nested field selection
    let mut fields = HashSet::new();
    fields.insert("patterns.success_rate".to_string());
    fields.insert("insights.total_episodes".to_string());

    let selector = FieldSelector::new(fields);

    let result = json!({
        "episodes": [],
        "patterns": {
            "success_rate": 0.85,
            "tool_sequence": ["tool1", "tool2"],
            "effectiveness": 9.5
        },
        "insights": {
            "total_episodes": 42,
            "relevant_patterns": 5,
            "avg_duration": 120
        }
    });

    let filtered = selector.apply(&result).unwrap();
    let filtered_str = serde_json::to_string(&filtered).unwrap();

    // Should contain only selected nested fields
    assert!(filtered_str.contains("success_rate"));
    assert!(filtered_str.contains("total_episodes"));
    assert!(!filtered_str.contains("tool_sequence"));
    assert!(!filtered_str.contains("avg_duration"));
}

#[test]
fn test_token_reduction_calculation() {
    // Simulate realistic query_memory response
    let full_response = json!({
        "episodes": [
            {
                "id": "ep1",
                "task_description": "Debug authentication issue",
                "steps": [
                    {"action": "check_logs", "result": "found error"},
                    {"action": "inspect_code", "result": "identified bug"},
                    {"action": "apply_fix", "result": "resolved"}
                ],
                "outcome": "success",
                "reflection": "The issue was caused by incorrect token validation...",
                "reward": {"total": 0.95}
            },
            {
                "id": "ep2",
                "task_description": "Optimize database query",
                "steps": [
                    {"action": "profile_query", "result": "slow join"},
                    {"action": "add_index", "result": "improved"}
                ],
                "outcome": "success",
                "reflection": "Adding index on foreign key improved performance by 10x...",
                "reward": {"total": 0.88}
            }
        ],
        "patterns": [
            {"type": "ToolSequence", "tools": ["check_logs", "inspect_code", "apply_fix"], "success_rate": 0.92}
        ],
        "insights": {
            "total_episodes": 2,
            "relevant_patterns": 1,
            "success_rate": 0.915
        }
    });

    // Request only IDs and descriptions
    let mut fields = HashSet::new();
    fields.insert("episodes.id".to_string());
    fields.insert("episodes.task_description".to_string());
    fields.insert("insights.total_episodes".to_string());

    let selector = FieldSelector::new(fields);
    let filtered = selector.apply(&full_response).unwrap();

    let full_size = serde_json::to_string(&full_response).unwrap().len();
    let filtered_size = serde_json::to_string(&filtered).unwrap().len();
    let reduction_percent = ((full_size - filtered_size) as f64 / full_size as f64) * 100.0;

    println!(
        "Full response: {} bytes (~{} tokens)",
        full_size,
        full_size / 4
    );
    println!(
        "Filtered response: {} bytes (~{} tokens)",
        filtered_size,
        filtered_size / 4
    );
    println!("Token reduction: {:.1}%", reduction_percent);

    // Should achieve significant reduction
    assert!(
        reduction_percent >= 40.0,
        "Should achieve at least 40% reduction for selective fields"
    );
}

#[test]
fn test_empty_fields_array() {
    // Empty fields array should return all fields
    let args = json!({
        "fields": []
    });

    let selector = FieldSelector::from_request(&args);

    let result = json!({"test": "data"});
    let filtered = selector.apply(&result).unwrap();

    assert_eq!(result, filtered);
}

#[test]
fn test_invalid_fields_parameter() {
    // Invalid fields parameter should default to returning all
    let args = json!({
        "fields": "not_an_array"
    });

    let selector = FieldSelector::from_request(&args);

    let result = json!({"test": "data"});
    let filtered = selector.apply(&result).unwrap();

    assert_eq!(result, filtered);
}
