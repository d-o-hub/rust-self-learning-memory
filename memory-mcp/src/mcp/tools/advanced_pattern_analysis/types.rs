//! # Advanced Pattern Analysis Types
//!
//! Type definitions for advanced pattern analysis including input, output, config,
//! summary, and performance metrics.

use crate::patterns::{predictive, statistical};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input parameters for advanced pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedPatternAnalysisInput {
    /// Type of analysis to perform
    pub analysis_type: AnalysisType,
    /// Time series data for analysis (variable_name -> values)
    pub time_series_data: HashMap<String, Vec<f64>>,
    /// Analysis configuration
    pub config: Option<AnalysisConfig>,
}

/// Types of analysis available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    /// Statistical analysis (correlations, changepoints, trends)
    Statistical,
    /// Predictive analysis (forecasting, anomalies, causality)
    Predictive,
    /// Comprehensive analysis (all methods)
    Comprehensive,
}

/// Configuration for analysis parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Significance level for statistical tests (default: 0.05)
    pub significance_level: Option<f64>,
    /// Forecast horizon for predictive analysis (default: 10)
    pub forecast_horizon: Option<usize>,
    /// Anomaly detection sensitivity (default: 0.5)
    pub anomaly_sensitivity: Option<f64>,
    /// Enable causal inference (default: true)
    pub enable_causal_inference: Option<bool>,
    /// Maximum data points to analyze (default: 10,000)
    pub max_data_points: Option<usize>,
    /// Enable parallel processing (default: true)
    pub parallel_processing: Option<bool>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            significance_level: Some(0.05),
            forecast_horizon: Some(10),
            anomaly_sensitivity: Some(0.5),
            enable_causal_inference: Some(true),
            max_data_points: Some(10_000),
            parallel_processing: Some(true),
        }
    }
}

/// Output from advanced pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedPatternAnalysisOutput {
    /// Statistical analysis results (if performed)
    pub statistical_results: Option<statistical::StatisticalResults>,
    /// Predictive analysis results (if performed)
    pub predictive_results: Option<predictive::PredictiveResults>,
    /// Analysis summary
    pub summary: AnalysisSummary,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Summary of analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Number of variables analyzed
    pub variables_analyzed: usize,
    /// Key findings
    pub key_findings: Vec<String>,
    /// Recommendations based on analysis
    pub recommendations: Vec<String>,
    /// Confidence level of results
    pub confidence_level: f64,
}

/// Performance metrics for the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total analysis time in milliseconds
    pub total_time_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}
