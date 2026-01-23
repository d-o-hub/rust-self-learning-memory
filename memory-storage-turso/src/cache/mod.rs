//! Cache integration for Turso storage
//!
//! This module provides caching layers that integrate with TursoStorage
//! to improve read performance by reducing database queries.
//!
//! ## Architecture
//!
//! ```text
//! Client → CachedTursoStorage → AdaptiveCache → TursoStorage → Database
//!                                     ↓
//!                             Cache Hit (fast path)
//! ```
//!
//! ## Components
//!
//! - `config`: Cache configuration types
//! - `wrapper`: CachedTursoStorage implementation
//! - `query_cache`: Query result caching

mod config;
mod wrapper;

#[cfg(test)]
mod tests;

pub use config::{CacheConfig, CacheStats};
pub use wrapper::CachedTursoStorage;
