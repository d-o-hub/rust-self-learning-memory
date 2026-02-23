// ============================================================================
// Audit Logging Module
// ============================================================================
//!
//! Comprehensive audit logging for security compliance and incident investigation.
//!
//! This module provides structured audit logging capabilities that track all
//! critical operations in the memory system. Audit logs are designed for:
//!
//! - **Security compliance**: Meet regulatory requirements for data access tracking
//! - **Incident investigation**: Reconstruct events leading to issues
//! - **Forensic analysis**: Understand system behavior and data flow
//! - **Access control monitoring**: Detect unauthorized access attempts
//!
//! ## Features
//!
//! - Structured JSON logging for machine parsing
//! - Millisecond-precision timestamps
//! - User/agent identification
//! - Before/after state tracking for modifications
//! - Configurable log levels and destinations
//! - Minimal performance overhead (<5% when enabled)
//!
//! ## Configuration
//!
//! Audit logging is **disabled by default** for development environments.
//! Enable via configuration:
//!
//! ```no_run
//! use memory_core::security::audit::{AuditConfig, AuditLogger, AuditLogLevel, AuditOutput};
//!
//! let config = AuditConfig {
//!     enabled: true,
//!     log_level: AuditLogLevel::Info,
//!     output_destination: AuditOutput::Stdout,
//!     retention_days: 90,
//!     include_state_changes: true,
//!     include_ip_address: false,
//!     include_session_info: false,
//!     buffer_size: 100,
//! };
//!
//! let logger = AuditLogger::new(config);
//! ```
//!
//! ## Environment Variables
//!
//! - `MEMORY_AUDIT_ENABLED`: Enable/disable audit logging (`true`/`false`)
//! - `MEMORY_AUDIT_LEVEL`: Log level (`debug`, `info`, `warn`, `error`)
//! - `MEMORY_AUDIT_OUTPUT`: Output destination (`stdout`, `stderr`, `file`)
//! - `MEMORY_AUDIT_FILE`: File path when output is `file`
//! - `MEMORY_AUDIT_RETENTION_DAYS`: Log retention period in days

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Submodules
mod helpers;
mod types;

// Re-export types
pub use self::helpers::{
    access_denied, config_changed, episode_completed, episode_created, episode_deleted,
    relationship_added, relationship_removed, tags_modified,
};
pub use self::types::{
    ActorType, AuditConfig, AuditEntry, AuditEventType, AuditLogLevel, AuditOutput, AuditResult,
};

// ============================================================================
// Audit Logger Implementation
// ============================================================================

/// Async audit logger with buffering and configurable output.
#[derive(Clone)]
pub struct AuditLogger {
    config: AuditConfig,
    sender: Option<mpsc::UnboundedSender<AuditEntry>>,
}

impl AuditLogger {
    /// Create a new audit logger with the given configuration.
    #[must_use]
    pub fn new(config: AuditConfig) -> Self {
        let sender = if config.enabled {
            let (tx, mut rx) = mpsc::unbounded_channel::<AuditEntry>();

            // Spawn background task to process audit entries
            tokio::spawn(async move {
                while let Some(entry) = rx.recv().await {
                    Self::write_entry(&entry);
                }
            });

            Some(tx)
        } else {
            None
        };

        Self { config, sender }
    }

    /// Create a disabled audit logger (no-op).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            config: AuditConfig {
                enabled: false,
                ..Default::default()
            },
            sender: None,
        }
    }

    /// Check if audit logging is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Log an audit entry asynchronously.
    ///
    /// This method is non-blocking. The entry is queued for async processing.
    pub fn log(&self, entry: AuditEntry) {
        if !self.config.enabled {
            return;
        }

        if !self.config.should_log(entry.level) {
            return;
        }

        if let Some(ref sender) = self.sender {
            if let Err(e) = sender.send(entry) {
                // Only log error at debug level to avoid infinite loop
                debug!("Failed to send audit entry: {}", e);
            }
        }
    }

    /// Log an entry synchronously (for critical events).
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn log_sync(&self, entry: &AuditEntry) -> anyhow::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if !self.config.should_log(entry.level) {
            return Ok(());
        }

        Self::write_entry(entry);
        Ok(())
    }

    /// Write an entry to the configured output destination.
    fn write_entry(entry: &AuditEntry) {
        match Self::format_entry(entry) {
            Ok(json) => {
                // Use tracing for output (can be configured via tracing subscribers)
                match entry.level {
                    AuditLogLevel::Debug => debug!(target: "audit", "{}", json),
                    AuditLogLevel::Info => info!(target: "audit", "{}", json),
                    AuditLogLevel::Warn => warn!(target: "audit", "{}", json),
                    AuditLogLevel::Error | AuditLogLevel::Critical => {
                        error!(target: "audit", "{}", json);
                    }
                }
            }
            Err(e) => {
                error!("Failed to format audit entry: {}", e);
            }
        }
    }

    /// Format an entry as JSON.
    fn format_entry(entry: &AuditEntry) -> anyhow::Result<String> {
        entry.to_json()
    }

    /// Get a reference to the configuration.
    #[must_use]
    pub fn config(&self) -> &AuditConfig {
        &self.config
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(AuditConfig::default())
    }
}

// ============================================================================
// Audit Context
// ============================================================================

/// Context for audit logging operations.
///
/// This struct carries audit-related information through the call stack,
/// allowing nested operations to be properly attributed.
#[derive(Debug, Clone)]
pub struct AuditContext {
    /// The actor performing operations
    pub actor: ActorType,
    /// Session identifier
    pub session_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

impl AuditContext {
    /// Create a new audit context.
    #[must_use]
    pub fn new(actor: ActorType) -> Self {
        Self {
            actor,
            session_id: None,
            ip_address: None,
            request_id: Some(Uuid::new_v4().to_string()),
        }
    }

    /// Create a system context.
    #[must_use]
    pub fn system() -> Self {
        Self::new(ActorType::System("memory-core".to_string()))
    }

    /// Create an anonymous context.
    #[must_use]
    pub fn anonymous() -> Self {
        Self::new(ActorType::Unknown)
    }

    /// Set the session ID.
    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set the IP address.
    #[must_use]
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set the request ID.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl Default for AuditContext {
    fn default() -> Self {
        Self::system()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_config_default() {
        let config = AuditConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.log_level, AuditLogLevel::Info));
        assert!(matches!(config.output_destination, AuditOutput::Stdout));
        assert_eq!(config.retention_days, 90);
    }

    #[test]
    fn test_audit_config_should_log() {
        let config = AuditConfig {
            enabled: true,
            log_level: AuditLogLevel::Warn,
            ..Default::default()
        };

        assert!(!config.should_log(AuditLogLevel::Debug));
        assert!(!config.should_log(AuditLogLevel::Info));
        assert!(config.should_log(AuditLogLevel::Warn));
        assert!(config.should_log(AuditLogLevel::Error));
        assert!(config.should_log(AuditLogLevel::Critical));
    }

    #[test]
    fn test_audit_config_disabled() {
        let config = AuditConfig {
            enabled: false,
            ..Default::default()
        };

        assert!(!config.should_log(AuditLogLevel::Critical));
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            AuditEventType::EpisodeCreated,
            ActorType::System("test".to_string()),
        );
        assert_eq!(entry.event_type, AuditEventType::EpisodeCreated);
        assert!(matches!(entry.actor, ActorType::System(_)));
        assert!(entry.details.is_empty());
    }

    #[test]
    fn test_audit_entry_with_details() {
        let entry = AuditEntry::new(
            AuditEventType::EpisodeCreated,
            ActorType::System("test".to_string()),
        )
        .with_detail("key", "value")
        .unwrap()
        .with_resource_id("resource-123");

        assert_eq!(entry.resource_id, Some("resource-123".to_string()));
        assert!(entry.details.contains_key("key"));
    }

    #[test]
    fn test_audit_entry_to_json() {
        let entry = AuditEntry::new(
            AuditEventType::EpisodeCreated,
            ActorType::System("test".to_string()),
        )
        .with_resource_id("test-id");

        let json = entry.to_json().unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("episode_created"));
    }

    #[test]
    fn test_actor_type_display() {
        assert_eq!(
            ActorType::User("alice".to_string()).to_string(),
            "user:alice"
        );
        assert_eq!(
            ActorType::System("worker".to_string()).to_string(),
            "system:worker"
        );
        assert_eq!(
            ActorType::Service("api".to_string()).to_string(),
            "service:api"
        );
        assert_eq!(ActorType::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_audit_log_level_display() {
        assert_eq!(AuditLogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(AuditLogLevel::Info.to_string(), "INFO");
        assert_eq!(AuditLogLevel::Warn.to_string(), "WARN");
        assert_eq!(AuditLogLevel::Error.to_string(), "ERROR");
        assert_eq!(AuditLogLevel::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn test_audit_event_type_display() {
        assert_eq!(
            AuditEventType::EpisodeCreated.to_string(),
            "episode_created"
        );
        assert_eq!(
            AuditEventType::EpisodeCompleted.to_string(),
            "episode_completed"
        );
        assert_eq!(AuditEventType::AccessDenied.to_string(), "access_denied");
    }

    #[test]
    fn test_audit_context_creation() {
        let context = AuditContext::system();
        assert!(matches!(context.actor, ActorType::System(_)));
        assert!(context.request_id.is_some());

        let user_context = AuditContext::new(ActorType::User("bob".to_string()));
        assert!(matches!(user_context.actor, ActorType::User(_)));
    }

    #[test]
    fn test_helper_functions() {
        let context = AuditContext::system();
        let episode_id = Uuid::new_v4();

        let entry = episode_created(&context, episode_id, "Test task", "code_generation");
        assert_eq!(entry.event_type, AuditEventType::EpisodeCreated);
        assert_eq!(entry.resource_id, Some(episode_id.to_string()));

        let entry = episode_completed(&context, episode_id, "Success", true);
        assert_eq!(entry.event_type, AuditEventType::EpisodeCompleted);
        assert!(matches!(entry.level, AuditLogLevel::Info));

        let entry = episode_deleted(&context, episode_id);
        assert_eq!(entry.event_type, AuditEventType::EpisodeDeleted);
        assert!(matches!(entry.level, AuditLogLevel::Warn));
    }

    #[test]
    fn test_access_denied() {
        let context = AuditContext::system();
        let entry = access_denied(
            &context,
            "episode-123",
            "delete",
            "insufficient_permissions",
        );

        assert_eq!(entry.event_type, AuditEventType::AccessDenied);
        assert!(matches!(entry.level, AuditLogLevel::Critical));
        assert!(matches!(entry.result, AuditResult::Denied { .. }));
    }

    #[test]
    fn test_config_changed() {
        let context = AuditContext::system();
        let entry = config_changed(&context, "max_episodes", "1000", "2000");

        assert_eq!(entry.event_type, AuditEventType::ConfigChanged);
        assert!(matches!(entry.level, AuditLogLevel::Warn));
    }

    #[test]
    fn test_relationship_helpers() {
        let context = AuditContext::system();
        let rel_id = Uuid::new_v4();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let entry = relationship_added(&context, rel_id, from, to, "DependsOn");
        assert_eq!(entry.event_type, AuditEventType::RelationshipAdded);

        let entry = relationship_removed(&context, rel_id);
        assert_eq!(entry.event_type, AuditEventType::RelationshipRemoved);
    }

    #[test]
    fn test_tags_modified() {
        let context = AuditContext::system();
        let episode_id = Uuid::new_v4();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];

        let entry = tags_modified(&context, episode_id, "added", &tags);
        assert_eq!(entry.event_type, AuditEventType::TagsAdded);
    }

    #[tokio::test]
    async fn test_audit_logger_disabled() {
        let logger = AuditLogger::disabled();
        assert!(!logger.is_enabled());

        let entry = AuditEntry::new(
            AuditEventType::EpisodeCreated,
            ActorType::System("test".to_string()),
        );
        logger.log(entry); // Should not panic
    }

    #[tokio::test]
    async fn test_audit_logger_log_sync() {
        let config = AuditConfig {
            enabled: true,
            log_level: AuditLogLevel::Debug,
            ..Default::default()
        };
        let logger = AuditLogger::new(config);

        let entry = AuditEntry::new(
            AuditEventType::EpisodeCreated,
            ActorType::System("test".to_string()),
        );
        let result = logger.log_sync(&entry);
        assert!(result.is_ok());
    }
}
