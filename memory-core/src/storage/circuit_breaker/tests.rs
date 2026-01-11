//! Circuit breaker tests.

#![allow(unused_imports)]

use std::sync::Arc;
use std::time::Duration;

use crate::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::Error;

/// Helper function to create a test circuit breaker with custom config
#[allow(dead_code)]
fn create_test_circuit_breaker(failure_threshold: u32, timeout_secs: u64) -> CircuitBreaker {
    let config = CircuitBreakerConfig {
        failure_threshold,
        timeout: Duration::from_secs(timeout_secs),
        half_open_test_period: Duration::from_secs(1),
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(1600),
    };
    CircuitBreaker::new(config)
}

#[tokio::test]
async fn test_circuit_breaker_starts_closed() {
    let cb = create_test_circuit_breaker(5, 30);
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_successful_operation() {
    let cb = create_test_circuit_breaker(5, 30);

    let result = cb.call(|| async { Ok::<i32, Error>(42) }).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(cb.state().await, CircuitState::Closed);

    let stats = cb.stats().await;
    assert_eq!(stats.total_calls, 1);
    assert_eq!(stats.successful_calls, 1);
    assert_eq!(stats.failed_calls, 0);
}

#[tokio::test]
async fn test_failed_operation() {
    let cb = create_test_circuit_breaker(5, 30);

    let result = cb
        .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
        .await;

    assert!(result.is_err());
    assert_eq!(cb.state().await, CircuitState::Closed); // Still closed after 1 failure

    let stats = cb.stats().await;
    assert_eq!(stats.total_calls, 1);
    assert_eq!(stats.successful_calls, 0);
    assert_eq!(stats.failed_calls, 1);
    assert_eq!(stats.consecutive_failures, 1);
}

#[tokio::test]
async fn test_circuit_opens_after_threshold() {
    let cb = create_test_circuit_breaker(5, 30);

    // Trigger 5 consecutive failures
    for i in 0..5 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;

        if i < 4 {
            assert_eq!(cb.state().await, CircuitState::Closed);
        }
    }

    // Circuit should now be open
    assert_eq!(cb.state().await, CircuitState::Open);

    let stats = cb.stats().await;
    assert_eq!(stats.circuit_opened_count, 1);
    assert_eq!(stats.consecutive_failures, 5);
}

#[tokio::test]
async fn test_circuit_rejects_when_open() {
    let cb = create_test_circuit_breaker(3, 30);

    // Open the circuit
    for _ in 0..3 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    assert_eq!(cb.state().await, CircuitState::Open);

    // Next call should be rejected
    let result = cb.call(|| async { Ok::<i32, Error>(42) }).await;

    assert!(matches!(result, Err(Error::CircuitBreakerOpen)));

    let stats = cb.stats().await;
    assert_eq!(stats.rejected_calls, 1);
}

#[tokio::test]
async fn test_circuit_transitions_to_half_open() {
    let cb = create_test_circuit_breaker(3, 1); // 1 second timeout

    // Open the circuit
    for _ in 0..3 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    assert_eq!(cb.state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Next call should transition to half-open
    let _ = cb.call(|| async { Ok::<i32, Error>(42) }).await;

    assert_eq!(cb.state().await, CircuitState::Closed); // Success closes circuit
}

#[tokio::test]
async fn test_half_open_success_closes_circuit() {
    let cb = create_test_circuit_breaker(2, 1);

    // Open the circuit
    for _ in 0..2 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    assert_eq!(cb.state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Successful call should close the circuit
    let result = cb.call(|| async { Ok::<i32, Error>(42) }).await;

    assert!(result.is_ok());
    assert_eq!(cb.state().await, CircuitState::Closed);

    let stats = cb.stats().await;
    assert_eq!(stats.consecutive_failures, 0);
}

#[tokio::test]
async fn test_half_open_failure_reopens_circuit() {
    let cb = create_test_circuit_breaker(2, 1);

    // Open the circuit
    for _ in 0..2 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    assert_eq!(cb.state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Failed call should reopen the circuit
    let _ = cb
        .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
        .await;

    assert_eq!(cb.state().await, CircuitState::Open);

    let stats = cb.stats().await;
    assert_eq!(stats.circuit_opened_count, 2); // Opened twice
}

#[tokio::test]
async fn test_success_resets_consecutive_failures() {
    let cb = create_test_circuit_breaker(5, 30);

    // 2 failures
    for _ in 0..2 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    let stats = cb.stats().await;
    assert_eq!(stats.consecutive_failures, 2);

    // 1 success should reset
    let _ = cb.call(|| async { Ok::<i32, Error>(42) }).await;

    let stats = cb.stats().await;
    assert_eq!(stats.consecutive_failures, 0);
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_exponential_backoff() {
    let cb = create_test_circuit_breaker(5, 30);

    assert_eq!(cb.calculate_backoff(0), Duration::from_millis(100));
    assert_eq!(cb.calculate_backoff(1), Duration::from_millis(200));
    assert_eq!(cb.calculate_backoff(2), Duration::from_millis(400));
    assert_eq!(cb.calculate_backoff(3), Duration::from_millis(800));
    assert_eq!(cb.calculate_backoff(4), Duration::from_millis(1600));
    assert_eq!(cb.calculate_backoff(5), Duration::from_millis(1600)); // Max cap
}

#[tokio::test]
async fn test_non_recoverable_errors_dont_affect_circuit() {
    let cb = create_test_circuit_breaker(3, 30);

    // Non-recoverable errors shouldn't increment consecutive failures
    for _ in 0..5 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::InvalidInput("test error".to_string())) })
            .await;
    }

    let stats = cb.stats().await;
    assert_eq!(stats.consecutive_failures, 0);
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_reset_circuit_breaker() {
    let cb = create_test_circuit_breaker(2, 30);

    // Open the circuit
    for _ in 0..2 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    assert_eq!(cb.state().await, CircuitState::Open);

    // Reset
    cb.reset().await;

    assert_eq!(cb.state().await, CircuitState::Closed);
    let stats = cb.stats().await;
    assert_eq!(stats.consecutive_failures, 0);
}

#[tokio::test]
#[allow(clippy::excessive_nesting)]
async fn test_concurrent_operations() {
    let cb = Arc::new(create_test_circuit_breaker(10, 30));

    let mut handles = vec![];

    // Execute 20 concurrent operations
    for i in 0..20 {
        let cb_clone = Arc::clone(&cb);
        let handle = tokio::spawn(async move {
            let result_value = if i % 2 == 0 {
                Ok::<i32, Error>(i)
            } else {
                Err(Error::Storage("test error".to_string()))
            };
            cb_clone
                .call(|| async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    result_value
                })
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let _ = handle.await;
    }

    let stats = cb.stats().await;
    assert_eq!(stats.total_calls, 20);
    assert_eq!(stats.successful_calls, 10);
    assert_eq!(stats.failed_calls, 10);
}

#[tokio::test]
async fn test_exactly_threshold_failures() {
    let cb = create_test_circuit_breaker(5, 30);

    // Exactly 5 failures
    for _ in 0..5 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    assert_eq!(cb.state().await, CircuitState::Open);

    let stats = cb.stats().await;
    assert_eq!(stats.consecutive_failures, 5);
    assert_eq!(stats.circuit_opened_count, 1);
}

#[tokio::test]
async fn test_rapid_failures() {
    let cb = create_test_circuit_breaker(3, 30);

    // Rapid successive failures
    for _ in 0..10 {
        let _ = cb
            .call(|| async { Err::<i32, Error>(Error::Storage("test error".to_string())) })
            .await;
    }

    let stats = cb.stats().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    // Should have rejected some calls
    assert!(stats.rejected_calls > 0);
}
