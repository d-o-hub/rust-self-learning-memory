use super::*;

#[test]
fn test_config_defaults() {
    let config = AgentFsConfig::default();
    assert!(!config.enabled, "Default config should be disabled");
    assert_eq!(config.external_weight, 0.3);
    assert!(config.db_path.is_empty(), "Default db_path should be empty");
    assert!(config.sanitize_parameters);
}

#[test]
fn test_provider_creation() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: false,
        external_weight: 0.5,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    assert_eq!(provider.name(), "agentfs");
    assert_eq!(provider.config().external_weight, 0.5);
}

#[test]
fn test_config_validation_valid() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: true,
        external_weight: 0.5,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    assert!(provider.validate_config().is_ok());
}

#[test]
fn test_config_validation_invalid_weight() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: true,
        external_weight: 1.5, // Invalid: > 1.0
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    let result = provider.validate_config();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ExternalSignalError::InvalidConfig(_)
    ));
}

#[test]
fn test_config_validation_missing_path_when_enabled() {
    let config = AgentFsConfig {
        db_path: String::new(), // Empty
        enabled: true,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    let result = provider.validate_config();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ExternalSignalError::ConfigMissing(_)
    ));
}

#[test]
fn test_config_validation_disabled_no_path_required() {
    // Disabled provider doesn require db_path
    let config = AgentFsConfig {
        db_path: String::new(), // Empty OK when disabled
        enabled: false,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    assert!(provider.validate_config().is_ok());
}

#[tokio::test]
async fn test_disabled_provider_returns_empty_signals() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: false, // Disabled
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    let episode = crate::episode::Episode::new(
        "test-episode".to_string(),
        crate::types::TaskContext::default(),
        crate::types::TaskType::Testing,
    );

    let signals = provider.get_signals(&episode).await.unwrap();
    assert!(
        signals.tool_signals.is_empty(),
        "Disabled provider should return empty signals"
    );
    assert_eq!(signals.confidence, 0.0);
    assert_eq!(signals.provider, "agentfs");
}

#[tokio::test]
async fn test_enabled_provider_with_invalid_path_returns_error() {
    let config = AgentFsConfig {
        db_path: "/nonexistent/path/to/db".to_string(),
        enabled: true,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    let health = provider.health_check().await;

    // Should be unhealthy because path doesn't exist/can't connect
    assert!(matches!(health, ProviderHealth::Unhealthy(_)));
}

#[tokio::test]
async fn test_health_check_disabled_returns_healthy() {
    let config = AgentFsConfig {
        db_path: String::new(),
        enabled: false,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);
    let health = provider.health_check().await;
    assert!(health.is_healthy(), "Disabled provider should be healthy");
}

#[test]
fn test_sanitize_parameters_object() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: true,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);

    let params = serde_json::json!({
        "query": "sensitive data",
        "api_key": "secret123",
        "limit": 10
    });

    let sanitized = provider.sanitize_parameters(&params);

    // Check structure preserved
    if let serde_json::Value::Object(map) = sanitized {
        assert!(map.contains_key("query"));
        assert!(map.contains_key("api_key"));
        assert!(map.contains_key("limit"));
        // All values should be redacted
        for (_, value) in map {
            assert_eq!(
                value,
                serde_json::Value::String("[REDACTED]".to_string()),
                "All values should be redacted"
            );
        }
    } else {
        panic!("Expected sanitized result to be an object");
    }
}

#[test]
fn test_sanitize_parameters_disabled() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: true,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: false, // Disabled sanitization
    };

    let provider = AgentFsProvider::new(config);

    let params = serde_json::json!({
        "query": "sensitive data"
    });

    let sanitized = provider.sanitize_parameters(&params);

    // Should return original when sanitization disabled
    assert_eq!(sanitized, params);
}

#[test]
fn test_sanitize_parameters_non_object() {
    let config = AgentFsConfig {
        db_path: "/tmp/test.db".to_string(),
        enabled: true,
        external_weight: 0.3,
        min_correlation_samples: 10,
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);

    // Test non-object value
    let params = serde_json::json!("string value");
    let sanitized = provider.sanitize_parameters(&params);
    assert_eq!(
        sanitized,
        serde_json::Value::String("[REDACTED]".to_string())
    );
}

#[tokio::test]
async fn test_fetch_tool_stats_with_real_sdk() {
    use tempfile::NamedTempFile;
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap().to_string();

    // Initialize SDK and record some stats
    // Signature: record(name, started_at, completed_at, parameters, result, error)
    // Status is "success" when error is None, "error" when error is Some
    let tc = ToolCalls::new(&db_path).await.unwrap();
    tc.record("test_tool", 0, 100, None, None, None)
        .await
        .unwrap();
    tc.record("test_tool", 0, 200, None, None, Some("test error"))
        .await
        .unwrap();
    tc.record("other_tool", 0, 50, None, None, None)
        .await
        .unwrap();

    let config = AgentFsConfig {
        db_path: db_path.clone(),
        enabled: true,
        external_weight: 0.3,
        min_correlation_samples: 1, // Low to match our test data
        sanitize_parameters: true,
    };

    let provider = AgentFsProvider::new(config);

    // Create an episode that uses "test_tool"
    let mut episode = crate::episode::Episode::new(
        "test-episode".to_string(),
        crate::types::TaskContext::default(),
        crate::types::TaskType::Testing,
    );
    let mut step =
        crate::episode::ExecutionStep::new(1, "test_tool".to_string(), "test action".to_string());
    step.set_parameters(serde_json::json!({}));
    step.result = Some(crate::types::ExecutionResult::Success {
        output: "ok".to_string(),
    });
    step.latency_ms = 100;
    episode.add_step(step);

    let stats = provider.fetch_tool_stats(&episode).await;

    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].tool_name, "test_tool");
    assert_eq!(stats[0].sample_count, 2);
    assert_eq!(stats[0].success_rate, 0.5);
}
