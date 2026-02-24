//! Tests for token optimization features (dynamic loading and field projection)
//!
//! These tests measure and verify the token reduction achieved by:
//! 1. Dynamic tool loading (only core tools in initial list_tools response)
//! 2. Field projection (only requested fields in query responses)

use memory_mcp::server::tools::field_projection::FieldSelector;
use memory_mcp::server::tools::registry::create_default_registry;
use serde_json::json;

/// Estimate token count for a JSON value
///
/// Note: This is a rough estimate. Actual token count depends on the tokenizer.
/// For OpenAI's cl100k_base tokenizer, a common approximation is:
/// - ~4 characters per token for text
/// - ~1 token per 75 bytes for JSON structure
fn estimate_tokens(value: &serde_json::Value) -> usize {
    let json_str = serde_json::to_string(value).unwrap_or_default();
    // Rough estimate: 1 token per 4 characters
    json_str.len() / 4
}

/// Test that dynamic tool loading reduces initial tool list size
#[tokio::test]
async fn test_dynamic_loading_reduces_initial_tool_list() {
    let registry = create_default_registry();

    // Initially only core tools should be loaded
    let core_tools = registry.get_core_tools();
    let total_tools = registry.total_tool_count();

    println!("Core tools: {}", core_tools.len());
    println!("Total tools available: {}", total_tools);

    // Core tools should be significantly fewer than total tools
    assert!(core_tools.len() < total_tools / 2);

    // Estimate token reduction
    let core_tokens = estimate_tokens(&json!(core_tools));

    // Estimate what all tools would cost (assuming ~170 tokens per tool on average)
    let estimated_all_tokens = total_tools * 170;

    let reduction_percentage =
        ((estimated_all_tokens - core_tokens) as f64 / estimated_all_tokens as f64) * 100.0;

    println!("Core tools tokens: {}", core_tokens);
    println!("Estimated all tools tokens: {}", estimated_all_tokens);
    println!("Token reduction: {:.1}%", reduction_percentage);

    // Keep a strong reduction target while allowing normal toolset growth.
    assert!(reduction_percentage > 75.0);
}

/// Test that field projection reduces query response size
#[test]
fn test_field_projection_reduces_response_size() {
    // Full response
    let full_response = json!({
        "episodes": [
            {
                "id": "episode-1",
                "task_description": "Implement feature X",
                "domain": "web-api",
                "start_time": 1234567890,
                "end_time": Some(1234567900),
                "steps": [
                    {
                        "step_number": 1,
                        "tool": "bash",
                        "action": "run tests",
                        "parameters": {"command": "cargo test"},
                        "result": {"type": "success", "output": "test passed"}
                    }
                ],
                "outcome": {
                    "type": "success",
                    "verdict": "Feature X implemented successfully"
                },
                "reward": {
                    "total": 0.95,
                    "components": {"code_quality": 0.9, "test_coverage": 1.0}
                },
                "reflection": "The implementation went smoothly",
                "patterns": ["pattern-1", "pattern-2"]
            }
        ],
        "patterns": [
            {
                "id": "pattern-1",
                "description": "Test-first development",
                "success_rate": 0.95
            }
        ],
        "insights": {
            "total_episodes": 1,
            "relevant_patterns": 1,
            "success_rate": 0.95
        }
    });

    // Request only specific fields
    let fields = vec![
        "episodes.id".to_string(),
        "episodes.task_description".to_string(),
        "patterns.success_rate".to_string(),
    ];

    let selector = FieldSelector::new(fields.into_iter().collect());
    let filtered = selector.apply(&full_response).unwrap();

    let full_tokens = estimate_tokens(&full_response);
    let filtered_tokens = estimate_tokens(&filtered);

    let reduction_percentage =
        ((full_tokens - filtered_tokens) as f64 / full_tokens as f64) * 100.0;

    println!("Full response tokens: {}", full_tokens);
    println!("Filtered response tokens: {}", filtered_tokens);
    println!("Token reduction: {:.1}%", reduction_percentage);

    // Filtered response should be significantly smaller
    assert!(filtered_tokens < full_tokens);

    // Should achieve at least 30% reduction for this example
    assert!(reduction_percentage > 30.0);

    // Verify that only requested fields are present
    assert!(filtered["episodes"][0].get("id").is_some());
    assert!(filtered["episodes"][0].get("task_description").is_some());
    assert!(filtered["episodes"][0].get("steps").is_none()); // Not requested
    assert!(filtered["episodes"][0].get("outcome").is_none()); // Not requested

    assert!(filtered["patterns"][0].get("success_rate").is_some());
    assert!(filtered["patterns"][0].get("description").is_none()); // Not requested

    assert!(filtered.get("insights").is_none()); // Not requested
}

/// Test field projection with complex nested structures
#[test]
fn test_field_projection_with_nesting() {
    let complex_response = json!({
        "episodes": [
            {
                "id": "1",
                "task_description": "Task 1",
                "outcome": {
                    "type": "success",
                    "verdict": "Good",
                    "artifacts": ["artifact1", "artifact2"]
                },
                "reward": {
                    "total": 0.9,
                    "components": {
                        "code_quality": 0.85,
                        "test_coverage": 0.95
                    }
                }
            }
        ],
        "patterns": [
            {
                "id": "p1",
                "success_rate": 0.88,
                "tool_sequence": ["bash", "edit"]
            }
        ]
    });

    // Request deeply nested fields
    let fields = vec![
        "episodes.id".to_string(),
        "episodes.outcome.type".to_string(),
        "episodes.reward.components.code_quality".to_string(),
        "patterns.tool_sequence".to_string(),
    ];

    let selector = FieldSelector::new(fields.into_iter().collect());
    let filtered = selector.apply(&complex_response).unwrap();

    // Verify deeply nested field selection works
    assert_eq!(filtered["episodes"][0]["id"], "1");
    assert_eq!(filtered["episodes"][0]["outcome"]["type"], "success");
    assert_eq!(
        filtered["episodes"][0]["reward"]["components"]["code_quality"],
        0.85
    );
    assert_eq!(filtered["patterns"][0]["tool_sequence"][0], "bash");

    // Verify non-requested fields are not present
    assert!(filtered["episodes"][0]["outcome"].get("verdict").is_none());
    assert!(filtered["episodes"][0]["reward"].get("total").is_none());
    assert!(filtered["patterns"][0].get("success_rate").is_none());
}

/// Test that field projection is backward compatible (no fields = all fields)
#[test]
fn test_field_projection_backward_compatible() {
    let response = json!({
        "id": "test",
        "name": "Test",
        "value": 42
    });

    // No field selection
    let selector = FieldSelector::new(std::collections::HashSet::new());
    let filtered = selector.apply(&response).unwrap();

    // Should return all fields
    assert_eq!(filtered, response);
}

/// Test lazy loading of extended tools
#[tokio::test]
async fn test_lazy_loading_extended_tools() {
    let registry = create_default_registry();

    // Initially, only core tools are loaded
    let initial_count = registry.loaded_tool_count();
    let core_tools = registry.get_core_tools();
    assert_eq!(initial_count, core_tools.len());

    // Use a known extended tool (bulk_episodes is an extended tool, not in core)
    let extended_name = "bulk_episodes";

    // Verify it's not in core tools
    assert!(
        !core_tools.iter().any(|t| t.name == extended_name),
        "bulk_episodes should be an extended tool, not in core"
    );

    // Load the extended tool
    let tool = registry.load_tool(extended_name).await;
    assert!(tool.is_some());

    // Now it should be loaded
    assert_eq!(registry.loaded_tool_count(), initial_count + 1);
}

/// Test that tool registry tracks usage correctly
#[tokio::test]
async fn test_token_tool_usage_tracking() {
    let registry = create_default_registry();

    // Use a core tool multiple times
    for _ in 0..5 {
        registry.load_tool("query_memory").await;
    }

    let usage = registry.get_tool_usage("query_memory");
    assert_eq!(usage, 5);

    // Use another tool fewer times
    for _ in 0..2 {
        registry.load_tool("health_check").await;
    }

    let usage2 = registry.get_tool_usage("health_check");
    assert_eq!(usage2, 2);
}

/// Test token reduction for typical query scenarios
#[test]
fn test_real_world_token_reduction() {
    // Typical query response (moderately sized)
    let typical_response = json!({
        "episodes": [
            {
                "id": "ep-1",
                "task_description": "Implement user authentication",
                "domain": "web-api",
                "task_type": "code_generation",
                "complexity": "moderate",
                "start_time": 1234567890,
                "end_time": Some(1234568000),
                "steps": [
                    {
                        "step_number": 1,
                        "tool": "edit",
                        "action": "Create auth module",
                        "parameters": {"file": "src/auth.rs"},
                        "result": {"type": "success"}
                    },
                    {
                        "step_number": 2,
                        "tool": "bash",
                        "action": "Run tests",
                        "parameters": {"command": "cargo test"},
                        "result": {"type": "success", "output": "All tests passed"}
                    }
                ],
                "outcome": {
                    "type": "success",
                    "verdict": "Authentication system implemented with JWT tokens"
                },
                "reward": {
                    "total": 0.92,
                    "components": {
                        "code_quality": 0.9,
                        "test_coverage": 0.95,
                        "documentation": 0.85
                    }
                },
                "tags": ["auth", "jwt", "security"]
            },
            {
                "id": "ep-2",
                "task_description": "Fix authentication bug",
                "domain": "web-api",
                "task_type": "debugging",
                "start_time": 1234568100,
                "end_time": Some(1234568200),
                "steps": [
                    {
                        "step_number": 1,
                        "tool": "bash",
                        "action": "Check logs",
                        "parameters": {"command": "tail -f logs/app.log"},
                        "result": {"type": "success", "output": "Found token validation error"}
                    }
                ],
                "outcome": {
                    "type": "success",
                    "verdict": "Fixed token expiration check"
                },
                "reward": {"total": 0.88}
            }
        ],
        "patterns": [
            {
                "id": "pat-1",
                "success_rate": 0.91,
                "tool_sequence": ["edit", "bash", "edit"]
            }
        ],
        "insights": {
            "total_episodes": 2,
            "relevant_patterns": 1,
            "success_rate": 0.9
        }
    });

    // Scenario 1: Client only needs episode IDs and descriptions
    let fields1 = vec![
        "episodes.id".to_string(),
        "episodes.task_description".to_string(),
    ];
    let selector1 = FieldSelector::new(fields1.into_iter().collect());
    let filtered1 = selector1.apply(&typical_response).unwrap();

    let reduction1 = ((estimate_tokens(&typical_response) - estimate_tokens(&filtered1)) as f64
        / estimate_tokens(&typical_response) as f64)
        * 100.0;

    println!(
        "Scenario 1 - IDs and descriptions only: {:.1}% reduction",
        reduction1
    );
    assert!(reduction1 > 50.0); // Should achieve >50% reduction

    // Scenario 2: Client needs full episode data but no patterns
    let fields2 = vec![
        "episodes.id".to_string(),
        "episodes.task_description".to_string(),
        "episodes.outcome".to_string(),
        "episodes.reward".to_string(),
    ];
    let selector2 = FieldSelector::new(fields2.into_iter().collect());
    let filtered2 = selector2.apply(&typical_response).unwrap();

    let reduction2 = ((estimate_tokens(&typical_response) - estimate_tokens(&filtered2)) as f64
        / estimate_tokens(&typical_response) as f64)
        * 100.0;

    println!(
        "Scenario 2 - Episodes without patterns: {:.1}% reduction",
        reduction2
    );
    assert!(reduction2 > 20.0); // Should achieve >20% reduction

    // Scenario 3: Client only needs success statistics
    let fields3 = vec![
        "insights.total_episodes".to_string(),
        "insights.success_rate".to_string(),
    ];
    let selector3 = FieldSelector::new(fields3.into_iter().collect());
    let filtered3 = selector3.apply(&typical_response).unwrap();

    let reduction3 = ((estimate_tokens(&typical_response) - estimate_tokens(&filtered3)) as f64
        / estimate_tokens(&typical_response) as f64)
        * 100.0;

    println!("Scenario 3 - Only statistics: {:.1}% reduction", reduction3);
    assert!(reduction3 > 80.0); // Should achieve >80% reduction
}

/// Benchmark test to show token reduction metrics
#[test]
fn test_token_reduction_metrics() {
    let test_cases = vec![
        // Small response
        (
            json!({"id": "1", "name": "Test", "value": 42}),
            vec!["id".to_string()],
            60.0, // Expected at least 60% reduction
        ),
        // Medium response
        (
            json!({
                "episodes": [{"id": "1", "task": "Test", "steps": 5, "outcome": "success"}],
                "total": 1
            }),
            vec!["episodes.id".to_string(), "episodes.task".to_string()],
            40.0, // Expected at least 40% reduction
        ),
        // Large response
        (
            json!({
                "episodes": [
                    {"id": "1", "task": "A", "steps": 10, "outcome": "success", "reward": 0.9},
                    {"id": "2", "task": "B", "steps": 8, "outcome": "failure", "reward": 0.3}
                ],
                "patterns": [{"id": "p1", "rate": 0.8}],
                "stats": {"total": 2, "success_rate": 0.6}
            }),
            vec!["episodes.id".to_string(), "stats.success_rate".to_string()],
            70.0, // Expected at least 70% reduction
        ),
    ];

    for (i, (response, fields, min_reduction)) in test_cases.into_iter().enumerate() {
        let selector = FieldSelector::new(fields.into_iter().collect());
        let filtered = selector.apply(&response).unwrap();

        let full_tokens = estimate_tokens(&response);
        let filtered_tokens = estimate_tokens(&filtered);
        let reduction = ((full_tokens - filtered_tokens) as f64 / full_tokens as f64) * 100.0;

        println!(
            "Test case {}: {:.1}% reduction ({} -> {} tokens)",
            i + 1,
            reduction,
            full_tokens,
            filtered_tokens
        );

        assert!(
            reduction >= min_reduction,
            "Test case {} expected at least {}% reduction, got {:.1}%",
            i + 1,
            min_reduction,
            reduction
        );
    }
}
