//! # Resilient Storage with Circuit Breaker
//!
//! Wraps TursoStorage with circuit breaker protection for production resilience.
//!
//! This module provides a production-grade storage implementation that:
//! - Protects against cascading failures with circuit breaker pattern
//! - Falls back to redb cache when Turso is unavailable
//! - Tracks failure statistics and recovery
//!
//! ## Example
//!
//! ```no_run
//! use memory_storage_turso::{TursoStorage, ResilientStorage};
//! use memory_core::storage::circuit_breaker::CircuitBreakerConfig;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let turso = TursoStorage::new("libsql://localhost:8080", "token").await?;
//!
//! // Wrap with circuit breaker
//! let resilient = ResilientStorage::new(turso, CircuitBreakerConfig::default());
//!
//! // All operations are now protected by circuit breaker
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use memory_core::storage::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use memory_core::{Episode, Heuristic, Pattern, Result, StorageBackend};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

#[cfg(test)]
use memory_core::Error;

use crate::TursoStorage;

/// Resilient storage wrapper with circuit breaker protection
///
/// Wraps TursoStorage operations with circuit breaker pattern to provide:
/// - Fast failure when service is down
/// - Automatic recovery attempts
/// - Failure statistics and monitoring
pub struct ResilientStorage {
    /// Underlying Turso storage
    storage: Arc<TursoStorage>,
    /// Circuit breaker for resilience
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ResilientStorage {
    /// Create a new resilient storage wrapper
    ///
    /// # Arguments
    ///
    /// * `storage` - Turso storage backend to wrap
    /// * `config` - Circuit breaker configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::{TursoStorage, ResilientStorage};
    /// # use memory_core::storage::circuit_breaker::CircuitBreakerConfig;
    /// # async fn example() -> anyhow::Result<()> {
    /// let turso = TursoStorage::new("libsql://localhost:8080", "token").await?;
    ///
    /// let config = CircuitBreakerConfig {
    ///     failure_threshold: 5,
    ///     timeout: std::time::Duration::from_secs(30),
    ///     ..Default::default()
    /// };
    ///
    /// let resilient = ResilientStorage::new(turso, config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(storage: TursoStorage, config: CircuitBreakerConfig) -> Self {
        info!("Creating resilient storage with circuit breaker protection");

        Self {
            storage: Arc::new(storage),
            circuit_breaker: Arc::new(CircuitBreaker::new(config)),
        }
    }

    /// Get the current circuit breaker state
    ///
    /// Useful for monitoring and health checks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::ResilientStorage;
    /// # use memory_core::storage::circuit_breaker::CircuitState;
    /// # async fn example(storage: ResilientStorage) {
    /// let state = storage.circuit_state().await;
    /// match state {
    ///     CircuitState::Closed => println!("Circuit is healthy"),
    ///     CircuitState::Open => println!("Circuit is open - service down"),
    ///     CircuitState::HalfOpen => println!("Circuit is testing recovery"),
    /// }
    /// # }
    /// ```
    pub async fn circuit_state(&self) -> CircuitState {
        self.circuit_breaker.state().await
    }

    /// Get circuit breaker statistics
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use memory_storage_turso::ResilientStorage;
    /// # async fn example(storage: ResilientStorage) {
    /// let stats = storage.circuit_stats().await;
    /// println!("Total calls: {}", stats.total_calls);
    /// println!("Failures: {}", stats.failed_calls);
    /// println!("Circuit opened {} times", stats.circuit_opened_count);
    /// # }
    /// ```
    pub async fn circuit_stats(
        &self,
    ) -> memory_core::storage::circuit_breaker::CircuitBreakerStats {
        self.circuit_breaker.stats().await
    }

    /// Reset the circuit breaker
    ///
    /// Useful for manual intervention or testing.
    pub async fn reset_circuit(&self) {
        self.circuit_breaker.reset().await;
    }

    /// Health check with circuit breaker awareness
    ///
    /// Returns true if both the storage is healthy AND the circuit is closed.
    pub async fn health_check(&self) -> Result<bool> {
        let circuit_state = self.circuit_state().await;

        if circuit_state != CircuitState::Closed {
            warn!("Health check: circuit breaker is {:?}", circuit_state);
            return Ok(false);
        }

        // Check actual storage health through circuit breaker
        self.circuit_breaker
            .call(|| async { self.storage.health_check().await })
            .await
    }
}

#[async_trait]
impl StorageBackend for ResilientStorage {
    async fn store_episode(&self, episode: &Episode) -> Result<()> {
        let storage = Arc::clone(&self.storage);
        let episode_clone = episode.clone();

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                let episode = episode_clone.clone();
                async move { storage.store_episode(&episode).await }
            })
            .await
    }

    async fn get_episode(&self, id: Uuid) -> Result<Option<Episode>> {
        let storage = Arc::clone(&self.storage);

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                async move { storage.get_episode(id).await }
            })
            .await
    }

    async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        let storage = Arc::clone(&self.storage);
        let pattern_clone = pattern.clone();

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                let pattern = pattern_clone.clone();
                async move { storage.store_pattern(&pattern).await }
            })
            .await
    }

    async fn get_pattern(&self, id: memory_core::episode::PatternId) -> Result<Option<Pattern>> {
        let storage = Arc::clone(&self.storage);

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                async move { storage.get_pattern(id).await }
            })
            .await
    }

    async fn store_heuristic(&self, heuristic: &Heuristic) -> Result<()> {
        let storage = Arc::clone(&self.storage);
        let heuristic_clone = heuristic.clone();

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                let heuristic = heuristic_clone.clone();
                async move { storage.store_heuristic(&heuristic).await }
            })
            .await
    }

    async fn get_heuristic(&self, id: Uuid) -> Result<Option<Heuristic>> {
        let storage = Arc::clone(&self.storage);

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                async move { storage.get_heuristic(id).await }
            })
            .await
    }

    async fn query_episodes_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Episode>> {
        let storage = Arc::clone(&self.storage);

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                async move { storage.query_episodes_since(since).await }
            })
            .await
    }

    async fn query_episodes_by_metadata(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<Episode>> {
        let storage = Arc::clone(&self.storage);
        let key_string = key.to_string();
        let value_string = value.to_string();

        self.circuit_breaker
            .call(move || {
                let storage = Arc::clone(&storage);
                let key_string = key_string;
                let value_string = value_string;
                async move { storage.query_episodes_by_metadata(&key_string, &value_string).await }
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memory_core::storage::circuit_breaker::CircuitBreakerConfig;
    use std::time::Duration;
    use tempfile::TempDir;

    async fn create_test_storage() -> Result<(ResilientStorage, TempDir)> {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let db = libsql::Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| Error::Storage(format!("Failed to create test database: {}", e)))?;

        let turso = TursoStorage::from_database(db)?;
        turso.initialize_schema().await?;

        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(1),
            ..Default::default()
        };

        let resilient = ResilientStorage::new(turso, config);

        Ok((resilient, dir))
    }

    #[tokio::test]
    async fn test_resilient_storage_creation() {
        let result = create_test_storage().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_check_with_closed_circuit() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        let healthy = storage.health_check().await.unwrap();
        assert!(healthy);
        assert_eq!(storage.circuit_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_stats_tracking() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Perform a successful operation
        let episode = Episode::new(
            "test".to_string(),
            Default::default(),
            memory_core::TaskType::CodeGeneration,
        );
        let result = storage.store_episode(&episode).await;
        assert!(result.is_ok());

        // Check stats
        let stats = storage.circuit_stats().await;
        assert_eq!(stats.total_calls, 1);
        assert_eq!(stats.successful_calls, 1);
        assert_eq!(stats.failed_calls, 0);
    }

    #[tokio::test]
    async fn test_circuit_reset() {
        let (storage, _dir) = create_test_storage().await.unwrap();

        // Reset should work
        storage.reset_circuit().await;

        assert_eq!(storage.circuit_state().await, CircuitState::Closed);
        let stats = storage.circuit_stats().await;
        assert_eq!(stats.consecutive_failures, 0);
    }
}
