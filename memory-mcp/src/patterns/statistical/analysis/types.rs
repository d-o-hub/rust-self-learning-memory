//! # Statistical Analysis Types
//!
//! Type definitions for statistical analysis including BOCPD (Bayesian Online
//! Change Point Detection), changepoint detection, correlation analysis, and
//! trend detection.

use serde::{Deserialize, Serialize};

/// Configuration for statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalConfig {
    /// Significance level for statistical tests (default: 0.05)
    pub significance_level: f64,
    /// Maximum number of data points to analyze (default: 10,000)
    pub max_data_points: usize,
    /// Enable parallel processing (default: true)
    pub parallel_processing: bool,
    /// Bayesian changepoint detection parameters
    pub changepoint_config: ChangepointConfig,
}

impl Default for StatisticalConfig {
    fn default() -> Self {
        Self {
            significance_level: 0.05,
            max_data_points: 10_000,
            parallel_processing: true,
            changepoint_config: ChangepointConfig {
                hazard_rate: 100.0,
                expected_run_length: 50.0,
            },
        }
    }
}

/// Configuration for changepoint detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangepointConfig {
    /// Hazard rate for Bayesian changepoint detection
    pub hazard_rate: f64,
    /// Expected run length for truncated BOCPD
    pub expected_run_length: f64,
}

impl Default for ChangepointConfig {
    fn default() -> Self {
        Self {
            hazard_rate: 250.0,
            expected_run_length: 250.0,
        }
    }
}

/// BOCD (Bayesian Online Change Point Detection) state
#[derive(Debug, Clone)]
pub struct BOCPDState {
    /// Posterior probabilities over run lengths (in log space)
    pub log_posterior: Vec<f64>,
    /// Current hazard rate
    pub hazard_rate: f64,
    /// Circular buffer for data
    pub data_buffer: std::collections::VecDeque<f64>,
    /// Maximum buffer size
    pub max_buffer_size: usize,
    /// Number of processed points
    pub processed_points: usize,
}

/// Configuration for BOCD detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOCPDConfig {
    /// Hazard rate for change point detection
    pub hazard_rate: f64,
    /// Expected run length (for truncation)
    pub expected_run_length: usize,
    /// Maximum number of run length hypotheses
    pub max_run_length_hypotheses: usize,
    /// Alert threshold for change point confidence
    pub alert_threshold: f64,
    /// Maximum buffer size for data
    pub buffer_size: usize,
}

impl Default for BOCPDConfig {
    fn default() -> Self {
        Self {
            hazard_rate: 250.0,
            expected_run_length: 100,
            max_run_length_hypotheses: 200,
            alert_threshold: 0.7,
            buffer_size: 50,
        }
    }
}

/// BOCPD detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BOCPDResult {
    /// Detected changepoint index
    pub changepoint_index: Option<usize>,
    /// Posterior probability of changepoint
    pub changepoint_probability: f64,
    /// Maximum a posteriori run length
    pub map_run_length: usize,
    /// Posterior distribution over run lengths
    pub run_length_distribution: Vec<f64>,
    /// Detection confidence
    pub confidence: f64,
}

/// Statistical analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalResults {
    /// Correlation coefficients with significance
    pub correlations: Vec<CorrelationResult>,
    /// Detected changepoints
    pub changepoints: Vec<ChangepointResult>,
    /// Trend analysis results
    pub trends: Vec<TrendResult>,
    /// Analysis metadata
    pub metadata: AnalysisMetadata,
}

/// Correlation analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    /// Variable pair
    pub variables: (String, String),
    /// Pearson correlation coefficient
    pub coefficient: f64,
    /// P-value for significance test
    pub p_value: f64,
    /// Whether correlation is statistically significant
    pub significant: bool,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
}

/// Changepoint detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangepointResult {
    /// Index where changepoint was detected
    pub index: usize,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Type of changepoint detected
    pub change_type: ChangeType,
}

/// Types of changes that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    /// Mean shift
    MeanShift,
    /// Variance change
    VarianceChange,
    /// Trend change
    TrendChange,
    /// Unknown/unspecified
    Unknown,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendResult {
    /// Variable name
    pub variable: String,
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0 to 1.0)
    pub strength: f64,
    /// Statistical significance
    pub significant: bool,
}

/// Trend directions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Increasing trend
    Increasing,
    /// Decreasing trend
    Decreasing,
    /// No clear trend
    Stationary,
    /// Oscillating pattern
    Oscillating,
}

/// Analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Number of data points analyzed
    pub data_points: usize,
    /// Analysis duration in milliseconds
    pub duration_ms: u64,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Processing method used
    pub processing_method: String,
}
