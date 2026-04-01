//! Snapshot tests for MCP tool responses and schemas
//!
//! These tests verify that tool definitions, responses, and error formats
//! remain consistent across changes. Part of ADR-033 Phase 6.

use do_memory_mcp::mcp::tools::checkpoint::{
    CheckpointEpisodeInput, CheckpointEpisodeOutput, GetHandoffPackInput, GetHandoffPackOutput,
};
use do_memory_mcp::mcp::tools::embeddings::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, EmbeddingProviderStatusInput,
    EmbeddingProviderStatusOutput, EmbeddingSearchResult, GenerateEmbeddingInput,
    GenerateEmbeddingOutput, ProviderTestResult, QuerySemanticMemoryInput,
    QuerySemanticMemoryOutput, SearchByEmbeddingInput, SearchByEmbeddingOutput, SemanticResult,
    TestEmbeddingsOutput,
};
use do_memory_mcp::mcp::tools::pattern_search::{
    PatternResult, ScoreBreakdownResult, SearchPatternsInput, SearchPatternsOutput,
};
use do_memory_mcp::mcp::tools::recommendation_feedback::{
    RecommendationStatsOutput, RecordRecommendationFeedbackInput, RecordRecommendationSessionInput,
    TaskOutcomeJson,
};
use do_memory_mcp::types::{
    ErrorType, ExecutionContext, ExecutionResult, ExecutionStats, SecurityViolationType, Tool,
};
use insta::assert_json_snapshot;
use serde_json::json;

/// Test that Tool struct serialization produces consistent output
#[test]
fn test_tool_definition_serialization() {
    let tool = Tool::new(
        "query_memory".to_string(),
        "Query episodic memory for relevant past experiences".to_string(),
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query describing the task"
                },
                "domain": {
                    "type": "string",
                    "description": "Task domain"
                }
            },
            "required": ["query", "domain"]
        }),
    );

    assert_json_snapshot!(tool);
}

/// Test execution result success format
#[test]
fn test_execution_result_success() {
    let result = ExecutionResult::Success {
        output: r#"{"sum": 42, "message": "Hello from sandbox"}"#.to_string(),
        stdout: "Processing...\nDone!".to_string(),
        stderr: "".to_string(),
        execution_time_ms: 150,
    };

    assert_json_snapshot!(result);
}

/// Test execution result error format
#[test]
fn test_execution_result_error() {
    let result = ExecutionResult::Error {
        message: "ReferenceError: undefined_variable is not defined".to_string(),
        error_type: ErrorType::Runtime,
        stdout: "".to_string(),
        stderr: "Error at line 5: undefined_variable".to_string(),
    };

    assert_json_snapshot!(result);
}

/// Test execution result timeout format
#[test]
fn test_execution_result_timeout() {
    let result = ExecutionResult::Timeout {
        elapsed_ms: 5000,
        partial_output: Some("Processing step 1...".to_string()),
    };

    assert_json_snapshot!(result);
}

/// Test execution result security violation format
#[test]
fn test_execution_result_security_violation() {
    use do_memory_mcp::types::SecurityViolationType;

    let result = ExecutionResult::SecurityViolation {
        reason: "Attempted filesystem access to /etc/passwd".to_string(),
        violation_type: SecurityViolationType::FileSystemAccess,
    };

    assert_json_snapshot!(result);
}

/// Test execution stats serialization
#[test]
fn test_execution_stats_serialization() {
    let stats = ExecutionStats {
        total_executions: 150,
        successful_executions: 142,
        failed_executions: 8,
        timeout_count: 3,
        security_violations: 1,
        avg_execution_time_ms: 245.5,
    };

    assert_json_snapshot!(stats);
}

/// Test execution context serialization
#[test]
fn test_execution_context_serialization() {
    let mut ctx = ExecutionContext::new(
        "Calculate sum of array".to_string(),
        json!({
            "numbers": [1, 2, 3, 4, 5],
            "operation": "sum"
        }),
    );

    // Add some environment variables
    ctx.env.insert("DEBUG".to_string(), "true".to_string());
    ctx.env.insert("NODE_ENV".to_string(), "test".to_string());

    // Serialize and verify structure (avoid direct snapshot due to HashMap ordering)
    let json_str = serde_json::to_string_pretty(&ctx).unwrap();
    assert!(json_str.contains("\"task\": \"Calculate sum of array\""));
    assert!(json_str.contains("\"DEBUG\": \"true\""));
    assert!(json_str.contains("\"NODE_ENV\": \"test\""));
    assert!(json_str.contains("\"numbers\": ["));
    assert!(json_str.contains("\"operation\": \"sum\""));
}

/// Test multiple tool definitions in a collection
#[test]
fn test_tool_definitions_collection() {
    let tools = vec![
        Tool::new(
            "query_memory".to_string(),
            "Query episodic memory".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 10}
                },
                "required": ["query"]
            }),
        ),
        Tool::new(
            "analyze_patterns".to_string(),
            "Analyze patterns from episodes".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "task_type": {"type": "string"},
                    "min_success_rate": {"type": "number", "default": 0.7}
                },
                "required": ["task_type"]
            }),
        ),
        Tool::new(
            "health_check".to_string(),
            "Check server health".to_string(),
            json!({"type": "object", "properties": {}}),
        ),
    ];

    assert_json_snapshot!(tools);
}

// =============================================================================
// Embedding Tool Type Snapshots
// =============================================================================

/// Test GenerateEmbeddingInput serialization
#[test]
fn test_generate_embedding_input() {
    let input = GenerateEmbeddingInput {
        text: "This is a sample text for embedding generation.".to_string(),
        normalize: true,
    };

    assert_json_snapshot!(input);
}

/// Test GenerateEmbeddingOutput serialization
#[test]
fn test_generate_embedding_output() {
    let output = GenerateEmbeddingOutput {
        embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        dimension: 5,
        model: "text-embedding-3-small".to_string(),
        provider: "openai".to_string(),
        generation_time_ms: 42.5,
        normalized: true,
        token_count: Some(8),
    };

    assert_json_snapshot!(output);
}

/// Test SearchByEmbeddingInput serialization
#[test]
fn test_search_by_embedding_input() {
    let input = SearchByEmbeddingInput {
        embedding: vec![0.1, 0.2, 0.3],
        limit: 10,
        similarity_threshold: 0.7,
        domain: Some("web-api".to_string()),
        task_type: Some("code-generation".to_string()),
    };

    assert_json_snapshot!(input);
}

/// Test SearchByEmbeddingOutput serialization
#[test]
fn test_search_by_embedding_output() {
    let output = SearchByEmbeddingOutput {
        results_found: 2,
        results: vec![
            EmbeddingSearchResult {
                episode_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
                similarity_score: 0.95,
                task_description: "Implement REST API endpoint".to_string(),
                domain: "web-api".to_string(),
                task_type: "code-generation".to_string(),
                outcome: Some("Successfully implemented GET /users endpoint".to_string()),
                timestamp: 1709827200,
            },
            EmbeddingSearchResult {
                episode_id: "550e8400-e29b-41d4-a716-446655440002".to_string(),
                similarity_score: 0.82,
                task_description: "Create authentication middleware".to_string(),
                domain: "web-api".to_string(),
                task_type: "code-generation".to_string(),
                outcome: None,
                timestamp: 1709740800,
            },
        ],
        embedding_dimension: 1536,
        search_time_ms: 15.3,
        provider: "openai".to_string(),
    };

    assert_json_snapshot!(output);
}

/// Test EmbeddingProviderStatusInput serialization
#[test]
fn test_embedding_provider_status_input() {
    let input = EmbeddingProviderStatusInput {
        test_connectivity: true,
    };

    assert_json_snapshot!(input);
}

/// Test EmbeddingProviderStatusOutput serialization
#[test]
fn test_embedding_provider_status_output() {
    let output = EmbeddingProviderStatusOutput {
        configured: true,
        available: true,
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        dimension: 1536,
        similarity_threshold: 0.7,
        batch_size: 100,
        cache_enabled: true,
        metadata: json!({
            "api_version": "v1",
            "rate_limit": "3000 per minute"
        }),
        test_result: Some(ProviderTestResult {
            success: true,
            duration_ms: 125,
            sample_embedding: vec![0.0123, -0.0456, 0.0789],
            error: None,
        }),
        warnings: vec![],
    };

    assert_json_snapshot!(output);
}

/// Test ProviderTestResult serialization (success case)
#[test]
fn test_provider_test_result_success() {
    let result = ProviderTestResult {
        success: true,
        duration_ms: 125,
        sample_embedding: vec![0.0123, -0.0456, 0.0789],
        error: None,
    };

    assert_json_snapshot!(result);
}

/// Test ProviderTestResult serialization (failure case)
#[test]
fn test_provider_test_result_failure() {
    let result = ProviderTestResult {
        success: false,
        duration_ms: 5000,
        sample_embedding: vec![],
        error: Some("Connection timeout after 5000ms".to_string()),
    };

    assert_json_snapshot!(result);
}

/// Test ConfigureEmbeddingsInput serialization
#[test]
fn test_configure_embeddings_input() {
    let input = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-small".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        similarity_threshold: Some(0.75),
        batch_size: Some(50),
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    assert_json_snapshot!(input);
}

/// Test ConfigureEmbeddingsOutput serialization
#[test]
fn test_configure_embeddings_output() {
    let output = ConfigureEmbeddingsOutput {
        success: true,
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        dimension: 1536,
        message: "Successfully configured OpenAI embedding provider".to_string(),
        warnings: vec![],
    };

    assert_json_snapshot!(output);
}

/// Test QuerySemanticMemoryInput serialization
#[test]
fn test_query_semantic_memory_input() {
    let input = QuerySemanticMemoryInput {
        query: "How do I implement rate limiting?".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.6),
        domain: Some("web-api".to_string()),
        task_type: None,
    };

    assert_json_snapshot!(input);
}

/// Test QuerySemanticMemoryOutput serialization
#[test]
fn test_query_semantic_memory_output() {
    let output = QuerySemanticMemoryOutput {
        results_found: 1,
        results: vec![SemanticResult {
            episode_id: "550e8400-e29b-41d4-a716-446655440003".to_string(),
            similarity_score: 0.88,
            task_description: "Implement rate limiting middleware".to_string(),
            domain: "web-api".to_string(),
            task_type: "implementation".to_string(),
            outcome: Some("Added token bucket rate limiter with 100 req/min limit".to_string()),
            timestamp: 1709654400,
        }],
        embedding_dimension: 1536,
        query_time_ms: 28.7,
        provider: "openai".to_string(),
    };

    assert_json_snapshot!(output);
}

/// Test TestEmbeddingsOutput serialization (available)
#[test]
fn test_embeddings_output_available() {
    let output = TestEmbeddingsOutput {
        available: true,
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        dimension: 1536,
        test_time_ms: 125,
        sample_embedding: vec![0.0123, -0.0456, 0.0789, 0.0234, -0.0567],
        message: "Embedding provider is available and working correctly".to_string(),
        errors: vec![],
    };

    assert_json_snapshot!(output);
}

/// Test TestEmbeddingsOutput serialization (unavailable)
#[test]
fn test_embeddings_output_unavailable() {
    let output = TestEmbeddingsOutput {
        available: false,
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        dimension: 0,
        test_time_ms: 5000,
        sample_embedding: vec![],
        message: "Embedding provider test failed".to_string(),
        errors: vec!["Connection timeout after 5000ms".to_string()],
    };

    assert_json_snapshot!(output);
}

/// Test all SecurityViolationType variants
#[test]
fn test_security_violation_types() {
    let violations = vec![
        SecurityViolationType::FileSystemAccess,
        SecurityViolationType::NetworkAccess,
        SecurityViolationType::ProcessExecution,
        SecurityViolationType::MemoryLimit,
        SecurityViolationType::InfiniteLoop,
        SecurityViolationType::MaliciousCode,
    ];

    assert_json_snapshot!(violations);
}

/// Test all ErrorType variants
#[test]
fn test_error_types() {
    let errors = vec![
        ErrorType::Syntax,
        ErrorType::Runtime,
        ErrorType::Permission,
        ErrorType::Resource,
        ErrorType::Unknown,
    ];

    assert_json_snapshot!(errors);
}

// =============================================================================
// Checkpoint Tool Type Snapshots
// =============================================================================

#[test]
fn test_checkpoint_episode_input() {
    let input = CheckpointEpisodeInput {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        reason: "Agent switch".to_string(),
        note: Some("Testing checkpoint snapshots".to_string()),
    };
    assert_json_snapshot!(input);
}

#[test]
fn test_checkpoint_episode_output() {
    let output = CheckpointEpisodeOutput {
        success: true,
        checkpoint_id: "770e8400-e29b-41d4-a716-446655440000".to_string(),
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        step_number: 12,
        message: "Checkpoint created successfully".to_string(),
    };
    assert_json_snapshot!(output);
}

#[test]
fn test_get_handoff_pack_input() {
    let input = GetHandoffPackInput {
        checkpoint_id: "770e8400-e29b-41d4-a716-446655440000".to_string(),
    };
    assert_json_snapshot!(input);
}

#[test]
fn test_get_handoff_pack_output() {
    let output = GetHandoffPackOutput {
        success: true,
        handoff_pack: None,
        message: "Handoff pack retrieved".to_string(),
    };
    assert_json_snapshot!(output);
}

// =============================================================================
// Recommendation Feedback Tool Type Snapshots
// =============================================================================

#[test]
fn test_record_recommendation_session_input() {
    let input = RecordRecommendationSessionInput {
        episode_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        recommended_pattern_ids: vec!["p1".to_string(), "p2".to_string()],
        recommended_playbook_ids: vec!["play1".to_string()],
    };
    assert_json_snapshot!(input);
}

#[test]
fn test_record_recommendation_feedback_input() {
    let input = RecordRecommendationFeedbackInput {
        session_id: "990e8400-e29b-41d4-a716-446655440000".to_string(),
        applied_pattern_ids: vec!["p1".to_string()],
        consulted_episode_ids: vec!["ep-past".to_string()],
        outcome: TaskOutcomeJson::Success {
            verdict: "Worked great".to_string(),
            artifacts: vec!["auth.rs".to_string()],
        },
        agent_rating: Some(0.9),
    };
    assert_json_snapshot!(input);
}

#[test]
fn test_recommendation_stats_output() {
    let output = RecommendationStatsOutput {
        success: true,
        total_sessions: 10,
        total_feedback: 8,
        patterns_applied: 5,
        patterns_ignored: 3,
        adoption_rate: 0.625,
        success_after_adoption_rate: 0.8,
        avg_agent_rating: Some(0.85),
        message: "Stats retrieved".to_string(),
    };
    assert_json_snapshot!(output);
}

// =============================================================================
// Pattern Search Tool Type Snapshots
// =============================================================================

#[test]
fn test_search_patterns_input() {
    let input = SearchPatternsInput {
        query: "rust authentication".to_string(),
        domain: "web".to_string(),
        tags: vec!["security".to_string()],
        limit: 5,
        min_relevance: 0.5,
        filter_by_domain: true,
    };
    assert_json_snapshot!(input);
}

#[test]
fn test_search_patterns_output() {
    let output = SearchPatternsOutput {
        results: vec![PatternResult {
            id: "p1".to_string(),
            pattern_type: "tool_sequence".to_string(),
            description: "Common auth sequence".to_string(),
            relevance_score: 0.9,
            score_breakdown: ScoreBreakdownResult {
                semantic_similarity: 0.95,
                context_match: 0.8,
                effectiveness: 0.9,
                recency: 1.0,
                success_rate: 0.85,
            },
            success_rate: 0.85,
            domain: Some("web".to_string()),
            times_applied: 15,
        }],
        total_searched: 1,
        query: "rust authentication".to_string(),
    };
    assert_json_snapshot!(output);
}

// =============================================================================
// Playbook Type Snapshots (ADR-044 Feature 1)
// =============================================================================

/// Test PlaybookStep serialization
#[test]
fn test_playbook_step() {
    use do_memory_core::memory::playbook::PlaybookStep;

    let step = PlaybookStep::new(1, "Analyze existing authentication patterns".to_string())
        .with_tool_hint("pattern_search")
        .with_expected_result("List of relevant authentication patterns");

    assert_json_snapshot!(step);
}

/// Test PlaybookStep with minimal fields
#[test]
fn test_playbook_step_minimal() {
    use do_memory_core::memory::playbook::PlaybookStep;

    let step = PlaybookStep::new(2, "Run tests to verify changes".to_string());

    assert_json_snapshot!(step);
}

/// Test PlaybookPitfall serialization
#[test]
fn test_playbook_pitfall() {
    use do_memory_core::memory::playbook::PlaybookPitfall;

    let pitfall = PlaybookPitfall::new(
        "Don't skip the type checking step",
        "Missing type annotations can cause runtime errors",
    )
    .with_mitigation("Always run `cargo clippy` before committing");

    assert_json_snapshot!(pitfall);
}

/// Test PlaybookPitfall without mitigation
#[test]
fn test_playbook_pitfall_no_mitigation() {
    use do_memory_core::memory::playbook::PlaybookPitfall;

    let pitfall = PlaybookPitfall::new(
        "Avoid blocking operations in async context",
        "Will block the entire tokio runtime",
    );

    assert_json_snapshot!(pitfall);
}
