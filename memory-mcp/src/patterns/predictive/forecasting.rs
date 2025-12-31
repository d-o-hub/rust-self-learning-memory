//! # Forecasting Module
//!
//! ETS (Error, Trend, Seasonal) forecasting models for time series prediction.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use super::types::{
    ETSErrorType, ETSForecastResult, ETSModel, ETSModelSpec, ETSSeasonalType, ETSState,
    ETSTrendType, ForecastResult, PredictiveConfig, SeasonalityResult,
};

#[derive(Debug)]
pub struct ForecastingEngine {
    config: PredictiveConfig,
}

impl ForecastingEngine {
    pub fn new() -> Result<Self> {
        Self::with_config(PredictiveConfig::default())
    }
    pub fn with_config(config: PredictiveConfig) -> Result<Self> {
        Ok(Self { config })
    }

    #[instrument(skip(self, data))]
    pub fn forecast(&mut self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<ForecastResult>> {
        let mut results = Vec::new();
        info!("Generating forecasts for {} variables", data.len());
        for (var_name, series) in data {
            if series.len() < 5 {
                warn!("Skipping forecast for {}: insufficient data points", var_name);
                continue;
            }
            let forecast_result = self.forecast_variable(var_name, series)?;
            results.push(forecast_result);
        }
        debug!("Generated {} forecasts", results.len());
        Ok(results)
    }

    fn forecast_variable(&mut self, variable: &str, series: &[f64]) -> Result<ForecastResult> {
        let data = if series.len() > self.config.reservoir_size {
            series.iter().take(self.config.reservoir_size).copied().collect()
        } else {
            series.to_vec()
        };
        if data.len() < 2 {
            return Err(anyhow!("Insufficient data for ETS forecasting"));
        }
        let seasonality = self.detect_seasonality(&data)?;
        let period = seasonality.period;
        let best_result = self.select_and_fit_ets_model(&data, period)?;
        let forecasts = self.forecast_ets(&best_result.model, &best_result.state, self.config.forecast_horizon)?;
        let (lower_bounds, upper_bounds) = self.calculate_confidence_intervals(&forecasts, &best_result.state, self.config.forecast_horizon)?;
        Ok(ForecastResult {
            variable: variable.to_string(),
            point_forecasts: forecasts,
            lower_bounds,
            upper_bounds,
            fit_quality: best_result.fit_quality,
            method: format!("ETS-{}{}{}", best_result.model.error.as_str(), best_result.model.trend.as_str(), best_result.model.seasonal.as_str()),
        })
    }

    #[allow(dead_code)]
    fn calculate_fit_quality(&self, actual: &[f64], forecast: &[f64]) -> f64 {
        if actual.len() < 2 || forecast.is_empty() { return 0.0; }
        let n = actual.len().min(forecast.len().min(10));
        let start_idx = actual.len().saturating_sub(n);
        let mape: f64 = actual[start_idx..].iter().zip(&forecast[..n]).map(|(&a, &f)| {
            if a != 0.0 { (a - f).abs() / a.abs() } else { 0.0 }
        }).sum::<f64>() / n as f64;
        (1.0 - mape.min(1.0)).max(0.0)
    }

    fn detect_seasonality(&self, series: &[f64]) -> Result<SeasonalityResult> {
        if series.len() < 10 {
            return Ok(SeasonalityResult { period: 0, strength: 0.0 });
        }
        let max_period = (series.len() / 2).min(12);
        let mut strengths: Vec<(usize, f64)> = Vec::new();
        for period in 2..=max_period {
            if let Some(strength) = self.calculate_seasonal_strength(series, period) {
                strengths.push((period, strength));
            }
        }
        let Some((_, max_strength)) = strengths.iter().cloned().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)) else {
            return Ok(SeasonalityResult { period: 0, strength: 0.0 });
        };
        let tolerance = 0.02;
        let mut candidates: Vec<(usize, f64)> = strengths.into_iter().filter(|(_, s)| *s >= max_strength - tolerance).collect();
        candidates.sort_by(|a, b| a.0.cmp(&b.0));
        let (best_period, best_strength) = if let Some((p, s)) = candidates.iter().find(|(p, _)| (3..=5).contains(p)).copied() {
            (p, s)
        } else {
            candidates[0]
        };
        Ok(SeasonalityResult { period: if best_strength > 0.1 { best_period } else { 0 }, strength: best_strength })
    }

    fn calculate_seasonal_strength(&self, series: &[f64], period: usize) -> Option<f64> {
        if series.len() < period * 2 { return None; }
        let mut seasonal_means = vec![0.0f64; period];
        let mut counts = vec![0usize; period];
        for (i, &value) in series.iter().enumerate() {
            seasonal_means[i % period] += value;
            counts[i % period] += 1;
        }
        for (i, &count) in counts.iter().enumerate() {
            if count > 0 { seasonal_means[i] /= count as f64; }
        }
        let overall_mean: f64 = series.iter().sum::<f64>() / series.len() as f64;
        let variance: f64 = series.iter().map(|&x| (x - overall_mean).powi(2)).sum::<f64>() / series.len() as f64;
        let seasonal_variance: f64 = seasonal_means.iter().enumerate().map(|(i, &mean)| {
            let count = counts[i] as f64;
            count * (mean - overall_mean).powi(2)
        }).sum::<f64>() / series.len() as f64;
        if variance > 0.0 { Some((seasonal_variance / variance).sqrt()) } else { Some(0.0) }
    }

    fn select_and_fit_ets_model(&self, series: &[f64], period: usize) -> Result<ETSForecastResult> {
        if series.len() < 2 { return Err(anyhow!("ETS requires at least 2 observations")); }
        let models_to_try = self.generate_model_combinations(period);
        let mut best_result = None;
        let mut best_aic = f64::INFINITY;
        for model_spec in models_to_try {
            if let Ok(result) = self.fit_ets_model(series, &model_spec) {
                if result.aic < best_aic {
                    best_aic = result.aic;
                    best_result = Some(result);
                }
            }
        }
        best_result.ok_or_else(|| anyhow!("Failed to fit any ETS model"))
    }

    fn generate_model_combinations(&self, period: usize) -> Vec<ETSModelSpec> {
        let mut models = Vec::new();
        let error_types = [ETSErrorType::Additive, ETSErrorType::Multiplicative];
        let trend_types = [ETSTrendType::None, ETSTrendType::Additive, ETSTrendType::AdditiveDamped];
        let seasonal_types = if period > 0 {
            vec![ETSSeasonalType::None, ETSSeasonalType::Additive, ETSSeasonalType::Multiplicative]
        } else {
            vec![ETSSeasonalType::None]
        };
        for error in &error_types {
            for trend in &trend_types {
                for seasonal in &seasonal_types {
                    models.push(ETSModelSpec { error: *error, trend: *trend, seasonal: *seasonal });
                }
            }
        }
        models
    }

    fn fit_ets_model(&self, series: &[f64], model_spec: &ETSModelSpec) -> Result<ETSForecastResult> {
        let mut model = self.initialize_parameters(series, model_spec)?;
        let mut state = self.initialize_state(series, &model)?;
        model = self.optimize_parameters(series, model_spec, &state)?;
        state = self.refit_with_parameters(series, &model)?;
        let (log_likelihood, aic) = self.calculate_model_metrics(series, &model, &state)?;
        let fit_quality = self.calculate_ets_fit_quality(series, &model, &state)?;
        Ok(ETSForecastResult { model, state, forecasts: Vec::new(), lower_bounds: Vec::new(), upper_bounds: Vec::new(), fit_quality, aic, log_likelihood })
    }

    fn initialize_parameters(&self, series: &[f64], model_spec: &ETSModelSpec) -> Result<ETSModel> {
        let n = series.len();
        let alpha = 0.2;
        let beta = if matches!(model_spec.trend, ETSTrendType::None) { 0.0 } else { 0.1 };
        let gamma = if matches!(model_spec.seasonal, ETSSeasonalType::None) { 0.0 } else { 0.1 };
        let phi = 0.98;
        let initial_level = series[0];
        let initial_trend = if n > 1 { (series[n - 1] - series[0]) / (n - 1) as f64 } else { 0.0 };
        let mut initial_seasonal = Vec::new();
        if !matches!(model_spec.seasonal, ETSSeasonalType::None) {
            let period = self.estimate_period(series);
            if period > 0 {
                for i in 0..period {
                    let indices: Vec<usize> = (i..n).step_by(period).collect();
                    if !indices.is_empty() {
                        let seasonal_mean: f64 = indices.iter().map(|&idx| series[idx]).sum::<f64>() / indices.len() as f64;
                        initial_seasonal.push(seasonal_mean - initial_level);
                    } else {
                        initial_seasonal.push(0.0);
                    }
                }
            } else {
                initial_seasonal = vec![0.0];
            }
        }
        Ok(ETSModel { error: model_spec.error, trend: model_spec.trend, seasonal: model_spec.seasonal, alpha, beta, gamma, phi, initial_level, initial_trend, initial_seasonal })
    }

    fn initialize_state(&self, series: &[f64], model: &ETSModel) -> Result<ETSState> {
        let n = series.len();
        let level = model.initial_level;
        let trend = model.initial_trend;
        let mut seasonal = model.initial_seasonal.clone();
        if seasonal.is_empty() { seasonal = vec![0.0]; }
        Ok(ETSState { level, trend, seasonal, last_observation: if n > 0 { series[n - 1] } else { 0.0 }, n_obs: n })
    }

    fn optimize_parameters(&self, series: &[f64], model_spec: &ETSModelSpec, _initial_state: &ETSState) -> Result<ETSModel> {
        let mut best_model = self.initialize_parameters(series, model_spec)?;
        let mut best_log_likelihood = f64::NEG_INFINITY;
        let alpha_values = [0.1, 0.2, 0.3, 0.5, 0.7, 0.9];
        let beta_values = if matches!(model_spec.trend, ETSTrendType::None) { vec![0.0] } else { vec![0.1, 0.2, 0.3, 0.5] };
        let gamma_values = if matches!(model_spec.seasonal, ETSSeasonalType::None) { vec![0.0] } else { vec![0.0, 0.1, 0.2, 0.3, 0.5] };
        for &alpha in &alpha_values {
            for &beta in &beta_values {
                for &gamma in &gamma_values {
                    let mut test_model = best_model.clone();
                    test_model.alpha = alpha;
                    test_model.beta = beta;
                    test_model.gamma = gamma;
                    if let Ok(test_state) = self.refit_with_parameters(series, &test_model) {
                        if let Ok((log_likelihood, _)) = self.calculate_model_metrics(series, &test_model, &test_state) {
                            if log_likelihood > best_log_likelihood {
                                best_log_likelihood = log_likelihood;
                                best_model = test_model;
                            }
                        }
                    }
                }
            }
        }
        Ok(best_model)
    }

    fn refit_with_parameters(&self, series: &[f64], model: &ETSModel) -> Result<ETSState> {
        let mut state = self.initialize_state(series, model)?;
        for &observation in series.iter().skip(1) {
            state = self.update_ets_state(&state, observation, model)?;
        }
        Ok(state)
    }

    fn update_ets_state(&self, current_state: &ETSState, new_observation: f64, model: &ETSModel) -> Result<ETSState> {
        let mut new_state = current_state.clone();
        let fitted = self.calculate_fitted_value(current_state, model);
        let residual = match model.error {
            ETSErrorType::Additive => new_observation - fitted,
            ETSErrorType::Multiplicative => if fitted != 0.0 { new_observation / fitted } else { 0.0 },
        };
        new_state.level = model.alpha * residual * self.get_error_multiplier(model) + (1.0 - model.alpha) * (current_state.level + current_state.trend);
        new_state.trend = model.beta * (new_state.level - current_state.level) + (1.0 - model.beta) * self.get_damped_trend(current_state.trend, model.phi);
        if !new_state.seasonal.is_empty() && !matches!(model.seasonal, ETSSeasonalType::None) {
            let seasonal_index = (new_state.n_obs + 1) % new_state.seasonal.len();
            let seasonal_factor = match model.seasonal {
                ETSSeasonalType::Additive => residual * self.get_error_multiplier(model),
                ETSSeasonalType::Multiplicative => residual,
                ETSSeasonalType::None => 0.0,
            };
            new_state.seasonal[seasonal_index] = model.gamma * seasonal_factor + (1.0 - model.gamma) * current_state.seasonal[seasonal_index];
        }
        new_state.last_observation = new_observation;
        new_state.n_obs += 1;
        Ok(new_state)
    }

    fn calculate_fitted_value(&self, state: &ETSState, model: &ETSModel) -> f64 {
        let trend_component = match model.trend {
            ETSTrendType::None => 0.0,
            ETSTrendType::Additive => state.trend,
            ETSTrendType::AdditiveDamped => state.trend * model.phi,
        };
        let seasonal_component = if !state.seasonal.is_empty() && !matches!(model.seasonal, ETSSeasonalType::None) {
            let seasonal_index = state.n_obs % state.seasonal.len();
            match model.seasonal {
                ETSSeasonalType::Additive => state.seasonal[seasonal_index],
                ETSSeasonalType::Multiplicative => 1.0 + state.seasonal[seasonal_index],
                ETSSeasonalType::None => 0.0,
            }
        } else {
            1.0
        };
        match (model.error, model.seasonal) {
            (ETSErrorType::Additive, ETSSeasonalType::Additive) => state.level + trend_component + seasonal_component,
            (ETSErrorType::Additive, ETSSeasonalType::Multiplicative) => (state.level + trend_component) * seasonal_component,
            (ETSErrorType::Multiplicative, ETSSeasonalType::Additive) => (state.level + trend_component) + seasonal_component,
            (ETSErrorType::Multiplicative, ETSSeasonalType::Multiplicative) => (state.level + trend_component) * seasonal_component,
            _ => state.level + trend_component,
        }
    }

    fn forecast_ets(&self, model: &ETSModel, state: &ETSState, horizon: usize) -> Result<Vec<f64>> {
        let mut forecasts = Vec::with_capacity(horizon);
        for h in 1..=horizon {
            let trend_component = match model.trend {
                ETSTrendType::None => 0.0,
                ETSTrendType::Additive => state.trend * h as f64,
                ETSTrendType::AdditiveDamped => state.trend * model.phi.powi(h as i32 - 1) * (1.0 - model.phi.powi(h as i32)) / (1.0 - model.phi),
            };
            let seasonal_component = if !state.seasonal.is_empty() && !matches!(model.seasonal, ETSSeasonalType::None) {
                let seasonal_index = (state.n_obs + h) % state.seasonal.len();
                match model.seasonal {
                    ETSSeasonalType::Additive => state.seasonal[seasonal_index],
                    ETSSeasonalType::Multiplicative => 1.0 + state.seasonal[seasonal_index],
                    ETSSeasonalType::None => 0.0,
                }
            } else {
                1.0
            };
            let forecast = match (model.error, model.seasonal) {
                (ETSErrorType::Additive, ETSSeasonalType::Additive) => state.level + trend_component + seasonal_component,
                (ETSErrorType::Additive, ETSSeasonalType::Multiplicative) => (state.level + trend_component) * seasonal_component,
                (ETSErrorType::Multiplicative, ETSSeasonalType::Additive) => (state.level + trend_component) + seasonal_component,
                (ETSErrorType::Multiplicative, ETSSeasonalType::Multiplicative) => (state.level + trend_component) * seasonal_component,
                _ => state.level + trend_component,
            };
            forecasts.push(forecast);
        }
        Ok(forecasts)
    }

    fn calculate_confidence_intervals(&self, forecasts: &[f64], state: &ETSState, horizon: usize) -> Result<(Vec<f64>, Vec<f64>)> {
        let residual_variance = self.estimate_residual_variance(state);
        let z_score = 1.96;
        let mut lower_bounds = Vec::with_capacity(horizon);
        let mut upper_bounds = Vec::with_capacity(horizon);
        for (h, &forecast) in forecasts.iter().enumerate() {
            let uncertainty = z_score * (residual_variance * (h as f64 + 1.0)).sqrt();
            lower_bounds.push(forecast - uncertainty);
            upper_bounds.push(forecast + uncertainty);
        }
        Ok((lower_bounds, upper_bounds))
    }

    fn estimate_residual_variance(&self, _state: &ETSState) -> f64 { 1.0 }

    fn calculate_ets_fit_quality(&self, series: &[f64], model: &ETSModel, _state: &ETSState) -> Result<f64> {
        if series.len() < 2 { return Ok(0.0); }
        let mut squared_errors = Vec::new();
        let mut current_state = self.initialize_state(series, model)?;
        for &observation in series.iter().skip(1) {
            let fitted = self.calculate_fitted_value(&current_state, model);
            let error = (observation - fitted).powi(2);
            squared_errors.push(error);
            current_state = self.update_ets_state(&current_state, observation, model)?;
        }
        let mse: f64 = squared_errors.iter().sum::<f64>() / squared_errors.len() as f64;
        let variance: f64 = series.iter().map(|&x| (x - series.iter().sum::<f64>() / series.len() as f64).powi(2)).sum::<f64>() / series.len() as f64;
        if variance > 0.0 { Ok((1.0 - mse / variance).clamp(0.0, 1.0)) } else { Ok(0.0) }
    }

    fn calculate_model_metrics(&self, series: &[f64], model: &ETSModel, _state: &ETSState) -> Result<(f64, f64)> {
        let mut log_likelihood = 0.0;
        let mut current_state = self.initialize_state(series, model)?;
        for &observation in series.iter().skip(1) {
            let fitted = self.calculate_fitted_value(&current_state, model);
            let residual = observation - fitted;
            log_likelihood += -0.5 * (residual.powi(2) + (2.0 * std::f64::consts::PI).ln());
            current_state = self.update_ets_state(&current_state, observation, model)?;
        }
        let n_params = 3 + match model.trend { ETSTrendType::None => 0, _ => 1 } + match model.seasonal { ETSSeasonalType::None => 0, _ => self.estimate_period(series) };
        let aic = 2.0 * n_params as f64 - 2.0 * log_likelihood;
        Ok((log_likelihood, aic))
    }

    fn estimate_period(&self, series: &[f64]) -> usize {
        let max_period = (series.len() / 2).min(12);
        let mut best_period = 0;
        let mut best_autocorr = 0.0;
        for period in 2..=max_period {
            if let Some(autocorr) = self.calculate_autocorrelation(series, period) {
                if autocorr.abs() > best_autocorr {
                    best_autocorr = autocorr.abs();
                    best_period = period;
                }
            }
        }
        best_period
    }

    fn calculate_autocorrelation(&self, series: &[f64], lag: usize) -> Option<f64> {
        if lag >= series.len() { return None; }
        let n = series.len() - lag;
        if n < 2 { return None; }
        let mean: f64 = series.iter().sum::<f64>() / series.len() as f64;
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        for i in 0..n {
            numerator += (series[i] - mean) * (series[i + lag] - mean);
            denominator += (series[i] - mean).powi(2);
        }
        if denominator > 0.0 { Some(numerator / denominator) } else { Some(0.0) }
    }

    fn get_error_multiplier(&self, model: &ETSModel) -> f64 {
        match model.error { ETSErrorType::Additive => 1.0, ETSErrorType::Multiplicative => model.alpha }
    }

    fn get_damped_trend(&self, trend: f64, phi: f64) -> f64 { trend * phi }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn test_forecast_generation() -> Result<()> {
        let mut engine = ForecastingEngine::new()?;
        let mut data = HashMap::new();
        data.insert("trend".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
        let forecasts = engine.forecast(&data)?;
        assert!(!forecasts.is_empty());
        let forecast = &forecasts[0];
        assert_eq!(forecast.variable, "trend");
        assert_eq!(forecast.point_forecasts.len(), 10);
        Ok(())
    }
    #[test]
    fn test_ets_seasonality_detection() -> Result<()> {
        let engine = ForecastingEngine::new()?;
        let seasonal_data: Vec<f64> = (0..20).map(|i| {
            let base = 10.0;
            let trend = i as f64 * 0.5;
            let seasonal = [0.0, 2.0, -1.0, 1.0][i % 4];
            base + trend + seasonal
        }).collect();
        let seasonality = engine.detect_seasonality(&seasonal_data)?;
        assert!(seasonality.strength > 0.05, "Should detect some seasonal strength");
        Ok(())
    }
    #[test]
    fn test_ets_additive_vs_multiplicative_selection() -> Result<()> {
        let engine = ForecastingEngine::new()?;
        let additive_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let result = engine.select_and_fit_ets_model(&additive_data, 0)?;
        assert!(result.aic.is_finite());
        assert!(result.fit_quality >= 0.0 && result.fit_quality <= 1.0);
        let multiplicative_data = vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0];
        let result2 = engine.select_and_fit_ets_model(&multiplicative_data, 0)?;
        assert!(result2.aic.is_finite());
        assert!(result2.fit_quality >= 0.0 && result2.fit_quality <= 1.0);
        Ok(())
    }
    #[test]
    fn test_ets_confidence_intervals() -> Result<()> {
        let engine = ForecastingEngine::new()?;
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let result = engine.select_and_fit_ets_model(&data, 0)?;
        let forecasts = engine.forecast_ets(&result.model, &result.state, 5)?;
        let (lower_bounds, upper_bounds) = engine.calculate_confidence_intervals(&forecasts, &result.state, 5)?;
        assert_eq!(forecasts.len(), 5);
        assert_eq!(lower_bounds.len(), 5);
        assert_eq!(upper_bounds.len(), 5);
        for i in 0..5 {
            assert!(lower_bounds[i] <= forecasts[i]);
            assert!(upper_bounds[i] >= forecasts[i]);
        }
        Ok(())
    }
    #[test]
    fn test_ets_parameter_estimation_convergence() -> Result<()> {
        let engine = ForecastingEngine::new()?;
        let data = vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5];
        let result = engine.fit_ets_model(&data, &ETSModelSpec { error: ETSErrorType::Additive, trend: ETSTrendType::Additive, seasonal: ETSSeasonalType::None })?;
        assert!(result.model.alpha > 0.0 && result.model.alpha < 1.0);
        assert!(result.model.beta > 0.0 && result.model.beta < 1.0);
        assert!(result.model.gamma >= 0.0 && result.model.gamma < 1.0);
        assert!(result.aic.is_finite());
        assert!(result.log_likelihood.is_finite());
        Ok(())
    }
    #[test]
    fn test_ets_edge_cases() -> Result<()> {
        let engine = ForecastingEngine::new()?;
        let single_obs = vec![5.0];
        let result = engine.select_and_fit_ets_model(&single_obs, 0);
        assert!(result.is_err());
        let two_obs = vec![1.0, 2.0];
        let result = engine.select_and_fit_ets_model(&two_obs, 0);
        assert!(result.is_ok());
        let constant_data = vec![5.0; 10];
        let result = engine.select_and_fit_ets_model(&constant_data, 0)?;
        assert!(result.aic.is_finite());
        let trend_data: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let result = engine.select_and_fit_ets_model(&trend_data, 0)?;
        assert!(result.fit_quality >= 0.0);
        Ok(())
    }
    #[test]
    fn test_ets_incremental_updates() -> Result<()> {
        let engine = ForecastingEngine::new()?;
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let model = engine.initialize_parameters(&data, &ETSModelSpec { error: ETSErrorType::Additive, trend: ETSTrendType::Additive, seasonal: ETSSeasonalType::None })?;
        let mut state = engine.initialize_state(&data, &model)?;
        let new_observation = 6.0;
        state = engine.update_ets_state(&state, new_observation, &model)?;
        assert_eq!(state.n_obs, 6);
        assert_eq!(state.last_observation, 6.0);
        Ok(())
    }
    #[test]
    fn test_ets_model_types() {
        assert_eq!(ETSErrorType::Additive.as_str(), "A");
        assert_eq!(ETSErrorType::Multiplicative.as_str(), "M");
        assert_eq!(ETSTrendType::None.as_str(), "N");
        assert_eq!(ETSTrendType::Additive.as_str(), "A");
        assert_eq!(ETSTrendType::AdditiveDamped.as_str(), "Ad");
        assert_eq!(ETSSeasonalType::None.as_str(), "N");
        assert_eq!(ETSSeasonalType::Additive.as_str(), "A");
        assert_eq!(ETSSeasonalType::Multiplicative.as_str(), "M");
    }
}
