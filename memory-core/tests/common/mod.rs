//! Shared test infrastructure for memory-core tests
//!
//! This module provides common utilities, fixtures, builders, and assertion helpers
//! to eliminate code duplication across test files.

pub mod assertions;
pub mod fixtures;
pub mod helpers;

// Re-export commonly used items for convenience
pub use assertions::*;
pub use fixtures::*;
pub use helpers::*;
