//! Tests for audit logging module

#[cfg(test)]
mod tests {
    use memory_mcp::server::audit::{
        AuditConfig, AuditDestination, AuditLogEntry, AuditLogLevel, AuditLogger,
    };
    use std::collections::HashSet;
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

        // Give a moment for the file to be written
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check that file was created and contains the log
        assert!(log_path.exists());
        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("test-client"));
        assert!(content.contains("create_episode"));
        assert!(content.contains("test-episode"));
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
