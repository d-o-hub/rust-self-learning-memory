// Monitoring tool handlers
//!
//! This module contains health_check and get_metrics tool handlers.

use anyhow::Result;

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

        let result = self.monitoring_endpoints.health_check().await?;

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
