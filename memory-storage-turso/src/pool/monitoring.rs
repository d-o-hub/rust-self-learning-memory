//! # Keep-Alive Pool Monitoring
//!
//! Monitoring and helper functions for keep-alive connection pool.

use std::time::Instant;

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

    /// Increment connections created
    pub fn inc_connections_created(&mut self) {
        self.total_connections_created += 1;
        self.active_connections += 1;
        self.update_activity();
    }

    /// Increment connections refreshed
    pub fn inc_connections_refreshed(&mut self) {
        self.total_connections_refreshed += 1;
    }

    /// Increment stale detections
    pub fn inc_stale_detected(&mut self) {
        self.total_stale_detected += 1;
    }

    /// Decrement active connections
    pub fn dec_active_connections(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    /// Calculate success rate for pings
    pub fn ping_success_rate(&self) -> f64 {
        let total = self.total_proactive_pings + self.total_ping_failures;
        if total == 0 {
            1.0
        } else {
            self.total_proactive_pings as f64 / total as f64
        }
    }
}
