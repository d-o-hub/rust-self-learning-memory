//! # Circuit Breaker Pattern for Storage Resilience
//!
//! Implements a production-grade circuit breaker to handle Turso failures gracefully.
//!
//! ## Circuit States
//!
//! - **Closed**: Normal operation, all requests pass through
//! - **Open**: Too many failures detected, requests fail immediately
//! - **Half-Open**: Testing if the service has recovered
//!
//! ## Configuration
//!
//! - Failure threshold: Configurable consecutive failures to open circuit
//! - Timeout: Duration before attempting recovery (OPEN -> `HALF_OPEN`)
//! - Half-open test period: Duration to test recovery before closing
//! - Exponential backoff: Progressive delays between retries
//!
//! ## Example
//!
//! ```no_run
//! use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = CircuitBreakerConfig::default();
//! let circuit_breaker = Arc::new(CircuitBreaker::new(config));
//!
//! // Execute operation with circuit breaker protection
//! let result = circuit_breaker.call(|| async {
//!     // Your Turso operation here
//!     Ok::<_, memory_core::Error>(())
//! }).await;
//! # Ok(())
//! # }
//! ```

use crate::{Error, Result};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,
    /// Too many failures - reject requests immediately
    Open,
    /// Testing recovery - allow limited requests
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit (default: 5)
    pub failure_threshold: u32,
    /// Duration to wait before attempting half-open state (default: 30s)
    pub timeout: Duration,
    /// Duration to test in half-open state (default: 10s)
    pub half_open_test_period: Duration,
    /// Base delay for exponential backoff (default: 100ms)
    pub base_delay: Duration,
    /// Maximum delay for exponential backoff (default: 1600ms)
    pub max_delay: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(30),
            half_open_test_period: Duration::from_secs(10),
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1600),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerStats {
    /// Total number of calls attempted
    pub total_calls: u64,
    /// Number of successful calls
    pub successful_calls: u64,
    /// Number of failed calls
    pub failed_calls: u64,
    /// Number of calls rejected due to open circuit
    pub rejected_calls: u64,
    /// Current consecutive failure count
    pub consecutive_failures: u32,
    /// Number of times circuit opened
    pub circuit_opened_count: u32,
}

/// Internal state of the circuit breaker
struct CircuitBreakerState {
    /// Current state of the circuit
    state: CircuitState,
    /// Statistics
    stats: CircuitBreakerStats,
    /// Timestamp when circuit was last opened
    last_failure_time: Option<Instant>,
    /// Timestamp when half-open state was entered
    half_open_started: Option<Instant>,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            stats: CircuitBreakerStats::default(),
            last_failure_time: None,
            half_open_started: None,
        }
    }
}

/// Production-grade circuit breaker for storage resilience
///
/// Protects against cascading failures by failing fast when a service is down,
/// and automatically attempting recovery.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Circuit breaker configuration
    ///
    /// # Example
    ///
    /// ```
    /// use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    ///
    /// let config = CircuitBreakerConfig {
    ///     failure_threshold: 3,
    ///     timeout: std::time::Duration::from_secs(10),
    ///     ..Default::default()
    /// };
    /// let circuit_breaker = CircuitBreaker::new(config);
    /// ```
    pub fn new(config: CircuitBreakerConfig) -> Self {
        info!(
            "Initializing circuit breaker: threshold={}, timeout={:?}",
            config.failure_threshold, config.timeout
        );

        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::default())),
        }
    }

    /// Execute an operation protected by the circuit breaker
    ///
    /// # Arguments
    ///
    /// * `operation` - Async closure that returns a Result
    ///
    /// # Returns
    ///
    /// Result of the operation, or `CircuitBreakerOpen` error if circuit is open
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_core::storage::circuit_breaker::CircuitBreaker;
    /// # use memory_core::storage::circuit_breaker::CircuitBreakerConfig;
    /// # async fn example() -> memory_core::Result<()> {
    /// let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
    /// let result = cb.call(|| async {
    ///     // Your database operation
    ///     Ok::<_, memory_core::Error>(())
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // Check current state and decide if we should allow the call
        let should_proceed = self.should_allow_request().await?;

        if !should_proceed {
            let mut state = self.state.write().await;
            state.stats.rejected_calls += 1;
            debug!("Circuit breaker rejecting request - circuit is open");
            return Err(Error::CircuitBreakerOpen);
        }

        // Increment total calls
        {
            let mut state = self.state.write().await;
            state.stats.total_calls += 1;
        }

        // Execute the operation
        let result = operation().await;

        // Record the result
        self.on_result(&result).await;

        result
    }

    /// Check if a request should be allowed based on circuit state
    async fn should_allow_request(&self) -> Result<bool> {
        let now = Instant::now();
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => Ok(true),
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = state.last_failure_time {
                    if now.duration_since(last_failure) >= self.config.timeout {
                        info!("Circuit breaker transitioning to half-open state");
                        state.state = CircuitState::HalfOpen;
                        state.half_open_started = Some(now);
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            CircuitState::HalfOpen => {
                // In half-open state, allow requests to test recovery
                Ok(true)
            }
        }
    }

    /// Record the result of an operation and update circuit state
    async fn on_result<T>(&self, result: &Result<T>) {
        let mut state = self.state.write().await;

        match result {
            Ok(_) => {
                state.stats.successful_calls += 1;
                self.on_success(&mut state).await;
            }
            Err(e) => {
                // Only count recoverable errors as failures
                if e.is_recoverable() {
                    state.stats.failed_calls += 1;
                    self.on_failure(&mut state).await;
                } else {
                    // Non-recoverable errors don't affect circuit state
                    debug!("Non-recoverable error, not affecting circuit: {}", e);
                }
            }
        }
    }

    /// Handle successful operation
    #[allow(clippy::unused_async)]
    async fn on_success(&self, state: &mut CircuitBreakerState) {
        match state.state {
            CircuitState::HalfOpen => {
                // Success in half-open state - close the circuit
                info!("Circuit breaker closing after successful recovery test");
                state.state = CircuitState::Closed;
                state.stats.consecutive_failures = 0;
                state.last_failure_time = None;
                state.half_open_started = None;
            }
            CircuitState::Closed => {
                // Reset consecutive failures on success
                if state.stats.consecutive_failures > 0 {
                    debug!(
                        "Resetting consecutive failures from {}",
                        state.stats.consecutive_failures
                    );
                    state.stats.consecutive_failures = 0;
                }
            }
            CircuitState::Open => {
                // This shouldn't happen, but reset if it does
                warn!("Unexpected success in open state");
            }
        }
    }

    /// Handle failed operation
    #[allow(clippy::unused_async)]
    async fn on_failure(&self, state: &mut CircuitBreakerState) {
        state.stats.consecutive_failures += 1;
        state.last_failure_time = Some(Instant::now());

        debug!(
            "Circuit breaker recorded failure {}/{}",
            state.stats.consecutive_failures, self.config.failure_threshold
        );

        match state.state {
            CircuitState::Closed => {
                if state.stats.consecutive_failures >= self.config.failure_threshold {
                    warn!(
                        "Circuit breaker opening after {} consecutive failures",
                        state.stats.consecutive_failures
                    );
                    state.state = CircuitState::Open;
                    state.stats.circuit_opened_count += 1;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                warn!("Circuit breaker reopening after failure in half-open state");
                state.state = CircuitState::Open;
                state.stats.circuit_opened_count += 1;
                state.half_open_started = None;
            }
            CircuitState::Open => {
                // Already open, just track the failure
            }
        }
    }

    /// Get the current state of the circuit breaker
    ///
    /// # Example
    ///
    /// ```
    /// # use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    /// # async fn example() {
    /// let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
    /// let state = cb.state().await;
    /// assert_eq!(state, CircuitState::Closed);
    /// # }
    /// ```
    pub async fn state(&self) -> CircuitState {
        let state = self.state.read().await;
        state.state
    }

    /// Get current statistics
    ///
    /// # Example
    ///
    /// ```
    /// # use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    /// # async fn example() {
    /// let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
    /// let stats = cb.stats().await;
    /// println!("Total calls: {}", stats.total_calls);
    /// # }
    /// ```
    pub async fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await;
        state.stats.clone()
    }

    /// Calculate exponential backoff delay based on attempt number
    ///
    /// # Arguments
    ///
    /// * `attempt` - Attempt number (0-based)
    ///
    /// # Returns
    ///
    /// Duration to wait before next attempt
    ///
    /// # Example
    ///
    /// ```
    /// # use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    /// let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
    /// let delay = cb.calculate_backoff(0); // 100ms
    /// let delay = cb.calculate_backoff(1); // 200ms
    /// let delay = cb.calculate_backoff(2); // 400ms
    /// let delay = cb.calculate_backoff(3); // 800ms
    /// let delay = cb.calculate_backoff(4); // 1600ms
    /// ```
    #[must_use]
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let delay = self.config.base_delay.as_millis() as u64 * 2u64.pow(attempt);
        let delay = Duration::from_millis(delay);
        std::cmp::min(delay, self.config.max_delay)
    }

    /// Reset the circuit breaker to closed state
    ///
    /// This is primarily useful for testing or manual intervention.
    ///
    /// # Example
    ///
    /// ```
    /// # use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    /// # async fn example() {
    /// let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
    /// cb.reset().await;
    /// assert_eq!(cb.state().await, CircuitState::Closed);
    /// # }
    /// ```
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        info!("Circuit breaker reset to closed state");
        state.state = CircuitState::Closed;
        state.stats.consecutive_failures = 0;
        state.last_failure_time = None;
        state.half_open_started = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test circuit breaker with custom config
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
}
