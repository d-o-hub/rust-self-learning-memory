//! Pattern clustering and deduplication
//!
//! This module provides functionality for:
//! - K-means clustering of episodes by pattern similarity
//! - Pattern deduplication based on similarity scores
//! - Pattern merging with confidence scoring

pub mod clusterer;
pub mod tests;
pub mod types;

pub use clusterer::PatternClusterer;
pub use types::{ClusterCentroid, ClusteringConfig, EpisodeCluster};
