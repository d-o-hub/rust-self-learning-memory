//! Audit logging types and configuration
//!
//! This module defines the core types, configuration, and log levels
//! for the audit logging system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Configuration for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Log destination (file, stdout, or both)
    pub destination: AuditDestination,
    /// Log file path (when destination includes file)
    pub file_path: Option<PathBuf>,
    /// Enable log rotation
    pub enable_rotation: bool,
    /// Maximum log file size in bytes before rotation
    pub max_file_size: u64,
    /// Maximum number of rotated log files to keep
    pub max_rotated_files: usize,
    /// Fields to redact from logs (sensitive data)
    pub redact_fields: HashSet<String>,
    /// Log level for audit events
    pub log_level: AuditLogLevel,
}

/// Audit log destination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditDestination {
    /// Log to stdout only
    Stdout,
    /// Log to file only
    File,
    /// Log to both stdout and file
    Both,
}

/// Audit log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLogLevel {
    /// Log all events
    Debug,
    /// Log informational and above
    Info,
    /// Log warnings and above
    Warn,
    /// Log errors only
    Error,
}

impl AuditLogLevel {
    /// Check if this level should log the given level
    pub fn should_log(&self, event_level: AuditLogLevel) -> bool {
        let self_value = *self as u8;
        let event_value = event_level as u8;
        self_value <= event_value
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        let mut redact_fields = HashSet::new();
        redact_fields.insert("password".to_string());
        redact_fields.insert("token".to_string());
        redact_fields.insert("secret".to_string());
        redact_fields.insert("api_key".to_string());
        redact_fields.insert("private_key".to_string());

        Self {
            enabled: true,
            destination: AuditDestination::Stdout,
            file_path: None,
            enable_rotation: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_rotated_files: 10,
            redact_fields,
            log_level: AuditLogLevel::Info,
        }
    }
}

impl AuditConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Check if audit logging is enabled
        if let Ok(enabled) = std::env::var("AUDIT_LOG_ENABLED") {
            config.enabled = enabled.parse().unwrap_or(true);
        }

        // Parse destination
        if let Ok(dest) = std::env::var("AUDIT_LOG_DESTINATION") {
            config.destination = match dest.to_lowercase().as_str() {
                "stdout" => AuditDestination::Stdout,
                "file" => AuditDestination::File,
                "both" => AuditDestination::Both,
                _ => AuditDestination::Stdout,
            };
        }

        // Parse file path
        if let Ok(path) = std::env::var("AUDIT_LOG_FILE_PATH") {
            config.file_path = Some(PathBuf::from(path));
        }

        // Parse rotation settings
        if let Ok(rotation) = std::env::var("AUDIT_LOG_ENABLE_ROTATION") {
            config.enable_rotation = rotation.parse().unwrap_or(true);
        }

        if let Ok(size) = std::env::var("AUDIT_LOG_MAX_FILE_SIZE") {
            config.max_file_size = size.parse().unwrap_or(100 * 1024 * 1024);
        }

        if let Ok(files) = std::env::var("AUDIT_LOG_MAX_ROTATED_FILES") {
            config.max_rotated_files = files.parse().unwrap_or(10);
        }

        // Parse log level
        if let Ok(level) = std::env::var("AUDIT_LOG_LEVEL") {
            config.log_level = match level.to_lowercase().as_str() {
                "debug" => AuditLogLevel::Debug,
                "info" => AuditLogLevel::Info,
                "warn" => AuditLogLevel::Warn,
                "error" => AuditLogLevel::Error,
                _ => AuditLogLevel::Info,
            };
        }

        // Parse redact fields
        if let Ok(fields) = std::env::var("AUDIT_LOG_REDACT_FIELDS") {
            config.redact_fields = fields
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        config
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Log level
    pub level: String,
    /// Client or user ID
    pub client_id: String,
    /// Operation performed
    pub operation: String,
    /// Operation result
    pub result: String,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_audit_config_from_env() {
        // Set environment variables
        std::env::set_var("AUDIT_LOG_ENABLED", "false");
        std::env::set_var("AUDIT_LOG_DESTINATION", "file");
        std::env::set_var("AUDIT_LOG_LEVEL", "debug");

        let config = AuditConfig::from_env();
        assert!(!config.enabled);
        assert_eq!(config.destination, AuditDestination::File);
        assert_eq!(config.log_level, AuditLogLevel::Debug);

        // Clean up
        std::env::remove_var("AUDIT_LOG_ENABLED");
        std::env::remove_var("AUDIT_LOG_DESTINATION");
        std::env::remove_var("AUDIT_LOG_LEVEL");
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
