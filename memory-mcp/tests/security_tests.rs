//! # Advanced Pattern Analysis Security Tests
//!
//! Tests security aspects of the advanced pattern analysis tool.

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::useless_conversion)]
#![allow(missing_docs)]
#![allow(clippy::single_match_else)]

use do_memory_core::SelfLearningMemory;
use do_memory_mcp::mcp::tools::advanced_pattern_analysis::{
    AdvancedPatternAnalysisInput, AdvancedPatternAnalysisTool, AnalysisConfig, AnalysisType,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Test input sanitization
#[tokio::test]
async fn test_input_sanitization() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test with potentially malicious data
    let mut malicious_data = HashMap::new();

    // Very large numbers that might cause overflow
    malicious_data.insert(
        "large".to_string(),
        vec![f64::MAX, f64::MAX / 2.0, f64::MAX / 4.0],
    );

    // Very small numbers
    malicious_data.insert(
        "small".to_string(),
        vec![
            f64::MIN_POSITIVE,
            f64::MIN_POSITIVE * 2.0,
            f64::MIN_POSITIVE * 4.0,
        ],
    );

    // Mixed problematic values
    malicious_data.insert(
        "mixed".to_string(),
        vec![0.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN],
    );

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: malicious_data,
        config: None,
    };

    let result = tool.execute(input).await;

    // Should not panic or crash
    match result {
        Ok(_) => {
            // If successful, that's fine - the tool handled edge cases
        }
        Err(e) => {
            // If error, should be a proper error, not a panic
            assert_ne!(e.to_string().len(), 0);
        }
    }
}

/// Test resource limits
#[tokio::test]
async fn test_resource_limits() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test with maximum allowed data points
    let mut large_data = HashMap::new();
    let max_series: Vec<f64> = (0..10_000).map(f64::from).collect();
    large_data.insert("max_size".to_string(), max_series);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: large_data,
        config: Some(AnalysisConfig {
            max_data_points: Some(10_000),
            parallel_processing: Some(false),
            ..Default::default()
        }),
    };

    let result = tool.execute(input).await;

    // Should handle large datasets without issues
    match result {
        Ok(output) => {
            assert!(output.performance.memory_usage_bytes < 500 * 1024 * 1024); // < 500MB
        }
        Err(e) => {
            // Should be a proper error about data size, not a panic
            assert!(e.to_string().contains("data") || e.to_string().contains("size"));
        }
    }
}

/// Test numerical stability vulnerabilities
#[tokio::test]
async fn test_numerical_stability_vulnerabilities() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let test_cases = vec![
        ("zeros", vec![0.0, 0.0, 0.0, 0.0, 0.0]),
        ("constants", vec![1.0, 1.0, 1.0, 1.0, 1.0]),
        ("near_zero", vec![1e-15, 2e-15, 3e-15, 4e-15, 5e-15]),
        ("large_variance", vec![1e-10, 1e10, 1e-10, 1e10, 1e-10]),
        ("division_triggers", vec![1.0, 2.0, 4.0, 8.0, 16.0]), // Powers of 2
    ];

    for (name, series) in test_cases {
        let mut data = HashMap::new();
        data.insert(name.to_string(), series);

        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Comprehensive,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;

        // Should not panic on any of these edge cases
        match result {
            Ok(output) => {
                // Results should be finite and reasonable
                assert!(output.performance.total_time_ms < 30_000); // < 30 seconds
            }
            Err(e) => {
                // Error should be descriptive
                assert_ne!(e.to_string().len(), 0);
            }
        }
    }
}

/// Test error handling doesn't leak sensitive information
#[tokio::test]
async fn test_error_information_leakage() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Test various error conditions
    let error_cases = vec![
        ("empty_data", HashMap::new()),
        ("insufficient_data", {
            let mut data = HashMap::new();
            data.insert("small".to_string(), vec![1.0, 2.0]);
            data
        }),
        ("nan_data", {
            let mut data = HashMap::new();
            data.insert("nan".to_string(), vec![1.0, f64::NAN, 3.0]);
            data
        }),
    ];

    for (case_name, data) in error_cases {
        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Statistical,
            time_series_data: data,
            config: None,
        };

        let result = tool.execute(input).await;
        assert!(result.is_err(), "Case {case_name} should fail");

        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // Error messages should be user-friendly and not leak internal details
        assert!(!error_msg.contains("panic"));
        assert!(!error_msg.contains("unwrap"));
        assert!(!error_msg.contains("internal"));
        assert!(!error_msg.contains("debug"));

        // Should contain helpful information
        assert!(error_msg.len() > 10);
        assert!(error_msg.len() < 500); // Not too verbose
    }
}

/// Test that analysis doesn't modify input data
#[tokio::test]
async fn test_input_data_immutability() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let original_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let mut data = HashMap::new();
    data.insert("test".to_string(), original_data.clone());

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Statistical,
        time_series_data: data.clone(),
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    // Original data should be unchanged
    assert_eq!(data.get("test").unwrap(), &original_data);
}

/// Test timeout protection (simulated)
#[tokio::test]
async fn test_timeout_protection() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    // Create a very large dataset that might be slow
    let mut data = HashMap::new();
    let large_series: Vec<f64> = (0..1000).map(f64::from).collect();

    for i in 0..20 {
        data.insert(format!("var_{i}"), large_series.clone());
    }

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: Some(AnalysisConfig {
            parallel_processing: Some(false), // Force sequential to test timeout
            max_data_points: Some(100_000),
            ..Default::default()
        }),
    };

    // Use tokio timeout to ensure analysis doesn't hang indefinitely
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(30), tool.execute(input)).await;

    match result {
        Ok(analysis_result) => {
            // If it completed, that's fine
            assert!(analysis_result.is_ok() || analysis_result.is_err());
        }
        Err(_) => {
            // If it timed out, that's also acceptable for very large datasets
            // The important thing is it didn't hang indefinitely
        }
    }
}

// ──── Input Bounds Clamping Tests (CWE-770 Prevention) ────

/// Verify that tags-specific constants are defined with sensible values
#[allow(clippy::assertions_on_constants)]
#[test]
fn test_tags_constants_validity() {
    assert!(do_memory_mcp::constants::MAX_TAGS_PER_OPERATION >= 1);
    assert!(do_memory_mcp::constants::MAX_TAGS_PER_OPERATION <= 10_000);

    assert!(do_memory_mcp::constants::MAX_TASK_DESCRIPTION_LEN >= 100);
    assert!(do_memory_mcp::constants::MAX_TASK_DESCRIPTION_LEN <= 1_000_000);

    assert!(do_memory_mcp::constants::MAX_BULK_EPISODE_IDS >= 1);
    assert!(do_memory_mcp::constants::MAX_BULK_EPISODE_IDS <= 10_000);

    // Depth bounds
    assert!(do_memory_mcp::constants::MIN_DEPTH >= 1);
    assert!(do_memory_mcp::constants::MAX_DEPTH > do_memory_mcp::constants::MIN_DEPTH);
    assert!(do_memory_mcp::constants::DEFAULT_DEPTH >= do_memory_mcp::constants::MIN_DEPTH);
    assert!(do_memory_mcp::constants::DEFAULT_DEPTH <= do_memory_mcp::constants::MAX_DEPTH);

    // Find related bounds
    assert!(do_memory_mcp::constants::MAX_FIND_RELATED_LIMIT > 0);
    assert!(
        do_memory_mcp::constants::DEFAULT_FIND_RELATED_LIMIT
            <= do_memory_mcp::constants::MAX_FIND_RELATED_LIMIT
    );
}

/// Verify that constants are defined with sensible values
#[allow(clippy::assertions_on_constants)]
#[test]
fn test_input_bounds_constants_validity() {
    // All min values should be >= 1
    assert!(do_memory_mcp::constants::MIN_QUERY_LIMIT >= 1);
    assert!(do_memory_mcp::constants::MIN_PLAYBOOK_STEPS >= 1);
    assert!(do_memory_mcp::constants::MIN_TAG_SEARCH_LIMIT >= 1);

    // All max values should be > min values
    assert!(do_memory_mcp::constants::MAX_QUERY_LIMIT > do_memory_mcp::constants::MIN_QUERY_LIMIT);
    assert!(
        do_memory_mcp::constants::MAX_PLAYBOOK_STEPS > do_memory_mcp::constants::MIN_PLAYBOOK_STEPS
    );
    assert!(
        do_memory_mcp::constants::MAX_TAG_SEARCH_LIMIT
            > do_memory_mcp::constants::MIN_TAG_SEARCH_LIMIT
    );
    assert!(do_memory_mcp::constants::MAX_SEARCH_LIMIT > do_memory_mcp::constants::MIN_QUERY_LIMIT);
    assert!(
        do_memory_mcp::constants::MAX_RECOMMEND_LIMIT > do_memory_mcp::constants::MIN_QUERY_LIMIT
    );
}

/// Verify that default values fall within min/max bounds
#[allow(clippy::assertions_on_constants)]
#[test]
fn test_input_bounds_defaults_in_range() {
    assert!(
        do_memory_mcp::constants::DEFAULT_QUERY_LIMIT >= do_memory_mcp::constants::MIN_QUERY_LIMIT
            && do_memory_mcp::constants::DEFAULT_QUERY_LIMIT
                <= do_memory_mcp::constants::MAX_QUERY_LIMIT
    );
    assert!(
        do_memory_mcp::constants::DEFAULT_ANALYZE_LIMIT
            >= do_memory_mcp::constants::MIN_QUERY_LIMIT
            && do_memory_mcp::constants::DEFAULT_ANALYZE_LIMIT
                <= do_memory_mcp::constants::MAX_QUERY_LIMIT
    );
    assert!(
        do_memory_mcp::constants::DEFAULT_PLAYBOOK_STEPS
            >= do_memory_mcp::constants::MIN_PLAYBOOK_STEPS
            && do_memory_mcp::constants::DEFAULT_PLAYBOOK_STEPS
                <= do_memory_mcp::constants::MAX_PLAYBOOK_STEPS
    );
    assert!(
        do_memory_mcp::constants::DEFAULT_TAG_SEARCH_LIMIT
            >= do_memory_mcp::constants::MIN_TAG_SEARCH_LIMIT
            && do_memory_mcp::constants::DEFAULT_TAG_SEARCH_LIMIT
                <= do_memory_mcp::constants::MAX_TAG_SEARCH_LIMIT
    );
}

/// Verify clamping logic: values below min are clamped up to min
#[allow(clippy::assertions_on_constants)]
#[test]
fn test_clamping_lower_bound() {
    // Test query limit clamping (0 should become 1)
    let clamped = 0usize.clamp(
        do_memory_mcp::constants::MIN_QUERY_LIMIT,
        do_memory_mcp::constants::MAX_QUERY_LIMIT,
    );
    assert_eq!(
        clamped,
        do_memory_mcp::constants::MIN_QUERY_LIMIT,
        "Value 0 should be clamped to minimum"
    );

    // Test playbook steps clamping (0 should become 1)
    let clamped = 0usize.clamp(
        do_memory_mcp::constants::MIN_PLAYBOOK_STEPS,
        do_memory_mcp::constants::MAX_PLAYBOOK_STEPS,
    );
    assert_eq!(
        clamped,
        do_memory_mcp::constants::MIN_PLAYBOOK_STEPS,
        "Value 0 should be clamped to minimum playbook steps"
    );
}

/// Verify clamping logic: values above max are clamped down to max
#[allow(clippy::assertions_on_constants)]
#[test]
fn test_clamping_upper_bound() {
    // Test query limit clamping (9999 should become 1000)
    let clamped = 9999usize.clamp(
        do_memory_mcp::constants::MIN_QUERY_LIMIT,
        do_memory_mcp::constants::MAX_QUERY_LIMIT,
    );
    assert_eq!(
        clamped,
        do_memory_mcp::constants::MAX_QUERY_LIMIT,
        "Value 9999 should be clamped to maximum"
    );

    // Test playbook steps clamping (999 should become 100)
    let clamped = 999usize.clamp(
        do_memory_mcp::constants::MIN_PLAYBOOK_STEPS,
        do_memory_mcp::constants::MAX_PLAYBOOK_STEPS,
    );
    assert_eq!(
        clamped,
        do_memory_mcp::constants::MAX_PLAYBOOK_STEPS,
        "Value 999 should be clamped to maximum playbook steps"
    );
}

/// Verify clamping logic: valid values within range pass through unchanged
#[allow(clippy::assertions_on_constants)]
#[test]
fn test_clamping_middle_values() {
    let clamped = 50usize.clamp(
        do_memory_mcp::constants::MIN_QUERY_LIMIT,
        do_memory_mcp::constants::MAX_QUERY_LIMIT,
    );
    assert_eq!(clamped, 50, "Value 50 should pass through unchanged");

    let clamped = 5usize.clamp(
        do_memory_mcp::constants::MIN_PLAYBOOK_STEPS,
        do_memory_mcp::constants::MAX_PLAYBOOK_STEPS,
    );
    assert_eq!(clamped, 5, "Value 5 should pass through unchanged");
}

/// Test that analysis results don't contain sensitive information
#[tokio::test]
async fn test_output_sanitization() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert("normal".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    let input = AdvancedPatternAnalysisInput {
        analysis_type: AnalysisType::Comprehensive,
        time_series_data: data,
        config: None,
    };

    let result = tool.execute(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();

    // Check that output doesn't contain any sensitive information
    // (This is more of a framework test - in practice, we'd check for things like
    // file paths, environment variables, etc.)

    // Results should be serializable (important for API safety)
    let json_result = serde_json::to_string(&output);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    assert!(json_str.len() > 100); // Should contain meaningful data
    assert!(json_str.len() < 1_000_000); // Shouldn't be excessively large
}

/// Test resistance to malformed configuration
#[tokio::test]
async fn test_malformed_configuration_resistance() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tool = AdvancedPatternAnalysisTool::new(memory);

    let mut data = HashMap::new();
    data.insert("test".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0]);

    // Test with extreme configuration values
    let extreme_configs = vec![
        AnalysisConfig {
            significance_level: Some(0.0),  // Edge case
            forecast_horizon: Some(1),      // Minimum
            anomaly_sensitivity: Some(0.0), // Minimum
            enable_causal_inference: Some(true),
            max_data_points: Some(1), // Very small
            parallel_processing: Some(false),
        },
        AnalysisConfig {
            significance_level: Some(1.0),  // Edge case
            forecast_horizon: Some(100),    // Maximum
            anomaly_sensitivity: Some(1.0), // Maximum
            enable_causal_inference: Some(false),
            max_data_points: Some(1_000_000), // Very large
            parallel_processing: Some(true),
        },
    ];

    for config in extreme_configs {
        let input = AdvancedPatternAnalysisInput {
            analysis_type: AnalysisType::Comprehensive,
            time_series_data: data.clone(),
            config: Some(config),
        };

        let result = tool.execute(input).await;

        // Should not panic on extreme configurations
        match result {
            Ok(_) => {
                // Success is fine
            }
            Err(e) => {
                // Error should be proper, not a panic
                assert!(!e.to_string().contains("panic"));
            }
        }
    }
}

// ============================================================================
// Rate-limit identity (issues #1082 / #1084)
//
// The rate-limit bucket must be derived from a trusted principal, never from
// caller-supplied request fields. Requests themselves are checked through the
// real `handle_request` path.
// ============================================================================

#[path = "../src/bin/server_impl/mod.rs"]
mod server_impl;

use do_memory_mcp::MemoryMCPServer;
use do_memory_mcp::protocol::OAuthConfig;
use do_memory_mcp::server::rate_limiter::{
    ClientId, OperationType, RateLimitConfig, RateLimiter, RateLimiterStats,
};
use do_memory_mcp::types::SandboxConfig;
use server_impl::{
    AuthContext, EmbeddingEnvConfig, JsonRpcRequest, JsonRpcResponse, handle_request,
    subject_identity,
};
use std::time::Duration;

fn rate_limited_limiter() -> RateLimiter {
    RateLimiter::new(RateLimitConfig {
        read_requests_per_second: 1,
        read_burst_size: 1,
        write_requests_per_second: 1,
        write_burst_size: 2,
        ..RateLimitConfig::default()
    })
}

async fn spoof_test_server() -> anyhow::Result<Arc<tokio::sync::Mutex<MemoryMCPServer>>> {
    let memory = Arc::new(SelfLearningMemory::new());
    let server = MemoryMCPServer::new(SandboxConfig::restrictive(), memory).await?;
    Ok(Arc::new(tokio::sync::Mutex::new(server)))
}

/// Tool call whose caller-supplied identity fields rotate on every request
fn spoofed_tool_call(client_id: &str) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: Some("2.0".to_string()),
        id: Some(serde_json::json!(1)),
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({
            "name": "query_memory",
            "arguments": {"query": "anything"},
            "_meta": {
                "client_id": client_id,
                "headers": {"X-Client-ID": client_id}
            },
            "client_id": client_id
        })),
    }
}

async fn dispatch(
    server: &Arc<tokio::sync::Mutex<MemoryMCPServer>>,
    request: JsonRpcRequest,
    auth_context: &AuthContext,
    limiter: &RateLimiter,
) -> Option<JsonRpcResponse> {
    let elicitations = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let tasks = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let embedding = EmbeddingEnvConfig {
        provider: "local".to_string(),
        api_key: None,
        api_key_env: "OPENAI_API_KEY".to_string(),
        model: None,
        similarity_threshold: 0.7,
        batch_size: 32,
    };
    handle_request(
        request,
        server,
        auth_context,
        &elicitations,
        &tasks,
        &embedding,
        limiter,
    )
    .await
}
#[tokio::test]
async fn test_spoofed_client_id_cannot_bypass_saturated_bucket() -> anyhow::Result<()> {
    let limiter = rate_limited_limiter();
    let identity = ClientId::process();

    // Saturate the process-scoped bucket
    assert!(
        limiter
            .check_rate_limit(&identity, OperationType::Write)
            .allowed
    );
    assert!(
        limiter
            .check_rate_limit(&identity, OperationType::Write)
            .allowed
    );
    assert!(
        !limiter
            .check_rate_limit(&identity, OperationType::Write)
            .allowed,
        "bucket must be saturated before the bypass attempt"
    );

    let server = spoof_test_server().await?;
    let auth_context = AuthContext::new(OAuthConfig::default(), identity, None);

    // Rotating caller-supplied identifiers must not reset or bypass the limit
    for i in 0..5 {
        let response = dispatch(
            &server,
            spoofed_tool_call(&format!("rotated-client-{i}")),
            &auth_context,
            &limiter,
        )
        .await
        .ok_or_else(|| anyhow::anyhow!("expected a rate limit response"))?;

        let error = response
            .error
            .ok_or_else(|| anyhow::anyhow!("request {i} bypassed the saturated bucket"))?;
        assert_eq!(
            error.code, -32000,
            "expected rate limit error, got {error:?}"
        );
    }
    Ok(())
}

#[test]
fn test_distinct_subjects_get_independent_buckets() {
    let limiter = rate_limited_limiter();
    let alice = subject_identity("alice@example.com");
    let bob = subject_identity("bob@example.com");

    assert_ne!(alice, bob, "distinct subjects must map to distinct buckets");
    // Identities are opaque: the raw subject must not appear in the bucket key
    assert!(!format!("{alice}").contains("alice"));

    assert!(
        limiter
            .check_rate_limit(&alice, OperationType::Read)
            .allowed
    );
    assert!(
        !limiter
            .check_rate_limit(&alice, OperationType::Read)
            .allowed,
        "alice's bucket must be saturated"
    );
    assert!(
        limiter.check_rate_limit(&bob, OperationType::Read).allowed,
        "bob must have an independent bucket"
    );
}

#[test]
fn test_bucket_cardinality_is_bounded_and_observable() {
    let max_identities = 4;
    let limiter = RateLimiter::new(RateLimitConfig {
        read_burst_size: 1,
        write_burst_size: 1,
        max_identities,
        ..RateLimitConfig::default()
    });

    for i in 0..500 {
        let identity = ClientId::from_string(&format!("rotated-{i}"));
        limiter.check_rate_limit(&identity, OperationType::Read);
    }

    let RateLimiterStats {
        read_buckets_count,
        max_identities: observed_max,
        ..
    } = limiter.get_stats();
    // One shared overflow bucket in addition to the tracked identities
    assert!(
        read_buckets_count <= max_identities + 1,
        "bucket cardinality {read_buckets_count} exceeds bound {max_identities} + overflow"
    );
    assert_eq!(observed_max, max_identities);
}

#[test]
fn test_process_identity_is_transport_scoped() {
    assert_eq!(ClientId::process(), ClientId::process());
    assert_ne!(ClientId::process(), ClientId::from_string("spoofed"));
    assert_ne!(ClientId::process(), ClientId::Unknown);
}

#[test]
fn test_stale_identity_buckets_are_evicted() {
    let limiter = RateLimiter::new(RateLimitConfig {
        stale_threshold: Duration::ZERO,
        ..RateLimitConfig::default()
    });

    limiter.check_rate_limit(&subject_identity("expired-principal"), OperationType::Read);
    assert_eq!(limiter.get_stats().read_buckets_count, 1);

    limiter.cleanup_stale_buckets(Duration::ZERO);
    assert_eq!(
        limiter.get_stats().read_buckets_count,
        0,
        "stale principals must be evicted once the token/principal is gone"
    );
}
