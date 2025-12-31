//! # Shared Types for Predictive Analysis
//!
//! Common types used across forecasting, anomaly detection, and causal inference.

use serde::{Deserialize, Serialize};

/// Configuration for predictive analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveConfig {
    /// Forecast horizon (default: 10)
    pub forecast_horizon: usize,
    /// Anomaly detection sensitivity (0.0 to 1.0, default: 0.5)
    pub anomaly_sensitivity: f64,
    /// Enable causal inference (default: true)
    pub enable_causal_inference: bool,
    /// Reservoir sampling size for large datasets (default: 1000)
    pub reservoir_size: usize,
}

impl Default for PredictiveConfig {
    fn default() -> Self {
        Self {
            forecast_horizon: 10,
            anomaly_sensitivity: 0.5,
            enable_causal_inference: true,
            reservoir_size: 1000,
        }
    }
}

/// Forecasting results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    /// Variable name
    pub variable: String,
    /// Point forecasts
    pub point_forecasts: Vec<f64>,
    /// Lower confidence bounds
    pub lower_bounds: Vec<f64>,
    /// Upper confidence bounds
    pub upper_bounds: Vec<f64>,
    /// Model fit quality (0.0 to 1.0)
    pub fit_quality: f64,
    /// Forecast method used
    pub method: String,
}

/// Anomaly detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Variable name
    pub variable: String,
    /// Indices of detected anomalies
    pub anomaly_indices: Vec<usize>,
    /// Anomaly scores for each point
    pub anomaly_scores: Vec<f64>,
    /// Detection method used
    pub method: String,
    /// Detection confidence
    pub confidence: f64,
}

/// Causal inference results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalResult {
    /// Cause variable
    pub cause: String,
    /// Effect variable
    pub effect: String,
    /// Causal strength (0.0 to 1.0)
    pub strength: f64,
    /// Statistical significance
    pub significant: bool,
    /// Causal relationship type
    pub relationship_type: CausalType,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
}

/// Types of causal relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CausalType {
    /// Direct causation
    Direct,
    /// Indirect causation through mediators
    Indirect,
    /// Spurious correlation
    Spurious,
    /// No causal relationship
    None,
}

/// Comprehensive predictive analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveResults {
    /// Forecasting results
    pub forecasts: Vec<ForecastResult>,
    /// Anomaly detection results
    pub anomalies: Vec<AnomalyResult>,
    /// Causal inference results
    pub causal_relationships: Vec<CausalResult>,
    /// Analysis metadata
    pub metadata: PredictiveMetadata,
}

/// Predictive analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveMetadata {
    /// Number of variables analyzed
    pub variables_analyzed: usize,
    /// Analysis duration in milliseconds
    pub duration_ms: u64,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// Methods used
    pub methods_used: Vec<String>,
}

/// ETS model types for different variations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ETSErrorType {
    Additive,
    Multiplicative,
}

impl ETSErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ETSErrorType::Additive => "A",
            ETSErrorType::Multiplicative => "M",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ETSTrendType {
    None,
    Additive,
    AdditiveDamped,
}

impl ETSTrendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ETSTrendType::None => "N",
            ETSTrendType::Additive => "A",
            ETSTrendType::AdditiveDamped => "Ad",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ETSSeasonalType {
    None,
    Additive,
    Multiplicative,
}

impl ETSSeasonalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ETSSeasonalType::None => "N",
            ETSSeasonalType::Additive => "A",
            ETSSeasonalType::Multiplicative => "M",
        }
    }
}

/// Seasonality detection result
#[derive(Debug, Clone)]
pub struct SeasonalityResult {
    pub period: usize,
    pub strength: f64,
}

/// ETS model specification for testing
#[derive(Debug, Clone, Copy)]
pub struct ETSModelSpec {
    pub error: ETSErrorType,
    pub trend: ETSTrendType,
    pub seasonal: ETSSeasonalType,
}

/// Complete ETS model specification
#[derive(Debug, Clone)]
pub struct ETSModel {
    pub error: ETSErrorType,
    pub trend: ETSTrendType,
    pub seasonal: ETSSeasonalType,
    pub alpha: f64, // Level smoothing
    pub beta: f64,  // Trend smoothing
    pub gamma: f64, // Seasonal smoothing
    pub phi: f64,   // Damping parameter
    pub initial_level: f64,
    pub initial_trend: f64,
    pub initial_seasonal: Vec<f64>,
}

/// ETS model state for forecasting
#[derive(Debug, Clone)]
pub struct ETSState {
    pub level: f64,
    pub trend: f64,
    pub seasonal: Vec<f64>,
    pub last_observation: f64,
    pub n_obs: usize,
}

/// Forecasting results with ETS metadata
#[derive(Debug, Clone)]
pub struct ETSForecastResult {
    pub model: ETSModel,
    pub state: ETSState,
    pub forecasts: Vec<f64>,
    pub lower_bounds: Vec<f64>,
    pub upper_bounds: Vec<f64>,
    pub fit_quality: f64,
    pub aic: f64,
    pub log_likelihood: f64,
}
