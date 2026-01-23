//! # Advanced Pattern Analysis Validator
//!
//! Input validation and data preparation for advanced pattern analysis.

use anyhow::{anyhow, Result};
use std::collections::HashMap;

use super::types::{AdvancedPatternAnalysisInput, AnalysisConfig};

/// Validator for advanced pattern analysis inputs
pub struct InputValidator;

impl InputValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self
    }

    /// Validate the complete input
    pub fn validate(&self, input: &AdvancedPatternAnalysisInput) -> Result<()> {
        self.validate_time_series_data(input)?;
        self.validate_config(&input.config)?;
        Ok(())
    }

    /// Validate time series data
    fn validate_time_series_data(&self, input: &AdvancedPatternAnalysisInput) -> Result<()> {
        if input.time_series_data.is_empty() {
            return Err(anyhow!("No time series data provided"));
        }

        for (var_name, series) in &input.time_series_data {
            if series.is_empty() {
                return Err(anyhow!("Variable '{}' has no data points", var_name));
            }
            if series.len() < 3 {
                return Err(anyhow!(
                    "Variable '{}' has insufficient data points (minimum 3)",
                    var_name
                ));
            }
            if !series.iter().all(|&x| x.is_finite()) {
                return Err(anyhow!(
                    "Variable '{}' contains non-finite values",
                    var_name
                ));
            }
        }

        Ok(())
    }

    /// Validate analysis configuration
    fn validate_config(&self, config: &Option<AnalysisConfig>) -> Result<()> {
        if let Some(cfg) = config {
            if let Some(sig) = cfg.significance_level {
                if !(0.0..=1.0).contains(&sig) {
                    return Err(anyhow!("Significance level must be between 0.0 and 1.0"));
                }
            }
            if let Some(sens) = cfg.anomaly_sensitivity {
                if !(0.0..=1.0).contains(&sens) {
                    return Err(anyhow!("Anomaly sensitivity must be between 0.0 and 1.0"));
                }
            }
        }

        Ok(())
    }
}

/// Data preparer for cleaning and preprocessing time series data
pub struct DataPreparer;

impl DataPreparer {
    /// Create a new data preparer
    pub fn new() -> Self {
        Self
    }

    /// Prepare and validate data for analysis
    pub fn prepare(
        &self,
        raw_data: &HashMap<String, Vec<f64>>,
    ) -> Result<HashMap<String, Vec<f64>>> {
        let mut prepared_data = HashMap::new();

        for (var_name, series) in raw_data {
            // Remove any remaining non-finite values (shouldn't happen after validation)
            let clean_series: Vec<f64> =
                series.iter().copied().filter(|&x| x.is_finite()).collect();

            if clean_series.len() >= 3 {
                prepared_data.insert(var_name.clone(), clean_series);
            }
        }

        Ok(prepared_data)
    }

    /// Check if data meets minimum requirements for analysis
    pub fn meets_minimum_requirements(
        &self,
        data: &HashMap<String, Vec<f64>>,
        min_points: usize,
    ) -> bool {
        data.values().all(|series| series.len() >= min_points)
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DataPreparer {
    fn default() -> Self {
        Self::new()
    }
}
