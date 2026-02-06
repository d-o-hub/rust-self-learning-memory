//! Audit logging operations - Relationship operations
//!
//! This module provides logging methods for relationship-related operations.

use super::core::AuditLogger;
use super::types::AuditLogLevel;
use serde_json::json;

impl AuditLogger {
    /// Log relationship addition between episodes
    pub async fn log_add_relationship(
        &self,
        client_id: &str,
        from_episode_id: &str,
        to_episode_id: &str,
        relationship_type: &str,
        relationship_id: &str,
        success: bool,
    ) {
        let metadata = json!({
            "from_episode_id": from_episode_id,
            "to_episode_id": to_episode_id,
            "relationship_type": relationship_type,
            "relationship_id": relationship_id
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "add_episode_relationship",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log relationship removal
    pub async fn log_remove_relationship(
        &self,
        client_id: &str,
        relationship_id: &str,
        success: bool,
    ) {
        let metadata = json!({
            "relationship_id": relationship_id
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "remove_episode_relationship",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log get episode relationships query
    pub async fn log_get_relationships(
        &self,
        client_id: &str,
        episode_id: &str,
        total_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "total_count": total_count
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "get_episode_relationships",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log find related episodes query
    pub async fn log_find_related(
        &self,
        client_id: &str,
        episode_id: &str,
        count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "count": count
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "find_related_episodes",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log check relationship exists query
    pub async fn log_check_relationship(
        &self,
        client_id: &str,
        from_episode_id: &str,
        to_episode_id: &str,
        relationship_type: &str,
        exists: bool,
        success: bool,
    ) {
        let metadata = json!({
            "from_episode_id": from_episode_id,
            "to_episode_id": to_episode_id,
            "relationship_type": relationship_type,
            "exists": exists
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "check_relationship_exists",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log dependency graph generation
    pub async fn log_dependency_graph(
        &self,
        client_id: &str,
        episode_id: &str,
        node_count: usize,
        edge_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "episode_id": episode_id,
            "node_count": node_count,
            "edge_count": edge_count
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "get_dependency_graph",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log cycle validation check
    pub async fn log_validate_cycles(
        &self,
        client_id: &str,
        from_episode_id: &str,
        to_episode_id: &str,
        relationship_type: &str,
        would_create_cycle: bool,
        success: bool,
    ) {
        let metadata = json!({
            "from_episode_id": from_episode_id,
            "to_episode_id": to_episode_id,
            "relationship_type": relationship_type,
            "would_create_cycle": would_create_cycle
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "validate_no_cycles",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log topological order computation
    pub async fn log_topological_order(
        &self,
        client_id: &str,
        input_count: usize,
        output_count: usize,
        has_cycles: bool,
        success: bool,
    ) {
        let metadata = json!({
            "input_count": input_count,
            "output_count": output_count,
            "has_cycles": has_cycles
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "get_topological_order",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }
}
