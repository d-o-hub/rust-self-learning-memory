// ============================================================================
// Audit Types Module
// ============================================================================
//!
//! Type definitions for audit logging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Severity level for audit log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AuditLogLevel {
    /// Detailed debugging information
    Debug,
    /// General informational events
    #[default]
    Info,
    /// Warning conditions
    Warn,
    /// Error conditions
    Error,
    /// Critical security events
    Critical,
}

impl fmt::Display for AuditLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Output destination for audit logs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AuditOutput {
    /// Write to standard output
    #[default]
    Stdout,
    /// Write to standard error
    Stderr,
    /// Write to a file
    File(String),
    /// Write to multiple destinations
    Multiple(Vec<AuditOutput>),
    /// No output (disabled)
    None,
}

/// Types of audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuditEventType {
    // Episode lifecycle events
    EpisodeCreated,
    EpisodeCompleted,
    EpisodeDeleted,
    EpisodeRetrieved,
    EpisodeUpdated,

    // Step events
    StepLogged,
    StepsFlushed,

    // Relationship events
    RelationshipAdded,
    RelationshipRemoved,
    RelationshipRetrieved,

    // Tag events
    TagsAdded,
    TagsRemoved,
    TagsSet,

    // Pattern events
    PatternExtracted,
    PatternRetrieved,

    // Configuration events
    ConfigChanged,

    // Security events
    AccessDenied,
    AuthenticationSuccess,
    AuthenticationFailure,

    // System events
    SystemStartup,
    SystemShutdown,
    StorageOperation,
}

impl fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::EpisodeCreated => "episode_created",
            Self::EpisodeCompleted => "episode_completed",
            Self::EpisodeDeleted => "episode_deleted",
            Self::EpisodeRetrieved => "episode_retrieved",
            Self::EpisodeUpdated => "episode_updated",
            Self::StepLogged => "step_logged",
            Self::StepsFlushed => "steps_flushed",
            Self::RelationshipAdded => "relationship_added",
            Self::RelationshipRemoved => "relationship_removed",
            Self::RelationshipRetrieved => "relationship_retrieved",
            Self::TagsAdded => "tags_added",
            Self::TagsRemoved => "tags_removed",
            Self::TagsSet => "tags_set",
            Self::PatternExtracted => "pattern_extracted",
            Self::PatternRetrieved => "pattern_retrieved",
            Self::ConfigChanged => "config_changed",
            Self::AccessDenied => "access_denied",
            Self::AuthenticationSuccess => "auth_success",
            Self::AuthenticationFailure => "auth_failure",
            Self::SystemStartup => "system_startup",
            Self::SystemShutdown => "system_shutdown",
            Self::StorageOperation => "storage_operation",
        };
        write!(f, "{name}")
    }
}

/// Actor type performing the action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    /// Human user
    User(String),
    /// Automated system component
    System(String),
    /// External service or API client
    Service(String),
    /// Unknown or anonymous actor
    Unknown,
}

impl Default for ActorType {
    fn default() -> Self {
        Self::System("memory-core".to_string())
    }
}

impl fmt::Display for ActorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User(id) => write!(f, "user:{id}"),
            Self::System(name) => write!(f, "system:{name}"),
            Self::Service(name) => write!(f, "service:{name}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Result of an audited operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AuditResult {
    /// Operation completed successfully
    #[default]
    Success,
    /// Operation failed with an error
    Failure { reason: String },
    /// Operation was denied due to access control
    Denied { reason: String },
}

/// Configuration for audit logging.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // Independent config flags are appropriate here
pub struct AuditConfig {
    /// Whether audit logging is enabled
    pub enabled: bool,
    /// Minimum log level to record
    pub log_level: AuditLogLevel,
    /// Where to output audit logs
    pub output_destination: AuditOutput,
    /// Number of days to retain logs (for file-based storage)
    pub retention_days: u32,
    /// Include before/after state in modification events
    pub include_state_changes: bool,
    /// Include IP address information (if available)
    pub include_ip_address: bool,
    /// Include session identifiers (if available)
    pub include_session_info: bool,
    /// Buffer size for async logging
    pub buffer_size: usize,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for development
            log_level: AuditLogLevel::Info,
            output_destination: AuditOutput::Stdout,
            retention_days: 90,
            include_state_changes: true,
            include_ip_address: false,
            include_session_info: false,
            buffer_size: 1000,
        }
    }
}

impl AuditConfig {
    /// Create configuration from environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(enabled) = std::env::var("MEMORY_AUDIT_ENABLED") {
            config.enabled = matches!(enabled.to_lowercase().as_str(), "true" | "1" | "yes" | "on");
        }

        if let Ok(level) = std::env::var("MEMORY_AUDIT_LEVEL") {
            config.log_level = match level.to_lowercase().as_str() {
                "debug" => AuditLogLevel::Debug,
                "info" => AuditLogLevel::Info,
                "warn" | "warning" => AuditLogLevel::Warn,
                "error" => AuditLogLevel::Error,
                "critical" => AuditLogLevel::Critical,
                _ => AuditLogLevel::Info,
            };
        }

        if let Ok(output) = std::env::var("MEMORY_AUDIT_OUTPUT") {
            config.output_destination = match output.to_lowercase().as_str() {
                "stdout" => AuditOutput::Stdout,
                "stderr" => AuditOutput::Stderr,
                "file" => {
                    let path = std::env::var("MEMORY_AUDIT_FILE")
                        .unwrap_or_else(|_| "audit.log".to_string());
                    AuditOutput::File(path)
                }
                "none" | "disabled" => AuditOutput::None,
                _ => AuditOutput::Stdout,
            };
        }

        if let Ok(retention) = std::env::var("MEMORY_AUDIT_RETENTION_DAYS") {
            if let Ok(days) = retention.parse() {
                config.retention_days = days;
            }
        }

        config
    }

    /// Check if a given log level should be logged.
    #[must_use]
    pub fn should_log(&self, level: AuditLogLevel) -> bool {
        if !self.enabled {
            return false;
        }

        let level_value = match level {
            AuditLogLevel::Debug => 0,
            AuditLogLevel::Info => 1,
            AuditLogLevel::Warn => 2,
            AuditLogLevel::Error => 3,
            AuditLogLevel::Critical => 4,
        };

        let config_value = match self.log_level {
            AuditLogLevel::Debug => 0,
            AuditLogLevel::Info => 1,
            AuditLogLevel::Warn => 2,
            AuditLogLevel::Error => 3,
            AuditLogLevel::Critical => 4,
        };

        level_value >= config_value
    }
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry
    pub entry_id: Uuid,
    /// Timestamp when the event occurred (UTC, millisecond precision)
    pub timestamp: DateTime<Utc>,
    /// Type of event
    pub event_type: AuditEventType,
    /// Severity level
    pub level: AuditLogLevel,
    /// Actor who performed the action
    pub actor: ActorType,
    /// Target resource identifier (e.g., episode ID)
    pub resource_id: Option<String>,
    /// Event-specific details
    pub details: HashMap<String, serde_json::Value>,
    /// State before modification (if applicable)
    pub before_state: Option<serde_json::Value>,
    /// State after modification (if applicable)
    pub after_state: Option<serde_json::Value>,
    /// Result of the operation
    pub result: AuditResult,
    /// IP address of the actor (if available)
    pub ip_address: Option<String>,
    /// Session identifier (if available)
    pub session_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl AuditEntry {
    /// Create a new audit entry with the current timestamp.
    #[must_use]
    pub fn new(event_type: AuditEventType, actor: ActorType) -> Self {
        Self {
            entry_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            level: AuditLogLevel::Info,
            actor,
            resource_id: None,
            details: HashMap::new(),
            before_state: None,
            after_state: None,
            result: AuditResult::Success,
            ip_address: None,
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the log level.
    #[must_use]
    pub fn with_level(mut self, level: AuditLogLevel) -> Self {
        self.level = level;
        self
    }

    /// Set the resource ID.
    #[must_use]
    pub fn with_resource_id(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    /// Add a detail field.
    #[must_use = "returns Self for builder pattern"]
    pub fn with_detail(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> anyhow::Result<Self> {
        let value = serde_json::to_value(value)?;
        self.details.insert(key.into(), value);
        Ok(self)
    }

    /// Set the before state.
    #[must_use = "returns Self for builder pattern"]
    pub fn with_before_state(mut self, state: impl Serialize) -> anyhow::Result<Self> {
        self.before_state = Some(serde_json::to_value(state)?);
        Ok(self)
    }

    /// Set the after state.
    #[must_use = "returns Self for builder pattern"]
    pub fn with_after_state(mut self, state: impl Serialize) -> anyhow::Result<Self> {
        self.after_state = Some(serde_json::to_value(state)?);
        Ok(self)
    }

    /// Set the operation result.
    #[must_use]
    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    /// Set the IP address.
    #[must_use]
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set the session ID.
    #[must_use]
    pub fn with_session_id(mut self, session: impl Into<String>) -> Self {
        self.session_id = Some(session.into());
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert to JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
