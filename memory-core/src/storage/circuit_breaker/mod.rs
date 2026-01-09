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

mod states;
mod tests;

pub use states::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState};
