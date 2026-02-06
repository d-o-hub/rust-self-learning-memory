//! Cache integration for Turso storage
//!
//! This module provides caching layers that integrate with TursoStorage
//! to improve read performance by reducing database queries.
//!
//! ## Architecture
//!
//! ```text
//! Client → CachedTursoStorage → AdaptiveTTLCache → TursoStorage
//!                                            ↓
//!                                    TTL Adaptation Engine
//!                                    Background Cleanup
//! ```
//!
//! ## Components
//!
//! - `config`: Cache configuration types (CacheConfig, CacheStats)
//! - `wrapper`: CachedTursoStorage implementation with transparent caching
//! - `adaptive_ttl`: Generic adaptive TTL cache with access pattern tracking
//! - `ttl_config`: TTL configuration types and validation
//! - `query_cache`: Advanced query result caching with smart invalidation
//! - `invalidation`: Cache invalidation strategies and management
//!
//! ## Usage
//!
//! ```no_run
//! use memory_storage_turso::{TursoStorage, CacheConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Use default cache configuration
//! let storage = TursoStorage::new("file:test.db", "").await?;
//! let cached = storage.with_cache_default();
//!
//! // Or create a custom cache configuration
//! let storage2 = TursoStorage::new("file:test2.db", "").await?;
//! let config = CacheConfig::default();
//! let cached2 = storage2.with_cache(config);
//!
//! // Use cached storage for all operations
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Usage (AdaptiveTTLCache)
//!
//! ```no_run
//! use memory_storage_turso::cache::{
//!     AdaptiveTTLCache, TTLConfig, TTLConfigError
//! };
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), TTLConfigError> {
//! // Create a custom adaptive TTL cache
//! let config = TTLConfig::new()
//!     .with_base_ttl(Duration::from_secs(600))
//!     .with_max_entries(5000);
//!
//! let cache = AdaptiveTTLCache::<String, String>::new(config)?;
//!
//! // Use the cache
//! cache.insert("key".to_string(), "value".to_string()).await;
//! let value = cache.get(&"key".to_string()).await;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Query Caching with Smart Invalidation
//!
//! ```no_run
//! use memory_storage_turso::cache::{
//!     AdvancedQueryCache, AdvancedQueryCacheConfig, InvalidationManager,
//!     InvalidationConfig, TableDependency, QueryKey
//! };
//! use std::time::Duration;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create advanced query cache
//! let config = AdvancedQueryCacheConfig::default();
//! let (cache, invalidation_rx) = AdvancedQueryCache::new(config);
//!
//! // Create invalidation manager
//! let inv_config = InvalidationConfig::default();
//! let (manager, event_tx) = InvalidationManager::new(inv_config, cache.clone());
//!
//! // Cache a query result
//! let key = QueryKey::from_sql("SELECT * FROM episodes");
//! let data = b"serialized results".to_vec();
//! let deps = vec![TableDependency::Episodes];
//! cache.put(key, data, deps);
//!
//! // Invalidate when episodes table changes
//! // event_tx.send(InvalidationEvent::TableModified { ... })?;
//! # Ok(())
//! # }
//! ```

// Generic adaptive TTL cache implementation
pub mod adaptive_ttl;
mod config;
pub mod integration;
pub mod invalidation;
pub mod query_cache;
mod ttl_config;
mod wrapper;

pub use adaptive_ttl::{AdaptiveTTLCache, CacheEntry, CacheStats, CacheStatsSnapshot};
pub use config::{CacheConfig, CacheStats as LegacyCacheStats};
pub use integration::{CachedQueryStorage, InvalidationMessage};
pub use invalidation::{
    CrudOperation, InvalidationConfig, InvalidationEvent, InvalidationManager, InvalidationMetrics,
    InvalidationRule, InvalidationRuleBuilder, InvalidationStrategy, InvalidationTarget,
    SchemaChangeType,
};
pub use query_cache::{
    AdvancedCacheStats, AdvancedQueryCache, AdvancedQueryCacheConfig, CachedResult,
    InvalidationMessage as QueryCacheInvalidationMessage, QueryKey, QueryType, TableDependency,
};
pub use ttl_config::{TTLConfig, TTLConfigError};
pub use wrapper::CachedTursoStorage;
