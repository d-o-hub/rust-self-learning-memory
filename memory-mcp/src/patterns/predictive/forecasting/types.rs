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

/// Comprehensive predictive analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveResults {
    /// Forecasting results
    pub forecasts: Vec<ForecastResult>,
    /// Anomaly detection results
    pub anomalies: Vec<super::super::anomaly::AnomalyResult>,
    /// Causal inference results
    pub causal_relationships: Vec<super::super::causal::CausalResult>,
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
