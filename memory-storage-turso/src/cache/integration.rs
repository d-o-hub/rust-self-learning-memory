//! Integration of AdvancedQueryCache with TursoStorage
//!
//! This module provides a bridge between the TursoStorage backend and the
//! advanced query result caching system with smart invalidation.

use crate::cache::invalidation::{
    InvalidationConfig, InvalidationEvent, InvalidationManager, InvalidationRuleBuilder,
    InvalidationTarget,
};
use crate::cache::query_cache::{
    AdvancedCacheStats, AdvancedQueryCache, AdvancedQueryCacheConfig, QueryKey, TableDependency,
};
use crate::TursoStorage;
use anyhow::Result;
use memory_core::{Episode, Pattern};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Storage wrapper with advanced query caching
pub struct CachedQueryStorage {
    /// Underlying Turso storage
    storage: Arc<TursoStorage>,
    /// Query cache
    cache: AdvancedQueryCache,
    /// Invalidation manager
    invalidation_manager: Option<InvalidationManager>,
    /// Event sender for invalidation
    event_tx: Option<mpsc::UnboundedSender<InvalidationEvent>>,
}

impl CachedQueryStorage {
    /// Create a new cached query storage with default configuration
    pub fn new(storage: TursoStorage) -> (Self, mpsc::UnboundedReceiver<InvalidationMessage>) {
        let cache_config = AdvancedQueryCacheConfig::default();
        Self::with_config(storage, cache_config)
    }

    /// Create with custom cache configuration
    pub fn with_config(
        storage: TursoStorage,
        cache_config: AdvancedQueryCacheConfig,
    ) -> (Self, mpsc::UnboundedReceiver<InvalidationMessage>) {
        let (cache, invalidation_rx) = AdvancedQueryCache::new(cache_config);

        let cached_storage = Self {
            storage: Arc::new(storage),
            cache: cache.clone(),
            invalidation_manager: None,
            event_tx: None,
        };

        (cached_storage, invalidation_rx)
    }

    /// Create with full invalidation support
    pub fn with_invalidation(
        storage: TursoStorage,
        cache_config: AdvancedQueryCacheConfig,
        invalidation_config: InvalidationConfig,
    ) -> (
        Self,
        mpsc::UnboundedReceiver<InvalidationMessage>,
        mpsc::UnboundedSender<InvalidationEvent>,
    ) {
        let (cache, invalidation_rx) = AdvancedQueryCache::new(cache_config);
        let (manager, event_tx) = InvalidationManager::new(invalidation_config, cache.clone());

        // Add default rules
        Self::setup_default_rules(&manager);

        let cached_storage = Self {
            storage: Arc::new(storage),
            cache,
            invalidation_manager: Some(manager),
            event_tx: Some(event_tx.clone()),
        };

        (cached_storage, invalidation_rx, event_tx)
    }

    /// Setup default invalidation rules
    fn setup_default_rules(manager: &InvalidationManager) {
        // Episodes queries depend on episodes and steps tables
        manager.add_rule(
            InvalidationRuleBuilder::new("%episodes%")
                .depends_on(TableDependency::Episodes)
                .depends_on(TableDependency::Steps)
                .with_priority(10)
                .build(),
        );

        // Pattern queries depend on patterns table
        manager.add_rule(
            InvalidationRuleBuilder::new("%patterns%")
                .depends_on(TableDependency::Patterns)
                .with_priority(10)
                .build(),
        );

        // Statistics queries have shorter TTL
        manager.add_rule(
            InvalidationRuleBuilder::new("%count%")
                .depends_on(TableDependency::Episodes)
                .depends_on(TableDependency::Patterns)
                .depends_on(TableDependency::Steps)
                .with_ttl(Duration::from_secs(30))
                .with_priority(5)
                .build(),
        );

        // Search queries
        manager.add_rule(
            InvalidationRuleBuilder::new("%search%")
                .depends_on(TableDependency::Episodes)
                .depends_on(TableDependency::Patterns)
                .depends_on(TableDependency::Embeddings)
                .with_ttl(Duration::from_secs(120))
                .with_priority(8)
                .build(),
        );
    }

    /// Execute a cached query
    pub async fn query_cached<F, Fut, T>(
        &self,
        sql: &str,
        params: &[&dyn ToString],
        fetch_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let key = QueryKey::new(sql, params);

        // Try to get from cache
        if let Some(cached_data) = self.cache.get(&key) {
            debug!("Cache hit for query: {}", sql);
            return match postcard::from_bytes(&cached_data) {
                Ok(result) => Ok(result),
                Err(e) => {
                    warn!("Failed to deserialize cached result: {}", e);
                    // Fall through to fetch
                    fetch_fn().await
                }
            };
        }

        // Cache miss - fetch from storage
        debug!("Cache miss for query: {}", sql);
        let result = fetch_fn().await?;

        // Serialize and cache the result
        match postcard::to_allocvec(&result) {
            Ok(data) => {
                let dependencies = TableDependency::from_query(sql);
                self.cache.put(key, data, dependencies);
            }
            Err(e) => {
                warn!("Failed to serialize result for caching: {}", e);
            }
        }

        Ok(result)
    }

    /// Query episodes with caching
    pub async fn query_episodes_cached(
        &self,
        sql: &str,
        params: &[&dyn ToString],
    ) -> Result<Vec<Episode>> {
        self.query_cached(sql, params, || async {
            // This would call the actual storage method
            // For now, return empty vec as placeholder
            Ok(Vec::new())
        })
        .await
    }

    /// Query patterns with caching
    pub async fn query_patterns_cached(
        &self,
        sql: &str,
        params: &[&dyn ToString],
    ) -> Result<Vec<Pattern>> {
        self.query_cached(sql, params, || async {
            // This would call the actual storage method
            // For now, return empty vec as placeholder
            Ok(Vec::new())
        })
        .await
    }

    /// Invalidate cache by table
    pub fn invalidate_table(&self, table: TableDependency) {
        self.cache.invalidate_by_table(&table);

        // Also send event if manager is running
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(InvalidationEvent::TableModified {
                table,
                operation: crate::cache::invalidation::CrudOperation::Update,
                affected_rows: 0,
            });
        }
    }

    /// Invalidate all cache entries
    pub fn invalidate_all(&self) {
        self.cache.clear();

        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(InvalidationEvent::ManualInvalidation {
                target: InvalidationTarget::All,
                reason: "Manual cache clear".to_string(),
            });
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> AdvancedCacheStats {
        self.cache.stats()
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Clear expired entries
    pub fn clear_expired(&self) -> usize {
        self.cache.clear_expired()
    }

    /// Get the underlying storage
    pub fn storage(&self) -> &TursoStorage {
        &self.storage
    }

    /// Get the cache reference
    pub fn cache(&self) -> &AdvancedQueryCache {
        &self.cache
    }

    /// Start the invalidation manager
    pub async fn start_invalidation_manager(self) {
        if let Some(manager) = self.invalidation_manager {
            info!("Starting invalidation manager");
            manager.run().await;
        }
    }
}

impl Clone for CachedQueryStorage {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            cache: self.cache.clone(),
            invalidation_manager: self.invalidation_manager.clone(),
            event_tx: self.event_tx.clone(),
        }
    }
}

/// Invalidation message type (re-exported from query_cache)
pub use crate::cache::query_cache::InvalidationMessage;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::QueryType;

    #[test]
    fn test_cached_query_storage_creation() {
        // This would need a real TursoStorage instance
        // For now, just verify the types compile correctly
    }

    #[test]
    fn test_query_key_creation() {
        let sql = "SELECT * FROM episodes WHERE domain = ?";
        let key = QueryKey::new(sql, &[&"test_domain"]);

        assert_eq!(key.query_type, QueryType::Episode);
    }

    #[test]
    fn test_table_dependency_detection() {
        let sql = "SELECT e.*, s.* FROM episodes e JOIN steps s ON e.episode_id = s.episode_id";
        let deps = TableDependency::from_query(sql);

        assert!(deps.contains(&TableDependency::Episodes));
        assert!(deps.contains(&TableDependency::Steps));
    }
}
