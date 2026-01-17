//!
//! Ground truth data for pattern accuracy tests
//!

use chrono::Duration;
use memory_core::Pattern;
use uuid::Uuid;

use super::helpers::create_test_context;

/// Ground truth: Known successful tool sequences
pub fn create_ground_truth_tool_sequences() -> Vec<Pattern> {
    let context = create_test_context("api-testing", Some("rust"));

    vec![
        // Sequence 1: Read -> Parse -> Validate
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "file_reader".to_string(),
                "json_parser".to_string(),
                "validator".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.95,
            avg_latency: Duration::milliseconds(150),
            occurrence_count: 10,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 2: Connect -> Query -> Process
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "db_connector".to_string(),
                "query_executor".to_string(),
                "result_processor".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.92,
            avg_latency: Duration::milliseconds(200),
            occurrence_count: 8,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 3: Auth -> Verify -> Grant
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "authenticator".to_string(),
                "token_verifier".to_string(),
                "access_granter".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.98,
            avg_latency: Duration::milliseconds(80),
            occurrence_count: 15,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 4: Fetch -> Transform -> Store
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "data_fetcher".to_string(),
                "transformer".to_string(),
                "storage_writer".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.88,
            avg_latency: Duration::milliseconds(250),
            occurrence_count: 12,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 5: Build -> Test -> Deploy
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "builder".to_string(),
                "test_runner".to_string(),
                "deployer".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.85,
            avg_latency: Duration::milliseconds(5000),
            occurrence_count: 20,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 6: Request -> Validate -> Response
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "http_handler".to_string(),
                "input_validator".to_string(),
                "response_builder".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.93,
            avg_latency: Duration::milliseconds(120),
            occurrence_count: 18,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 7: Parse -> Compile -> Execute
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "parser".to_string(),
                "compiler".to_string(),
                "executor".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.90,
            avg_latency: Duration::milliseconds(300),
            occurrence_count: 7,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 8: Monitor -> Analyze -> Alert
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "monitor".to_string(),
                "analyzer".to_string(),
                "alerter".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.87,
            avg_latency: Duration::milliseconds(180),
            occurrence_count: 9,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 9: Serialize -> Compress -> Send
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "serializer".to_string(),
                "compressor".to_string(),
                "sender".to_string(),
            ],
            context: context.clone(),
            success_rate: 0.94,
            avg_latency: Duration::milliseconds(160),
            occurrence_count: 11,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Sequence 10: Load -> Cache -> Serve
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![
                "loader".to_string(),
                "cache_manager".to_string(),
                "server".to_string(),
            ],
            context,
            success_rate: 0.96,
            avg_latency: Duration::milliseconds(90),
            occurrence_count: 16,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
    ]
}

/// Ground truth: Known decision points
pub fn create_ground_truth_decision_points() -> Vec<Pattern> {
    let context = create_test_context("api-testing", Some("rust"));

    vec![
        // Decision 1: Check cache validity
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Check if cache is valid".to_string(),
            action: "cache_validator".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 45,
                failure_count: 5,
                total_count: 50,
                avg_duration_secs: 0.05,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 2: Verify permissions
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Verify user has permissions".to_string(),
            action: "permission_checker".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 38,
                failure_count: 12,
                total_count: 50,
                avg_duration_secs: 0.08,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 3: Check resource availability
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Check if resource is available".to_string(),
            action: "resource_checker".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 42,
                failure_count: 8,
                total_count: 50,
                avg_duration_secs: 0.06,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 4: Validate input format
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Verify input format is correct".to_string(),
            action: "format_validator".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 47,
                failure_count: 3,
                total_count: 50,
                avg_duration_secs: 0.04,
            },
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Decision 5: Check rate limit
        Pattern::DecisionPoint {
            id: Uuid::new_v4(),
            condition: "Check if rate limit exceeded".to_string(),
            action: "rate_limiter".to_string(),
            outcome_stats: memory_core::OutcomeStats {
                success_count: 40,
                failure_count: 10,
                total_count: 50,
                avg_duration_secs: 0.03,
            },
            context,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
    ]
}

/// Ground truth: Known error recovery patterns
pub fn create_ground_truth_error_recoveries() -> Vec<Pattern> {
    let context = create_test_context("api-testing", Some("rust"));

    vec![
        // Recovery 1: Connection timeout
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Connection timeout".to_string(),
            recovery_steps: vec![
                "retry_connector: Retry with exponential backoff".to_string(),
                "fallback_connector: Try alternate endpoint".to_string(),
            ],
            success_rate: 0.85,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 2: Authentication failure
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Authentication failed".to_string(),
            recovery_steps: vec![
                "token_refresher: Refresh authentication token".to_string(),
                "re_authenticator: Re-authenticate with credentials".to_string(),
            ],
            success_rate: 0.92,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 3: Resource not found
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Resource not found".to_string(),
            recovery_steps: vec![
                "cache_invalidator: Clear stale cache".to_string(),
                "resource_loader: Reload resource from source".to_string(),
            ],
            success_rate: 0.78,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 4: Parse error
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Parse error".to_string(),
            recovery_steps: vec![
                "fallback_parser: Try alternative parser".to_string(),
                "error_handler: Return default value".to_string(),
            ],
            success_rate: 0.88,
            context: context.clone(),
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
        // Recovery 5: Rate limit exceeded
        Pattern::ErrorRecovery {
            id: Uuid::new_v4(),
            error_type: "Rate limit exceeded".to_string(),
            recovery_steps: vec![
                "backoff_handler: Wait and retry after delay".to_string(),
                "queue_manager: Queue request for later".to_string(),
            ],
            success_rate: 0.95,
            context,
            effectiveness: memory_core::PatternEffectiveness::new(),
        },
    ]
}
