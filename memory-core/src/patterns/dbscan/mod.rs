//! # DBSCAN Anomaly Detection for Episodes
//!
//! This module implements the DBSCAN (Density-Based Spatial Clustering of
//! Applications with Noise) algorithm for detecting anomalous episodes.
//!
//! DBSCAN is ideal for this use case because:
//! - It doesn't require specifying the number of clusters upfront
//! - It naturally identifies outliers as noise (points not belonging to any cluster)
//! - It can find clusters of arbitrary shape
//!
//! ## Usage
//!
//! ```rust
//! use memory_core::patterns::DBSCANAnomalyDetector;
//!
//! let detector = DBSCANAnomalyDetector::new();
//! let anomalies = detector.detect_anomalies(&episodes).await;
//! ```
//!
//! ## Integration
//!
//! The detector is integrated into the learning cycle and can be called
//! during episode completion to identify unusual patterns.

pub mod algorithms;
pub mod detector;
pub mod tests;
pub mod types;

pub use detector::DBSCANAnomalyDetector;
pub use types::{
    Anomaly, AnomalyReason, ClusterCentroid, DBSCANClusterResult, DBSCANConfig, DBSCANStats,
    EpisodeCluster, FeatureWeights,
};
