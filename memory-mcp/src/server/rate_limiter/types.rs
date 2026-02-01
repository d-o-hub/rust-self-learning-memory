//! Types for rate limiting

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    pub enabled: bool,
    /// Requests per second for read operations
    pub read_requests_per_second: u32,
    /// Burst size for read operations
    pub read_burst_size: u32,
    /// Requests per second for write operations
    pub write_requests_per_second: u32,
    /// Burst size for write operations
    pub write_burst_size: u32,
    /// Interval for cleaning up stale client entries
    pub cleanup_interval: Duration,
    /// Header name to extract client ID from
    pub client_id_header: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            read_requests_per_second: 100,
            read_burst_size: 150,
            write_requests_per_second: 20,
            write_burst_size: 30,
            cleanup_interval: Duration::from_secs(60),
            client_id_header: "X-Client-ID".to_string(),
        }
    }
}

impl RateLimitConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let enabled = std::env::var("MCP_RATE_LIMIT_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        let read_requests_per_second = std::env::var("MCP_RATE_LIMIT_READ_RPS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(100);

        let read_burst_size = std::env::var("MCP_RATE_LIMIT_READ_BURST")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(150);

        let write_requests_per_second = std::env::var("MCP_RATE_LIMIT_WRITE_RPS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(20);

        let write_burst_size = std::env::var("MCP_RATE_LIMIT_WRITE_BURST")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(30);

        let cleanup_interval_secs = std::env::var("MCP_RATE_LIMIT_CLEANUP_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60);

        let client_id_header = std::env::var("MCP_RATE_LIMIT_CLIENT_ID_HEADER")
            .unwrap_or_else(|_| "X-Client-ID".to_string());

        Self {
            enabled,
            read_requests_per_second,
            read_burst_size,
            write_requests_per_second,
            write_burst_size,
            cleanup_interval: Duration::from_secs(cleanup_interval_secs),
            client_id_header,
        }
    }
}

/// Type of operation for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Read operations (queries, gets)
    Read,
    /// Write operations (creates, updates, deletes)
    Write,
}

impl OperationType {
    /// Determine operation type from MCP method name
    pub fn from_method(method: &str) -> Self {
        // Write operations that modify state
        let write_methods = [
            "tools/call",
            "batch/execute",
            "elicitation/request",
            "elicitation/data",
            "elicitation/cancel",
            "task/create",
            "task/update",
            "task/complete",
            "task/cancel",
        ];

        if write_methods.contains(&method) {
            OperationType::Write
        } else {
            OperationType::Read
        }
    }
}

/// Client identifier for rate limiting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientId {
    /// Client identified by IP address
    Ip(String),
    /// Client identified by custom ID
    Id(String),
    /// Unknown client (fallback)
    Unknown,
}

impl ClientId {
    /// Create a client ID from a string identifier
    pub fn from_string(id: &str) -> Self {
        if id.is_empty() {
            ClientId::Unknown
        } else {
            ClientId::Id(id.to_string())
        }
    }

    /// Create a client ID from IP address
    pub fn from_ip(ip: &str) -> Self {
        if ip.is_empty() {
            ClientId::Unknown
        } else {
            ClientId::Ip(ip.to_string())
        }
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientId::Ip(ip) => write!(f, "ip:{}", ip),
            ClientId::Id(id) => write!(f, "id:{}", id),
            ClientId::Unknown => write!(f, "unknown"),
        }
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining tokens in the bucket
    pub remaining: u32,
    /// Time until the bucket resets (for rate limit headers)
    pub reset_after: Duration,
    /// Maximum allowed requests (for rate limit headers)
    pub limit: u32,
    /// Retry after duration if rate limited
    pub retry_after: Option<Duration>,
}

/// Statistics about the rate limiter
#[derive(Debug, Clone, Serialize)]
pub struct RateLimiterStats {
    /// Number of active read buckets
    pub read_buckets_count: usize,
    /// Number of active write buckets
    pub write_buckets_count: usize,
    /// Whether rate limiting is enabled
    pub enabled: bool,
    /// Read configuration (rps, burst)
    pub read_config: (u32, u32),
    /// Write configuration (rps, burst)
    pub write_config: (u32, u32),
}
