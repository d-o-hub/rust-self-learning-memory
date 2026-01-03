use anyhow::Result;

use super::ets_types::{
    ETSErrorType, ETSForecastResult, ETSModel, ETSModelSpec, ETSSeasonalType, ETSState,
    ETSTrendType,
};

impl super::engine::ForecastingEngine {
    pub(super) fn generate_model_combinations(&self, period: usize) -> Vec<ETSModelSpec> {
        let mut models = Vec::new();

        let error_types = [ETSErrorType::Additive, ETSErrorType::Multiplicative];
        let trend_types = [
            ETSTrendType::None,
            ETSTrendType::Additive,
            ETSTrendType::AdditiveDamped,
        ];
        let seasonal_types = if period > 0 {
            vec![
                ETSSeasonalType::None,
                ETSSeasonalType::Additive,
                ETSSeasonalType::Multiplicative,
            ]
        } else {
            vec![ETSSeasonalType::None]
        };

        for error in &error_types {
            for trend in &trend_types {
                for seasonal in &seasonal_types {
                    models.push(ETSModelSpec {
                        error: *error,
                        trend: *trend,
                        seasonal: *seasonal,
                    });
                }
            }
        }

        models
    }

    /// Fit ETS model using Maximum Likelihood Estimation
    pub(super) fn fit_ets_model(
        &self,
        series: &[f64],
        model_spec: &ETSModelSpec,
    ) -> Result<ETSForecastResult> {
        // Initialize parameters
        let mut model = self.initialize_parameters(series, model_spec)?;
        let mut state = self.initialize_state(series, &model)?;

        // Optimize parameters using MLE
        model = self.optimize_parameters(series, model_spec, &state)?;

        // Refit with optimized parameters
        state = self.refit_with_parameters(series, &model)?;

        // Calculate fitted values
        let fitted: Vec<f64> = (0..series.len())
            .map(|i| {
                let obs_state = ETSState {
                    level: state.level,
                    trend: state.trend,
                    seasonal: state.seasonal.clone(),
                    last_observation: if i > 0 { series[i - 1] } else { series[0] },
                    n_obs: i,
                };
                self.calculate_fitted_value(&obs_state, &model)
            })
            .collect();

        // Calculate fit quality and model metrics
        let (_, rmse, _mape) = self.calculate_model_metrics(series, &fitted);
        let fit_quality = self.calculate_ets_fit_quality(series, &fitted, &model);

        // Calculate simplified log-likelihood and AIC
        let log_likelihood = -rmse * series.len() as f64; // Simplified
        let aic = series.len() as f64 * (rmse.ln() + 1.0) + 6.0; // Simplified AIC

        Ok(ETSForecastResult {
            model,
            state,
            forecasts: Vec::new(), // Will be filled by caller
            lower_bounds: Vec::new(),
            upper_bounds: Vec::new(),
            fit_quality,
            aic,
            log_likelihood,
        })
    }

    /// Initialize ETS parameters with heuristics
    fn initialize_parameters(&self, series: &[f64], model_spec: &ETSModelSpec) -> Result<ETSModel> {
        let n = series.len();

        // Simple heuristics for initial parameter values
        let alpha = 0.2;
        let beta = if matches!(model_spec.trend, ETSTrendType::None) {
            0.0
        } else {
            0.1
        };
        let gamma = if matches!(model_spec.seasonal, ETSSeasonalType::None) {
            0.0
        } else {
            0.1
        };
        let phi = 0.98;

        // Calculate initial level and trend
        let initial_level = series[0];
        let initial_trend = if n > 1 {
            (series[n - 1] - series[0]) / (n - 1) as f64
        } else {
            0.0
        };

        // Calculate initial seasonal components
        let mut initial_seasonal = Vec::new();
        if !matches!(model_spec.seasonal, ETSSeasonalType::None) {
            let period = self.estimate_period(series);
            if period > 0 {
                for i in 0..period {
                    let indices: Vec<usize> = (i..n).step_by(period).collect();
                    if !indices.is_empty() {
                        let seasonal_mean: f64 =
                            indices.iter().map(|&idx| series[idx]).sum::<f64>()
                                / indices.len() as f64;
                        initial_seasonal.push(seasonal_mean - initial_level);
                    } else {
                        initial_seasonal.push(0.0);
                    }
                }
            } else {
                initial_seasonal = vec![0.0];
            }
        }

        Ok(ETSModel {
            error: model_spec.error,
            trend: model_spec.trend,
            seasonal: model_spec.seasonal,
            alpha,
            beta,
            gamma,
            phi,
            initial_level,
            initial_trend,
            initial_seasonal,
        })
    }

    /// Initialize ETS state from data
    fn initialize_state(&self, series: &[f64], model: &ETSModel) -> Result<ETSState> {
        let n = series.len();
        let level = model.initial_level;
        let trend = model.initial_trend;

        let mut seasonal = model.initial_seasonal.clone();
        if seasonal.is_empty() {
            seasonal = vec![0.0];
        }

        Ok(ETSState {
            level,
            trend,
            seasonal,
            last_observation: if n > 0 { series[n - 1] } else { 0.0 },
            n_obs: n,
        })
    }

    /// Optimize ETS parameters using a simplified BFGS-like approach
    fn optimize_parameters(
        &self,
        series: &[f64],
        model_spec: &ETSModelSpec,
        _initial_state: &ETSState,
    ) -> Result<ETSModel> {
        // Simplified parameter optimization - in practice, use proper optimization library
        let mut best_model = self.initialize_parameters(series, model_spec)?;
        let mut best_log_likelihood = f64::NEG_INFINITY;

        // Grid search over reasonable parameter values
        let alpha_values = [0.1, 0.2, 0.3, 0.5, 0.7, 0.9];
        let beta_values = if matches!(model_spec.trend, ETSTrendType::None) {
            vec![0.0]
        } else {
            // Ensure beta stays strictly positive when trend is enabled.
            vec![0.1, 0.2, 0.3, 0.5]
        };
        let gamma_values = if matches!(model_spec.seasonal, ETSSeasonalType::None) {
            vec![0.0]
        } else {
            vec![0.0, 0.1, 0.2, 0.3, 0.5]
        };

        for &alpha in &alpha_values {
            for &beta in &beta_values {
                for &gamma in &gamma_values {
                    let mut test_model = best_model.clone();
                    test_model.alpha = alpha;
                    test_model.beta = beta;
                    test_model.gamma = gamma;

                    if let Ok(test_state) = self.refit_with_parameters(series, &test_model) {
                        // Calculate fitted values for this test model
                        let fitted: Vec<f64> = (0..series.len())
                            .map(|i| {
                                let obs_state = ETSState {
                                    level: test_state.level,
                                    trend: test_state.trend,
                                    seasonal: test_state.seasonal.clone(),
                                    last_observation: if i > 0 { series[i - 1] } else { series[0] },
                                    n_obs: i,
                                };
                                self.calculate_fitted_value(&obs_state, &test_model)
                            })
                            .collect();
                        let (_, rmse, _) = self.calculate_model_metrics(series, &fitted);
                        let log_likelihood = -rmse * series.len() as f64;

                        if log_likelihood > best_log_likelihood {
                            best_log_likelihood = log_likelihood;
                            best_model = test_model;
                        }
                    }
                }
            }
        }

        Ok(best_model)
    }

    /// Refit ETS model with given parameters
    fn refit_with_parameters(&self, series: &[f64], model: &ETSModel) -> Result<ETSState> {
        let mut state = self.initialize_state(series, model)?;

        for &observation in series.iter().skip(1) {
            state = self.update_ets_state(&state, observation, model)?;
        }

        Ok(state)
    }

    /// Update ETS state with new observation (for incremental updates)
    fn update_ets_state(
        &self,
        current_state: &ETSState,
        new_observation: f64,
        model: &ETSModel,
    ) -> Result<ETSState> {
        let mut new_state = current_state.clone();

        // Calculate fitted value
        let fitted = self.calculate_fitted_value(current_state, model);

        // Calculate residual
        let residual = match model.error {
            ETSErrorType::Additive => new_observation - fitted,
            ETSErrorType::Multiplicative => {
                if fitted != 0.0 {
                    new_observation / fitted
                } else {
                    0.0
                }
            }
        };

        // Update components
        new_state.level = model.alpha * residual * self.get_error_multiplier(model)
            + (1.0 - model.alpha) * (current_state.level + current_state.trend);

        new_state.trend = model.beta * (new_state.level - current_state.level)
            + (1.0 - model.beta) * self.get_damped_trend(current_state.trend, model.phi);

        if !new_state.seasonal.is_empty() && !matches!(model.seasonal, ETSSeasonalType::None) {
            let seasonal_index = (new_state.n_obs + 1) % new_state.seasonal.len();
            let seasonal_factor = match model.seasonal {
                ETSSeasonalType::Additive => residual * self.get_error_multiplier(model),
                ETSSeasonalType::Multiplicative => residual,
                ETSSeasonalType::None => 0.0,
            };

            new_state.seasonal[seasonal_index] = model.gamma * seasonal_factor
                + (1.0 - model.gamma) * current_state.seasonal[seasonal_index];
        }

        new_state.last_observation = new_observation;
        new_state.n_obs += 1;

        Ok(new_state)
    }
}

impl super::engine::ForecastingEngine {
    pub(super) fn estimate_period(&self, series: &[f64]) -> usize {
        if series.len() < 20 {
            return 0;
        }

        let max_period = series.len() / 4;
        let mut best_period = 0;
        let mut best_acf = 0.0;

        for period in 2..=max_period.min(24) {
            if let Some(acf) = self.calculate_autocorrelation(series, period) {
                if acf.abs() > best_acf {
                    best_acf = acf.abs();
                    best_period = period;
                }
            }
        }
        best_period
    }

    pub(super) fn calculate_autocorrelation(&self, series: &[f64], lag: usize) -> Option<f64> {
        if series.len() <= lag {
            return None;
        }

        let n = series.len() - lag;
        let mean: f64 = series.iter().sum::<f64>() / series.len() as f64;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..n {
            numerator += (series[i] - mean) * (series[i + lag] - mean);
            denominator += (series[i] - mean).powi(2);
        }

        if denominator > 0.0 {
            Some(numerator / denominator)
        } else {
            Some(0.0)
        }
    }

    pub(super) fn get_error_multiplier(&self, model: &ETSModel) -> f64 {
        match model.error {
            ETSErrorType::Additive => 1.0,
            ETSErrorType::Multiplicative => model.alpha,
        }
    }

    pub(super) fn get_damped_trend(&self, trend: f64, phi: f64) -> f64 {
        trend * phi
    }

    pub(super) fn calculate_fitted_value(&self, state: &ETSState, model: &ETSModel) -> f64 {
        let trend_component = match model.trend {
            ETSTrendType::None => 0.0,
            ETSTrendType::Additive => state.trend,
            ETSTrendType::AdditiveDamped => state.trend * model.phi,
        };

        let seasonal_component =
            if !state.seasonal.is_empty() && !matches!(model.seasonal, ETSSeasonalType::None) {
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
            (ETSErrorType::Additive, ETSSeasonalType::Additive) => {
                state.level + trend_component + seasonal_component
            }
            (ETSErrorType::Additive, ETSSeasonalType::Multiplicative) => {
                (state.level + trend_component) * seasonal_component
            }
            (ETSErrorType::Multiplicative, ETSSeasonalType::Additive) => {
                (state.level + trend_component) + seasonal_component
            }
            (ETSErrorType::Multiplicative, ETSSeasonalType::Multiplicative) => {
                (state.level + trend_component) * seasonal_component
            }
            _ => state.level + trend_component,
        }
    }
}
