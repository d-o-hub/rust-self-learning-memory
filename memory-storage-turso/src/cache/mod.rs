//! Cache integration for Turso storage
//!
//! This module provides caching layers that integrate with TursoStorage
//! to improve read performance by reducing database queries.

// Allow unexpected cfg warnings for adaptive-cache feature that may not be enabled
#![allow(unexpected_cfgs)]
//!
//! ## Architecture
//!
//! ```text
//! Client → CachedTursoStorage → AdaptiveTtlCache → redb AdaptiveCache → TursoStorage
//!                                            ↓
//!                                    Memory Pressure Monitor
//!                                    TTL Adaptation Engine
//! ```
//!
//! ## Components
//!
//! - `config`: Cache configuration types (CacheConfig, CacheStats)
//! - `wrapper`: CachedTursoStorage implementation with transparent caching
//! - `adaptive_ttl`: Advanced adaptive TTL cache with memory pressure awareness
//!
//! ## Usage
//!
//! ```no_run
//! use memory_storage_turso::{TursoStorage, CacheConfig, AdaptiveTtlCache, AdaptiveTtlConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let storage = TursoStorage::new("file:test.db", "").await?;
//!
//! // Use default adaptive cache configuration
//! let cached = storage.with_cache_default();
//!
//! // Or create a custom adaptive TTL cache
//! let adaptive_config = AdaptiveTtlConfig::default();
//! let adaptive_cache = AdaptiveTtlCache::<String>::new(adaptive_config);
//!
//! // Use cached storage for all operations
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "adaptive-cache")]
mod adaptive_ttl;
mod config;
mod wrapper;

#[cfg(test)]
mod tests;

pub use config::{CacheConfig, CacheStats};
pub use wrapper::CachedTursoStorage;

#[cfg(feature = "adaptive-cache")]
pub use adaptive_ttl::{
    AdaptiveTtlCache, AdaptiveTtlConfig, AdaptiveTtlStats, AdaptiveTtlStatsSnapshot,
    CacheEffectivenessReport, PressureLevel,
};
