//! Circuit breaker implementation for storage resilience.

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
