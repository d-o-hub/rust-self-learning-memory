use anyhow::Result;
use tracing::debug;

use super::ets_types::{ETSErrorType, ETSModel, ETSSeasonalType, ETSState, ETSTrendType};

impl super::engine::ForecastingEngine {
    /// Main ETS forecasting function
    pub(super) fn forecast_ets(
        &self,
        model: &ETSModel,
        state: &ETSState,
        horizon: usize,
    ) -> Result<Vec<f64>> {
        let mut forecasts = Vec::with_capacity(horizon);

        for h in 1..=horizon {
            // Forecast h steps ahead
            let trend_component = match model.trend {
                ETSTrendType::None => 0.0,
                ETSTrendType::Additive => state.trend * h as f64,
                ETSTrendType::AdditiveDamped => {
                    let mut phi_sum = 0.0;
                    for i in 1..=h {
                        phi_sum += model.phi.powi(i as i32);
                    }
                    state.trend * phi_sum
                }
            };

            let seasonal_component =
                if !state.seasonal.is_empty() && !matches!(model.seasonal, ETSSeasonalType::None) {
                    let seasonal_index = (state.n_obs + h - 1) % state.seasonal.len();
                    state.seasonal[seasonal_index]
                } else {
                    0.0
                };

            let forecast = match (model.error, model.seasonal) {
                (ETSErrorType::Additive, ETSSeasonalType::Additive) => {
                    state.level + trend_component + seasonal_component
                }
                (ETSErrorType::Additive, ETSSeasonalType::Multiplicative) => {
                    (state.level + trend_component) * (1.0 + seasonal_component)
                }
                (ETSErrorType::Multiplicative, ETSSeasonalType::Additive) => {
                    (state.level + trend_component) + seasonal_component
                }
                (ETSErrorType::Multiplicative, ETSSeasonalType::Multiplicative) => {
                    (state.level + trend_component) * (1.0 + seasonal_component)
                }
                _ => state.level + trend_component,
            };

            forecasts.push(forecast);
        }

        debug!("Generated {} forecasts", forecasts.len());
        Ok(forecasts)
    }

    pub(super) fn calculate_confidence_intervals(
        &self,
        _model: &ETSModel,
        forecasts: &[f64],
        _state: &ETSState,
        _confidence: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let residual_std = self.estimate_residual_variance(_state).sqrt();
        let z_score = 1.96; // 95% confidence

        let lower_bounds: Vec<f64> = forecasts
            .iter()
            .enumerate()
            .map(|(i, &f)| f - z_score * residual_std * ((i + 1) as f64).sqrt())
            .collect();

        let upper_bounds: Vec<f64> = forecasts
            .iter()
            .enumerate()
            .map(|(i, &f)| f + z_score * residual_std * ((i + 1) as f64).sqrt())
            .collect();

        (lower_bounds, upper_bounds)
    }

    pub(super) fn estimate_residual_variance(&self, _state: &ETSState) -> f64 {
        // Simplified - would use actual residuals in production
        0.1
    }

    pub(super) fn calculate_ets_fit_quality(
        &self,
        _actual: &[f64],
        _fitted: &[f64],
        _model: &ETSModel,
    ) -> f64 {
        // Calculate AIC or similar metric
        let n = _actual.len() as f64;
        let k = 3.0; // Number of parameters (simplified)

        // SSE
        let sse: f64 = _actual
            .iter()
            .zip(_fitted.iter())
            .map(|(a, f)| (a - f).powi(2))
            .sum();

        // AIC = n * ln(SSE/n) + 2k
        let aic = n * (sse / n).ln() + 2.0 * k;
        -aic // Return negative so higher is better
    }

    pub(super) fn calculate_model_metrics(
        &self,
        _actual: &[f64],
        _fitted: &[f64],
    ) -> (f64, f64, f64) {
        let n = _actual.len() as f64;

        // MAE
        let mae: f64 = _actual
            .iter()
            .zip(_fitted.iter())
            .map(|(a, f)| (a - f).abs())
            .sum::<f64>()
            / n;

        // RMSE
        let mse: f64 = _actual
            .iter()
            .zip(_fitted.iter())
            .map(|(a, f)| (a - f).powi(2))
            .sum::<f64>()
            / n;
        let rmse = mse.sqrt();

        // MAPE
        let mape: f64 = _actual
            .iter()
            .zip(_fitted.iter())
            .filter(|(a, _)| a.abs() > 1e-10)
            .map(|(a, f)| ((a - f).abs() / a.abs()) * 100.0)
            .sum::<f64>()
            / n;

        (mae, rmse, mape)
    }
}
