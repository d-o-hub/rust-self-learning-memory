//! Monitoring capabilities for MCP server
//!
//! This module provides basic monitoring functionality for the MCP server,
//! including episode creation rate tracking, performance monitoring, and health checks.

pub mod core;
pub mod endpoints;
pub mod types;

pub use core::MonitoringSystem;
pub use endpoints::MonitoringEndpoints;
pub use types::{EpisodeMetrics, HealthStatus, MonitoringConfig, MonitoringStats};
