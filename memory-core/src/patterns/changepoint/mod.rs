//! # Changepoint Detection for Pattern Monitoring
//!
//! Implements changepoint detection using the PELT (Pruned Exact Linear Time) algorithm
//! from the augurs-changepoint crate. This module detects significant changes in
//! pattern metrics over time, enabling adaptive pattern learning.
//!
//! ## Example
//!
//! ```
//! use memory_core::patterns::changepoint::{ChangepointDetector, ChangepointConfig};
//!
//! // Create detector with default settings (needs to be mutable for detection)
//! let mut detector = ChangepointDetector::new(ChangepointConfig::default());
//!
//! // Simulate pattern success rate time series
//! let metrics = vec![
//!     0.8, 0.82, 0.81, 0.79, 0.83, // Normal variation
//!     0.45, 0.48, 0.42, 0.44,      // Drop (changepoint detected)
//!     0.46, 0.47, 0.45, 0.48,      // New baseline
//! ];
//!
//! // Detect changepoints
//! let changepoints = detector.detect_changepoints(&metrics).unwrap();
//! println!("Detected {} changepoints", changepoints.len());
//! ```
//!
//! ## Integration with Monitoring
//!
//! The changepoint detector integrates with the agent monitoring system to:
//! - Detect significant changes in pattern success rates
//! - Identify shifts in task execution metrics
//! - Trigger pattern recalibration when drift is detected

pub mod algorithms;
pub mod detector;
pub mod tests;
pub mod types;

pub use detector::ChangepointDetector;
pub use types::{
    ChangeDirection, ChangeType, Changepoint, ChangepointConfig, ChangepointError,
    SegmentComparison, SegmentComparisonConfig, SegmentStats,
};
