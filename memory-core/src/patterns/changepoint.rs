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
//! // Create detector with default settings
//! let detector = ChangepointDetector::new(ChangepointConfig::default());
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

use anyhow::{anyhow, Context, Result};
use augurs_changepoint::{DefaultArgpcpDetector, Detector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, instrument, warn};
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Error, Debug)]
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

/// Detector for identifying changepoints in pattern metrics time series
#[derive(Debug, Clone)]
pub struct ChangepointDetector {
    /// Detection configuration
    config: ChangepointConfig,
    /// ARPCP detector instance
    detector: DefaultArgpcpDetector,
    /// Historical baselines for adaptive thresholding
    baselines: HashMap<String, BaselineStats>,
    /// Cache of recent detections
    recent_detections: Vec<Changepoint>,
}

/// Baseline statistics for adaptive thresholding
#[derive(Debug, Clone, Default)]
struct BaselineStats {
    /// Historical mean
    mean: f64,
    /// Historical standard deviation
    std_dev: f64,
    /// Number of observations
    count: usize,
    /// Last computed baseline
    last_update: chrono::DateTime<chrono::Utc>,
}

impl ChangepointDetector {
    /// Create a new changepoint detector with default configuration
    #[must_use]
    pub fn new(config: ChangepointConfig) -> Self {
        Self::with_config(config)
    }

    /// Create a detector with explicit configuration
    #[must_use]
    pub fn with_config(config: ChangepointConfig) -> Self {
        let validated_config = config.validated();

        Self {
            config: validated_config,
            detector: DefaultArgpcpDetector::default(),
            baselines: HashMap::new(),
            recent_detections: Vec::new(),
        }
    }

    /// Detect changepoints in a time series of values
    ///
    /// # Errors
    ///
    /// Returns `ChangepointError::InsufficientData` if fewer than `min_observations` values
    /// are provided, or `ChangepointError::InvalidData` if the data contains invalid values.
    #[instrument(skip(self))]
    pub fn detect_changepoints(&mut self, values: &[f64]) -> Result<Vec<Changepoint>> {
        // Validate input
        if values.len() < self.config.min_observations {
            return Err(anyhow!(ChangepointError::InsufficientData {
                have: values.len(),
                need: self.config.min_observations,
            }));
        }

        // Check for NaN or infinite values
        if values.iter().any(|v| !v.is_finite()) {
            return Err(anyhow!(ChangepointError::InvalidData {
                message: "Data contains NaN or infinite values".to_string(),
            }));
        }

        // Update adaptive baseline if enabled
        if self.config.adaptive_threshold {
            self.update_baseline("default", values);
        }

        // Run changepoint detection using ARPCP algorithm
        let changepoint_indices = self
            .detector
            .detect_changepoints(values)
            .map_err(|e| {
                anyhow!(ChangepointError::AlgorithmError {
                    message: e.to_string(),
                })
            })
            .context("Changelog detection failed")?;

        // Convert raw detections to our Changepoint struct
        let mut changepoints = self.convert_detections(values, &changepoint_indices);

        // Filter by minimum distance between changepoints
        changepoints = self.filter_by_min_distance(changepoints);

        // Filter by probability threshold
        changepoints.retain(|cp| cp.probability >= self.config.min_probability);

        // Update recent detections cache
        self.recent_detections = changepoints.clone();

        debug!(
            num_changepoints = changepoints.len(),
            min_probability = self.config.min_probability,
            min_distance = self.config.min_distance,
            "Detected changepoints"
        );

        Ok(changepoints)
    }

    /// Detect changepoints in a specific metric time series
    ///
    /// # Errors
    ///
    /// Returns errors from `detect_changepoints`.
    #[instrument(skip(self))]
    pub fn detect_metric_changepoints(
        &mut self,
        metric_name: &str,
        values: &[f64],
    ) -> Result<Vec<Changepoint>> {
        // Update metric-specific baseline
        if self.config.adaptive_threshold {
            self.update_baseline(metric_name, values);
        }

        // Run detection with adjusted threshold based on historical data
        let base_threshold = self.config.min_probability;
        self.config.min_probability = self.get_adaptive_threshold(metric_name);

        let result = self.detect_changepoints(values);

        // Restore original threshold
        self.config.min_probability = base_threshold;

        result
    }

    /// Analyze segments between changepoints
    #[instrument(skip(self, values))]
    pub fn analyze_segments(
        &self,
        values: &[f64],
        changepoints: &[Changepoint],
    ) -> Vec<(usize, usize, SegmentStats)> {
        if changepoints.is_empty() {
            // Return entire series as single segment
            let stats = compute_segment_stats(values);
            return vec![(0, values.len(), stats)];
        }

        let mut segments = Vec::new();
        let mut prev_idx = 0usize;

        for cp in changepoints {
            let end_idx = cp.index;
            if end_idx > prev_idx {
                let stats = compute_segment_stats(&values[prev_idx..end_idx]);
                segments.push((prev_idx, end_idx, stats));
            }
            prev_idx = cp.index;
        }

        // Add final segment
        if prev_idx < values.len() {
            let stats = compute_segment_stats(&values[prev_idx..]);
            segments.push((prev_idx, values.len(), stats));
        }

        segments
    }

    /// Compare two segments of the time series
    #[instrument(skip(self))]
    pub fn compare_segments(
        &self,
        values: &[f64],
        seg1: (usize, usize),
        seg2: (usize, usize),
        config: SegmentComparisonConfig,
    ) -> Result<SegmentComparison> {
        // Validate indices
        if seg1.1 <= seg1.0 || seg2.1 <= seg2.0 {
            return Err(anyhow!(ChangepointError::SegmentError {
                message: "Invalid segment boundaries".to_string(),
            }));
        }

        if seg1.1 > values.len() || seg2.1 > values.len() {
            return Err(anyhow!(ChangepointError::SegmentError {
                message: "Segment extends beyond data".to_string(),
            }));
        }

        let data1 = &values[seg1.0..seg1.1];
        let data2 = &values[seg2.0..seg2.1];

        let stats1 = compute_segment_stats(data1);
        let stats2 = compute_segment_stats(data2);

        // Calculate effect size (Cohen's d)
        let pooled_std = ((stats1.std_dev.powi(2) + stats2.std_dev.powi(2)) / 2.0).sqrt();
        let effect_size = if pooled_std > 1e-10 {
            (stats2.mean - stats1.mean) / pooled_std
        } else {
            0.0
        };

        // Calculate mean difference
        let mean_difference = stats2.mean - stats1.mean;

        // Simplified p-value approximation (t-test like)
        let n1 = stats1.count as f64;
        let n2 = stats2.count as f64;
        let se = pooled_std * (1.0 / n1 + 1.0 / n2).sqrt();
        let t_stat = if se > 1e-10 {
            mean_difference / se
        } else {
            0.0
        };

        // Approximate p-value using normal distribution
        let p_value = 2.0 * (1.0 - normal_cdf(t_stat.abs()));

        Ok(SegmentComparison {
            effect_size: effect_size.abs(),
            is_significant: effect_size.abs() >= config.min_effect_size
                && p_value < config.significance_level,
            mean_difference,
            p_value,
        })
    }

    /// Get the current adaptive threshold for a metric
    fn get_adaptive_threshold(&self, metric_name: &str) -> f64 {
        if let Some(baseline) = self.baselines.get(metric_name) {
            // Use baseline std_dev to adjust threshold
            // Higher variability = higher threshold needed
            let base_threshold = self.config.min_probability;
            let variability_factor = (baseline.std_dev / baseline.mean.abs().max(0.001)).min(2.0);
            (base_threshold * (0.5 + variability_factor * 0.5)).min(0.95)
        } else {
            self.config.min_probability
        }
    }

    /// Update baseline statistics for adaptive thresholding
    fn update_baseline(&mut self, metric_name: &str, values: &[f64]) {
        let stats = compute_segment_stats(values);
        let entry = self.baselines.entry(metric_name.to_string()).or_default();

        // Exponential moving average for baseline
        let alpha = 0.3;
        entry.mean = entry.mean * (1.0 - alpha) + stats.mean * alpha;
        entry.std_dev = entry.std_dev * (1.0 - alpha) + stats.std_dev * alpha;
        entry.count += values.len();
        entry.last_update = chrono::Utc::now();
    }

    /// Convert raw detections to Changepoint structs
    fn convert_detections(
        &self,
        values: &[f64],
        changepoint_indices: &[usize],
    ) -> Vec<Changepoint> {
        let mut changepoints = Vec::new();

        for (i, &cp_index) in changepoint_indices.iter().enumerate() {
            // Calculate probability based on position and surrounding values
            let probability = self.calculate_changepoint_probability(values, cp_index, i);

            // Determine change type by analyzing adjacent segments
            let change_type = self.classify_change_type(values, cp_index);

            // Calculate magnitude of change
            let magnitude = self.calculate_change_magnitude(values, cp_index);

            // Determine change direction
            let direction = self.determine_direction(values, cp_index);

            // Calculate confidence interval
            let ci = self.compute_confidence_interval(values, cp_index, i);

            changepoints.push(Changepoint {
                id: Uuid::new_v4(),
                index: cp_index,
                probability,
                confidence_interval: ci,
                change_type,
                magnitude,
                direction,
                detected_at: chrono::Utc::now(),
            });
        }

        changepoints
    }

    /// Calculate changepoint probability based on surrounding data
    fn calculate_changepoint_probability(
        &self,
        values: &[f64],
        cp_index: usize,
        detection_index: usize,
    ) -> f64 {
        // Base probability on detection order (earlier detections are more reliable)
        let base_prob = if detection_index == 0 {
            self.config.min_probability.max(0.7)
        } else {
            self.config.min_probability
        };

        // Adjust based on local variance
        let window = 5;
        let start = cp_index.saturating_sub(window);
        let end = (cp_index + window).min(values.len().saturating_sub(1));

        if start < end {
            let segment = &values[start..end];
            let mean: f64 = segment.iter().sum::<f64>() / segment.len() as f64;
            let variance: f64 =
                segment.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / segment.len() as f64;
            let std_dev = variance.sqrt();

            // Higher variance reduces confidence
            let variance_factor = (1.0 - (std_dev / mean.abs().max(0.001))).clamp(0.5, 1.0);
            (base_prob * variance_factor).clamp(0.0, 1.0)
        } else {
            base_prob
        }
    }

    /// Classify the type of change at a changepoint
    fn classify_change_type(&self, values: &[f64], cp_index: usize) -> ChangeType {
        let window = 5;
        let start = cp_index.saturating_sub(window);
        let end = (cp_index + window).min(values.len());

        let before_stats = compute_segment_stats(&values[start..cp_index]);
        let after_stats = compute_segment_stats(&values[cp_index..end]);

        let mean_change = (after_stats.mean - before_stats.mean).abs();
        let var_before = before_stats.std_dev.powi(2);
        let var_after = after_stats.std_dev.powi(2);
        let var_change = (var_after - var_before).abs();

        let mean_threshold = before_stats.std_dev * 0.5;
        let var_threshold = var_before * 0.3;

        let mean_significant = mean_change > mean_threshold;
        let var_significant = var_change > var_threshold;

        match (mean_significant, var_significant) {
            (true, false) => ChangeType::MeanShift,
            (false, true) => ChangeType::VarianceChange,
            (true, true) => ChangeType::MixedChange,
            (false, false) => ChangeType::Unknown,
        }
    }

    /// Calculate the magnitude of change at a changepoint
    fn calculate_change_magnitude(&self, values: &[f64], cp_index: usize) -> f64 {
        let window = 5;
        let start = cp_index.saturating_sub(window);
        let end = (cp_index + window).min(values.len());

        let before_stats = compute_segment_stats(&values[start..cp_index]);
        let after_stats = compute_segment_stats(&values[cp_index..end]);

        // Standardized mean difference
        let pooled_std =
            ((before_stats.std_dev.powi(2) + after_stats.std_dev.powi(2)) / 2.0).sqrt();
        if pooled_std > 1e-10 {
            (after_stats.mean - before_stats.mean).abs() / pooled_std
        } else {
            0.0
        }
    }

    /// Determine the direction of change
    fn determine_direction(&self, values: &[f64], cp_index: usize) -> ChangeDirection {
        let window = 5;
        let start = cp_index.saturating_sub(window);
        let end = (cp_index + window).min(values.len());

        let before_stats = compute_segment_stats(&values[start..cp_index]);
        let after_stats = compute_segment_stats(&values[cp_index..end]);

        let mean_diff = after_stats.mean - before_stats.mean;

        if mean_diff > before_stats.std_dev * 0.2 {
            ChangeDirection::Increase
        } else if mean_diff < -(before_stats.std_dev * 0.2) {
            ChangeDirection::Decrease
        } else {
            ChangeDirection::Mixed
        }
    }

    /// Compute confidence interval for changepoint location
    fn compute_confidence_interval(
        &self,
        values: &[f64],
        cp_index: usize,
        detection_index: usize,
    ) -> (usize, usize) {
        let half_window = self.config.min_distance / 2;

        // Base interval on min_distance
        let lower = if detection_index == 0 {
            0
        } else {
            cp_index.saturating_sub(half_window)
        };

        let upper = (cp_index + half_window).min(values.len().saturating_sub(1));

        (lower, upper)
    }

    /// Filter changepoints to ensure minimum distance
    fn filter_by_min_distance(&self, mut changepoints: Vec<Changepoint>) -> Vec<Changepoint> {
        if changepoints.is_empty() {
            return changepoints;
        }

        changepoints.sort_by(|a, b| a.index.cmp(&b.index));

        let mut filtered = Vec::with_capacity(changepoints.len());
        let mut last_cp_index = 0usize;

        for cp in changepoints {
            if cp.index >= last_cp_index + self.config.min_distance {
                filtered.push(cp);
                last_cp_index = cp.index;
            }
        }

        filtered
    }

    /// Get recent changepoint detections
    #[must_use]
    pub fn get_recent_detections(&self) -> &[Changepoint] {
        &self.recent_detections
    }

    /// Clear detection history
    pub fn clear_history(&mut self) {
        self.recent_detections.clear();
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &ChangepointConfig {
        &self.config
    }
}

/// Compute statistics for a segment of values
#[must_use]
fn compute_segment_stats(values: &[f64]) -> SegmentStats {
    if values.is_empty() {
        return SegmentStats::default();
    }

    let count = values.len();
    let mean: f64 = values.iter().sum::<f64>() / count as f64;

    let variance: f64 = if count > 1 {
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count - 1) as f64
    } else {
        0.0
    };

    let std_dev = variance.sqrt();

    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    SegmentStats {
        count,
        mean,
        std_dev,
        min,
        max,
    }
}

/// Standard normal CDF approximation
#[inline]
fn normal_cdf(x: f64) -> f64 {
    // Approximation of the error function
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changepoint_config_validation() {
        let config = ChangepointConfig {
            min_probability: 1.5,     // Invalid, should clamp
            min_distance: 0,          // Invalid, should clamp
            significance_level: -0.1, // Invalid, should clamp
            adaptive_threshold: true,
            min_observations: 3, // Invalid, should clamp to 5
        }
        .validated();

        assert_eq!(config.min_probability, 1.0);
        assert_eq!(config.min_distance, 1);
        assert_eq!(config.significance_level, 0.0);
        assert_eq!(config.min_observations, 5);
    }

    #[test]
    fn test_detect_changepoints_insufficient_data() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());
        let values = vec![0.5, 0.6, 0.7];

        let result = detector.detect_changepoints(&values);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<ChangepointError>(),
            Some(ChangepointError::InsufficientData { .. })
        ));
    }

    #[test]
    fn test_detect_changepoints_invalid_data() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());
        let values = vec![0.5, f64::NAN, 0.7];

        let result = detector.detect_changepoints(&values);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<ChangepointError>(),
            Some(ChangepointError::InvalidData { .. })
        ));
    }

    #[test]
    fn test_detect_changepoint_mean_shift() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create a series with a clear mean shift
        let values: Vec<f64> = vec![
            0.8, 0.82, 0.81, 0.79, 0.83, 0.80, 0.81, 0.82, // Normal ~0.81
            0.45, 0.48, 0.42, 0.44, 0.46, 0.47, 0.45, 0.48, // Drop to ~0.45
        ];

        let changepoints = detector.detect_changepoints(&values).unwrap();

        // Should detect at least one changepoint
        assert!(!changepoints.is_empty());

        // Changepoint should be in the transition zone
        let first_cp = &changepoints[0];
        assert!((8..12).contains(&first_cp.index));
    }

    #[test]
    fn test_detect_changepoint_increasing_trend() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create a series with increasing trend
        let values: Vec<f64> = (0..30)
            .map(|i| 0.5 + (i as f64 * 0.02) + (rand::random::<f64>() * 0.05))
            .collect();

        let changepoints = detector.detect_changepoints(&values).unwrap();

        // May or may not detect depending on PELT sensitivity
        // Just verify it runs without error
        let _ = changepoints;
    }

    #[test]
    fn test_analyze_segments() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();

        let changepoints = vec![Changepoint {
            id: Uuid::new_v4(),
            index: 10,
            probability: 0.9,
            confidence_interval: (8, 12),
            change_type: ChangeType::MeanShift,
            magnitude: 1.0,
            direction: ChangeDirection::Increase,
            detected_at: chrono::Utc::now(),
        }];

        let segments = detector.analyze_segments(&values, &changepoints);

        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            (
                0,
                10,
                SegmentStats {
                    count: 10,
                    mean: 4.5,
                    std_dev: 2.87,
                    min: 0.0,
                    max: 9.0
                }
            )
        );
        assert_eq!(
            segments[1],
            (
                10,
                20,
                SegmentStats {
                    count: 10,
                    mean: 14.5,
                    std_dev: 2.87,
                    min: 10.0,
                    max: 19.0
                }
            )
        );
    }

    #[test]
    fn test_compare_segments() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());
        let values: Vec<f64> = (0..20).map(|i| i as f64).collect();

        let comparison = detector
            .compare_segments(
                &values,
                (0, 10),
                (10, 20),
                SegmentComparisonConfig::default(),
            )
            .unwrap();

        assert!(comparison.is_significant);
        assert!(comparison.effect_size > 0.0);
        assert!((comparison.mean_difference - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_filter_by_min_distance() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        // Create changepoints too close together
        let mut changepoints = vec![
            Changepoint {
                id: Uuid::new_v4(),
                index: 5,
                probability: 0.9,
                confidence_interval: (3, 7),
                change_type: ChangeType::MeanShift,
                magnitude: 1.0,
                direction: ChangeDirection::Increase,
                detected_at: chrono::Utc::now(),
            },
            Changepoint {
                id: Uuid::new_v4(),
                index: 8,
                probability: 0.8,
                confidence_interval: (6, 10),
                change_type: ChangeType::MeanShift,
                magnitude: 0.8,
                direction: ChangeDirection::Increase,
                detected_at: chrono::Utc::now(),
            },
            Changepoint {
                id: Uuid::new_v4(),
                index: 15,
                probability: 0.7,
                confidence_interval: (13, 17),
                change_type: ChangeType::MeanShift,
                magnitude: 0.7,
                direction: ChangeDirection::Increase,
                detected_at: chrono::Utc::now(),
            },
        ];

        let filtered = detector.filter_by_min_distance(changepoints);

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].index, 5);
        assert_eq!(filtered[1].index, 15);
    }

    #[test]
    fn test_classify_change_type() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        // Mean shift only
        let values1: Vec<f64> = vec![
            0.5, 0.5, 0.5, 0.5, 0.5, // Before
            0.8, 0.8, 0.8, 0.8, 0.8, // After - mean shift
        ];
        let change_type = detector.classify_change_type(&values1, 5);
        assert_eq!(change_type, ChangeType::MeanShift);

        // Variance change only
        let values2: Vec<f64> = vec![
            0.5, 0.5, 0.5, 0.5, 0.5, // Before - stable
            0.3, 0.7, 0.4, 0.6, 0.2, // After - more variable
        ];
        let change_type = detector.classify_change_type(&values2, 5);
        assert!(matches!(
            change_type,
            ChangeType::VarianceChange | ChangeType::MixedChange
        ));
    }

    #[test]
    fn test_determine_direction() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        // Increase
        let values_inc: Vec<f64> = vec![0.5; 10].into_iter().chain(vec![0.8; 10]).collect();
        let direction = detector.determine_direction(&values_inc, 10);
        assert_eq!(direction, ChangeDirection::Increase);

        // Decrease
        let values_dec: Vec<f64> = vec![0.8; 10].into_iter().chain(vec![0.5; 10]).collect();
        let direction = detector.determine_direction(&values_dec, 10);
        assert_eq!(direction, ChangeDirection::Decrease);
    }

    #[test]
    fn test_compute_segment_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = compute_segment_stats(&values);

        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!((stats.min - 1.0).abs() < 0.001);
        assert!((stats.max - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_empty_segment_stats() {
        let values: Vec<f64> = vec![];
        let stats = compute_segment_stats(&values);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_get_recent_detections() {
        let detector = ChangepointDetector::new(ChangepointConfig::default());

        let detections = detector.get_recent_detections();
        assert!(detections.is_empty());
    }

    #[test]
    fn test_clear_history() {
        let mut detector = ChangepointDetector::new(ChangepointConfig::default());

        let values: Vec<f64> = (0..30)
            .map(|i| {
                let base = if i < 15 { 0.5 } else { 0.8 };
                base + rand::random::<f64>() * 0.1
            })
            .collect();

        let _ = detector.detect_changepoints(&values).unwrap();
        assert!(!detector.get_recent_detections().is_empty());

        detector.clear_history();
        assert!(detector.get_recent_detections().is_empty());
    }

    #[test]
    fn test_normal_cdf() {
        // Test CDF at known points
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }
}
