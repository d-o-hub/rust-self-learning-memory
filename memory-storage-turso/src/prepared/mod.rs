//! Prepared Statement Cache
//!
//! Provides connection-aware caching of prepared statement metadata to track
//! query performance and optimize statement reuse.
//!
//! ## Features
//!
//! - **Connection-aware**: Tracks statements per connection using ConnectionId
//! - **Thread-safe**: Uses `parking_lot::RwLock` for concurrent access
//! - **LRU Eviction**: Automatically evicts least recently used statements
//! - **Statistics Tracking**: Tracks hits, misses, evictions, and preparation times
//! - **Configurable Size**: Default 100 statements per connection, configurable
//!
//! ## Architecture
//!
//! The cache uses a two-level structure:
//! ```text
//! ConnectionId -> { SQL -> CachedStatementMetadata }
//! ```
//!
//! Note: Due to libsql::Statement not implementing Clone or Send, we track
//! metadata rather than the actual statement objects. SQLite's internal
//! statement cache provides the actual performance benefits.
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_turso::prepared::{PreparedStatementCache, ConnectionId};
//!
//! let cache = PreparedStatementCache::new(100);
//! let conn_id = cache.get_connection_id();
//!
//! // Record a cache hit
//! cache.record_hit(conn_id, "SELECT 1");
//!
//! // Record a cache miss (statement was prepared)
//! cache.record_miss(conn_id, "SELECT 1", 100); // 100 microseconds
//! ```

pub mod cache;
pub use cache::{PreparedCacheConfig, PreparedCacheStats, PreparedStatementCache};

// Re-export ConnectionId from pool module
pub use crate::pool::ConnectionId;

#[cfg(test)]
mod tests;
