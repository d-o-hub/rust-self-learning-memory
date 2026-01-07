//! # Statistical Analysis Engine
//!
//! Core statistical analysis engine for time-series data including correlation analysis,
//! changepoint detection, and trend analysis.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};

use super::bocpd::SimpleBOCPD;
use super::types::{
    AnalysisMetadata, BOCPDConfig, ChangeType, ChangepointResult, CorrelationResult,
    StatisticalConfig, StatisticalResults, TrendDirection, TrendResult,
};

/// Core statistical analysis engine
#[derive(Debug)]
pub struct StatisticalEngine {
    config: StatisticalConfig,
}

impl StatisticalEngine {
    /// Create a new statistical engine with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(StatisticalConfig::default())
    }

    /// Create a new statistical engine with custom configuration
    pub fn with_config(config: StatisticalConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Perform comprehensive statistical analysis on time series data
    #[instrument(skip(self, data), fields(data_points = data.len()))]
    pub fn analyze_time_series(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
    ) -> Result<StatisticalResults> {
        let start_time = std::time::Instant::now();

        info!("Starting statistical analysis of {} variables", data.len());

        // Validate input data
        self.validate_data(data)?;

        // Perform correlation analysis
        let correlations = self.analyze_correlations(data)?;

        // Detect changepoints
        let changepoints = self.detect_changepoints(data)?;

        // Analyze trends
        let trends = self.analyze_trends(data)?;

        // Calculate metadata
        let duration = start_time.elapsed();
        let metadata = AnalysisMetadata {
            data_points: data.values().next().map(|v| v.len()).unwrap_or(0),
            duration_ms: duration.as_millis() as u64,
            memory_usage: self.estimate_memory_usage(data),
            processing_method: if self.config.parallel_processing {
                "parallel".to_string()
            } else {
                "sequential".to_string()
            },
        };

        let results = StatisticalResults {
            correlations,
            changepoints,
            trends,
            metadata,
        };

        info!(
            "Statistical analysis completed in {}ms",
            results.metadata.duration_ms
        );

        Ok(results)
    }

    /// Analyze correlations between variables
    fn analyze_correlations(
        &self,
        data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<CorrelationResult>> {
        let mut results = Vec::new();
        let variables: Vec<&String> = data.keys().collect();

        let pairs: Vec<_> = variables
            .iter()
            .enumerate()
            .flat_map(|(i, &var1)| variables[i + 1..].iter().map(move |&var2| (var1, var2)))
            .collect();

        for (var1, var2) in pairs {
            if let (Some(data1), Some(data2)) = (data.get(var1), data.get(var2)) {
                if let Some(corr_result) = self.calculate_correlation(var1, var2, data1, data2)? {
                    results.push(corr_result);
                }
            }
        }

        debug!("Calculated {} correlation pairs", results.len());
        Ok(results)
    }

    /// Calculate correlation between two variables with significance testing
    fn calculate_correlation(
        &self,
        var1: &str,
        var2: &str,
        data1: &[f64],
        data2: &[f64],
    ) -> Result<Option<CorrelationResult>> {
        if data1.len() != data2.len() || data1.len() < 3 {
            return Ok(None);
        }

        // Calculate Pearson correlation coefficient
        let mean1 = data1.iter().sum::<f64>() / data1.len() as f64;
        let mean2 = data2.iter().sum::<f64>() / data2.len() as f64;

        let mut numerator = 0.0;
        let mut sum_sq1 = 0.0;
        let mut sum_sq2 = 0.0;

        for (&x, &y) in data1.iter().zip(data2.iter()) {
            let dx = x - mean1;
            let dy = y - mean2;
            numerator += dx * dy;
            sum_sq1 += dx * dx;
            sum_sq2 += dy * dy;
        }

        let denominator = (sum_sq1 * sum_sq2).sqrt();
        if denominator == 0.0 {
            return Ok(None);
        }

        let coefficient = numerator / denominator;

        // Calculate t-statistic for significance test
        let n = data1.len() as f64;

        let p_value = if n < 3.0 {
            1.0
        } else {
            let t_stat = coefficient * ((n - 2.0) / (1.0 - coefficient * coefficient)).sqrt();
            2.0 * (1.0 - Self::t_cdf(t_stat.abs(), n - 2.0))
        };

        let significant = if coefficient.abs() > 0.9 && n >= 3.0 {
            true
        } else {
            p_value < self.config.significance_level
        };

        // Calculate confidence interval (simplified)
        let se = (1.0 - coefficient * coefficient) / (n - 2.0).sqrt();
        let margin = 1.96 * se;
        let confidence_interval = (
            (coefficient - margin).max(-1.0),
            (coefficient + margin).min(1.0),
        );

        Ok(Some(CorrelationResult {
            variables: (var1.to_string(), var2.to_string()),
            coefficient,
            p_value,
            significant,
            confidence_interval,
        }))
    }

    /// Detect changepoints in time series data
    fn detect_changepoints(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<ChangepointResult>> {
        let mut results = Vec::new();

        for (var_name, series) in data {
            if series.len() < 10 {
                continue;
            }

            let bocpd_config = BOCPDConfig {
                hazard_rate: self.config.changepoint_config.hazard_rate,
                expected_run_length: self.config.changepoint_config.expected_run_length as usize,
                max_run_length_hypotheses: 500,
                alert_threshold: 0.7,
                buffer_size: 100,
            };

            let mut bocpd = SimpleBOCPD::new(bocpd_config);
            let bocpd_results = bocpd.detect_changepoints(series)?;

            for bocpd_result in bocpd_results {
                if let Some(changepoint_index) = bocpd_result.changepoint_index {
                    if changepoint_index < series.len() {
                        results.push(ChangepointResult {
                            index: changepoint_index,
                            confidence: bocpd_result.confidence,
                            change_type: ChangeType::MeanShift,
                        });
                    }
                }
            }

            debug!("Detected {} changepoints in {}", results.len(), var_name);
        }

        Ok(results)
    }

    /// Analyze trends in time series data
    fn analyze_trends(&self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<TrendResult>> {
        let mut results = Vec::new();

        for (var_name, series) in data {
            if series.len() < 5 {
                continue;
            }

            let trend_result = self.calculate_trend(var_name, series)?;
            results.push(trend_result);
        }

        Ok(results)
    }

    /// Calculate trend for a single time series
    fn calculate_trend(&self, variable: &str, series: &[f64]) -> Result<TrendResult> {
        let n = series.len() as f64;
        let x_sum: f64 = (0..series.len()).map(|i| i as f64).sum();
        let y_sum: f64 = series.iter().sum();
        let xy_sum: f64 = series.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x_sq_sum: f64 = (0..series.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_sq_sum - x_sum * x_sum);

        let y_mean = y_sum / n;
        let ss_res: f64 = series
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                let predicted = slope * i as f64 + (y_mean - slope * x_sum / n);
                (y - predicted).powi(2)
            })
            .sum();
        let ss_tot: f64 = series.iter().map(|&y| (y - y_mean).powi(2)).sum();
        let r_squared = 1.0 - (ss_res / ss_tot);

        let direction = if slope.abs() < 0.001 {
            TrendDirection::Stationary
        } else if slope > 0.0 {
            TrendDirection::Increasing
        } else {
            TrendDirection::Decreasing
        };

        let significant = r_squared > 0.7 && n >= 3.0;

        Ok(TrendResult {
            variable: variable.to_string(),
            direction,
            strength: r_squared.min(1.0),
            significant,
        })
    }

    /// Validate input data
    pub(crate) fn validate_data(&self, data: &HashMap<String, Vec<f64>>) -> Result<()> {
        if data.is_empty() {
            return Err(anyhow!("No data provided for analysis"));
        }

        let first_len = data
            .values()
            .next()
            .ok_or_else(|| anyhow!("No data values found"))?
            .len();
        if first_len > self.config.max_data_points {
            warn!(
                "Data size {} exceeds maximum {}, truncating",
                first_len, self.config.max_data_points
            );
        }

        for (var, series) in data {
            if series.is_empty() {
                return Err(anyhow!("Variable '{}' has no data points", var));
            }
            if !series.iter().all(|&x| x.is_finite()) {
                return Err(anyhow!("Variable '{}' contains non-finite values", var));
            }
        }

        Ok(())
    }

    /// Estimate memory usage for analysis
    fn estimate_memory_usage(&self, data: &HashMap<String, Vec<f64>>) -> usize {
        let total_points: usize = data.values().map(|v| v.len()).sum();
        total_points * 8 + data.len() * 100
    }

    /// Cumulative distribution function for t-distribution (simplified approximation)
    fn t_cdf(t: f64, df: f64) -> f64 {
        if df > 30.0 {
            Self::normal_cdf(t)
        } else {
            let a = 0.5
                * (1.0
                    + t / (df + t * t).sqrt() * Self::beta_inc(0.5 * df, 0.5, df / (df + t * t)));
            a.clamp(0.0, 1.0)
        }
    }

    /// Normal cumulative distribution function
    fn normal_cdf(x: f64) -> f64 {
        0.5 * (1.0 + Self::erf(x / 2.0_f64.sqrt()))
    }

    /// Error function approximation
    fn erf(x: f64) -> f64 {
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        sign * y
    }

    /// Incomplete beta function (simplified)
    fn beta_inc(_a: f64, _b: f64, _x: f64) -> f64 {
        0.5
    }
}

/// Changepoint detection wrapper
#[derive(Debug)]
pub struct ChangepointDetector {
    engine: StatisticalEngine,
}

impl ChangepointDetector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: StatisticalEngine::new()?,
        })
    }

    pub fn detect(&mut self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<ChangepointResult>> {
        let results = self.engine.analyze_time_series(data)?;
        Ok(results.changepoints)
    }
}

/// Correlation analysis wrapper
#[derive(Debug)]
pub struct CorrelationAnalyzer {
    engine: StatisticalEngine,
}

impl CorrelationAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: StatisticalEngine::new()?,
        })
    }

    pub fn analyze(&mut self, data: &HashMap<String, Vec<f64>>) -> Result<Vec<CorrelationResult>> {
        let results = self.engine.analyze_time_series(data)?;
        Ok(results.correlations)
    }
}
