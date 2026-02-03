//! # Batch Operations Module
//!
//! Optimized batch operations for episodes and patterns using transactions
//! and prepared statements for 4-6x throughput improvement.
//!
//! ## Modules
//!
//! - `episode_batch` - Episode batch operations
//! - `combined_batch` - Combined episode + pattern batch
//! - `query_batch` - Batch query operations
//! - `pattern_core` - Pattern storage batch operations
//! - `pattern_types` - Pattern batch types
//! - `heuristic_core` - Heuristic storage batch operations
//! - `heuristic_types` - Heuristic batch types

pub mod combined_batch;
pub mod episode_batch;
pub mod query_batch;

// Pattern batch modules
pub mod pattern_core;
pub mod pattern_types;

// Heuristic batch modules
pub mod heuristic_core;
pub mod heuristic_types;

// Tests
#[cfg(test)]
pub mod heuristic_tests;
#[cfg(test)]
pub mod pattern_tests;
