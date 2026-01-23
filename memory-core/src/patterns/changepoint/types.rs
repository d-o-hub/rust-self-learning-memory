//! # Changepoint Types
//!
//! Type definitions for the changepoint detection system.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for changepoint detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangepointConfig {
    /// Minimum changepoint probability threshold (0.0 to 1.0)
    pub min_probability: f64,
    /// Minimum distance between changepoints (in observations)
    pub min_distance: usize,
    /// Significance level for change detection
    pub significance_level: f64,
    /// Enable adaptive thresholding based on historical data
    pub adaptive_threshold: bool,
    /// Minimum observations required for detection
    pub min_observations: usize,
}

impl Default for ChangepointConfig {
    fn default() -> Self {
        Self {
            min_probability: 0.5,
            min_distance: 5,
            significance_level: 0.05,
            adaptive_threshold: true,
            min_observations: 10,
        }
        .validated()
    }
}

impl ChangepointConfig {
    /// Validate configuration values
    #[must_use]
    pub fn validated(mut self) -> Self {
        self.min_probability = self.min_probability.clamp(0.0, 1.0);
        self.min_distance = self.min_distance.max(1);
        self.significance_level = self.significance_level.clamp(0.0, 1.0);
        self.min_observations = self.min_observations.max(5);
        self
    }
}

/// A detected changepoint in the time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changepoint {
    /// Unique identifier for this changepoint
    pub id: Uuid,
    /// Index in the time series where changepoint occurs
    pub index: usize,
    /// Probability that this is a true changepoint
    pub probability: f64,
    /// Confidence interval for the changepoint location
    pub confidence_interval: (usize, usize),
    /// Type of change detected
    pub change_type: ChangeType,
    /// Magnitude of the change (absolute value)
    pub magnitude: f64,
    /// Direction of change (increase/decrease)
    pub direction: ChangeDirection,
    /// Timestamp when changepoint was detected
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Type of statistical change detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Mean value shift
    MeanShift,
    /// Change in variance/volatility
    VarianceChange,
    /// Both mean and variance changed
    MixedChange,
    /// Unknown/uncategorized change
    Unknown,
}

/// Direction of the detected change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDirection {
    /// Value increased at changepoint
    Increase,
    /// Value decreased at changepoint
    Decrease,
    /// No clear direction (mixed changes)
    Mixed,
}

/// Statistics for a segment of the time series
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SegmentStats {
    /// Number of observations in segment
    pub count: usize,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

/// Configuration for segment comparison
#[derive(Debug, Clone)]
pub struct SegmentComparisonConfig {
    /// Minimum effect size for considering a change significant
    pub min_effect_size: f64,
    /// Statistical significance threshold
    pub significance_level: f64,
}

impl Default for SegmentComparisonConfig {
    fn default() -> Self {
        Self {
            min_effect_size: 0.2,
            significance_level: 0.05,
        }
    }
}

/// Result of comparing two segments
#[derive(Debug, Clone)]
pub struct SegmentComparison {
    /// Effect size (Cohen's d)
    pub effect_size: f64,
    /// Whether change is statistically significant
    pub is_significant: bool,
    /// Estimated mean difference
    pub mean_difference: f64,
    /// P-value for the difference
    pub p_value: f64,
}

/// Error types for changepoint detection
#[derive(thiserror::Error, Debug)]
pub enum ChangepointError {
    #[error("Insufficient data points: have {have}, need at least {need}")]
    InsufficientData { have: usize, need: usize },

    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    #[error("Detection algorithm error: {message}")]
    AlgorithmError { message: String },

    #[error("Segment analysis error: {message}")]
    SegmentError { message: String },
}
