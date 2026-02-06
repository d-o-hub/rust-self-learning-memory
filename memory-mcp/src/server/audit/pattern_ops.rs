//! Audit logging operations - Pattern operations
//!
//! This module provides logging methods for pattern-related operations.

use super::core::AuditLogger;
use super::types::AuditLogLevel;
use serde_json::json;

impl AuditLogger {
    /// Log pattern analysis request
    pub async fn log_pattern_analysis(
        &self,
        client_id: &str,
        task_type: &str,
        result_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "task_type": task_type,
            "result_count": result_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "analyze_patterns",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log advanced pattern analysis
    pub async fn log_advanced_pattern_analysis(
        &self,
        client_id: &str,
        analysis_type: &str,
        success: bool,
    ) {
        let metadata = json!({
            "analysis_type": analysis_type
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "advanced_pattern_analysis",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log pattern search
    pub async fn log_pattern_search(
        &self,
        client_id: &str,
        domain: &str,
        result_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "domain": domain,
            "result_count": result_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "search_patterns",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }

    /// Log pattern recommendation request
    pub async fn log_recommend_patterns(
        &self,
        client_id: &str,
        domain: &str,
        recommendation_count: usize,
        success: bool,
    ) {
        let metadata = json!({
            "domain": domain,
            "recommendation_count": recommendation_count
        });

        self.log_event(
            AuditLogLevel::Debug,
            client_id,
            "recommend_patterns",
            if success { "success" } else { "failure" },
            metadata,
        )
        .await;
    }
}
