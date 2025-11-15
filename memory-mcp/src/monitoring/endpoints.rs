//! Monitoring endpoints for MCP server

use super::core::MonitoringSystem;
use super::types::{HealthCheck, MonitoringStats, PerformanceMetrics};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tracing::debug;

/// Monitoring endpoints handler
pub struct MonitoringEndpoints {
    /// Monitoring system
    monitoring: Arc<MonitoringSystem>,
}

impl MonitoringEndpoints {
    /// Create new monitoring endpoints
    pub fn new(monitoring: Arc<MonitoringSystem>) -> Self {
        Self { monitoring }
    }

    /// Handle health check endpoint
    pub async fn health_check(&self) -> Result<serde_json::Value> {
        debug!("Handling health check request");

        let health = self.monitoring.health_check().await;

        Ok(json!({
            "status": match &health.status {
                super::types::HealthStatus::Healthy => "healthy",
                super::types::HealthStatus::Warning { .. } => "warning",
                super::types::HealthStatus::Unhealthy { .. } => "unhealthy",
            },
            "timestamp": health.timestamp,
            "components": health.components.into_iter()
                .map(|(name, component)| {
                    (name, json!({
                        "status": match &component.status {
                            super::types::HealthStatus::Healthy => "healthy",
                            super::types::HealthStatus::Warning { .. } => "warning",
                            super::types::HealthStatus::Unhealthy { .. } => "unhealthy",
                        },
                        "details": component.details,
                        "last_check": component.last_check
                    }))
                })
                .collect::<serde_json::Map<String, serde_json::Value>>()
        }))
    }

    /// Handle metrics endpoint
    pub async fn metrics(&self) -> Result<serde_json::Value> {
        debug!("Handling metrics request");

        let stats = self.monitoring.get_stats();
        let performance = self.monitoring.get_performance();
        let active_requests = self.monitoring.active_request_count().await;

        Ok(json!({
            "monitoring_stats": stats,
            "performance_metrics": performance,
            "active_requests": active_requests,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }))
    }

    /// Handle stats endpoint (alias for metrics)
    pub async fn stats(&self) -> Result<serde_json::Value> {
        self.metrics().await
    }

    /// Handle episode metrics endpoint
    pub async fn episode_metrics(&self) -> Result<serde_json::Value> {
        debug!("Handling episode metrics request");

        let stats = self.monitoring.get_stats();

        Ok(json!({
            "episode_metrics": stats.episode_metrics,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }))
    }

    /// Handle performance metrics endpoint
    pub async fn performance_metrics(&self) -> Result<serde_json::Value> {
        debug!("Handling performance metrics request");

        let performance = self.monitoring.get_performance();

        Ok(json!({
            "performance_metrics": performance,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }))
    }

    /// Handle system info endpoint
    pub async fn system_info(&self) -> Result<serde_json::Value> {
        debug!("Handling system info request");

        let stats = self.monitoring.get_stats();

        Ok(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": stats.uptime_seconds,
            "memory_usage_mb": stats.memory_usage_mb,
            "cpu_usage_percent": stats.cpu_usage_percent,
            "total_requests": stats.total_requests,
            "avg_response_time_ms": stats.avg_response_time_ms,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }))
    }

    /// Check if monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        self.monitoring.config().enabled
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::MonitoringConfig;
    use super::*;

    #[tokio::test]
    async fn test_monitoring_endpoints() {
        let config = MonitoringConfig::default();
        let monitoring = Arc::new(MonitoringSystem::new(config));
        let endpoints = MonitoringEndpoints::new(monitoring);

        // Test health check
        let health = endpoints.health_check().await.unwrap();
        assert!(health.get("status").is_some());
        assert!(health.get("components").is_some());

        // Test metrics
        let metrics = endpoints.metrics().await.unwrap();
        assert!(metrics.get("monitoring_stats").is_some());
        assert!(metrics.get("performance_metrics").is_some());

        // Test episode metrics
        let episode_metrics = endpoints.episode_metrics().await.unwrap();
        assert!(episode_metrics.get("episode_metrics").is_some());

        // Test system info
        let system_info = endpoints.system_info().await.unwrap();
        assert!(system_info.get("version").is_some());
        assert!(system_info.get("uptime_seconds").is_some());
    }
}
