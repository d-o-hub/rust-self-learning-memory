//! Audit logging operations - Tag operations
//!
//! This module provides logging methods for tag-related operations.

use super::core::AuditLogger;
use super::types::AuditLogLevel;
use serde_json::json;

impl AuditLogger {
    /// Log tag addition to episode
    pub async fn log_add_tags(
        &self,
        client_id: &str,
        episode_id: &str,
        tags: &[String],
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "tags": tags
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "add_episode_tags",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log tag removal from episode
    pub async fn log_remove_tags(
        &self,
        client_id: &str,
        episode_id: &str,
        tags: &[String],
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "tags": tags
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "remove_episode_tags",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log tag replacement on episode
    pub async fn log_set_tags(
        &self,
        client_id: &str,
        episode_id: &str,
        tags: &[String],
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "tags": tags
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "set_episode_tags",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log tag search
    pub async fn log_search_tags(&self, client_id: &str, tags: &[String], result_count: usize) {
        let metadata = json!({
            "tags": tags,
            "result_count": result_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "search_episodes_by_tags",
            "success",
            metadata,
        )
        .await;
    }
}
