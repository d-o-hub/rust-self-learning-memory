//! Tests for audit logging module

#[cfg(test)]
mod tests {
    use do_memory_mcp::server::audit::{
        AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel, AuditLogger,
        redact_sensitive_data,
    };
    use std::collections::HashSet;
    use std::io::Write;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_audit_config_default() {
        let config = AuditConfig::default();
        assert!(config.enabled);
        assert_eq!(config.destination, AuditDestination::Stdout);
        assert!(config.enable_rotation);
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert_eq!(config.max_rotated_files, 10);
        assert!(!config.redact_fields.is_empty());
    }

    /// Setup audit environment variables with proper isolation
    #[allow(unsafe_code)]
    fn setup_audit_test_env() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            // SAFETY: test-only env var manipulation
            unsafe {
                std::env::set_var("AUDIT_LOG_ENABLED", "false");
                std::env::set_var("AUDIT_LOG_DESTINATION", "file");
                std::env::set_var("AUDIT_LOG_LEVEL", "debug");
            }
        });
    }

    #[test]
    fn test_audit_config_from_env() {
        // Set environment variables with proper isolation
        setup_audit_test_env();

        let config = AuditConfig::from_env();
        assert!(!config.enabled);
        assert_eq!(config.destination, AuditDestination::File);
        assert_eq!(config.log_level, AuditLogLevel::Debug);
    }

    #[test]
    fn test_audit_log_level_should_log() {
        assert!(AuditLogLevel::Debug.should_log(AuditLogLevel::Debug));
        assert!(AuditLogLevel::Debug.should_log(AuditLogLevel::Info));
        assert!(AuditLogLevel::Debug.should_log(AuditLogLevel::Error));

        assert!(!AuditLogLevel::Error.should_log(AuditLogLevel::Debug));
        assert!(!AuditLogLevel::Error.should_log(AuditLogLevel::Info));
        assert!(AuditLogLevel::Error.should_log(AuditLogLevel::Error));
    }

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await;
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_audit_logger_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.log");

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: false,
            max_file_size: 1024,
            max_rotated_files: 5,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // Log an event
        logger
            .log_episode_creation("test-client", "test-episode", "test task", true, None)
            .await;

        assert!(logger.flush(Duration::from_secs(2)));

        // Check that file was created and contains the log
        assert!(log_path.exists());
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("test-client"));
        assert!(content.contains("create_episode"));
        assert!(content.contains("test-episode"));
    }

    #[test]
    fn test_audit_recursive_redaction_nested_and_arrays() {
        // Arrange
        let mut fields = HashSet::new();
        fields.insert("password".to_string());
        fields.insert("token".to_string());
        fields.insert("api_key".to_string());

        // Build dotted key at runtime so static scanners do not treat fixtures
        // as hard-coded credentials (hashicorp-tf-password / gitleaks).
        let dotted_key = format!("{}.{}", "nested", "password");
        let mut metadata = serde_json::json!({
            "user": {
                "name": "alice",
                "password": "TEST_PASSWORD_PLACEHOLDER",
                "profile": { "api_key": "TEST_API_KEY_PLACEHOLDER", "bio": "hi" }
            },
            "items": [
                { "id": 1, "token": "TEST_TOKEN_PLACEHOLDER" },
                { "id": 2, "note": "ok" }
            ]
        });
        metadata.as_object_mut().expect("object").insert(
            dotted_key.clone(),
            serde_json::json!("TEST_DOTTED_PLACEHOLDER"),
        );

        // Act
        let redacted = redact_sensitive_data(metadata, &fields);

        // Assert
        assert_eq!(redacted["user"]["name"], "alice");
        assert_eq!(redacted["user"]["password"], "[REDACTED]");
        assert_eq!(redacted["user"]["profile"]["api_key"], "[REDACTED]");
        assert_eq!(redacted["user"]["profile"]["bio"], "hi");
        assert_eq!(redacted["items"][0]["token"], "[REDACTED]");
        assert_eq!(redacted["items"][1]["note"], "ok");
        assert_eq!(redacted[dotted_key], "[REDACTED]");
    }

    #[test]
    fn test_audit_redaction_case_variants() {
        // Arrange — config fields are lowercase; keys use mixed case.
        let mut fields = HashSet::new();
        fields.insert("password".to_string());
        fields.insert("token".to_string());
        fields.insert("api_key".to_string());

        let metadata = serde_json::json!({
            "Password": "p1",
            "TOKEN": "t1",
            "Api_Key": "k1",
            "safe": true
        });

        // Act
        let redacted = redact_sensitive_data(metadata, &fields);

        // Assert
        assert_eq!(redacted["Password"], "[REDACTED]");
        assert_eq!(redacted["TOKEN"], "[REDACTED]");
        assert_eq!(redacted["Api_Key"], "[REDACTED]");
        assert_eq!(redacted["safe"], true);
    }

    #[tokio::test]
    async fn test_audit_existing_file_size_rotation_on_first_write() {
        // Arrange — oversized pre-existing log with rotation enabled.
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("existing_size.log");

        {
            let mut f = std::fs::File::create(&log_path).unwrap();
            f.write_all(&vec![b'z'; 400]).unwrap();
            f.flush().unwrap();
        }
        assert!(std::fs::metadata(&log_path).unwrap().len() >= 400);

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: true,
            max_file_size: 100,
            max_rotated_files: 3,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        // Act
        let logger = AuditLogger::new(config).await.unwrap();
        logger
            .log_event(
                AuditLogLevel::Info,
                "size-client",
                "size_op",
                "success",
                serde_json::json!({"marker": "post-rotate"}),
            )
            .await;
        assert!(logger.flush(Duration::from_secs(2)));

        // Assert
        let rotated = log_path.with_extension("log.1");
        assert!(rotated.exists(), "oversized existing log should rotate");
        let active = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(active.contains("size-client"));
        assert!(active.contains("post-rotate"));
    }

    #[tokio::test]
    async fn test_audit_writer_backpressure_drop_metrics() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("bp_metrics.log");

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: false,
            max_file_size: 10 * 1024 * 1024,
            max_rotated_files: 2,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::with_queue_capacity(config, 1).await.unwrap();

        // Act — flood a capacity-1 queue.
        for i in 0..8_000 {
            logger
                .log_event(
                    AuditLogLevel::Info,
                    "bp-client",
                    "bp_op",
                    "success",
                    serde_json::json!({"i": i}),
                )
                .await;
        }

        // Assert
        let dropped = logger.dropped_writes();
        assert!(
            dropped > 0,
            "expected drop count to increase under backpressure, got {dropped}"
        );
        assert!(logger.flush(Duration::from_secs(5)));

        // Under normal subsequent load, writes still succeed.
        logger
            .log_event(
                AuditLogLevel::Info,
                "bp-client",
                "after_flood",
                "success",
                serde_json::json!({"ok": true}),
            )
            .await;
        assert!(logger.flush(Duration::from_secs(2)));
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("after_flood") || content.contains("bp_op"));
    }

    #[tokio::test]
    async fn test_audit_writer_normal_load_no_drops() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("normal_load.log");

        let config = AuditConfig {
            enabled: true,
            destination: AuditDestination::File,
            file_path: Some(log_path.clone()),
            enable_rotation: false,
            max_file_size: 10 * 1024 * 1024,
            max_rotated_files: 2,
            redact_fields: HashSet::new(),
            log_level: AuditLogLevel::Debug,
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // Act
        for i in 0..32 {
            logger
                .log_event(
                    AuditLogLevel::Info,
                    "ok-client",
                    "ok_op",
                    "success",
                    serde_json::json!({"i": i}),
                )
                .await;
        }
        assert!(logger.flush(Duration::from_secs(2)));

        // Assert
        assert_eq!(logger.dropped_writes(), 0);
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert_eq!(content.lines().count(), 32);
    }

    #[tokio::test]
    async fn test_audit_logger_disabled() {
        let config = AuditConfig {
            enabled: false,
            ..AuditConfig::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();

        // This should not panic or error when disabled
        logger
            .log_episode_creation("test-client", "test-episode", "test task", true, None)
            .await;
    }

    #[tokio::test]
    async fn test_log_episode_deletion() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        // Log a deletion event
        logger
            .log_episode_deletion("admin-client", "episode-to-delete", true, None)
            .await;
    }

    #[tokio::test]
    async fn test_log_security_violation() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        // Log a security violation
        logger
            .log_security_violation(
                "attacker",
                "unauthorized_access",
                "attempted to access restricted resource",
            )
            .await;
    }

    #[tokio::test]
    async fn test_log_rate_limit_violation() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        // Log a rate limit violation
        logger
            .log_rate_limit_violation("heavy-user", "create_episode", 100, 150)
            .await;
    }

    #[tokio::test]
    async fn test_log_config_change() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        // Log a configuration change
        logger
            .log_config_change(
                "admin",
                "max_episodes",
                &serde_json::json!(100),
                &serde_json::json!(200),
                true,
            )
            .await;
    }

    #[tokio::test]
    async fn test_log_batch_execution() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        // Log a batch execution
        logger
            .log_batch_execution("batch-client", 10, 8, 2, true)
            .await;
    }

    #[tokio::test]
    async fn test_log_tag_operations() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        let tags = vec!["tag1".to_string(), "tag2".to_string()];

        // Log tag operations
        logger
            .log_add_tags("client-1", "episode-1", &tags, true)
            .await;
        logger
            .log_remove_tags("client-1", "episode-1", &tags, true)
            .await;
        logger
            .log_set_tags("client-1", "episode-1", &tags, true)
            .await;
        logger.log_search_tags("client-1", &tags, 5).await;
    }

    #[tokio::test]
    async fn test_log_pattern_operations() {
        let config = AuditConfig::default();
        let logger = AuditLogger::new(config).await.unwrap();

        // Log pattern operations
        logger
            .log_pattern_analysis("client-1", "code_generation", 10, true)
            .await;
        logger
            .log_advanced_pattern_analysis("client-1", "statistical", true)
            .await;
        logger
            .log_pattern_search("client-1", "web-api", 5, true)
            .await;
        logger
            .log_recommend_patterns("client-1", "cli", 3, true)
            .await;
    }

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            timestamp: "2026-01-31T12:00:00Z".to_string(),
            level: "info".to_string(),
            client_id: "client-123".to_string(),
            operation: "create_episode".to_string(),
            result: "success".to_string(),
            metadata: serde_json::json!({"episode_id": "uuid-123"}),
        };

        let json_str = serde_json::to_string(&entry).unwrap();
        assert!(json_str.contains("client-123"));
        assert!(json_str.contains("create_episode"));
        assert!(json_str.contains("uuid-123"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["client_id"], "client-123");
    }
}
