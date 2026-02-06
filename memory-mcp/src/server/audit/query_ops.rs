//! Audit logging operations - Query and batch operations
//!
//! This module provides logging methods for query and batch operations.

use super::core::AuditLogger;
use super::types::AuditLogLevel;
use serde_json::json;

impl AuditLogger {
    // === Batch Operations ===

    /// Log batch execution
    pub async fn log_batch_execution(
        &self,
        client_id: &str,
        operation_count: usize,
        success_count: usize,
        failure_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "operation_count": operation_count,
            "success_count": success_count,
            "failure_count": failure_count
        });

        self.log_event(
            AuditLogLevel::Info,
            client_id,
            "batch_execute",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log bulk episode retrieval
    pub async fn log_bulk_episodes(&self, client_id: &str, episode_count: usize, success: bool) {
        let metadata = json!({
            "episode_count": episode_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "bulk_episodes",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    // === Memory Query Operations ===

    /// Log memory query
    pub async fn log_memory_query(
        &self,
        client_id: &str,
        query: &str,
        result_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "query": query,
            "result_count": result_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "query_memory",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log semantic memory query
    pub async fn log_semantic_query(
        &self,
        client_id: &str,
        query: &str,
        result_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "query": query,
            "result_count": result_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "query_semantic_memory",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }
}
