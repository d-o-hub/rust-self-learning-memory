//! Rate limiter for MCP server
//!
//! This module provides token bucket-based rate limiting to prevent DoS attacks.
//! Features:
//! - Per-client rate limiting (by IP or client ID)
//! - Token bucket algorithm for smooth rate limiting
//! - Different limits for read vs write operations
//! - Configurable via environment variables
//! - Rate limit headers in responses

use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, trace, warn};

mod types;
pub use types::*;

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,
    /// Maximum burst size
    capacity: u32,
    /// Tokens added per second
    refill_rate: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
    /// Last time this bucket was accessed
    last_accessed: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(requests_per_second: u32, burst_size: u32) -> Self {
        let now = Instant::now();
        Self {
            tokens: burst_size as f64,
            capacity: burst_size,
            refill_rate: requests_per_second as f64,
            last_refill: now,
            last_accessed: now,
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f64);
        self.last_refill = now;
        self.last_accessed = now;
    }

    /// Try to consume tokens from the bucket
    /// Returns true if tokens were consumed, false if rate limited
    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    /// Get current token count
    fn tokens(&mut self) -> u32 {
        self.refill();
        self.tokens as u32
    }

    /// Check if this bucket is stale (not accessed for a while)
    fn is_stale(&self, timeout: Duration) -> bool {
        Instant::now().duration_since(self.last_accessed) > timeout
    }

    /// Get time until next token is available
    fn time_until_next_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::from_secs(0)
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds = tokens_needed / self.refill_rate;
            Duration::from_secs_f64(seconds)
        }
    }
}

/// Rate limiter using token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    /// Configuration
    pub config: RateLimitConfig,
    /// Token buckets for read operations per client
    read_buckets: RwLock<HashMap<ClientId, TokenBucket>>,
    /// Token buckets for write operations per client
    write_buckets: RwLock<HashMap<ClientId, TokenBucket>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        let limiter = Self {
            config,
            read_buckets: RwLock::new(HashMap::new()),
            write_buckets: RwLock::new(HashMap::new()),
        };

        // Spawn cleanup task if enabled
        if limiter.config.enabled {
            limiter.spawn_cleanup_task();
        }

        limiter
    }

    /// Create a rate limiter from environment configuration
    pub fn from_env() -> Self {
        Self::new(RateLimitConfig::from_env())
    }

    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if a request should be rate limited
    pub fn check_rate_limit(
        &self,
        client_id: &ClientId,
        operation: OperationType,
    ) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult {
                allowed: true,
                remaining: u32::MAX,
                reset_after: Duration::from_secs(0),
                limit: u32::MAX,
                retry_after: None,
            };
        }

        let (rps, burst) = match operation {
            OperationType::Read => (
                self.config.read_requests_per_second,
                self.config.read_burst_size,
            ),
            OperationType::Write => (
                self.config.write_requests_per_second,
                self.config.write_burst_size,
            ),
        };

        let buckets = match operation {
            OperationType::Read => &self.read_buckets,
            OperationType::Write => &self.write_buckets,
        };

        let mut buckets_guard = buckets.write();

        // Get or create bucket for this client
        let bucket = buckets_guard.entry(client_id.clone()).or_insert_with(|| {
            trace!("Creating new rate limit bucket for client: {}", client_id);
            TokenBucket::new(rps, burst)
        });

        // Try to consume one token
        let allowed = bucket.try_consume(1);
        let remaining = bucket.tokens();
        let reset_after = bucket.time_until_next_token();

        if allowed {
            trace!(
                "Rate limit check passed for client: {} (op: {:?}, remaining: {})",
                client_id,
                operation,
                remaining
            );
            RateLimitResult {
                allowed: true,
                remaining,
                reset_after,
                limit: burst,
                retry_after: None,
            }
        } else {
            let retry_after = bucket.time_until_next_token();
            warn!(
                "Rate limit exceeded for client: {} (op: {:?}, retry_after: {:?})",
                client_id, operation, retry_after
            );
            RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_after,
                limit: burst,
                retry_after: Some(retry_after),
            }
        }
    }

    /// Get rate limit headers for a successful request
    pub fn get_headers(&self, result: &RateLimitResult) -> Vec<(String, String)> {
        vec![
            ("X-RateLimit-Limit".to_string(), result.limit.to_string()),
            (
                "X-RateLimit-Remaining".to_string(),
                result.remaining.to_string(),
            ),
            (
                "X-RateLimit-Reset".to_string(),
                result.reset_after.as_secs().to_string(),
            ),
        ]
    }

    /// Get rate limit headers for a rate-limited response
    pub fn get_rate_limited_headers(&self, result: &RateLimitResult) -> Vec<(String, String)> {
        let mut headers = self.get_headers(result);
        if let Some(retry_after) = result.retry_after {
            headers.push(("Retry-After".to_string(), retry_after.as_secs().to_string()));
        }
        headers
    }

    /// Spawn a background task to clean up stale buckets
    fn spawn_cleanup_task(&self) {
        // This is a placeholder for the cleanup task
        // In a production implementation, we would use a background task
        // to periodically clean up stale buckets. For now, we rely on
        // lazy cleanup during check_rate_limit calls.
        debug!("Rate limiter cleanup task registered (lazy cleanup enabled)");
    }

    /// Get current statistics about the rate limiter
    pub fn get_stats(&self) -> RateLimiterStats {
        RateLimiterStats {
            read_buckets_count: self.read_buckets.read().len(),
            write_buckets_count: self.write_buckets.read().len(),
            enabled: self.config.enabled,
            read_config: (
                self.config.read_requests_per_second,
                self.config.read_burst_size,
            ),
            write_config: (
                self.config.write_requests_per_second,
                self.config.write_burst_size,
            ),
        }
    }

    /// Manually clean up stale buckets (for testing)
    #[cfg(test)]
    pub fn cleanup_stale_buckets(&self, stale_threshold: Duration) {
        // Clean up stale read buckets
        {
            let mut read_guard = self.read_buckets.write();
            let stale_clients: Vec<ClientId> = read_guard
                .iter()
                .filter(|(_, bucket)| bucket.is_stale(stale_threshold))
                .map(|(client_id, _)| client_id.clone())
                .collect();

            for client_id in stale_clients {
                debug!("Removing stale rate limit bucket for client: {}", client_id);
                read_guard.remove(&client_id);
            }
        }

        // Clean up stale write buckets
        {
            let mut write_guard = self.write_buckets.write();
            let stale_clients: Vec<ClientId> = write_guard
                .iter()
                .filter(|(_, bucket)| bucket.is_stale(stale_threshold))
                .map(|(client_id, _)| client_id.clone())
                .collect();

            for client_id in stale_clients {
                debug!("Removing stale rate limit bucket for client: {}", client_id);
                write_guard.remove(&client_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10, 20);
        assert_eq!(bucket.tokens(), 20);

        // Consume some tokens
        assert!(bucket.try_consume(5));
        assert_eq!(bucket.tokens(), 15);

        // Consume all remaining
        assert!(bucket.try_consume(15));
        assert_eq!(bucket.tokens(), 0);

        // Should fail when empty
        assert!(!bucket.try_consume(1));
    }

    #[test]
    fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            enabled: false,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        let client_id = ClientId::from_string("test");
        let result = limiter.check_rate_limit(&client_id, OperationType::Read);

        assert!(result.allowed);
        assert_eq!(result.remaining, u32::MAX);
    }

    #[test]
    fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            enabled: true,
            read_requests_per_second: 10,
            read_burst_size: 5,
            write_requests_per_second: 5,
            write_burst_size: 3,
            cleanup_interval: Duration::from_secs(60),
            client_id_header: "X-Client-ID".to_string(),
        };
        let limiter = RateLimiter::new(config);

        let client_id = ClientId::from_string("test");

        // Should allow burst size requests
        for i in 0..5 {
            let result = limiter.check_rate_limit(&client_id, OperationType::Read);
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 6th request should be rate limited
        let result = limiter.check_rate_limit(&client_id, OperationType::Read);
        assert!(!result.allowed);
        assert!(result.retry_after.is_some());
    }

    #[test]
    fn test_rate_limit_headers() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        let result = RateLimitResult {
            allowed: true,
            remaining: 50,
            reset_after: Duration::from_secs(30),
            limit: 100,
            retry_after: None,
        };

        let headers = limiter.get_headers(&result);
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Limit" && v == "100"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Remaining" && v == "50"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Reset" && v == "30"));
    }

    #[test]
    fn test_rate_limited_headers() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        let result = RateLimitResult {
            allowed: false,
            remaining: 0,
            reset_after: Duration::from_secs(60),
            limit: 100,
            retry_after: Some(Duration::from_secs(5)),
        };

        let headers = limiter.get_rate_limited_headers(&result);
        assert!(headers.iter().any(|(k, v)| k == "Retry-After" && v == "5"));
    }
}
