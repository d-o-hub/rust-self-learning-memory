//! Monitoring capabilities for MCP server
//!
//! This module provides basic monitoring functionality for the MCP server,
//! including episode creation rate tracking, performance monitoring, and health checks.

pub mod core;
pub mod endpoints;
pub mod health_probe;
pub mod types;

pub use core::MonitoringSystem;
pub use endpoints::MonitoringEndpoints;
pub use health_probe::{
    PROBE_TIMEOUT, SYNC_NOT_CONFIGURED, build_health_response, overall_status, probe_backend,
    probe_outcome,
};
pub use types::{BackendStatus, EpisodeMetrics, HealthStatus, MonitoringConfig, MonitoringStats};
