//! Integration tests for rate limiting
//!
//! These tests verify that rate limiting is properly integrated into the MCP server.

use memory_mcp::server::rate_limiter::{
    ClientId, OperationType, RateLimitConfig, RateLimitResult, RateLimiter,
};
use std::time::Duration;

#[test]
fn test_rate_limit_config_from_env_defaults() {
    // Test that default configuration is reasonable
    let config = RateLimitConfig::default();
    assert!(config.enabled);
    assert_eq!(config.read_requests_per_second, 100);
    assert_eq!(config.read_burst_size, 150);
    assert_eq!(config.write_requests_per_second, 20);
    assert_eq!(config.write_burst_size, 30);
    assert_eq!(config.cleanup_interval, Duration::from_secs(60));
    assert_eq!(config.client_id_header, "X-Client-ID");
}

#[test]
fn test_operation_type_classification() {
    // Read operations
    assert_eq!(
        OperationType::from_method("initialize"),
        OperationType::Read
    );
    assert_eq!(
        OperationType::from_method("tools/list"),
        OperationType::Read
    );
    assert_eq!(OperationType::from_method("task/list"), OperationType::Read);
    assert_eq!(
        OperationType::from_method("completion/complete"),
        OperationType::Read
    );
    assert_eq!(
        OperationType::from_method(".well-known/oauth-protected-resource"),
        OperationType::Read
    );

    // Write operations
    assert_eq!(
        OperationType::from_method("tools/call"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("batch/execute"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("elicitation/request"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("elicitation/data"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("elicitation/cancel"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("task/create"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("task/update"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("task/complete"),
        OperationType::Write
    );
    assert_eq!(
        OperationType::from_method("task/cancel"),
        OperationType::Write
    );
}

#[test]
fn test_rate_limiter_per_client_isolation() {
    let config = RateLimitConfig {
        enabled: true,
        read_requests_per_second: 10,
        read_burst_size: 3,
        write_requests_per_second: 5,
        write_burst_size: 2,
        cleanup_interval: Duration::from_secs(60),
        client_id_header: "X-Client-ID".to_string(),
    };
    let limiter = RateLimiter::new(config);

    let client1 = ClientId::from_string("client1");
    let client2 = ClientId::from_string("client2");

    // Exhaust client1's burst
    for _ in 0..3 {
        assert!(
            limiter
                .check_rate_limit(&client1, OperationType::Read)
                .allowed
        );
    }
    assert!(
        !limiter
            .check_rate_limit(&client1, OperationType::Read)
            .allowed
    );

    // Client2 should still have tokens
    assert!(
        limiter
            .check_rate_limit(&client2, OperationType::Read)
            .allowed
    );
    assert!(
        limiter
            .check_rate_limit(&client2, OperationType::Read)
            .allowed
    );
    assert!(
        limiter
            .check_rate_limit(&client2, OperationType::Read)
            .allowed
    );
    assert!(
        !limiter
            .check_rate_limit(&client2, OperationType::Read)
            .allowed
    );
}

#[test]
fn test_rate_limiter_read_write_separation() {
    let config = RateLimitConfig {
        enabled: true,
        read_requests_per_second: 10,
        read_burst_size: 5,
        write_requests_per_second: 2,
        write_burst_size: 2,
        cleanup_interval: Duration::from_secs(60),
        client_id_header: "X-Client-ID".to_string(),
    };
    let limiter = RateLimiter::new(config);

    let client = ClientId::from_string("test_client");

    // Exhaust write tokens
    assert!(
        limiter
            .check_rate_limit(&client, OperationType::Write)
            .allowed
    );
    assert!(
        limiter
            .check_rate_limit(&client, OperationType::Write)
            .allowed
    );
    assert!(
        !limiter
            .check_rate_limit(&client, OperationType::Write)
            .allowed
    );

    // Read tokens should still be available
    assert!(
        limiter
            .check_rate_limit(&client, OperationType::Read)
            .allowed
    );
    assert!(
        limiter
            .check_rate_limit(&client, OperationType::Read)
            .allowed
    );
    assert!(
        limiter
            .check_rate_limit(&client, OperationType::Read)
            .allowed
    );
}

#[test]
fn test_rate_limit_result_structure() {
    let result = RateLimitResult {
        allowed: true,
        remaining: 42,
        reset_after: Duration::from_secs(30),
        limit: 100,
        retry_after: None,
    };

    assert!(result.allowed);
    assert_eq!(result.remaining, 42);
    assert_eq!(result.reset_after, Duration::from_secs(30));
    assert_eq!(result.limit, 100);
    assert!(result.retry_after.is_none());
}

#[test]
fn test_rate_limit_result_rate_limited() {
    let result = RateLimitResult {
        allowed: false,
        remaining: 0,
        reset_after: Duration::from_secs(60),
        limit: 100,
        retry_after: Some(Duration::from_secs(5)),
    };

    assert!(!result.allowed);
    assert_eq!(result.remaining, 0);
    assert!(result.retry_after.is_some());
    assert_eq!(result.retry_after.unwrap(), Duration::from_secs(5));
}

#[test]
fn test_client_id_creation() {
    let from_string = ClientId::from_string("my_client");
    assert_eq!(from_string.to_string(), "id:my_client");

    let from_ip = ClientId::from_ip("192.168.1.1");
    assert_eq!(from_ip.to_string(), "ip:192.168.1.1");

    let empty_string = ClientId::from_string("");
    assert_eq!(empty_string.to_string(), "unknown");

    let empty_ip = ClientId::from_ip("");
    assert_eq!(empty_ip.to_string(), "unknown");
}

#[test]
fn test_rate_limiter_stats() {
    let config = RateLimitConfig::default();
    let limiter = RateLimiter::new(config);

    // Initially no buckets
    let stats = limiter.get_stats();
    assert_eq!(stats.read_buckets_count, 0);
    assert_eq!(stats.write_buckets_count, 0);
    assert!(stats.enabled);
    assert_eq!(stats.read_config, (100, 150));
    assert_eq!(stats.write_config, (20, 30));

    // Create some buckets
    let client1 = ClientId::from_string("client1");
    let client2 = ClientId::from_string("client2");

    limiter.check_rate_limit(&client1, OperationType::Read);
    limiter.check_rate_limit(&client2, OperationType::Read);
    limiter.check_rate_limit(&client1, OperationType::Write);

    let stats = limiter.get_stats();
    assert_eq!(stats.read_buckets_count, 2);
    assert_eq!(stats.write_buckets_count, 1);
}

#[test]
fn test_rate_limit_headers_generation() {
    let config = RateLimitConfig::default();
    let limiter = RateLimiter::new(config);

    let result = RateLimitResult {
        allowed: true,
        remaining: 75,
        reset_after: Duration::from_secs(45),
        limit: 100,
        retry_after: None,
    };

    let headers = limiter.get_headers(&result);

    // Check that all expected headers are present
    let limit_header = headers.iter().find(|(k, _)| k == "X-RateLimit-Limit");
    let remaining_header = headers.iter().find(|(k, _)| k == "X-RateLimit-Remaining");
    let reset_header = headers.iter().find(|(k, _)| k == "X-RateLimit-Reset");

    assert!(limit_header.is_some());
    assert!(remaining_header.is_some());
    assert!(reset_header.is_some());

    assert_eq!(limit_header.unwrap().1, "100");
    assert_eq!(remaining_header.unwrap().1, "75");
    assert_eq!(reset_header.unwrap().1, "45");
}

#[test]
fn test_rate_limit_headers_when_rate_limited() {
    let config = RateLimitConfig::default();
    let limiter = RateLimiter::new(config);

    let result = RateLimitResult {
        allowed: false,
        remaining: 0,
        reset_after: Duration::from_secs(60),
        limit: 100,
        retry_after: Some(Duration::from_secs(10)),
    };

    let headers = limiter.get_rate_limited_headers(&result);

    // Should include standard headers plus Retry-After
    let retry_header = headers.iter().find(|(k, _)| k == "Retry-After");
    assert!(retry_header.is_some());
    assert_eq!(retry_header.unwrap().1, "10");
}

#[tokio::test]
async fn test_rate_limiter_disabled_allows_all() {
    let config = RateLimitConfig {
        enabled: false,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);

    let client = ClientId::from_string("test");

    // Should allow many requests when disabled
    for _ in 0..1000 {
        let result = limiter.check_rate_limit(&client, OperationType::Read);
        assert!(result.allowed);
        assert_eq!(result.remaining, u32::MAX);
    }
}

#[test]
fn test_burst_allowance() {
    let config = RateLimitConfig {
        enabled: true,
        read_requests_per_second: 1, // Very slow refill
        read_burst_size: 10,         // But large burst
        write_requests_per_second: 1,
        write_burst_size: 10,
        cleanup_interval: Duration::from_secs(60),
        client_id_header: "X-Client-ID".to_string(),
    };
    let limiter = RateLimiter::new(config);

    let client = ClientId::from_string("burst_test");

    // Should allow burst_size requests immediately
    for i in 0..10 {
        let result = limiter.check_rate_limit(&client, OperationType::Read);
        assert!(result.allowed, "Request {} should be allowed in burst", i);
    }

    // 11th request should be rate limited
    let result = limiter.check_rate_limit(&client, OperationType::Read);
    assert!(!result.allowed, "Request 11 should be rate limited");
}
