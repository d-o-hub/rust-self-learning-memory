//! Forecasting module with ETS (Error, Trend, Seasonality) models

pub mod engine;
pub mod ets_fitting;
pub mod ets_forecasting;
pub mod ets_types;
pub mod types;

// Re-export main types
pub use engine::ForecastingEngine;
pub use ets_types::{
    ETSErrorType, ETSForecastResult, ETSModel, ETSModelSpec, ETSSeasonalType, ETSState,
    ETSTrendType, SeasonalityResult,
};
pub use types::{ForecastResult, PredictiveConfig, PredictiveMetadata, PredictiveResults};
