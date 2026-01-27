//! Configuration and statistics for keep-alive connection pool

use std::time::Instant;

/// Configuration for keep-alive behavior
#[derive(Debug, Clone)]
pub struct KeepAliveConfig {
    /// Interval between keep-alive operations (default: 30 seconds)
    pub keep_alive_interval: std::time::Duration,
    /// Time after which a connection is considered stale (default: 60 seconds)
    pub stale_threshold: std::time::Duration,
    /// Enable proactive ping to keep connections alive
    pub enable_proactive_ping: bool,
    /// Timeout for ping operations
    pub ping_timeout: std::time::Duration,
}

impl Default for KeepAliveConfig {
    fn default() -> Self {
        Self {
            keep_alive_interval: std::time::Duration::from_secs(30),
            stale_threshold: std::time::Duration::from_secs(60),
            enable_proactive_ping: true,
            ping_timeout: std::time::Duration::from_secs(5),
        }
    }
}

/// Statistics for keep-alive pool monitoring
#[derive(Debug, Clone)]
pub struct KeepAliveStatistics {
    /// Total connections created
    pub total_connections_created: usize,
    /// Total connections refreshed (due to staleness)
    pub total_connections_refreshed: usize,
    /// Total stale connections detected
    pub total_stale_detected: usize,
    /// Total proactive pings sent
    pub total_proactive_pings: usize,
    /// Total ping failures
    pub total_ping_failures: usize,
    /// Current number of active connections
    pub active_connections: usize,
    /// Average time saved by avoiding stale reconnects (ms)
    pub avg_time_saved_ms: u64,
    /// Last activity timestamp
    pub last_activity: Instant,
}

impl Default for KeepAliveStatistics {
    fn default() -> Self {
        Self {
            total_connections_created: 0,
            total_connections_refreshed: 0,
            total_stale_detected: 0,
            total_proactive_pings: 0,
            total_ping_failures: 0,
            active_connections: 0,
            avg_time_saved_ms: 0,
            last_activity: Instant::now(),
        }
    }
}

impl KeepAliveStatistics {
    /// Update the last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
}
