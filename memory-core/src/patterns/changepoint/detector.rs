//! # Changepoint Detector
//!
//! Main detector implementation for identifying changepoints in time series data.

use super::algorithms::{compute_segment_stats, normal_cdf};
use super::types::{
    ChangeDirection, ChangeType, Changepoint, ChangepointConfig, ChangepointError,
    SegmentComparison, SegmentComparisonConfig, SegmentStats,
};
use anyhow::{Result, anyhow};
use augurs_changepoint::{DefaultArgpcpDetector, Detector};
use std::collections::HashMap;
use tracing::{debug, instrument, warn};
use uuid::Uuid;

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
        // Check for NaN or infinite values first (data quality check)
        if values.iter().any(|v| !v.is_finite()) {
            return Err(anyhow!(ChangepointError::InvalidData {
                message: "Data contains NaN or infinite values".to_string(),
            }));
        }

        // Validate input size
        if values.len() < self.config.min_observations {
            return Err(anyhow!(ChangepointError::InsufficientData {
                have: values.len(),
                need: self.config.min_observations,
            }));
        }

        // Update adaptive baseline if enabled
        if self.config.adaptive_threshold {
            self.update_baseline("default", values);
        }

        // Run changepoint detection using ARPCP algorithm
        let changepoint_indices = self.detector.detect_changepoints(values);

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
        super::algorithms::calculate_changepoint_probability(
            &self.config,
            values,
            cp_index,
            detection_index,
        )
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
            let cp_index = cp.index;
            if cp_index >= last_cp_index + self.config.min_distance {
                filtered.push(cp);
                last_cp_index = cp_index;
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
