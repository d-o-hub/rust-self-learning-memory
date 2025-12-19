//! Monitoring types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring collection
    pub enabled: bool,
    /// Maximum number of metrics to keep in memory
    pub max_metrics_history: usize,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Enable detailed performance tracking
    pub enable_performance_tracking: bool,
    /// Enable episode creation rate tracking
    pub enable_episode_tracking: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_metrics_history: 1000,
            health_check_interval_secs: 30,
            enable_performance_tracking: true,
            enable_episode_tracking: true,
        }
    }
}

/// Overall monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Episode metrics
    pub episode_metrics: EpisodeMetrics,
    /// Last health check timestamp
    pub last_health_check: u64,
    /// Health status
    pub health_status: HealthStatus,
}

/// Episode creation and tracking metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetrics {
    /// Total episodes created
    pub total_episodes_created: u64,
    /// Episodes created in the last hour
    pub episodes_last_hour: u64,
    /// Episodes created in the last 24 hours
    pub episodes_last_24h: u64,
    /// Average episodes per hour
    pub avg_episodes_per_hour: f64,
    /// Success rate for episode operations
    pub episode_success_rate: f64,
    /// Episode creation timestamps (for rate calculation)
    pub episode_timestamps: Vec<u64>,
}

impl Default for EpisodeMetrics {
    fn default() -> Self {
        Self {
            total_episodes_created: 0,
            episodes_last_hour: 0,
            episodes_last_24h: 0,
            avg_episodes_per_hour: 0.0,
            episode_success_rate: 100.0,
            episode_timestamps: Vec::new(),
        }
    }
}

/// Health status of the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has warnings but is operational
    Warning { message: String },
    /// System is unhealthy
    Unhealthy { message: String },
}

/// Individual request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Request ID
    pub request_id: String,
    /// Tool name
    pub tool_name: String,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp
    pub end_time: u64,
    /// Success flag
    pub success: bool,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// Performance metrics for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Tool name -> metrics
    pub tool_metrics: HashMap<String, ToolPerformance>,
    /// Overall system performance
    pub system_performance: SystemPerformance,
}

/// Performance metrics for a specific tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPerformance {
    /// Total calls
    pub total_calls: u64,
    /// Successful calls
    pub successful_calls: u64,
    /// Failed calls
    pub failed_calls: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Min response time
    pub min_response_time_ms: u64,
    /// Max response time
    pub max_response_time_ms: u64,
    /// Success rate percentage
    pub success_rate: f64,
}

impl Default for ToolPerformance {
    fn default() -> Self {
        Self {
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            avg_response_time_ms: 0.0,
            min_response_time_ms: u64::MAX,
            max_response_time_ms: 0,
            success_rate: 100.0,
        }
    }
}

/// System-wide performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    /// Total memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Active connections/requests
    pub active_requests: u64,
    /// Total uptime in seconds
    pub uptime_seconds: u64,
}

impl Default for SystemPerformance {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            active_requests: 0,
            uptime_seconds: 0,
        }
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Overall status
    pub status: HealthStatus,
    /// Component health checks
    pub components: HashMap<String, ComponentHealth>,
    /// Timestamp of check
    pub timestamp: u64,
}

/// Health status of individual components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Additional details
    pub details: Option<String>,
    /// Last check timestamp
    pub last_check: u64,
}

impl Default for MonitoringStats {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitoringStats {
    /// Create new monitoring stats
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            uptime_seconds: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            episode_metrics: EpisodeMetrics::default(),
            last_health_check: now,
            health_status: HealthStatus::Healthy,
        }
    }

    /// Update uptime
    pub fn update_uptime(&mut self, uptime_seconds: u64) {
        self.uptime_seconds = uptime_seconds;
    }

    /// Record a request
    pub fn record_request(&mut self, success: bool, response_time_ms: u64) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // Update average response time
        let total = self.total_requests as f64;
        self.avg_response_time_ms =
            (self.avg_response_time_ms * (total - 1.0) + response_time_ms as f64) / total;
    }

    /// Update system metrics
    pub fn update_system_metrics(&mut self, memory_mb: f64, cpu_percent: f64) {
        self.memory_usage_mb = memory_mb;
        self.cpu_usage_percent = cpu_percent;
    }

    /// Record episode creation
    pub fn record_episode_creation(&mut self, success: bool) {
        self.episode_metrics.total_episodes_created += 1;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Keep only recent timestamps (last 24 hours)
        self.episode_metrics.episode_timestamps.push(now);
        // Ensure we don't underflow if timestamps are in the future; keep only timestamps <= now and within 24 hours
        self.episode_metrics
            .episode_timestamps
            .retain(|&ts| ts <= now && now - ts < 86400); // 24 hours

        // Update counts with saturating subtraction for safety
        let one_hour_ago = now.saturating_sub(3600);
        let twenty_four_hours_ago = now.saturating_sub(86400);

        self.episode_metrics.episodes_last_hour = self
            .episode_metrics
            .episode_timestamps
            .iter()
            .filter(|&&ts| ts >= one_hour_ago)
            .count() as u64;

        self.episode_metrics.episodes_last_24h = self
            .episode_metrics
            .episode_timestamps
            .iter()
            .filter(|&&ts| ts >= twenty_four_hours_ago)
            .count() as u64;

        // Calculate average episodes per hour
        if self.episode_metrics.episodes_last_24h > 0 {
            self.episode_metrics.avg_episodes_per_hour =
                self.episode_metrics.episodes_last_24h as f64 / 24.0;
        }

        // Update success rate (simplified - assuming all recent episodes succeeded)
        if success {
            self.episode_metrics.episode_success_rate = 100.0;
        } else {
            self.episode_metrics.episode_success_rate = 99.0; // Placeholder for error tracking
        }
    }
}
