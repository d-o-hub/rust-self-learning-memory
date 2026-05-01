//! Property-based tests for JSON roundtrips in memory-mcp
//!
//! These tests verify that MCP types survive JSON roundtrip serialization
//! without data loss, which is critical for the JSON-RPC protocol.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
#![allow(missing_docs)]

use do_memory_mcp::jsonrpc::{JsonRpcError, JsonRpcResponse};
use do_memory_mcp::types::{
    ErrorType, ExecutionContext, ExecutionResult, ExecutionStats, SecurityViolationType, Tool,
};
use proptest::prelude::*;
use serde_json::json;

// ============================================================================
// Tool Definition Roundtrips
// ============================================================================

proptest! {
    /// Tool JSON roundtrip preserves all fields
    #[test]
    fn tool_json_roundtrip(
        name in "[a-z_]{3,30}",
        description in "[a-zA-Z0-9 ]{5,100}",
    ) {
        let tool = Tool::new(
            name.clone(),
            description.clone(),
            json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        );

        let json = serde_json::to_string(&tool).expect("serialize to JSON");
        let deserialized: Tool = serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(tool.name, deserialized.name);
        prop_assert_eq!(tool.description, deserialized.description);
    }

    /// Tool with complex input schema roundtrip
    #[test]
    fn tool_complex_schema_roundtrip(
        name in "[a-z_]{3,20}",
    ) {
        let tool = Tool::new(
            name,
            "Complex tool".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "array_field": {
                        "type": "array",
                        "items": {"type": "integer"}
                    },
                    "nested": {
                        "type": "object",
                        "properties": {
                            "inner": {"type": "string"}
                        }
                    }
                },
                "required": ["array_field"]
            }),
        );

        let json = serde_json::to_string(&tool).expect("serialize to JSON");
        let deserialized: Tool = serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(tool.name, deserialized.name);
        // Verify schema structure is preserved
        prop_assert!(deserialized.input_schema.is_object());
    }
}

// ============================================================================
// Execution Context Roundtrips
// ============================================================================

proptest! {
    /// ExecutionContext JSON roundtrip preserves all fields
    #[test]
    fn execution_context_json_roundtrip(
        task in "[a-zA-Z0-9 ]{5,100}",
        key1 in "[a-z]{2,10}",
        value1 in "[a-z]{2,10}",
    ) {
        let mut ctx = ExecutionContext::new(
            task.clone(),
            json!({"key": "value"}),
        );
        ctx.env.insert(key1.clone(), value1.clone());

        let json = serde_json::to_string(&ctx).expect("serialize to JSON");
        let deserialized: ExecutionContext =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(ctx.task, deserialized.task);
        prop_assert_eq!(deserialized.env.get(&key1), Some(&value1));
    }

    /// ExecutionContext with nested JSON input roundtrip
    #[test]
    fn execution_context_nested_input_roundtrip(
        task in "[a-zA-Z0-9 ]{5,50}",
    ) {
        let ctx = ExecutionContext::new(
            task,
            json!({
                "nested": {
                    "deeply": {
                        "value": 42,
                        "items": [1, 2, 3]
                    }
                },
                "array": ["a", "b", "c"]
            }),
        );

        let json = serde_json::to_string(&ctx).expect("serialize to JSON");
        let deserialized: ExecutionContext =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(ctx.input, deserialized.input);
    }
}

// ============================================================================
// Execution Result Roundtrips
// ============================================================================

proptest! {
    /// ExecutionResult::Success JSON roundtrip
    #[test]
    fn execution_result_success_roundtrip(
        output in "[a-zA-Z0-9 ]{5,200}",
        stdout in "[a-zA-Z0-9 ]{0,100}",
        stderr in "[a-zA-Z0-9 ]{0,100}",
        execution_time_ms in 0u64..60000u64,
    ) {
        let result = ExecutionResult::Success {
            output: output.clone(),
            stdout,
            stderr,
            execution_time_ms,
        };

        let json = serde_json::to_string(&result).expect("serialize to JSON");
        let deserialized: ExecutionResult =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(result, deserialized);
    }

    /// ExecutionResult::Error JSON roundtrip
    #[test]
    fn execution_result_error_roundtrip(
        message in "[a-zA-Z0-9 ]{5,100}",
        error_type_idx in 0u8..5u8,
    ) {
        let error_type = match error_type_idx {
            0 => ErrorType::Syntax,
            1 => ErrorType::Runtime,
            2 => ErrorType::Permission,
            3 => ErrorType::Resource,
            _ => ErrorType::Unknown,
        };

        let result = ExecutionResult::Error {
            message: message.clone(),
            error_type,
            stdout: String::new(),
            stderr: "Error output".to_string(),
        };

        let json = serde_json::to_string(&result).expect("serialize to JSON");
        let deserialized: ExecutionResult =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(result, deserialized);
    }

    /// ExecutionResult::Timeout JSON roundtrip
    #[test]
    fn execution_result_timeout_roundtrip(
        elapsed_ms in 1000u64..60000u64,
        has_partial in proptest::bool::ANY,
        partial in "[a-zA-Z0-9 ]{0,100}",
    ) {
        let result = ExecutionResult::Timeout {
            elapsed_ms,
            partial_output: if has_partial { Some(partial) } else { None },
        };

        let json = serde_json::to_string(&result).expect("serialize to JSON");
        let deserialized: ExecutionResult =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(result, deserialized);
    }

    /// ExecutionResult::SecurityViolation JSON roundtrip
    #[test]
    fn execution_result_security_violation_roundtrip(
        reason in "[a-zA-Z0-9 ]{5,100}",
        violation_type_idx in 0u8..6u8,
    ) {
        let violation_type = match violation_type_idx {
            0 => SecurityViolationType::FileSystemAccess,
            1 => SecurityViolationType::NetworkAccess,
            2 => SecurityViolationType::ProcessExecution,
            3 => SecurityViolationType::MemoryLimit,
            4 => SecurityViolationType::InfiniteLoop,
            _ => SecurityViolationType::MaliciousCode,
        };

        let result = ExecutionResult::SecurityViolation {
            reason: reason.clone(),
            violation_type,
        };

        let json = serde_json::to_string(&result).expect("serialize to JSON");
        let deserialized: ExecutionResult =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(result, deserialized);
    }
}

// ============================================================================
// Execution Stats Invariants
// ============================================================================

proptest! {
    /// ExecutionStats success_rate invariant
    #[test]
    fn execution_stats_success_rate_invariant(
        total in 0u64..1000u64,
        successful_ratio in 0.0f64..1.0f64,
    ) {
        let successful = (total as f64 * successful_ratio) as u64;
        let failed = total - successful;

        let stats = ExecutionStats {
            total_executions: total,
            successful_executions: successful,
            failed_executions: failed,
            timeout_count: 0,
            security_violations: 0,
            avg_execution_time_ms: 100.0,
        };

        // Verify success rate is bounded [0, 100]
        let success_rate = stats.success_rate();
        prop_assert!((0.0..=100.0).contains(&success_rate));

        // Verify rate calculation
        if total > 0 {
            #[allow(clippy::cast_precision_loss)]
            let expected_rate = (successful as f64 / total as f64) * 100.0;
            prop_assert!((success_rate - expected_rate).abs() < 0.1);
        } else {
            prop_assert!((success_rate - 0.0_f64).abs() < f64::EPSILON);
        }
    }

    /// ExecutionStats record_execution updates correctly
    #[test]
    fn execution_stats_record_invariant(
        successes in 0u64..50u64,
        failures in 0u64..50u64,
        timeouts in 0u64..20u64,
        violations in 0u64..10u64,
    ) {
        let mut stats = ExecutionStats::default();

        // Record successes
        for _ in 0..successes {
            stats.record_execution(
                &ExecutionResult::Success {
                    output: "ok".to_string(),
                    stdout: String::new(),
                    stderr: String::new(),
                    execution_time_ms: 100,
                },
                100,
            );
        }

        // Record failures
        for _ in 0..failures {
            stats.record_execution(
                &ExecutionResult::Error {
                    message: "err".to_string(),
                    error_type: ErrorType::Runtime,
                    stdout: String::new(),
                    stderr: String::new(),
                },
                50,
            );
        }

        // Record timeouts
        for _ in 0..timeouts {
            stats.record_execution(
                &ExecutionResult::Timeout {
                    elapsed_ms: 5000,
                    partial_output: None,
                },
                5000,
            );
        }

        // Record security violations
        for _ in 0..violations {
            stats.record_execution(
                &ExecutionResult::SecurityViolation {
                    reason: "violation".to_string(),
                    violation_type: SecurityViolationType::FileSystemAccess,
                },
                10,
            );
        }

        // Verify counts
        prop_assert_eq!(stats.total_executions, successes + failures + timeouts + violations);
        prop_assert_eq!(stats.successful_executions, successes);
        prop_assert_eq!(stats.failed_executions, failures + timeouts + violations);
        prop_assert_eq!(stats.timeout_count, timeouts);
        prop_assert_eq!(stats.security_violations, violations);
    }
}

// ============================================================================
// JSON-RPC Roundtrips
// ============================================================================

proptest! {
    /// JsonRpcResponse success roundtrip
    #[test]
    fn jsonrpc_response_success_roundtrip(
        id in 1i64..1000i64,
        result_value in "[a-zA-Z0-9]{5,50}",
    ) {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            result: Some(json!({"status": "success", "data": result_value})),
            error: None,
        };

        let json = serde_json::to_string(&response).expect("serialize to JSON");
        let deserialized: JsonRpcResponse =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        prop_assert_eq!(response.id, deserialized.id);
        prop_assert!(deserialized.result.is_some());
        prop_assert!(deserialized.error.is_none());
    }

    /// JsonRpcResponse error roundtrip
    #[test]
    fn jsonrpc_response_error_roundtrip(
        id in 1i64..1000i64,
        code in -32000i32..-32000i32 + 100,
        message in "[a-zA-Z0-9 ]{5,50}",
    ) {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.clone(),
                data: Some(json!({"details": "Additional info"})),
            }),
        };

        let json = serde_json::to_string(&response).expect("serialize to JSON");
        let deserialized: JsonRpcResponse =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(response.jsonrpc, deserialized.jsonrpc);
        prop_assert!(deserialized.error.is_some());
        let err = deserialized.error.unwrap();
        prop_assert_eq!(err.code, code);
        prop_assert_eq!(err.message, message);
    }

    /// JsonRpcError roundtrip
    #[test]
    fn jsonrpc_error_roundtrip(
        code in -32700i32..-32000i32,
        message in "[a-zA-Z0-9 ]{5,100}",
    ) {
        let error = JsonRpcError {
            code,
            message: message.clone(),
            data: Some(json!({"timestamp": 1_234_567_890})),
        };

        let json = serde_json::to_string(&error).expect("serialize to JSON");
        let deserialized: JsonRpcError =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(error.code, deserialized.code);
        prop_assert_eq!(error.message, deserialized.message);
    }
}

// ============================================================================
// Error Type Invariants
// ============================================================================

proptest! {
    /// ErrorType roundtrip through serialization
    #[test]
    fn error_type_roundtrip(
        error_type_idx in 0u8..5u8,
    ) {
        let error_type = match error_type_idx {
            0 => ErrorType::Syntax,
            1 => ErrorType::Runtime,
            2 => ErrorType::Permission,
            3 => ErrorType::Resource,
            _ => ErrorType::Unknown,
        };

        let json = serde_json::to_string(&error_type).expect("serialize to JSON");
        let deserialized: ErrorType =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(error_type, deserialized);
    }

    /// SecurityViolationType roundtrip through serialization
    #[test]
    fn security_violation_type_roundtrip(
        violation_type_idx in 0u8..6u8,
    ) {
        let violation_type = match violation_type_idx {
            0 => SecurityViolationType::FileSystemAccess,
            1 => SecurityViolationType::NetworkAccess,
            2 => SecurityViolationType::ProcessExecution,
            3 => SecurityViolationType::MemoryLimit,
            4 => SecurityViolationType::InfiniteLoop,
            _ => SecurityViolationType::MaliciousCode,
        };

        let json = serde_json::to_string(&violation_type).expect("serialize to JSON");
        let deserialized: SecurityViolationType =
            serde_json::from_str(&json).expect("deserialize from JSON");

        prop_assert_eq!(violation_type, deserialized);
    }
}

// ============================================================================
// Determinism Tests
// ============================================================================

proptest! {
    /// Tool serialization is deterministic
    #[test]
    fn tool_serialization_determinism(
        name in "[a-z_]{3,20}",
        description in "[a-zA-Z0-9 ]{5,50}",
    ) {
        let tool = Tool::new(
            name,
            description,
            json!({"type": "object"}),
        );

        let json1 = serde_json::to_string(&tool).expect("serialize 1");
        let json2 = serde_json::to_string(&tool).expect("serialize 2");

        prop_assert_eq!(json1, json2);
    }

    /// ExecutionResult serialization is deterministic
    #[test]
    fn execution_result_serialization_determinism(
        output in "[a-zA-Z0-9 ]{5,100}",
        time_ms in 0u64..10000u64,
    ) {
        let result = ExecutionResult::Success {
            output,
            stdout: String::new(),
            stderr: String::new(),
            execution_time_ms: time_ms,
        };

        let json1 = serde_json::to_string(&result).expect("serialize 1");
        let json2 = serde_json::to_string(&result).expect("serialize 2");

        prop_assert_eq!(json1, json2);
    }

    /// JsonRpcResponse serialization is deterministic
    #[test]
    fn jsonrpc_response_determinism(
        id in 1i64..100i64,
    ) {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id)),
            result: Some(json!({"status": "ok"})),
            error: None,
        };

        let json1 = serde_json::to_string(&response).expect("serialize 1");
        let json2 = serde_json::to_string(&response).expect("serialize 2");

        prop_assert_eq!(json1, json2);
    }
}
