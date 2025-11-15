//! Core monitoring system implementation

use super::types::{
    ComponentHealth, HealthCheck, HealthStatus, MonitoringConfig, MonitoringStats,
    PerformanceMetrics, RequestMetrics, SystemPerformance, ToolPerformance,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::debug;

/// Core monitoring system
pub struct MonitoringSystem {
    /// Configuration
    config: MonitoringConfig,
    /// Current statistics
    stats: Arc<RwLock<MonitoringStats>>,
    /// Performance metrics
    performance: Arc<RwLock<PerformanceMetrics>>,
    /// Start time for uptime calculation
    start_time: Instant,
    /// Active request tracking
    active_requests: Arc<Mutex<HashMap<String, RequestMetrics>>>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        let start_time = Instant::now();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let stats = MonitoringStats {
            uptime_seconds: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            episode_metrics: Default::default(),
            last_health_check: now,
            health_status: HealthStatus::Healthy,
        };

        let performance = PerformanceMetrics {
            tool_metrics: HashMap::new(),
            system_performance: SystemPerformance::default(),
        };

        Self {
            config,
            stats: Arc::new(RwLock::new(stats)),
            performance: Arc::new(RwLock::new(performance)),
            start_time,
            active_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start request tracking
    pub async fn start_request(&self, request_id: String, tool_name: String) -> String {
        if !self.config.enabled {
            return request_id.clone();
        }

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let metrics = RequestMetrics {
            request_id: request_id.clone(),
            tool_name,
            start_time,
            end_time: 0,
            success: false,
            response_time_ms: 0,
            error_message: None,
        };

        let mut active = self.active_requests.lock().await;
        active.insert(request_id.clone(), metrics);

        debug!("Started tracking request: {}", request_id);
        request_id
    }

    /// End request tracking
    pub async fn end_request(
        &self,
        request_id: &str,
        success: bool,
        error_message: Option<String>,
    ) {
        if !self.config.enabled {
            return;
        }

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut active = self.active_requests.lock().await;
        if let Some(mut metrics) = active.remove(request_id) {
            metrics.end_time = end_time;
            metrics.success = success;
            metrics.response_time_ms = (end_time - metrics.start_time) * 1000; // Convert to ms
            metrics.error_message = error_message;

            // Update stats
            {
                let mut stats = self.stats.write();
                stats.record_request(success, metrics.response_time_ms);
            }

            // Update performance metrics
            {
                let mut perf = self.performance.write();
                let tool_perf = perf
                    .tool_metrics
                    .entry(metrics.tool_name.clone())
                    .or_default();

                tool_perf.total_calls += 1;
                if success {
                    tool_perf.successful_calls += 1;
                } else {
                    tool_perf.failed_calls += 1;
                }

                // Update response time stats
                let total_calls = tool_perf.total_calls as f64;
                tool_perf.avg_response_time_ms = (tool_perf.avg_response_time_ms
                    * (total_calls - 1.0)
                    + metrics.response_time_ms as f64)
                    / total_calls;

                tool_perf.min_response_time_ms =
                    tool_perf.min_response_time_ms.min(metrics.response_time_ms);
                tool_perf.max_response_time_ms =
                    tool_perf.max_response_time_ms.max(metrics.response_time_ms);

                tool_perf.success_rate =
                    (tool_perf.successful_calls as f64 / tool_perf.total_calls as f64) * 100.0;
            }

            debug!(
                "Ended tracking request: {} (success: {}, time: {}ms)",
                request_id, success, metrics.response_time_ms
            );
        }
    }

    /// Record episode creation
    pub fn record_episode_creation(&self, success: bool) {
        if !self.config.enabled || !self.config.enable_episode_tracking {
            return;
        }

        let mut stats = self.stats.write();
        stats.record_episode_creation(success);

        debug!("Recorded episode creation (success: {})", success);
    }

    /// Update system metrics
    pub fn update_system_metrics(&self, memory_mb: f64, cpu_percent: f64) {
        if !self.config.enabled {
            return;
        }

        let mut stats = self.stats.write();
        stats.update_system_metrics(memory_mb, cpu_percent);

        let mut perf = self.performance.write();
        perf.system_performance.memory_usage_mb = memory_mb;
        perf.system_performance.cpu_usage_percent = cpu_percent;
        perf.system_performance.uptime_seconds = self.start_time.elapsed().as_secs();

        debug!(
            "Updated system metrics: memory={}MB, cpu={}%%",
            memory_mb, cpu_percent
        );
    }

    /// Update uptime
    pub fn update_uptime(&self) {
        if !self.config.enabled {
            return;
        }

        let uptime = self.start_time.elapsed().as_secs();
        let mut stats = self.stats.write();
        stats.update_uptime(uptime);
    }

    /// Get current monitoring statistics
    pub fn get_stats(&self) -> MonitoringStats {
        self.stats.read().clone()
    }

    /// Get performance metrics
    pub fn get_performance(&self) -> PerformanceMetrics {
        self.performance.read().clone()
    }

    /// Perform health check
    pub async fn health_check(&self) -> HealthCheck {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check memory usage
        let memory_status = self.check_memory_health();
        components.insert(
            "memory".to_string(),
            ComponentHealth {
                name: "memory".to_string(),
                status: memory_status.clone(),
                details: Some(format!(
                    "Memory usage: {:.1}MB",
                    self.stats.read().memory_usage_mb
                )),
                last_check: now,
            },
        );
        if matches!(memory_status, HealthStatus::Unhealthy { .. }) {
            overall_status = memory_status;
        }

        // Check CPU usage
        let cpu_status = self.check_cpu_health();
        components.insert(
            "cpu".to_string(),
            ComponentHealth {
                name: "cpu".to_string(),
                status: cpu_status.clone(),
                details: Some(format!(
                    "CPU usage: {:.1}%",
                    self.stats.read().cpu_usage_percent
                )),
                last_check: now,
            },
        );
        if matches!(cpu_status, HealthStatus::Unhealthy { .. })
            && matches!(overall_status, HealthStatus::Healthy)
        {
            overall_status = cpu_status;
        }

        // Check request success rate
        let request_status = self.check_request_health();
        components.insert(
            "requests".to_string(),
            ComponentHealth {
                name: "requests".to_string(),
                status: request_status.clone(),
                details: Some(format!(
                    "Success rate: {:.1}%",
                    self.calculate_success_rate()
                )),
                last_check: now,
            },
        );
        if matches!(request_status, HealthStatus::Unhealthy { .. })
            && matches!(overall_status, HealthStatus::Healthy)
        {
            overall_status = request_status;
        }

        // Update last health check time
        {
            let mut stats = self.stats.write();
            stats.last_health_check = now;
            stats.health_status = overall_status.clone();
        }

        HealthCheck {
            status: overall_status,
            components,
            timestamp: now,
        }
    }

    /// Check memory health
    fn check_memory_health(&self) -> HealthStatus {
        let memory_mb = self.stats.read().memory_usage_mb;

        if memory_mb > 500.0 {
            // Over 500MB
            HealthStatus::Unhealthy {
                message: format!("High memory usage: {:.1}MB", memory_mb),
            }
        } else if memory_mb > 200.0 {
            // Over 200MB
            HealthStatus::Warning {
                message: format!("Elevated memory usage: {:.1}MB", memory_mb),
            }
        } else {
            HealthStatus::Healthy
        }
    }

    /// Check CPU health
    fn check_cpu_health(&self) -> HealthStatus {
        let cpu_percent = self.stats.read().cpu_usage_percent;

        if cpu_percent > 90.0 {
            // Over 90%
            HealthStatus::Unhealthy {
                message: format!("High CPU usage: {:.1}%", cpu_percent),
            }
        } else if cpu_percent > 70.0 {
            // Over 70%
            HealthStatus::Warning {
                message: format!("Elevated CPU usage: {:.1}%", cpu_percent),
            }
        } else {
            HealthStatus::Healthy
        }
    }

    /// Check request health
    fn check_request_health(&self) -> HealthStatus {
        let success_rate = self.calculate_success_rate();

        if success_rate < 80.0 {
            // Below 80%
            HealthStatus::Unhealthy {
                message: format!("Low success rate: {:.1}%", success_rate),
            }
        } else if success_rate < 95.0 {
            // Below 95%
            HealthStatus::Warning {
                message: format!("Moderate success rate: {:.1}%", success_rate),
            }
        } else {
            HealthStatus::Healthy
        }
    }

    /// Calculate overall success rate
    fn calculate_success_rate(&self) -> f64 {
        let stats = self.stats.read();
        if stats.total_requests == 0 {
            100.0
        } else {
            (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
        }
    }

    /// Get active request count
    pub async fn active_request_count(&self) -> usize {
        let active = self.active_requests.lock().await;
        active.len()
    }

    /// Get configuration
    pub fn config(&self) -> &MonitoringConfig {
        &self.config
    }
}
