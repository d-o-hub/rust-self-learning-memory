//! Audit logging operations - Security and configuration operations
//!
//! This module provides logging methods for security and configuration operations.

use super::core::AuditLogger;
use super::types::AuditLogLevel;
use serde_json::json;

impl AuditLogger {
    // === Configuration Operations ===

    /// Log configuration change
    pub async fn log_config_change(
        &self,
        client_id: &str,
        config_key: &str,
        old_value: &serde_json::Value,
        new_value: &serde_json::Value,
        success: bool,
    ) {
        let metadata = json!({
            "config_key": config_key,
            "old_value": old_value,
            "new_value": new_value
        });

        self.log_event(
            AuditLogLevel::Warn,
            client_id,
            "config_change",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log embedding configuration change
    pub async fn log_embedding_config(
        &self,
        client_id: &str,
        provider: &str,
        model: Option<&str>,
        success: bool,
    ) {
        let metadata = json!({
            "provider": provider,
            "model": model
        });

        self.log_event(
            AuditLogLevel::Warn,
            client_id,
            "embedding_config_change",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    // === Authentication & Security ===

    /// Log authentication event
    pub async fn log_authentication(
        &self,
        client_id: &str,
        auth_type: &str,
        success: bool,
        error: Option<&str>,
    ) {
        let metadata = json!({
            "auth_type": auth_type,
            "error": error
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "authentication",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log rate limit violation
    pub async fn log_rate_limit_violation(
        &self,
        client_id: &str,
        operation: &str,
        limit: u32,
        current_count: u32,
    ) {
        let metadata = json!({
            "operation": operation,
            "limit": limit,
            "current_count": current_count
        });

        self.log_event(
            AuditLogLevel::Warn,
            client_id,
            "rate_limit_violation",
            "blocked",
            metadata,
        )
        .await;
    }

    /// Log security violation
    pub async fn log_security_violation(
        &self,
        client_id: &str,
        violation_type: &str,
        details: &str,
    ) {
        let metadata = json!({
            "violation_type": violation_type,
            "details": details
        });

        self.log_event(
            AuditLogLevel::Error,
            client_id,
            "security_violation",
            "detected",
            metadata,
        )
        .await;
    }

    /// Log code execution event
    pub async fn log_code_execution(
        &self,
        client_id: &str,
        sandbox_type: &str,
        execution_time_ms: u64,
        success: bool,
        error: Option<&str>,
    ) {
        let metadata = json!({
            "sandbox_type": sandbox_type,
            "execution_time_ms": execution_time_ms,
            "error": error
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "code_execution",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    // === Relationship Operations ===

    /// Log relationship change
    pub async fn log_relationship_change(
        &self,
        client_id: &str,
        source_id: &str,
        target_id: &str,
        relationship_type: &str,
        operation: &str,
        success: bool,
    ) {
        let metadata = json!({
            "source_id": source_id,
            "target_id": target_id,
            "relationship_type": relationship_type,
            "operation": operation
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "relationship_change",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }
}
