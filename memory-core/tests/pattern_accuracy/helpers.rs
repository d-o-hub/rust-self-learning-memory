//!
//! Helper functions for pattern accuracy tests
//!

use chrono::Duration;
use memory_core::{
    ComplexityLevel, Episode, ExecutionResult, ExecutionStep, Pattern, TaskContext, TaskOutcome,
    TaskType,
};
use uuid::Uuid;

/// Create a test context for episodes
pub fn create_test_context(domain: &str, language: Option<&str>) -> TaskContext {
    TaskContext {
        language: language.map(std::string::ToString::to_string),
        framework: Some("tokio".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec!["test".to_string()],
    }
}

/// Create a successful execution step
pub fn create_success_step(step_number: usize, tool: &str, action: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Success {
        output: "OK".to_string(),
    });
    step.latency_ms = 100;
    step
}

/// Create a failed execution step
pub fn create_error_step(
    step_number: usize,
    tool: &str,
    action: &str,
    error_msg: &str,
) -> ExecutionStep {
    let mut step = ExecutionStep::new(step_number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Error {
        message: error_msg.to_string(),
    });
    step.latency_ms = 50;
    step
}

/// Create episodes that contain the ground truth patterns
pub fn create_episodes_with_patterns() -> Vec<Episode> {
    let mut episodes = Vec::new();
    let context = create_test_context("api-testing", Some("rust"));

    // Episode 1: File reading workflow
    let mut ep1 = Episode::new(
        "Read and validate config".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep1.add_step(create_success_step(1, "file_reader", "Read config file"));
    ep1.add_step(create_success_step(2, "json_parser", "Parse JSON content"));
    ep1.add_step(create_success_step(
        3,
        "validator",
        "Validate config schema",
    ));
    ep1.complete(TaskOutcome::Success {
        verdict: "Config validated".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep1);

    // Episode 2: Database query workflow
    let mut ep2 = Episode::new(
        "Fetch user data".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep2.add_step(create_success_step(
        1,
        "db_connector",
        "Connect to database",
    ));
    ep2.add_step(create_success_step(
        2,
        "query_executor",
        "Execute SELECT query",
    ));
    ep2.add_step(create_success_step(
        3,
        "result_processor",
        "Process query results",
    ));
    ep2.complete(TaskOutcome::Success {
        verdict: "Data fetched".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep2);

    // Episode 3: Authentication workflow
    let mut ep3 = Episode::new(
        "Authenticate user".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep3.add_step(create_success_step(
        1,
        "authenticator",
        "Verify credentials",
    ));
    ep3.add_step(create_success_step(2, "token_verifier", "Verify JWT token"));
    ep3.add_step(create_success_step(
        3,
        "access_granter",
        "Grant access permissions",
    ));
    ep3.complete(TaskOutcome::Success {
        verdict: "User authenticated".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep3);

    // Episode 4: Decision point - cache validation
    let mut ep4 = Episode::new(
        "Check cache".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep4.add_step(create_success_step(
        1,
        "cache_validator",
        "Check if cache is valid",
    ));
    ep4.add_step(create_success_step(2, "cache_reader", "Read from cache"));
    ep4.complete(TaskOutcome::Success {
        verdict: "Cache hit".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep4);

    // Episode 5: Error recovery - connection timeout
    let mut ep5 = Episode::new(
        "Handle connection error".to_string(),
        context.clone(),
        TaskType::Debugging,
    );
    ep5.add_step(create_error_step(
        1,
        "connector",
        "Connect to API",
        "Connection timeout",
    ));
    ep5.add_step(create_success_step(
        2,
        "retry_connector",
        "Retry with exponential backoff",
    ));
    ep5.add_step(create_success_step(
        3,
        "fallback_connector",
        "Try alternate endpoint",
    ));
    ep5.complete(TaskOutcome::Success {
        verdict: "Recovered from timeout".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep5);

    // Episode 6: Data transformation workflow
    let mut ep6 = Episode::new(
        "Transform and store data".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep6.add_step(create_success_step(1, "data_fetcher", "Fetch raw data"));
    ep6.add_step(create_success_step(
        2,
        "transformer",
        "Transform data format",
    ));
    ep6.add_step(create_success_step(3, "storage_writer", "Write to storage"));
    ep6.complete(TaskOutcome::Success {
        verdict: "Data stored".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep6);

    // Episode 7: Decision point - permissions check
    let mut ep7 = Episode::new(
        "Check permissions".to_string(),
        context.clone(),
        TaskType::CodeGeneration,
    );
    ep7.add_step(create_success_step(
        1,
        "permission_checker",
        "Verify user has permissions",
    ));
    ep7.add_step(create_success_step(
        2,
        "action_executor",
        "Execute authorized action",
    ));
    ep7.complete(TaskOutcome::Success {
        verdict: "Action authorized".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep7);

    // Episode 8: Error recovery - auth failure
    let mut ep8 = Episode::new(
        "Recover from auth failure".to_string(),
        context.clone(),
        TaskType::Debugging,
    );
    ep8.add_step(create_error_step(
        1,
        "auth_client",
        "Authenticate",
        "Authentication failed",
    ));
    ep8.add_step(create_success_step(
        2,
        "token_refresher",
        "Refresh authentication token",
    ));
    ep8.add_step(create_success_step(
        3,
        "re_authenticator",
        "Re-authenticate with credentials",
    ));
    ep8.complete(TaskOutcome::Success {
        verdict: "Re-authenticated successfully".to_string(),
        artifacts: vec![],
    });
    episodes.push(ep8);

    episodes
}
