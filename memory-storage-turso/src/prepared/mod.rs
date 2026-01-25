//! Prepared Statement Cache
//!
//! Caches compiled SQL statements to reduce parsing overhead by 80%.
//!
//! ## Features
//!
//! - **Thread-safe**: Uses `parking_lot::RwLock` for concurrent access
//! - **LRU Eviction**: Automatically evicts least recently used statements
//! - **Statistics Tracking**: Tracks hits, misses, and evictions
//! - **Configurable Size**: Default 100 statements, configurable via feature
//!
//! ## Usage
//!
//! ```rust
//! use memory_storage_turso::prepared::PreparedStatementCache;
//!
//! let cache = PreparedStatementCache::new(100);
//! ```

pub mod cache;
pub use cache::{PreparedCacheConfig, PreparedCacheStats, PreparedStatementCache};
