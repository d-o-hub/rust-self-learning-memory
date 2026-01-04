// Monitoring tool handlers
//!
//! This module contains health_check and get_metrics tool handlers.

use crate::unified_sandbox::SandboxBackend;
use anyhow::Result;
use serde_json::json;

impl crate::server::MemoryMCPServer {
    /// Execute the health_check tool
    ///
    /// # Returns
    ///
    /// Returns health check results
    pub async fn health_check(&self) -> Result<serde_json::Value> {
        self.track_tool_usage("health_check").await;

        // Start monitoring request
        let request_id = format!(
            "health_check_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "health_check".to_string())
            .await;

        let mut result = self.monitoring_endpoints.health_check().await?;

        // Attach unified sandbox health info
        let backend = self.sandbox.backend();
        let unified_metrics = self.sandbox.get_metrics().await;
        let health = self.sandbox.get_health_status().await;

        let sandbox_json = json!({
            "backend": match backend {
                SandboxBackend::NodeJs => "nodejs",
                SandboxBackend::Wasm => "wasm",
                SandboxBackend::Hybrid { .. } => "hybrid",
            },
            "wasmtime_pool": health.wasmtime_pool_stats.map(|m| json!({
                "total_executions": m.total_executions,
                "successful_executions": m.successful_executions,
                "failed_executions": m.failed_executions,
                "timeout_count": m.timeout_count,
                "security_violations": m.security_violations,
                "avg_execution_time_ms": m.avg_execution_time_ms,
                "peak_memory_bytes": m.peak_memory_bytes,
            })),
            "routing": json!({
                "total_executions": unified_metrics.total_executions,
                "node_executions": unified_metrics.node_executions,
                "wasm_executions": unified_metrics.wasm_executions,
                "node_success_rate": unified_metrics.node_success_rate,
                "wasm_success_rate": unified_metrics.wasm_success_rate,
                "node_avg_latency_ms": unified_metrics.node_avg_latency_ms,
                "wasm_avg_latency_ms": unified_metrics.wasm_avg_latency_ms,
            })
        });

        if let Some(obj) = result.as_object_mut() {
            obj.insert("sandbox".to_string(), sandbox_json);
        }

        // End monitoring request
        self.monitoring.end_request(&request_id, true, None).await;

        Ok(result)
    }

    /// Execute the get_metrics tool
    ///
    /// # Arguments
    ///
    /// * `metric_type` - Type of metrics to retrieve
    ///
    /// # Returns
    ///
    /// Returns monitoring metrics
    pub async fn get_metrics(&self, metric_type: Option<String>) -> Result<serde_json::Value> {
        self.track_tool_usage("get_metrics").await;

        // Start monitoring request
        let request_id = format!(
            "get_metrics_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        self.monitoring
            .start_request(request_id.clone(), "get_metrics".to_string())
            .await;

        let result = match metric_type.as_deref() {
            Some("performance") => self.monitoring_endpoints.performance_metrics().await,
            Some("episodes") => self.monitoring_endpoints.episode_metrics().await,
            Some("system") => self.monitoring_endpoints.system_info().await,
            _ => self.monitoring_endpoints.metrics().await,
        };

        // End monitoring request
        self.monitoring
            .end_request(&request_id, result.is_ok(), None)
            .await;

        result
    }
}
