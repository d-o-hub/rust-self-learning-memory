use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityResult {
    pub period: usize,
    pub strength: f64,
}

/// ETS model specification for testing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ETSModelSpec {
    pub error: ETSErrorType,
    pub trend: ETSTrendType,
    pub seasonal: ETSSeasonalType,
}

/// Complete ETS model specification
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ETSState {
    pub level: f64,
    pub trend: f64,
    pub seasonal: Vec<f64>,
    pub last_observation: f64,
    pub n_obs: usize,
}

/// Forecasting results with ETS metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
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
