//! Audit logging operations - Episode operations
//!
//! This module provides logging methods for episode-related operations.

use super::core::AuditLogger;
use super::types::AuditLogLevel;
use serde_json::json;

impl AuditLogger {
    /// Log episode creation
    pub async fn log_episode_creation(
        &self,
        client_id: &str,
        episode_id: &str,
        task_description: &str,
        success: bool,
        error: Option<&str>,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "task_description": task_description,
            "error": error
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "create_episode",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log episode modification
    pub async fn log_episode_modification(
        &self,
        client_id: &str,
        episode_id: &str,
        operation: &str,
        success: bool,
        error: Option<&str>,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "modification_type": operation,
            "error": error
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "modify_episode",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log episode deletion
    pub async fn log_episode_deletion(
        &self,
        client_id: &str,
        episode_id: &str,
        success: bool,
        error: Option<&str>,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "error": error
        });

        self.log_event(
            AuditLogLevel::Warn,
            client_id,
            "delete_episode",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log episode step addition
    pub async fn log_episode_step(
        &self,
        client_id: &str,
        episode_id: &str,
        step_number: u32,
        tool: &str,
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "step_number": step_number,
            "tool": tool
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "add_episode_step",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log episode completion
    pub async fn log_episode_completion(
        &self,
        client_id: &str,
        episode_id: &str,
        outcome: &str,
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "outcome": outcome
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "complete_episode",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }
}
