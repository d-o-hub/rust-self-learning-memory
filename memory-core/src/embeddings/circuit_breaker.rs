//! Circuit breaker pattern for embedding providers

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Circuit breaker to prevent cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    config: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,
    /// Number of consecutive successes to close circuit
    pub success_threshold: u32,
    /// How long to wait before attempting recovery (seconds)
    pub timeout_seconds: u64,
    /// Maximum attempts in half-open state before reopening
    pub half_open_max_attempts: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_seconds: 30,
            half_open_max_attempts: 3,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone)]
enum CircuitState {
    /// Normal operation - requests pass through
    Closed { consecutive_failures: u32 },
    /// Circuit is open - requests fail fast
    Open { opened_at: Instant },
    /// Testing recovery - limited requests allowed
    HalfOpen {
        attempts: u32,
        consecutive_successes: u32,
    },
}

/// Circuit breaker result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, allow request
    Closed,
    /// Circuit is open, reject request
    Open,
    /// Circuit is half-open, allow limited requests
    HalfOpen,
}

/// Error when circuit is open
#[derive(Debug, thiserror::Error)]
#[error("Circuit breaker is open - provider unavailable")]
pub struct CircuitOpenError;

impl CircuitBreaker {
    /// Create a new circuit breaker
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed {
                consecutive_failures: 0,
            })),
            config,
        }
    }

    /// Create a circuit breaker with default configuration
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Check if request is allowed
    ///
    /// Returns `Ok(())` if request should proceed, `Err` if circuit is open
    pub fn allow_request(&self) -> Result<(), CircuitOpenError> {
        let mut state = self.state.lock().expect(
            "CircuitBreaker: state lock poisoned - this indicates a panic in circuit breaker code",
        );

        match *state {
            CircuitState::Closed { .. } => Ok(()),
            CircuitState::Open { opened_at } => {
                // Check if timeout has elapsed
                let elapsed = opened_at.elapsed();
                if elapsed >= Duration::from_secs(self.config.timeout_seconds) {
                    // Transition to half-open
                    tracing::info!("Circuit breaker transitioning to half-open");
                    *state = CircuitState::HalfOpen {
                        attempts: 0,
                        consecutive_successes: 0,
                    };
                    Ok(())
                } else {
                    Err(CircuitOpenError)
                }
            }
            CircuitState::HalfOpen {
                attempts,
                consecutive_successes,
            } => {
                if attempts < self.config.half_open_max_attempts {
                    // Increment attempt counter on each allow_request() call
                    *state = CircuitState::HalfOpen {
                        attempts: attempts + 1,
                        consecutive_successes,
                    };
                    Ok(())
                } else {
                    Err(CircuitOpenError)
                }
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let mut state = self.state.lock().expect(
            "CircuitBreaker: state lock poisoned - this indicates a panic in circuit breaker code",
        );

        match *state {
            CircuitState::Closed { .. } => {
                // Reset failure counter on success
                *state = CircuitState::Closed {
                    consecutive_failures: 0,
                };
            }
            CircuitState::HalfOpen {
                attempts,
                consecutive_successes,
            } => {
                let new_successes = consecutive_successes + 1;
                if new_successes >= self.config.success_threshold {
                    // Transition back to closed
                    tracing::info!("Circuit breaker closing - recovery successful");
                    *state = CircuitState::Closed {
                        consecutive_failures: 0,
                    };
                } else {
                    *state = CircuitState::HalfOpen {
                        attempts,
                        consecutive_successes: new_successes,
                    };
                }
            }
            CircuitState::Open { .. } => {
                // Shouldn't happen, but handle gracefully
                tracing::warn!("Received success while circuit is open");
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        let mut state = self.state.lock().expect(
            "CircuitBreaker: state lock poisoned - this indicates a panic in circuit breaker code",
        );

        match *state {
            CircuitState::Closed {
                consecutive_failures,
            } => {
                let new_failures = consecutive_failures + 1;
                if new_failures >= self.config.failure_threshold {
                    // Open circuit
                    tracing::warn!("Circuit breaker opening after {} failures", new_failures);
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                } else {
                    *state = CircuitState::Closed {
                        consecutive_failures: new_failures,
                    };
                }
            }
            CircuitState::HalfOpen {
                attempts: _,
                consecutive_successes: _,
            } => {
                // Failure during recovery - reopen circuit
                tracing::warn!("Circuit breaker reopening - recovery failed");
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
            }
            CircuitState::Open { .. } => {
                // Already open, do nothing
            }
        }
    }

    /// Get current circuit state
    #[must_use]
    pub fn state(&self) -> CircuitBreakerState {
        let state = self.state.lock().expect(
            "CircuitBreaker: state lock poisoned - this indicates a panic in circuit breaker code",
        );
        match *state {
            CircuitState::Closed { .. } => CircuitBreakerState::Closed,
            CircuitState::Open { .. } => CircuitBreakerState::Open,
            CircuitState::HalfOpen { .. } => CircuitBreakerState::HalfOpen,
        }
    }

    /// Reset circuit breaker to closed state
    pub fn reset(&self) {
        let mut state = self.state.lock().expect(
            "CircuitBreaker: state lock poisoned - this indicates a panic in circuit breaker code",
        );
        *state = CircuitState::Closed {
            consecutive_failures: 0,
        };
        tracing::info!("Circuit breaker manually reset");
    }

    /// Get circuit breaker statistics
    #[must_use]
    pub fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.lock().expect(
            "CircuitBreaker: state lock poisoned - this indicates a panic in circuit breaker code",
        );
        match *state {
            CircuitState::Closed {
                consecutive_failures,
            } => CircuitBreakerStats {
                state: CircuitBreakerState::Closed,
                consecutive_failures,
                consecutive_successes: 0,
                opened_at: None,
            },
            CircuitState::Open { opened_at } => CircuitBreakerStats {
                state: CircuitBreakerState::Open,
                consecutive_failures: 0,
                consecutive_successes: 0,
                opened_at: Some(opened_at),
            },
            CircuitState::HalfOpen {
                attempts: _,
                consecutive_successes,
            } => CircuitBreakerStats {
                state: CircuitBreakerState::HalfOpen,
                consecutive_failures: 0,
                consecutive_successes,
                opened_at: None,
            },
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitBreakerState,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub opened_at: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        assert!(cb.allow_request().is_ok());
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_opens_after_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);
        assert!(cb.allow_request().is_err());
    }

    #[test]
    fn test_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_seconds: 0,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);

        std::thread::sleep(Duration::from_millis(10));
        assert!(cb.allow_request().is_ok());
        assert_eq!(cb.state(), CircuitBreakerState::HalfOpen);
    }

    #[test]
    fn test_success_closes_circuit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout_seconds: 0,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(10));

        cb.allow_request()
            .expect("Circuit breaker should allow request in test context");
        cb.record_success();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_half_open_limits_attempts() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_seconds: 0,
            half_open_max_attempts: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Open and transition to half-open
        cb.record_failure();
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(10));
        assert!(cb.allow_request().is_ok());

        // First attempt allowed
        assert!(cb.allow_request().is_ok());

        // Second attempt allowed
        assert!(cb.allow_request().is_ok());

        // Third attempt rejected (max is 2)
        assert!(cb.allow_request().is_err());
    }

    #[test]
    fn test_circuit_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Open circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitBreakerState::Open);

        // Reset
        cb.reset();
        assert_eq!(cb.state(), CircuitBreakerState::Closed);
        assert!(cb.allow_request().is_ok());
    }
}
