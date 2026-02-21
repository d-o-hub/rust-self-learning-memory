use anyhow::{Result, anyhow};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use super::ets_types::{ETSForecastResult, SeasonalityResult};
use super::types::{ForecastResult, PredictiveConfig};

pub struct ForecastingEngine {
    config: PredictiveConfig,
}

impl ForecastingEngine {
    /// Create a new forecasting engine
    pub fn new() -> Result<Self> {
        Self::with_config(PredictiveConfig::default())
    }

    /// Create a new forecasting engine with custom config
    pub fn with_config(config: PredictiveConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Generate forecasts for time series data
    #[instrument(skip(self, data))]
    pub fn forecast(&mut self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<ForecastResult>> {
        let mut results = Vec::new();

        info!("Generating forecasts for {} variables", data.len());

        for (var_name, series) in data {
            if series.len() < 5 {
                warn!(
                    "Skipping forecast for {}: insufficient data points",
                    var_name
                );
                continue;
            }

            let forecast_result = self.forecast_variable(var_name, series)?;
            results.push(forecast_result);
        }

        debug!("Generated {} forecasts", results.len());
        Ok(results)
    }

    /// Main ETS forecasting function - replaces placeholder
    fn forecast_variable(&mut self, variable: &str, series: &[f64]) -> Result<ForecastResult> {
        // Sample data if too large
        let data = if series.len() > self.config.reservoir_size {
            series
                .iter()
                .take(self.config.reservoir_size)
                .copied()
                .collect()
        } else {
            series.to_vec()
        };

        if data.len() < 2 {
            return Err(anyhow!("Insufficient data for ETS forecasting"));
        }

        // Detect seasonality
        let seasonality = self.detect_seasonality(&data)?;
        let period = seasonality.period;

        // Try all ETS model combinations and select best
        let best_result = self.select_and_fit_ets_model(&data, period)?;

        // Generate multi-step forecasts
        let forecasts = self.forecast_ets(
            &best_result.model,
            &best_result.state,
            self.config.forecast_horizon,
        )?;

        // Calculate confidence intervals
        let (lower_bounds, upper_bounds) = self.calculate_confidence_intervals(
            &best_result.model,
            &forecasts,
            &best_result.state,
            0.95, // 95% confidence
        );

        Ok(ForecastResult {
            variable: variable.to_string(),
            point_forecasts: forecasts,
            lower_bounds,
            upper_bounds,
            fit_quality: best_result.fit_quality,
            method: format!(
                "ETS-{}{}{}",
                best_result.model.error.as_str(),
                best_result.model.trend.as_str(),
                best_result.model.seasonal.as_str()
            ),
        })
    }

    /// Calculate forecast fit quality
    #[allow(dead_code)]
    fn calculate_fit_quality(&self, actual: &[f64], forecast: &[f64]) -> f64 {
        if actual.len() < 2 || forecast.is_empty() {
            return 0.0;
        }

        // Simple MAPE calculation for last few points
        let n = actual.len().min(forecast.len().min(10));
        let start_idx = actual.len().saturating_sub(n);

        let mape: f64 = actual[start_idx..]
            .iter()
            .zip(&forecast[..n])
            .map(|(&a, &f)| {
                if a != 0.0 {
                    (a - f).abs() / a.abs()
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / n as f64;

        // Convert MAPE to quality score (lower MAPE = higher quality)
        (1.0 - mape.min(1.0)).max(0.0)
    }

    /// Automatic seasonality detection using autocorrelation
    fn detect_seasonality(&self, series: &[f64]) -> Result<SeasonalityResult> {
        if series.len() < 10 {
            return Ok(SeasonalityResult {
                period: 0,
                strength: 0.0,
            });
        }

        let max_period = (series.len() / 2).min(12); // Limit seasonal periods

        // Collect strengths for each candidate period.
        let mut strengths: Vec<(usize, f64)> = Vec::new();
        for period in 2..=max_period {
            if let Some(strength) = self.calculate_seasonal_strength(series, period) {
                strengths.push((period, strength));
            }
        }

        let Some((_, max_strength)) = strengths
            .iter()
            .cloned()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        else {
            return Ok(SeasonalityResult {
                period: 0,
                strength: 0.0,
            });
        };

        // Prefer realistic short seasonal periods if they are close to the best score.
        // This reduces autocorrelation artifacts picking period=2 on small synthetic series.
        let tolerance = 0.02;
        let mut candidates: Vec<(usize, f64)> = strengths
            .into_iter()
            .filter(|(_, s)| *s >= max_strength - tolerance)
            .collect();
        candidates.sort_by(|a, b| a.0.cmp(&b.0));

        let (best_period, best_strength) = if let Some((p, s)) = candidates
            .iter()
            .find(|(p, _)| (3..=5).contains(p))
            .copied()
        {
            (p, s)
        } else {
            // Otherwise choose the smallest period among near-best candidates.
            candidates[0]
        };

        Ok(SeasonalityResult {
            period: if best_strength > 0.1 { best_period } else { 0 },
            strength: best_strength,
        })
    }

    /// Calculate seasonal strength for a given period
    fn calculate_seasonal_strength(&self, series: &[f64], period: usize) -> Option<f64> {
        if series.len() < period * 2 {
            return None;
        }

        let mut seasonal_means = vec![0.0f64; period];
        let mut counts = vec![0usize; period];

        for (i, &value) in series.iter().enumerate() {
            seasonal_means[i % period] += value;
            counts[i % period] += 1;
        }

        for (i, &count) in counts.iter().enumerate() {
            if count > 0 {
                seasonal_means[i] /= count as f64;
            }
        }

        let overall_mean: f64 = series.iter().sum::<f64>() / series.len() as f64;
        let variance: f64 = series
            .iter()
            .map(|&x| (x - overall_mean).powi(2))
            .sum::<f64>()
            / series.len() as f64;

        let seasonal_variance: f64 = seasonal_means
            .iter()
            .enumerate()
            .map(|(i, &mean)| {
                let count = counts[i] as f64;
                count * (mean - overall_mean).powi(2)
            })
            .sum::<f64>()
            / series.len() as f64;

        if variance > 0.0 {
            Some((seasonal_variance / variance).sqrt())
        } else {
            Some(0.0)
        }
    }

    /// Select and fit the best ETS model using information criteria
    fn select_and_fit_ets_model(&self, series: &[f64], period: usize) -> Result<ETSForecastResult> {
        if series.len() < 2 {
            return Err(anyhow!("ETS requires at least 2 observations"));
        }
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
}
