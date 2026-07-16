//! # Persistence Layer
//!
//! Handles backend traits, repository interfaces, query limits, and storage errors.
//! This is the **persistence** stage of the memory pipeline:
//! `Episode → [pre_storage: ingest] → [storage: persistence] → Backend`
//!
//! Responsibilities: backend trait definitions, repository interfaces,
//! query construction, error types, and circuit breaker patterns.

mod backend;
pub mod circuit_breaker;

pub use backend::StorageBackend;

/// Default limit for query operations (when not specified)
pub const DEFAULT_QUERY_LIMIT: usize = 100;

/// Maximum limit for query operations (prevents OOM)
pub const MAX_QUERY_LIMIT: usize = 1000;

/// Apply limit with defaults and bounds checking.
///
/// - If `limit` is None, returns the default limit (100)
/// - If `limit` is Some, clamps it to the maximum (1000)
#[must_use]
#[inline]
pub fn apply_query_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(l) => l.min(MAX_QUERY_LIMIT),
        None => DEFAULT_QUERY_LIMIT,
    }
}
