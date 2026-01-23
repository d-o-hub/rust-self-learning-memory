//! Advanced filtering for episodes
//!
//! This module provides comprehensive filtering capabilities for episodes,
//! allowing users to query episodes based on multiple criteria including
//! tags, dates, task types, outcomes, and more.

pub mod builder;
pub mod matcher;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export types for convenient public API access
pub use builder::EpisodeFilterBuilder;
pub use types::{EpisodeFilter, OutcomeType};
